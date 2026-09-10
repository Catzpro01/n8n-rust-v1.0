# ADR-v2-STORAGE-BATCH-WAL-OPTIMIZATION

## Status
**PROPOSED** - Sayembara ADR v2 "ZERO-SACRIFICE PERFORMANCE"

## Author
agent2 (Storage Engineer)

## Date
2026-09-09

## Problem Statement

Storage crate saat ini menggunakan SQLite default configuration (journal_mode=DELETE) dan single-row INSERT untuk lineage tracking. Pada high-volume workloads (>10k lineage edges per execution), ini menciptakan bottleneck:

1. **Single-row INSERT**: Setiap INSERT = 1 transaction flush ke disk → I/O bound
2. **journal_mode=DELETE**: SQLite menulis rollback journal terpisah → double write overhead
3. **No connection pooling**: Setiap operation buka/tutup connection → overhead handshake
4. **No batch API**: Caller harus loop INSERT satu-per-satu → tidak ada bulk optimization

**Impact pada sistem:**
- Execution dengan 50k lineage edges ≈ 50k individual INSERTs ≈ 2-5 detik (HDD) / 0.5-1 detik (SSD)
- Write amplification 2x (journal + database)
- Tidak scalable untuk multi-execution concurrent workloads

## Solution

### 1. WAL Mode (Write-Ahead Logging)
```rust
// migrations/005_wal_mode.sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;  // WAL works best with NORMAL
PRAGMA wal_autocheckpoint=1000;  // checkpoint every 1000 pages
```

**Benefit:**
- Readers don't block writers, writers don't block readers
- Single write (no rollback journal) → 50% write amplification reduction
- Better concurrency untuk read-heavy workloads (lineage queries)

### 2. Batch INSERT API
```rust
// New API in lineage.rs
pub fn insert_lineage_edges_batch(&self, edges: &[LineageEdge]) -> Result<(), StorageError> {
    let c = self.db.conn.lock().unwrap();
    c.execute("BEGIN TRANSACTION", [])?;
    
    let mut stmt = c.prepare(
        "INSERT INTO lineage_edge (execution_id, output_ref, repr, ...) VALUES (?1, ?2, ?3, ...)"
    )?;
    
    for edge in edges {
        stmt.execute(params![...])?;
    }
    
    c.execute("COMMIT", [])?;
    Ok(())
}
```

**Benefit:**
- 10k edges dalam 1 transaction = 1 disk flush (bukan 10k)
- Estimated 10-50x throughput improvement
- Atomic batch (all-or-nothing semantics)

### 3. Connection Pool (Optional Phase 2)
```rust
// Use r2d2 connection pool
pub struct StoragePool {
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}
```

**Benefit:**
- Reuse connections across operations
- Reduce handshake overhead untuk high-frequency queries

## Zero-Regression Proof

### Functional Equivalence
- ✅ SQLite WAL is ACID-compliant (same guarantees as DELETE mode)
- ✅ Batch INSERT uses same schema + constraints (G-2 CHECK, foreign keys)
- ✅ No API changes untuk existing callers (backward compatible)
- ✅ New batch API adalah addition, bukan replacement

### Test Coverage
- Existing 19/19 tests tetap PASS (zero functional changes)
- New test: `test_batch_insert_10k_edges` (verify atomicity + performance)
- Mutation testing: WAL pragma + batch transaction = 2 new mutants to kill

### Compatibility
- SQLite 3.7.0+ (2010) - semua modern systems support WAL
- No changes to migration 001-004 (backward compatible)
- Optional: migration 005 untuk enable WAL (non-breaking)

## Measurable Metrics Projection

### Benchmark Plan
```rust
// benches/storage_throughput.rs
fn bench_insert_10k_edges(c: &mut Criterion) {
    c.bench_function("single_insert_10k", |b| {
        b.iter(|| { /* 10k individual INSERTs */ })
    });
    
    c.bench_function("batch_insert_10k", |b| {
        b.iter(|| { /* 1 batch INSERT 10k edges */ })
    });
}
```

### Expected Improvements
| Metric | Current | After WAL | After WAL+Batch | Improvement |
|--------|---------|-----------|-----------------|-------------|
| 10k INSERT latency | 2.5s (HDD) | 1.8s | 0.15s | **16.7x** |
| 10k INSERT latency | 0.8s (SSD) | 0.5s | 0.05s | **16x** |
| Write amplification | 2.0x | 1.0x | 1.0x | **2x reduction** |
| Concurrent readers | Blocked | Non-blocking | Non-blocking | **∞ improvement** |
| Memory overhead | - | +4MB (WAL buffer) | +4MB | Acceptable (<500MB cap) |

### Verification Metrics
- Latency p50/p95/p99 untuk 1k/10k/100k INSERTs
- Throughput (edges/second) single vs batch
- Concurrent read/write throughput (readers/sec while writing)
- Memory footprint (RSS) dengan WAL buffer

## Mutation Verification Plan

### Mutant 1: Remove WAL pragma
```sql
-- Expected: test_wal_mode_enabled FAILS
-- Reason: PRAGMA journal_mode returns 'delete' instead of 'wal'
```

### Mutant 2: Remove BEGIN TRANSACTION from batch
```rust
// Expected: test_batch_insert_atomicity FAILS
// Reason: partial INSERT succeeds (non-atomic) → constraint violation
```

### Mutant 3: Change synchronous=NORMAL to FULL
```rust
// Expected: benchmark shows 30% slower write throughput
// Reason: FULL synchronous = extra fsync per transaction
```

## Implementation Plan

### Phase 1: WAL Mode (1-2 days)
1. Add migration 005_wal_mode.sql
2. Test: verify `PRAGMA journal_mode` returns 'wal'
3. Benchmark: measure write throughput before/after
4. Commit: migration + test + benchmark

### Phase 2: Batch INSERT API (2-3 days)
1. Implement `insert_lineage_edges_batch()` in lineage.rs
2. Add test: `test_batch_insert_10k_edges` (atomicity + performance)
3. Add benchmark: single vs batch throughput comparison
4. Commit: API + test + benchmark

### Phase 3: Connection Pool (Optional, 3-5 days)
1. Add r2d2 dependency
2. Implement StoragePool wrapper
3. Migrate existing callers to use pool
4. Benchmark: connection reuse overhead reduction

## Risk Assessment

### Low Risk
- ✅ SQLite WAL is production-proven (used by Firefox, Dropbox)
- ✅ Batch INSERT is standard SQL pattern
- ✅ No schema changes (backward compatible)
- ✅ Existing tests tetap PASS

### Mitigations
- **WAL checkpoint failure**: Set `wal_autocheckpoint=1000` (automatic)
- **Batch INSERT memory**: Process in chunks of 1k (avoid OOM)
- **Concurrent write conflict**: SQLite WAL handles this (queue writes)

## Conclusion

**Zero-Sacrifice Guarantee:**
- ✅ No feature removal
- ✅ No accuracy reduction
- ✅ No compatibility breaking
- ✅ Memory overhead <5MB (well within 500MB cap)

**Expected Impact:**
- 10-16x throughput improvement untuk high-volume workloads
- 2x write amplification reduction
- Non-blocking concurrent reads
- Scalable untuk multi-execution workloads

**Alignment with Project Goals:**
- Pilar 1 (Performa Ekstrem): ✅ 16x throughput, 2x write reduction
- Pilar 2 (Fitur Bermanfaat): ✅ Batch API untuk high-volume use cases
- Hard-cap <500MB: ✅ +4MB WAL buffer (acceptable)

## Verification Checklist

- [ ] Migration 005 WAL mode applied
- [ ] `PRAGMA journal_mode` returns 'wal'
- [ ] Batch INSERT API implemented
- [ ] 10k batch INSERT < 0.2s (SSD)
- [ ] Existing 19/19 tests PASS
- [ ] New mutation tests killed (WAL + batch atomicity)
- [ ] Benchmark results documented
- [ ] Memory overhead < 10MB

---

**Submitted for ADR v2 "ZERO-SACRIFICE PERFORMANCE" sayembara**
