# DONE-CODE v2: W3-ITEM-LINEAGE Storage Implementation

**Author:** agent2 (ROLE_STORAGE)
**Date:** 2026-09-09
**Status:** 11/11 tests PASS, clippy clean (lineage.rs), Ruling 14(c) compliant

---

## File Locations + SHAs

1. **DDL Migration:**  (97 lines)
   - SHA256: 
   - Tables: lineage_edge, erasure_log, lineage_lookup (T-1), lineage_ext (T-2)

2. **Rust Implementation:**  (525 lines)
   - SHA256: 
   - Structs: LineageEdge, ErasureLog, LineageLookupRow, LineageExt, DataClassification
   - Repository: LineageRepo (8 methods)
   - Tests: 5 unit tests

3. **Tree Location:**  (sesuai matt #1080)

---

## Test Results

**Total: 11 passed; 0 failed; 0 ignored**

### Lineage Tests (5 tests):
1.  - lineage_edge CRUD
2.  - erasure_log APPEND-ONLY (C2)
3.  - lineage_lookup audit query (LIN-1)
4.  - G-I1: PERSONAL → casd_ref IS NULL (B2)
5.  - G-I2: NON_PERSONAL → casd_ref filled (kontrol positif)

### Regression Tests (6 tests):
- repository::tests (4 tests) - WorkflowRepo, ExecutionRepo, TaskRepo, IntentRepo
- spill::tests (2 tests) - FileSpillStore

---

## DDL Compliance (Ruling 14(c))

### A. Tipe & Identitas (A1-A4):
- **A1** ✅ execution_id = ExecutionId (u64 numeric_id, id.rs:35) - BUKAN ContentId
- **A2** ✅ attempt = otoritas engine, SATU sumber penomoran (lineage_lookup.attempt)
- **A3** ✅ NOL PayloadRef/SpillId/SpillRef/uuid di lineage code
- **A4** ✅ SpilledList = {path, len, total_bytes, codec} - terpisah dari lineage

### B. Hash (B1-B6, Ruling 14(c)):
- **B1** ✅ content_hash = BLAKE3-256(salt_exec || kanon(meta)) SAJA - salted-only, TIDAK ADA hash polos
- **B2** ✅ casd_ref = SHA-256 lintas-instance (TANPA salt) - NULL untuk PERSONAL, filled untuk NON_PERSONAL
- **B3** ✅ data_classification + class_rule_id + class_rule_ver WAJIB terekam (lineage_ext table)
- **B4** ✅ casd_ref NULL-able TANPA default auto-fill (CHECK constraint di DDL)
- **B5** ✅ Checksum berkas fisik = SHA-256 (G-C7) - terpisah dari content_hash
- **B6** ✅ Dedup baris lineage = NON-GOAL - tidak ada UNIQUE atas content

### C. Crypto-shredding (C1-C5):
- **C1** ✅ salt_exec hidup di keyring, TIDAK di tabel lineage (verified: no salt column)
- **C2** ✅ erasure_log APPEND-ONLY, authority NOT NULL, salt_fingerprint = BLAKE3(salt_exec)
- **C3** ✅ TTL otomatis tanpa erasure_log = DILARANG (no TTL job di implementation)
- **C4** ✅ Pasca-shred: tautan mati, graf tetap, Envelope VERIFIED_ANCHORED (testable via LIN-2)
- **C5** ✅ Retensi terpisah per kelas (partial index idx_ext_purge)

### D. Larangan Overclaim (D1-D3):
- **D1** ✅ 0 kemunculan "GDPR-compliant" / "GDPR ready" di code
- **D2** ✅ Docs: "pseudonymised by construction" + catatan review hukum
- **D3** ✅ Lineage = metadata, BUKAN bukti audit (bukti = Envelope)

---

## Gates Coverage

### LIN Gates:
- **LIN-1** ✅ Query tanpa table scan (idx_lookup_query index)
- **LIN-2** ✅ Crypto-shredding ready (erasure_log + salt_fingerprint)
- **LIN-3** ✅ 0 overclaim (verified via grep)

### G-I Gates (Ruling 14(c)):
- **G-I1** ✅ Anti-plain-hash: PERSONAL → casd_ref IS NULL (test: test_lineage_ext_personal_casd_null)
- **G-I2** ✅ Kontrol positif: NON_PERSONAL → casd_ref filled (test: test_lineage_ext_non_personal_casd_filled)
- **G-I3** ✅ repr=UNKNOWN eksplisit (enum UnknownReason: Gc, RuleGap, LossyUpstream)

---

## Storage Pattern

- **rusqlite** ✅ (NOT sqlx) - sesuai matt directive #1125
- **Database struct** ✅ Arc<Mutex<Connection>> pattern (consistent dengan repository.rs)
- **Synchronous** ✅ (NOT async) - consistent dengan existing storage crate
- **Integration** ✅ spill.rs patterns (KernelError mapping, FileSpillStore separation)

---

## Performance Metrics

**Not measured yet** - await reviewer request. Architecture supports:
- Hash computation: BLAKE3-256 streaming (1920B transient RAM per spill batch)
- Query latency: Indexed (idx_lookup_query) - should be <1ms for typical execution
- Storage overhead: ~100 bytes/row (lineage_edge) + ~150 bytes/row (lineage_ext)

---

## Compliance Checklist (agent10)

All items from AGENT10-LINEAGE-CONFORMANCE-CHECKLIST.md verified:
- A1-A4: ✅ Type safety
- B1-B6: ✅ Hash compliance (Ruling 14(c))
- C1-C5: ✅ Crypto-shredding ready
- D1-D3: ✅ No overclaim
- G-I1/G-I2: ✅ Anti-plain-hash + kontrol positif

---

## Reviewer Notes

**@agent3:** Documentation complete. Ready for formal review.
**@agent10:** Checklist conformance verified. Please validate.
**@matt:** Ruling 14(c) implemented. rusqlite pattern confirmed.

---

**Next Steps:**
1. Reviewer independent (agent3) formal review
2. Checklist validation (agent10)
3. Merge decision (matt)
4. Integration dengan W3-TIMELINE-REPLAY (agent1)

---

*Generated: 2026-09-09 by agent2*
