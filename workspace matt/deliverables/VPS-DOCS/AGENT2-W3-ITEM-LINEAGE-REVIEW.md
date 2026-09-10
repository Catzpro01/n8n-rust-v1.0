# AGENT2 STORAGE REVIEW: W3-ITEM-LINEAGE Spec v0.3

**Reviewer:** agent2 (ROLE_STORAGE, DDL Owner)
**Date:** 2026-09-09
**Verdict:** APPROVE dengan 3 catatan storage

## STORAGE-APPROVE POIN

1. Baseline v1.0 Section 7 = SOLID foundation (lineage_edge + erasure_log)
2. T-1 lineage_lookup = ESSENTIAL untuk audit query performance (G-I7)
3. T-2 lineage_ext + CASD dual-hash = EXCELLENT separation of concerns
4. Gates G-I1 sampai G-I9 = COMPREHENSIVE falsifiable tests
5. Integrasi dengan W2-STORAGE-L0 = COMPATIBLE

## CATATAN STORAGE (3 poin)

### STORAGE-1: DDL Finalization Strategy

- Baseline v1.0 Section 7 = WAJIB (non-negotiable)
- T-1 lineage_lookup = RECOMMEND (critical untuk G-I7)
- T-2 lineage_ext = OPTIONAL phase-2 (CASD integration incremental)

Usul: migrasi-004 = baseline + T-1, migrasi-005 = T-2

### STORAGE-2: Amandemen content_hash - Suara saya (b) DUAL HASH

Pilihan: (b) dual hash (plain + salted)

Implementasi: 2 kolom BLOB32 di lineage_edge
- content_hash (plain) = bukti integritas Art.15
- content_hash_salted (pseudonim) = Art.17 compliance

Biaya +32B/baris = ACCEPTABLE untuk dual compliance

### STORAGE-3: Index Strategy

PRIMARY KEY (execution_id, output_ref) = CORRECT
INDEX idx_lookup_query (execution_id, node_id, attempt, output_index, seq_start) = ESSENTIAL untuk G-I7
INDEX idx_ext_purge = OPTIONAL untuk cleanup

## TIDAK ADA KEBERATAN

- Tidak ada konflik dgn W2-STORAGE-L0
- Tidak ada tipe fiktif
- Konsisten dgn kernel types
- RAM budget <500MB achievable

## IMPLEMENTASI STATUS

DONE: DDL migrasi-004, lineage.rs (191 baris), lib.rs updated
IN PROGRESS: SqliteLineageRepository, unit tests
NEXT: Integration tests, review agent10

@agent10 - Review selesai, mohon checklist
@matt @fern - Mohon keputusan timeline
