# SPESIFIKASI STORAGE LAYER — DRAF v0.1
**Author:** agent2 (Backend Engineer)
**Tanggal:** 2026-09-09
**Status:** DRAF — menunggu review matt & agent5
**Dependen pada:** PRD-2 (matt), audit agent1 (21 temuan)

---

## 1. TUJUAN

Dokumen ini mendetailkan desain SQLite storage layer untuk engine Rust n8n, menjawab temuan audit:
- F6: Lease premature untuk single-process
- F18: event_log rotation strategy
- F19: Crash antara spill-write dan DB-commit
- F20: Credential encryption key versioning

## 2. PRINSIP DESAIN

1. **SQLite WAL mode** — 1 writer + N reader paralel
2. **Single writer actor** — semua write lewat mpsc channel, hindari SQLITE_BUSY
3. **Write-ahead intent** — catat niat SEBELUM operasi, hapus SETELAH sukses
4. **Append-only event log** — tidak ada UPDATE/DELETE pada event yang sudah ditulis
5. **Versioned encryption** — setiap credential punya key version, mendukung rotasi

## 3. SKEMA DATABASE (migrasi 1)

```sql
-- === METADATA ===
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT
);

-- === WORKFLOW ===
CREATE TABLE workflow (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    nodes_json TEXT NOT NULL,
    connections_json TEXT NOT NULL,
    settings_json TEXT NOT NULL DEFAULT '{}',
    static_data TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE workflow_version (
    workflow_id TEXT NOT NULL REFERENCES workflow(id),
    version INTEGER NOT NULL,
    snapshot_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (workflow_id, version)
);

-- === ENCRYPTION (menjawab F20) ===
CREATE TABLE encryption_key (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version INTEGER NOT NULL UNIQUE,
    algorithm TEXT NOT NULL DEFAULT 'aes-256-gcm',
    key_hash TEXT NOT NULL,  -- SHA-256 dari key, BUKAN key itu sendiri
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    rotated_from INTEGER REFERENCES encryption_key(id),
    description TEXT
);

-- === CREDENTIAL (dengan key version) ===
CREATE TABLE credential (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    data_encrypted BLOB NOT NULL,
    enc_key_version INTEGER NOT NULL DEFAULT 1 REFERENCES encryption_key(version),
    enc_algorithm TEXT NOT NULL DEFAULT 'aes-256-gcm',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- === EXECUTION (tanpa lease — menjawab F6) ===
-- NOTE: lease_holder/lease_expires_at dihapus untuk Fase 1 (single-process).
-- Akan ditambah di Fase 5+ saat multi-worker didukung.
CREATE TABLE execution (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflow(id),
    workflow_version INTEGER NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('regular', 'manual', 'webhook', 'retry')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'waiting', 'success', 'failed', 'needs_review', 'canceled')),
    started_at TEXT,
    finished_at TEXT,
    error_code TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_execution_workflow ON execution(workflow_id);
CREATE INDEX idx_execution_status ON execution(status);

-- === EVENT LOG (append-only, menjawab F18) ===
-- Strategi: 1 tabel dengan batch checkpoint untuk VACUUM safety.
-- Alternatif di masa depan: partition per-execution (execution_NNN_events).
CREATE TABLE event_log (
    execution_id TEXT NOT NULL REFERENCES execution(id),
    seq INTEGER NOT NULL,
    ts TEXT NOT NULL DEFAULT (datetime('now')),
    schema_version INTEGER NOT NULL DEFAULT 1,
    event_type TEXT NOT NULL,
    event_json TEXT NOT NULL,
    PRIMARY KEY (execution_id, seq)
);

-- === TASK (tanpa lease di Fase 1) ===
CREATE TABLE task (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL REFERENCES execution(id),
    node_id TEXT NOT NULL,
    input_index INTEGER NOT NULL DEFAULT 0,
    attempt INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('pending', 'ready', 'running', 'waiting', 'success', 'failed', 'in_doubt', 'canceled')),
    hints_json TEXT DEFAULT '{}',
    -- LEASE: opsional, hanya diisi saat multi-worker (Fase 5+)
    lease_holder TEXT,
    lease_expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    finished_at TEXT,
    error_json TEXT,
    wake_at TEXT  -- untuk node Wait
);

CREATE INDEX idx_task_execution ON task(execution_id);
CREATE INDEX idx_task_status ON task(status);

-- === CHECKPOINT ===
CREATE TABLE checkpoint (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL REFERENCES execution(id),
    schema_version INTEGER NOT NULL DEFAULT 1,
    completed_ids_json TEXT NOT NULL DEFAULT '[]',
    in_flight_ids_json TEXT NOT NULL DEFAULT '[]',
    waiting_json TEXT DEFAULT '{}',
    event_seq INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- === NODE OUTPUT (dengan checksum — menjawab T2) ===
CREATE TABLE node_output (
    execution_id TEXT NOT NULL REFERENCES execution(id),
    node_id TEXT NOT NULL,
    output_index INTEGER NOT NULL DEFAULT 0,
    storage_kind TEXT NOT NULL CHECK (storage_kind IN ('inline', 'spill', 'blob')),
    spill_path TEXT,  -- relatif ke spill root
    checksum_blake3 TEXT,  -- BLAKE3 (lebih cepat dari SHA-256, tetap kriptografis)
    item_count INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (execution_id, node_id, output_index)
);

-- === WRITE-AHEAD INTENT (menjawab F19) ===
-- Mencatat niat SEBELUM operasi fisik. Orphan = crash sebelum commit.
CREATE TABLE spill_intent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id TEXT NOT NULL REFERENCES execution(id),
    intent_type TEXT NOT NULL CHECK (intent_type IN ('write_spill', 'delete_spill', 'write_event', 'update_task')),
    target_path TEXT,  -- untuk spill operations
    payload_json TEXT,  -- untuk event/task operations
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- === SPILL GC ===
CREATE TABLE spill_gc (
    spill_path TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL REFERENCES execution(id),
    ref_count INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- === VARIABLE ===
CREATE TABLE variable (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'string'
);
```

## 4. WRITE-AHEAD INTENT PROTOCOL (menjawab F19)

```
Untuk setiap operasi yang melibatkan spill file:

1. INSERT spill_intent (intent_type='write_spill', target_path='/path/to/spill')
2. COMMIT
3. Tulis spill file ke disk
4. INSERT node_output (spill_path, checksum)
5. INSERT event_log (TaskCompleted)
6. DELETE spill_intent WHERE id = ?
7. COMMIT

Crash recovery:
- Startup scan spill_intent untuk intent tanpa matching completion
- write_spill tanpa matching node_output = orphan spill → hapus file
- delete_spill tanpa matching file = sudah terhapus → noop
```

## 5. CREDENTIAL KEY ROTATION (menjawab F20)

```
Prosedur rotasi key:

1. Buat encryption_key baru dengan version = max(version) + 1
2. Untuk setiap credential dengan enc_key_version = old_version:
   a. Decrypt data dengan key lama
   b. Encrypt ulang dengan key baru
   c. UPDATE credential SET data_encrypted = ?, enc_key_version = new_version
3. Tandai key lama sebagai rotated (rotated_from)
4. Key lama TIDAK dihapus — masih dibutuhkan untuk decrypt credential lama jika rotasi gagal di tengah

Migration dari existing data:
- Semua credential tanpa enc_key_version = version 0 (legacy, plaintext atau format lama)
- Saat upgrade, wajib isi enc_key_version = 1 dan encrypt ulang
```

## 6. EVENT LOG ROTATION STRATEGY (menjawab F18)

**Opsi yang dipertimbangkan:**

| Opsi | Kelebihan | Kekurangan |
|------|-----------|------------|
| A. 1 tabel besar + VACUUM periodik | Simple | VACUUM block semua write |
| B. Partition per-execution | Isolasi bagus | Ribuan tabel untuk ribuan eksekusi |
| C. 1 tabel + batch archival | Kompromi | Complexity archival |

**Rekomendasi: Opsi A dengan modifikasi** — 1 tabel besar, tapi:
- Event log dipindahkan ke file terpisah setelah execution terminal (success/failed/canceled)
- File event di-spill ke disk (format: 1 file per execution, binary format)
- DB hanya menyimpan pointer: `execution.event_log_path TEXT`
- VACUUM hanya diperlukan periodik (monthly)

## 7. LEASE STRATEGY (menjawab F6)

**Fase 1 (single-process):**
- Kolom lease_holder dan lease_expires_at di task table = NULL selalu
- Tidak ada logika lease di executor
- Task status langsung berubah: pending → running → success/failed

**Fase 5+ (multi-worker):**
- Aktifkan lease: task.lease_holder = worker_id, lease_expires_at = now + timeout
- Background sweeper: scan task WHERE status='running' AND lease_expires_at < now()
- Recover action berdasarkan side_effect hint (Idempotent → requeue, NonIdempotent → InDoubt)

## 8. KONFIGURASI SQLITE PRAGMA

```sql
PRAGMA journal_mode = WAL;          -- Concurrent read + single write
PRAGMA synchronous = NORMAL;        -- Balance safety vs speed (WAL aman)
PRAGMA busy_timeout = 5000;         -- Tunggu 5 detik jika locked
PRAGMA cache_size = -64000;         -- 64MB cache
PRAGMA temp_store = MEMORY;         -- Temp tables di RAM
PRAGMA mmap_size = 268435456;       -- 256MB mmap untuk read performance
PRAGMA wal_autocheckpoint = 1000;   -- Checkpoint setiap 1000 page
```

## 9. KEPUTUSAN TERBUKA (untuk matt)

| ID | Keputusan | Rekomendasi agent2 |
|----|-----------|-------------------|
| S-1 | BLAKE3 vs SHA-256 untuk checksum? | BLAKE3 — 10x lebih cepat, tetap kriptografis |
| S-2 | Event log archival ke file? | Ya, setelah execution terminal |
| S-3 | Migration strategy untuk schema v2+? | Inline transformer + version gate |
| S-4 | sqlx offline mode (DATABASE_URL saat build)? | Commit .sqlx/ cache ke repo |
| S-5 | Batch commit interval? | Per-task atau per-10-task (configurable) |

## 10. METRIK VERIFIKASI

- [ ] 10.000 event_log row INSERT < 1 detik
- [ ] Concurrent read saat write: tidak ada SQLITE_BUSY (actor pattern)
- [ ] Crash recovery: 100 SIGKILL → 100 recovery tanpa data loss
- [ ] Credential rotasi: 1000 credential < 10 detik
- [ ] Spill orphan detection: scan startup < 1 detik untuk 10.000 spill file

---

*Dokumen ini DRAF. Mohon review dari matt (arsitektur) dan agent5 (security).*
