# AGENT1_CORE_ENGINE_SPEC — Core Async DAG Execution Engine

**Penulis:** agent1 (Core DAG Execution Engine)
**Tanggal:** 2026-09-09
**Status:** DRAFT v0.1 — menunggu review adversarial + keputusan OPEN
**Mandat:** Perintah fern #82 (spesifikasi crate), #93 (audit Section 1–5), #99 (adversarial auditor)
**Dependen pada:** PRD-1-N8N-ANALYSIS.md, PRD-2-RUST.md, audit agent1 #109–116 (temuan F1–F21, P1)
**Bukan kode:** dokumen ini spesifikasi. ZERO baris Rust sampai PRD Perfection Gate lolos (#93).

> **Aturan kejujuran dokumen ini.** Setiap angka punya sumber/cara ukur.
> Setiap yang belum diputuskan ditandai `OPEN-n` eksplisit — bukan tebakan.
> Referensi silang ke temuan audit memakai ID `[Fn]` dari pesan #111–116.

---

## 1. Ruang Lingkup

Crate yang dirancang di sini: `executor` + `scheduler` (nama kerja `n8n-core-rust`
di #82 dipetakan ke dua crate PRD-2 §3.2 agar tetap satu tanggung jawab per crate).

| Crate | Tanggung jawab | BUKAN tanggung jawab |
|---|---|---|
| `scheduler` | Urutan task, prioritas, admission control, retry/backoff, recovery planning | Menjalankan node, evaluasi expression |
| `executor` | Membangun context, resolusi parameter, memanggil `Node::execute`, menulis hasil | Memilih task mana berikutnya, menyimpan payload |

Kontrak antar-crate HANYA lewat trait `kernel` (PRD-2 §3.3). Tidak ada
ketergantungan langsung `executor → scheduler` atau sebaliknya; keduanya
berkomunikasi lewat tabel `task` + channel in-process (§5.4).

---

## 2. Model DAG

### 2.1 Struktur graf

Graf workflow adalah `petgraph::Graph<NodeData, EdgeData, Directed>`:

```rust
struct NodeData {
    id: String,          // uuid n8n, stabil, maks 64 char
    name: String,        // UNIK per workflow, maks 128 char; kunci expression $('Name')
    node_type: String,   // mis. "n8n-nodes-base.httpRequest", maks 128 char
    type_version: u32,
    position: (i32, i32),// koordinat canvas; dipakai ordering v1
    disabled: bool,
}

struct EdgeData {
    output_index: u32,   // branch ke-N dari source (IF=0/1, Switch=0..N)
    conn_type: ConnType, // Main | AiTool | AiModel | AiMemory | ...
}
```

### 2.2 Aturan validasi (gagal impor = error eksplisit, bukan diam)

| ID | Aturan | Kode error |
|---|---|---|
| V1 | Graf acyclic (cek `petgraph::algo::is_cyclic_directed`) | `E_DAG_CYCLE` |
| V2 | `name` unik per workflow (case-sensitive, persis n8n) | `E_DUP_NODE_NAME` |
| V3 | Setiap koneksi merujuk `name` yang ada | `E_DANGLING_EDGE` |
| V4 | `output_index` < jumlah output descriptor node | `E_BAD_BRANCH` |
| V5 | Node bertipe trigger tidak boleh punya input `Main` | `E_TRIGGER_HAS_INPUT` |
| V6 | Maks 10.000 node, 50.000 edge per workflow (batas DoS impor) | `E_WORKFLOW_TOO_LARGE` |

Batas V6: angka konservatif; workflow n8n nyata terbesar yang diketahui indirect
< 1.000 node. `OPEN-1`: kalibrasi ulang setelah korpus 50 tersedia.

### 2.3 Rename safety (jawaban atas PRD-1 §4.2)

Importer TIDAK me-rename. Jika dua workflow digabung (future), collision `name`
ditolak dengan `E_DUP_NODE_NAME`. Referensi expression `$(name)` di-resolve
saat plan; nama tak dikenal → `E_UNKNOWN_NODE_REF` pada node pemakai, dengan
pesan menyebut nama yang hilang.

---

## 3. Execution Order v0 / v1

### 3.1 Definisi eksak

**v1 (default):** depth-first per branch. Urutan successor diurutkan by
`(position.y ASC, position.x ASC, name ASC)` — tiebreaker `name` ditambahkan
karena PRD-1 §5.3 tidak mendefinisikan tiebreak posisi identik.

**v0 (legacy):** breadth-first by topological depth. Depth dihitung dari node
tanpa input. Node sedalam sama diurutkan dengan tiebreak yang sama dengan v1.

### 3.2 Algoritma planner (deterministik)

```
1. Topo-sort DAG (Kahn). Jika cycle → E_DAG_CYCLE (V1).
2. Hitung depth tiap node = 1 + max(depth predecessor), trigger = 0.
3. v1: DFS dari tiap trigger (urutan trigger by tiebreak), kunjungi successor
   terurut; catat branch_seq global menaik.
   v0: kelompokkan by depth ASC, dalam grup urutkan by tiebreak.
4. Emit daftar TaskDefinition { node_id, depth, branch_seq }.
```

Kompleksitas: O(V + E). Deterministik byte-per-byte untuk input sama
(syarat differential testing §10.2 PRD-2).

`OPEN-2`: perilaku n8n 2.x aktual untuk workflow `executionOrder: v0`
(fallback ke v1 vs dieksekusi) — temuan F-verifikasi PRD-1 (C3 di #87).
Sampai terverifikasi, kita implementasikan v0 sungguhan.

---

## 4. Task Lifecycle (state machine eksak)

Status task (kolom `task.status`):

```
Pending → Ready → Running → Succeeded | FailedTerminal
Pending → Ready → Running → FailedRetry → Ready (attempt+1, backoff)
Running → Waiting (Wait node / webhook-resume; wake_at diset)
Waiting → Ready (timer tiba / webhook datang / resume manual)
Running → InDoubt (crash dengan side_effect=NonIdempotent; terminal)
* → Cancelled (cancel manual; terminal)
```

Transisi ILEGAL (bug jika terjadi, wajib `debug_assert` + metric):
`Succeeded → *`, `FailedTerminal → *`, `InDoubt → *`, `Cancelled → *`,
`Pending → Running` (melewati Ready), `Waiting → Running` (melewati Ready).

**Enforcement lapis-DB (jawaban atas review agent2 #260):** `CHECK`
tidak bisa melihat nilai lama, jadi gunakan trigger:
`CREATE TRIGGER trg_task_transition BEFORE UPDATE OF status ON task`
yang `RAISE(ABORT, 'E_ILLEGAL_TRANSITION')` bila pasangan
`(OLD.status, NEW.status)` tidak ada di tabel transisi legal §4.
Defense-in-depth: Rust `debug_assert` (lapis 1) + trigger DB (lapis 2)
+ metric `illegal_transition_total` (lapis 3, alert bila > 0).

Kolom `attempt` mulai 1. `max_attempts = 1 + maxTries` (n8n `retryOnFail=false`
→ tepat 1 attempt). Backoff: `waitBetweenTries` ms eksak n8n; jika 0,
default 1.000 ms dengan jitter ±10% (didefinisikan di sini, bukan tebakan).

---

## 5. Scheduler

### 5.1 Prioritas (jawaban atas F16 — D56 didefinisikan di sini)

```rust
struct Priority { depth: u32, branch_seq: u32, retry_boost: u8 }
```

Ordering leksikografis `(depth ASC, branch_seq ASC, retry_boost DESC)`.
`retry_boost = min(attempt, 255)` — retry didahulukan agar tidak starvation
di belakang antrian panjang. Dihitung saat plan (depth, branch_seq) +
saat enqueue (retry_boost). Deterministik.

**Penyimpanan (jawaban atas review agent2 #260): TIGA kolom terpisah**
(`depth INTEGER, branch_seq INTEGER, retry_boost INTEGER`), BUKAN satu
INTEGER packed. Alasan: query `ORDER BY` langsung tanpa decode, tidak ada
trik inversi (255-boost) yang rawan salah, dan independen terhadap batas
V6. Indeks: `CREATE INDEX idx_task_sched ON task(status, depth,
branch_seq)`. Nilai ditulis scheduler saat enqueue; DB tidak pernah
menghitung ulang prioritas.

### 5.2 Admission control

Task `Ready` di-admit jika SEMUA terpenuhi:

1. `running_count < max_concurrent_tasks` (default: §6.1).
2. Governor level ≠ `Critical` (§7); jika `Pressure`, hanya task dengan
   `weight ≤ Weight::Medium` di-admit.
3. `max_concurrency` descriptor node (ResourceHint) belum tercapai.
4. Budget RAM proyeksi (`ram_footprint` kernel + `chunk × item`) < sisa budget.
5. Template Hub: `manifest_sig` Ed25519 TERVERIFIKASI-ULANG saat first-execution
   (verify-or-refuse; tutup TOCTOU install→run #602/#603; KONTRAK-engine bukan
   gate-SEC per #618/#619, disahkan fern #621).

Penolakan admission BUKAN error — task tetap `Ready`, dicoba lagi tick
berikutnya (tick = 10 ms, terkonfigurasi). Pengecualian: butir-5 GAGAL =
hard-refuse eksekusi template (integritas, bukan penjadwalan).

### 5.3 Single-process Fase 1: TANPA lease (jawaban atas F6)

Kolom `lease_holder`, `lease_expires_at` TIDAK ADA di skema Fase 1.
Klaim task = `UPDATE task SET status='Running' WHERE id=? AND status='Ready'`
dalam satu transaksi + writer tunggal (§5.4). race antar-thread mustahil
karena hanya writer yang menyentuh tabel task.

Lease dikembalikan di Fase 5 (distributed workers) sebagai migrasi skema
terversioning, BUKAN kolom menganggur sejak awal.

### 5.4 Single-writer actor (pola T1 agent2 / R-A agent1, DISETUJUI #96)

Semua tulis ke SQLite lewat SATU task tokio (`db_writer`) via
`tokio::mpsc::Sender<DbOp>` (kapasitas 1.024, bounded — penuh = backpressure,
bukan OOM). Executor/scheduler TIDAK PERNAH menyentuh `sqlx` langsung;
mereka mengirim `DbOp::{InsertTask, TransitionTask, AppendEvent, ...}`.

Konfigurasi SQLite (jawaban atas F-temuan WAL; spesialisasi agent1):

```sql
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA journal_size_limit = 67108864;  -- 64 MB, cegah WAL tak terbatas
```

Batch commit: writer flush tiap 5 ms ATAU 100 op (mana dulu).
Target terukur: 0 `SQLITE_BUSY` pada uji N-writer (§12).

---

## 6. Executor & Runtime Tokio

### 6.1 Konfigurasi runtime (jawaban atas F7)

```rust
worker_threads   = min(num_cpus::get(), 8)   // default; terkonfigurasi 1..=64
max_concurrent_tasks = worker_threads * 4    // default; terkonfigurasi
thread_stack_size    = 2 MiB                 // default tokio; eksplisit
```

`max_blocking_threads = 512` default tokio TIDAK diubah; operasi blocking
(file spill) memakai `spawn_blocking` dengan semaphore sendiri (32 slot)
agar tidak memonopoli pool blocking.

Rasional batas 8 worker: mesin target 2 CPU (PRD-1 §14); 8 cukup untuk
I/O-bound (HTTP) tanpa konteks-switch berlebih. `OPEN-3`: kalibrasi dengan
benchmark Fase 1 (workload CPU-bound vs I/O-bound).

### 6.2 Alur eksekusi satu task (eksak, melengkapi PRD-2 §6.2)

```
1. Ambil TaskDefinition + NodeDescriptor dari registry (§8).
2. Bangun NodeContext: input view (streaming jika BufferingMode::Streaming),
   PriorOutputs bridge (baca spill by node name — O(1) lookup index).
3. Resolusi parameter: kelompokkan expression by `eval_scope` (kontrak
   agent4 §11): `per_task` (executeOnce/kredensial/header statis) dievaluasi
   SEKALI per task; `per_item` (default) dievaluasi per item. Kedua kelompok
   masing-masing SATU eval_batch QuickJS (amortisasi setup context; PRD-2
   §8.2). Hasil per_task di-cache di NodeContext dan dipakai ulang untuk
   semua item task itu — TIDAK dievaluasi ulang per item (anti-deviasi:
   `$now` di header statis harus identik untuk semua item).
4. Panggil Node::execute dengan timeout = min(node_timeout, 300s default).
5. Hasil → ItemList::from_vec (kecil) / spill_from_iter (governor memutuskan,
   BUKAN node — PRD-2 §7.2).
6. Kirim DbOp batch ke db_writer: node_output row + TaskCompleted event.
7. Tandai task Succeeded (via writer). Intent spill dicatat SEBELUM tulis
   file (jawaban atas F19 — write-ahead intent).
```

Timeout eksekusi node: default 300 s, terkonfigurasi per node type.
Timeout = `FailedRetry` jika retry tersedia, else `FailedTerminal` + kode
`E_NODE_TIMEOUT`. `OPEN-4`: perilaku timeout n8n per node type
(perlu verifikasi upstream sebelum katalog deviasi final).

### 6.2.1 Determinism record (kontrak replay Fase-1a)

Replay (QA internal, TIMELINE, Envelope) butuh input nondeterministik yang
terekam, BUKAN snapshot state. Executor menulis satu record per execution
(sebelum task pertama jalan): `{wall_clock_ms, rng_seed (=seed W2-DETERM-ENFORCE),
node_versions, execution_id_logis, webhook_urls, env_allowlist, template_sha256,
content_version (keduanya per OPEN-HUB-1 #536), recordset_sha256 (jembatan
replay↔determinisme, AGENT10-W2-DETERM-ENFORCE-SPEC §6 #686)}`. Prioritas isi mengikuti
DETERMINISM-SOURCE-INVENTORY.md agent5 (eksternal 128/171 file via
record-replay HTTP; jam 28; cron DEV-A4 via Rosetta; random 3; exec-id 2;
webhookUrl 1). `$now` per_task di-cache (§6.2 langkah 3) konsisten dengan
jam-beku; `$env` hanya dari allowlist. Respons eksternal direkam oleh
lapisan record-replay (bukan executor) dan diindeks hash-request.

### 6.3 Checkpoint (jawaban atas temuan kolom JSON raksasa)

TIDAK ADA kolom `completed_ids`/`in_flight_ids` JSON. Sebagai gantinya tabel
ternormalisasi:

```sql
checkpoint_node(execution_id, node_id, status, attempt, finished_at,
                PRIMARY KEY (execution_id, node_id));
checkpoint_meta(execution_id PRIMARY KEY, event_seq, created_at);
```

Recovery = `SELECT` dua tabel ini. O(completed) baris, bukan O(1) blob
yang tumbuh tanpa batas. `waiting_json` diganti kolom `wake_at` di `task`
+ tabel `wait_state(execution_id, node_id, resume_key, spill_path)` (`spill_path` = `SpilledList.path`, handle data-plane; payload tak dibawa di kolom JSON. Dulu bernama `payload_ref` — diganti #1038 agar selaras kanon Ruling-3(c); alias historis dicatat di sini, bukan kolom ganda).

---

## 7. Memory Governor (angka eksak)

Budget: `min(50% RAM terdeteksi, 4 GiB)` default; floor 256 MiB; terkonfigurasi.
Level (PRD-2 §6.4 dipertahankan, ambang eksak):

| Level | Ambang | Aksi scheduler | Aksi executor |
|---|---|---|---|
| Normal | < 60% dari budget | admit semua yg lolos §5.2 | inline s/d max_inline_items |
| Pressure | 60–85% dari budget | hanya weight ≤ Medium | paksa spill list baru; max_inline_items ÷ 4 |
| Critical | > 85% dari budget | tolak admission baru | spill list inline terbesar; tugas baru ditunda |

`max_inline_items` default 10.000, `max_inline_bytes` default 1 MiB
(mana tercapai dulu). Pengukuran via `ItemList::ram_footprint()` (kernel)
+ `memory-stats` crate untuk RSS proses (sampling tiap 100 ms, bukan per task).

**INVARIAN ARSITEKTURAL (garis merah, dari analisis fork #369):**
baca spill WAJIB terindeks (`read_at`/`read_range` kernel) — baca blob-utuh
(`read() → Vec<u8>` seluruh file) DITOLAK di semua lapisan, karena menghapus
satu-satunya keunggulan memori terukur atas n8n (BENCH-A03 —
dengan kualifikasi #434/#461/#471: angka terukur-berdiri namun dihasilkan
kode contoh tanpa test-penjaga (penguncian-regresi = prasyarat pra-rilis);
angka-SPILLED (index 38,1MB/peak 50,4MB) transfer-antar-mesin, klaim
reduksi-takhingga/OOM-kill SPESIFIK-HOST dan dilarang dikutip tanpa
batas-memori-eksplisit). Executor,
replay-debug (TIMELINE), dan diff-harness hanya boleh memakai akses terindeks.

Anti-thrash: level turun butuh 5 s di bawah ambang bawah (hysteresis),
mencegah osilasi Pressure↔Critical.

---

## 8. Node Registry (jawaban atas F8)

Arah dependensi FINAL (mengikat, diverifikasi check-freeze):

```
nodes-core → workflow-registry → workflow (parser)
executor → workflow-registry (baca descriptor)
workflow-registry → kernel SAJA (trait Node, NodeDescriptor)
```

- Parser `workflow` TIDAK validasi parameter; ia emit `UnresolvedWorkflow`.
- `workflow-registry` resolve + validasi parameter → `ResolvedWorkflow`.
- Executor hanya menerima `ResolvedWorkflow` (type-state: tidak bisa
  mengeksekusi yang belum tervalidasi — enforcement kompilator).

Registry diisi saat startup dari `nodes-core` + `nodes-openapi` (inventory
statik, BUKAN plugin dinamis — prinsip satu binary §2).

### 8.1 Alias node deprecated (jawaban atas temuan matt #207)

Template lama memakai tipe node deprecated. Registry WAJIB memetakan alias
ini (bukan menolak impor), karena penolakan merusak kompatibilitas L1:

| Alias deprecated | Dipetakan ke | Dasar |
|---|---|---|
| `n8n-nodes-base.function` | `n8n-nodes-base.code` (mode JS run-once) | pendahulu Code node |
| `n8n-nodes-base.cron` | `n8n-nodes-base.scheduleTrigger` | pendahulu Schedule Trigger |

Aturan: alias di-resolve saat `UnresolvedWorkflow → ResolvedWorkflow` (§8),
dicatat di `plan.json` (`"aliases_resolved": [...]`) agar harness differential
agent5 bisa memverifikasi. Alias TIDAK didokumentasikan sebagai node Tier —
mereka shim kompatibilitas. Alias baru hanya ditambah lewat katalog deviasi
(DEV-xxx) + pemilik katalog, tidak diam-diam.

---

## 9. Error Taxonomy (kode eksak)

| Kode | Arti | Terminal? | Retry? |
|---|---|---|---|
| `E_DAG_CYCLE` | cycle di graf | ya (impor) | tidak |
| `E_DUP_NODE_NAME` | nama ganda | ya (impor) | tidak |
| `E_DANGLING_EDGE` | edge ke nama tak ada | ya (impor) | tidak |
| `E_BAD_BRANCH` | output_index invalid | ya (impor) | tidak |
| `E_TRIGGER_HAS_INPUT` | trigger ber-input | ya (impor) | tidak |
| `E_WORKFLOW_TOO_LARGE` | > batas V6 | ya (impor) | tidak |
| `E_UNKNOWN_NODE_REF` | $('Name') tak dikenal | tidak (plan) | tidak |
| `E_NODE_TIMEOUT` | eksekusi > timeout | tergantung retry | ya |
| `E_EXPR_EVAL` | expression gagal (dengan node+field+item_index) | tergantung onError | ya |
| `E_SPILL_CORRUPT` | varian-eksak InvalidMagic/ChecksumMismatch/OffsetOutOfRange (#750; naik-kelas dari Invalid-generik #746/#747) | ya (execution CORRUPTED) | tidak |
| `E_SPILL_IO` | gagal baca/tulis spill (varian SpillIo; infra-bukan-korupsi #750) | node FAILED + retry-kebijakan | ya |
| `E_SPILL_MISSING` | file spill hilang | ya (execution CORRUPTED) | tidak |
| `E_DB_WRITER_DOWN` | actor writer mati | ya (proses harus restart) | tidak |
| `E_BUDGET_EXCEEDED` | 1 execution > budget (pengaman terakhir) | ya (execution dibatalkan) | tidak |

Semua error membawa `{ code, node_id?, item_index?, retryable: bool,
caused_by? }`. Tidak ada string error tanpa kode. `E_DB_WRITER_DOWN`
adalah satu-satunya error yang menghentikan SELURUH proses (supervisor
restart; recovery dari checkpoint §6.3).

---

## 10. NFR & Cara Ukur (prinsip #6)

| ID | Metrik | Target Fase 1 | Cara ukur |
|---|---|---|---|
| N1 | Peak RSS vs n8n, workload identik | ≤ 25% | harness differential + `/usr/bin/time -v` |
| N2 | 5jt item di 2 GB | lolos, ≤ 150 MB RSS | BENCH-A03 direproduksi di engine DALAM MemoryMax=2G + swap-0 (cgroup/systemd-run, aturan #471) |
| N3 | SQLITE_BUSY pada uji N-writer | 0 per 100rb op | test konkurensi 64 task × 5.000 op |
| N4 | Determinisme planner | byte-identik 1.000 run | test seed + hash output plan |
| N5 | Recovery SIGKILL | 0 duplikasi side-effect | kill -9 di 10 titik + verifikasi |
| N6 | Leak spill file | 0 setelah 1.000 execution | orphan sweeper + hitung file |
| N7 | Cold start CLI | ≤ 500 ms | hyperfine, mesin target 2 CPU |
| N8 | Latensi plan workflow 1.000 node | ≤ 100 ms p99 | criterion bench |

---

## 11. Rencana Uji

1. **Unit:** state machine transisi (semua edge legal + semua edge ilegal
   di-assert), tiebreak ordering, backoff/jitter bounds, priority ordering.
2. **Property:** proptest plan determinism (acak DAG valid → hash stabil).
3. **Integration:** vertical slice Manual→Set→IF→HTTP(mock) end-to-end;
   crash-injection di 7 titik (§4 transisi).
4. **Differential hooks:** planner emit `plan.json` deterministik agar harness
   agent5 bisa bandingkan urutan eksekusi vs n8n (kontrak antar-agent).
5. **Soak:** 1.000 execution campuran + governor di 90% budget paksa;
   assert N6 + tidak ada deadlock writer (timeout op 30 s → E_DB_WRITER_DOWN).

---

## 12. Keputusan OPEN (butuh manusia/fern/matt — bukan tebakan)

| ID | Keputusan | Opsi | Rekomendasi agent1 |
|---|---|---|---|
| OPEN-1 | Batas V6 | 10k/50k vs kalibrasi korpus | pertahankan s/d korpus |
| OPEN-2 | Semantik v0 aktual n8n 2.x | v0-sungguhan vs fallback-v1 | v0-sungguhan s/d verifikasi |
| OPEN-3 | worker_threads default | min(cpu,8) vs =cpu | min(cpu,8) s/d bench |
| OPEN-4 | Timeout per node type | tiru n8n vs seragam 300s | seragam 300s s/d verifikasi |
| OPEN-5 | Hash checksum spill vs Envelope (T-11 DIPECAH per agent10 #518; status-final per matt #634) | SHA-256 spill / BLAKE3 Envelope | KEPUTUSAN-KERJA (final = matt + pengukuran): T-11a spill = SHA-256 PERTAHANKAN (konsisten E_SPILL_CORRUPT + data-plane nyata; biaya-migrasi nyata); T-11b Envelope = BLAKE3 (cakupan-eksplisit Envelope/audit-chain, BUKAN putusan-T-11); klaim-kecepatan DILARANG s/d ukur |

---

## 13. Traceability Temuan → Spec

| Temuan | Status di spec ini |
|---|---|
| F6 (lease YAGNI) | §5.3: lease dihapus Fase 1 |
| F7 (tokio tanpa angka) | §6.1: angka eksak + OPEN-3 |
| F8 (arah dependensi) | §8: registry + arah final |
| F16/D56 | §5.1: Priority didefinisikan |
| F19 (crash spill-vs-DB) | §6.2 langkah 7: write-ahead intent |
| F4 (index O(N)) | diteruskan ke agent3 (storage) sebagai syarat; engine berasumsi lookup O(1) |
| F18 (event_log skala) | diteruskan ke agent3; engine menulis event tanpa asumsi partisi |
| F20 (key_version) | diteruskan ke agent2 (API/auth); engine membaca kredensial terdekripsi via auth |

---

*Akhir spec v0.1. Siap untuk adversarial review oleh agent2–5 dan keputusan
OPEN oleh fern/matt/manusia sebelum Fase 1 dimulai.*
