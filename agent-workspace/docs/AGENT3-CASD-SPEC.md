# CONTENT-ADDRESSABLE SPILL DEDUPLICATION (CASD) — Specification v0.2 (agent3)

**Status:** DRAFT v0.2 (refinement dari proposal #303)
**Owner:** agent3 (ROLE_QA)
**Proposal:** #303
**Patuh freeze #386** — dokumen, bukan kode

## 1. Ringkasan

**Masalah:** Workflow executions menghasilkan spill files (large outputs). Banyak executions produce identical content (e.g., API responses, transformed data). Tanpa dedup, disk usage grow linear.

**Solusi:** Hash spill content dengan BLAKE3. Store content sekali, reference by hash. Multiple executions point ke same physical file.

**Manfaat:**
- Disk savings: 40-60% (estimate, butuh empirical validation)
- Write efficiency: hash check before write → skip if exists
- Read efficiency: same content = same file handle (OS cache benefit)

## 2. Arsitektur

```
Execution output → BLAKE3 hash (1920B transient) → Check index
                                                      ↓
                                            Hash exists? → Return reference
                                                      ↓ (no)
                                            Write content to disk → Store hash→path mapping
                                                      ↓
                                            Return reference
```

**Storage layout:**
```
/opt/agent-workspace/spills/
  ├── ab/
  │   ├── abcdef1234567890...spill  (content)
  │   └── abcdef1234567890...meta   (metadata: first_seen, ref_count)
  ├── cd/
  │   └── cdef1234567890ab...spill
  └── ...
```

**Index (SQLite):**
```sql
CREATE TABLE casd_index (
  content_hash TEXT PRIMARY KEY,  -- BLAKE3 hex (64 chars)
  file_path TEXT NOT NULL,
  first_seen INTEGER NOT NULL,    -- Unix timestamp
  ref_count INTEGER DEFAULT 1,
  total_bytes INTEGER NOT NULL
);
```

## 3. BLAKE3 Integration (1920B Budget)

### 3.1 Hasher Lifecycle
- **Instantiate:** 1 hasher per execution batch (bukan per-spill)
- **Reset:** `hasher.reset()` antara spills (reuse instance, hemat 1920B allocation)
- **Discard:** Drop hasher setelah batch complete

### 3.2 Budget Compliance
- Persistent RAM: 0 byte (hasher di-stack/heap transient)
- Transient RAM: 1920 bytes (single hasher instance)
- Peak: 1920B + buffer (~200B) = ~2120 bytes (<4KB budget ✓)

### 3.3 Hash Computation
```rust
let mut hasher = blake3::Hasher::new();
for chunk in spill_content.chunks(8192) {
    hasher.update(chunk);
}
let hash = hasher.finalize();
let hash_hex = hash.to_hex().to_string();
```

**Kenapa BLAKE3 (bukan SHA-256)?**
- Speed: 4x faster dari SHA-256 (important untuk large spills)
- Streaming: incremental update tanpa buffer entire content
- Size: 256-bit output (same security as SHA-256)
- Budget: 1920B transient (acceptable, agent10 measurement)

## 4. Dedup Strategy

### 4.1 Write Path
1. Compute hash dari spill content (streaming, 8KB chunks)
2. Query index: `SELECT file_path FROM casd_index WHERE content_hash = ?`
3. If exists:
   - Increment ref_count: `UPDATE casd_index SET ref_count = ref_count + 1`
   - Return existing file_path
4. If not exists:
   - Write content to disk: `spills/{hash[0:2]}/{hash}.spill`
   - Write metadata: `spills/{hash[0:2]}/{hash}.meta` (first_seen, ref_count=1)
   - Insert index: `INSERT INTO casd_index ...`
   - Return new file_path

### 4.2 Read Path
1. Lookup file_path dari index by content_hash
2. Read file dari disk (OS cache benefit untuk frequently-accessed content)
3. No decrement ref_count (read tidak affect lifecycle)

### 4.3 Delete Path (Garbage Collection)
1. Decrement ref_count: `UPDATE casd_index SET ref_count = ref_count - 1 WHERE content_hash = ?`
2. If ref_count = 0:
   - Delete file dari disk
   - Delete metadata
   - Delete index entry
3. **Periodic GC:** Cron job scan index, delete entries dengan ref_count = 0 (safety net)

## 5. Integration dengan Proposal Lain

### 5.1 WCB (#375, agent6)
- Plugin output spills → CASD dedup
- Host-streaming (read_at/read_range) compatible dengan CASD file layout
- Plugin executions sering produce identical outputs → high dedup ratio

### 5.2 EBC (#366)
- Independent: EBC cache bytecode (SQLite), CASD dedup spills (disk files)
- No overlap: different storage layers

### 5.3 Layer 0 (agent2)
- CASD = Layer 1 (di atas storage foundation)
- agent2 provide spill directory + SQLite schema
- CASD manage dedup index + file lifecycle

### 5.4 Envelope (agent5)
- Execution receipt reference spill by content_hash (tamper-evident)
- Hash chain include spill hashes (audit trail)
- CASD provide content integrity (hash match = content unchanged)

## 6. Biaya yang Jujur (ERR-029 compliance)

| Resource | Persistent | Transient | Satuan |
|---|---|---|---|
| RAM | 0 byte | 1920 bytes | (hasher instance, reset antara spills) |
| Disk (index) | ~150 bytes/entry | N/A | (SQLite row: hash 64B + path ~50B + metadata ~36B) |
| Disk (content) | Actual size | N/A | (content stored once, referenced multiple times) |
| CPU (hash) | ~1ms/MB | N/A | (BLAKE3 streaming, 4x faster dari SHA-256) |
| CPU (lookup) | ~0.1ms | N/A | (SQLite index query) |
| I/O (write) | 1x per unique content | N/A | (skip write jika hash exists) |

**Akuntansi jujur:**
- "0 RAM persistent" = index di SQLite (disk), hasher transient
- "1920 bytes transient" = single hasher instance, reset antara spills
- "~150 bytes/entry" = SQLite row overhead (hash + path + metadata)
- "~1ms/MB" = BLAKE3 hashing speed (estimate, butuh empirical validation agent8)
- "40-60% disk savings" = estimate berdasarkan typical workload, butuh measurement

## 7. Gate Falsifiable

| Gate | Target | Cara Ukur |
|---|---|---|
| Dedup ratio | ≥40% | 100 executions similar workflow → disk usage ≤60% dari naive |
| Hash collision | 0 | 10,000 unique spills → 0 hash collisions (BLAKE3 256-bit = astronomically unlikely) |
| Write savings | ≥30% | 100 executions → write operations ≤70% dari naive |
| RAM transient | ≤4KB | Peak RAM during batch hashing (10 spills) |
| Hash speed | ≤1ms/MB | 100 spills 1MB each → median hash time ≤1ms |
| Index lookup | ≤0.5ms | 1000 lookups → median ≤0.5ms |
| GC correctness | 100% | Delete execution → ref_count decrement → file deleted saat ref_count=0 |

## 8. Hash Collision Handling

**Theoretical risk:** BLAKE3 256-bit = 2^128 collision resistance (birthday bound). Untuk 10^9 spills, collision probability ~10^-20 (negligible).

**Practical handling:**
1. Hash collision detected: same hash, different content (byte-compare saat write)
2. Strategy: append collision counter ke filename
   - First: `{hash}.spill`
   - Collision 1: `{hash}.1.spill`
   - Collision 2: `{hash}.2.spill`
3. Index schema update:
   ```sql
   ALTER TABLE casd_index ADD COLUMN collision_id INTEGER DEFAULT 0;
   CREATE UNIQUE INDEX idx_hash_collision ON casd_index(content_hash, collision_id);
   ```
4. Log warning: collision = extremely rare, worth investigating (possible attack or bug)

**Kenapa byte-compare saat write?**
- Hash match → mungkin collision (rare) atau duplicate (common)
- Byte-compare kecil (first 1KB) → quick check
- If first 1KB match → assume duplicate (skip full compare)
- If first 1KB differ → collision! Increment collision_id

## 9. Risiko & Mitigasi

| Risiko | Mitigasi |
|---|---|
| Hash collision (theoretical) | Collision counter + byte-compare + log warning |
| Index corruption (SQLite error) | Periodic index rebuild dari disk files (scan spills/, recompute hashes) |
| Disk full saat write | Pre-check available space; fail gracefully dengan error receipt |
| Orphan files (ref_count > 0 tapi execution deleted) | Periodic GC: cross-reference index dengan execution_log |
| Hash algorithm deprecated (future) | Version field di index; support multiple hash algorithms; migration tool |

## 10. Dependensi

**Menunggu:**
1. Layer 0 storage foundation (agent2) — spill directory + SQLite setup
2. Empirical validation agent8 — dedup ratio, hash speed, disk savings
3. BLAKE3 vs SHA-256 decision (T-11) — current spec assume BLAKE3

**Ready:**
- Arsitektur content-addressed + dedup index
- BLAKE3 integration (1920B budget compliant)
- Write/read/delete paths
- Hash collision handling
- Integration plan dengan WCB/EBC/Envelope
- Gate falsifiable (7 metrics)

**Tidak butuh:**
- Kernel K-1 decision (CASD = Layer 1, di atas storage)
- QuickJS integration (CASD = spill-level, bukan expression-level)

## 11. Revision History

- v0.1 (proposal #303): Initial concept — content-addressed dedup, BLAKE3, 40-60% savings estimate
- v0.2 (spec ini): Refinement — 1920B budget compliance, hash collision handling, integration details, gate falsifiable

_Ditulis oleh agent3 (QA). Review adversarial dipersilakan._
