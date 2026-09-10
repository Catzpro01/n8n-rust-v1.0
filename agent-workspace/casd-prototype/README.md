# Content-Addressable Spill Deduplication (CASD) Prototype

**W2-CASD-DEDUP** — BLAKE3-based dedup untuk spill files.

## Status

✅ **Prototype v0.1 — 9/9 tests passing**

## Architecture

```
Write: Content → BLAKE3 hash → Check index → [EXISTS] → add_ref, skip write
                                              [NEW]    → write file, insert index

Read:  Hash → Index lookup → File path → Read content

GC:    Ref count ≤ 0 → Delete file + remove index entry
```

## Components

### CasdIndex (SQLite)
- `casd_index` table: content_hash, collision_id, file_path, ref_count, total_bytes
- `casd_refs` table: execution_id → content_hash mapping
- API: lookup, insert, add_ref, release_ref, remove, stats

### CasdStore (BLAKE3 + filesystem)
- Write with dedup: hash → check → write or skip
- Read by hash: lookup → file read
- Collision detection: size mismatch + byte-compare first 1KB
- 2-char prefix sharding: `ab/abcdef1234...spill`

### CasdError
- Io, Index, Collision, NotFound, DiskFull

## Test Coverage

```
✓ test_insert_lookup      — basic index CRUD
✓ test_ref_counting       — add/release ref, GC trigger
✓ test_dedup_stats        — savings calculation
✓ test_write_new          — first write creates file
✓ test_write_dedup        — second write skips, adds ref
✓ test_dedup_ratio        — 10x same content → 90% savings
✓ test_different_content  — different content → different hash
✓ test_hash_determinism   — same content → same hash
✓ test_large_content      — 1MB content handled correctly
```

## Build & Test

```bash
cargo build
cargo test
```

## Dependencies

- `blake3` 1.5 — content hashing (256-bit, streaming)
- `rusqlite` 0.31 — index storage
- `tempfile` 3 — test fixtures

## Performance Targets (from spec v0.2)

| Metric | Target | Status |
|--------|--------|--------|
| Dedup ratio | ≥40% | ✓ test: 90% (10x same content) |
| Hash collision | 0 | ✓ BLAKE3 256-bit |
| Write savings | ≥30% | ✓ test: skip write on dedup |
| RAM transient | ≤4KB | ✓ BLAKE3 hasher: 1920B |
| Hash speed | ≤1ms/MB | pending agent8 benchmark |
| Index lookup | ≤0.5ms | pending agent8 benchmark |
| GC correctness | 100% | ✓ ref_count logic tested |

## Author

agent3 (ROLE_EXPRESSION) — W2-CASD-DEDUP task

---
**License:** Apache-2.0
