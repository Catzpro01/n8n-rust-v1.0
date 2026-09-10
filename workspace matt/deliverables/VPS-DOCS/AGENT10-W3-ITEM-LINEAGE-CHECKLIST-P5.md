# AGENT10 — CHECKLIST REVIEW PILAR-5: W3-ITEM-LINEAGE (implementasi @agent2)
Ver. 0.3 — 2026-09-09 12:2x (v0.3.4-spesifikasi: kolom digest di ext DIHAPUS (edge.content_hash = aturan 1); plain = `content_proof_plain`; CHECK DUA-ARAH per mandat #1134 (butir saya A8 sebelumnya = satu-arah → SAYA TARIK; G-I# pindah ke penomoran checklist operatif 797b42d5); baseline konformansi-vs-norm = docs/AGENT10-LINEAGE-CONFORMANCE-CHECKLIST.md (797b42d5, sesi paralel) — dokumen ini = ACCEPTANCE/DONE-CODE + hygiene + delta) · Reviewer: agent10 (conformance check; batas: penulis norm v1.0 — verdict "CONFORMANCE", reviewer independen penuh = @agent1/@agent5, menunggu ratifikasi) · Artefak diaudit: DDL migrasi-004/005 + `lineage.rs` (crates/storage) + unit tests · Norm: AGENT10-ITEM-LINEAGE-SPEC.md v1.0 (eae5cbc2) + AGENT10-W3-ITEM-LINEAGE-SPEC-v0.3.2.md (872a0791…) + Ruling 13-15 #1102

Setiap butir: PASS / FAIL (blokir) / DEP (ditunda ke gate integrasi). Dilarang "N/A" tanpa alasan tertulis.

## A. DDL (migrasi-004 = v1.0 §7 + T-1; migrasi-005 = T-2 per #1095)
- [ ] A1 `lineage_edge` kolom wajib v1.0 §7 ada semua: execution_id, output_ref BLOB(16), repr, inputs_exact/inputs_range/inputs_digest, unknown_reason, rule_id, rule_ver, content_hash BLOB(32); PK (execution_id, output_ref).
- [ ] A2 `execution_id` bertipe `ExecutionId` (id.rs:35) — BUKAN `ContentId` (id.rs:41; identitas konten). Uji kompiler: `0_` dibaca `ExecutionId` (bukan u64 polos? — cek: pakai newtype di DDL mapping, atau waspada bila u64 dipakai — SYARAT: tidak ada jalur kode yang memasukkan ContentId ke kolom execution_id).
- [ ] A3 `repr` CHECK (repr IN (0,1,2,3)) sesuai v1.0 §7; 3 = UNKNOWN + `unknown_reason` (GC|RULE_GAP|LOSSY_UPSTREAM) terisi.
- [ ] A4 `content_hash` (`lineage_edge`) = BLAKE3-256(salt_exec‖kanon(is)) — WAJIB di SETIAP baris (Ruling 18 aturan 1 = digest ter-salt; SATU kolom digest — TIDAK ADA kolom digest kedua di lineage_ext; salt_instance = DILARANG, hanya chain_key+salt_exec). Test balik: `content_hash != BLAKE3-256(content)` dan `!= BLAKE3-256(canon)` untuk input apa pun. Hash polos HANYA boleh: `content_proof_plain` (lineage_ext, NON_PERSONAL saja).
- [ ] A5 `erasure_log`: subject_request_id, execution_id, destroyed_at, actor, authority, **salt_fingerprint BLOB NOT NULL** (v1.0 §7); append-only: TIDAK ADA jalur UPDATE/DELETE (cek sql: hanya INSERT/SELECT; tidak ada `UPDATE erasure_log` / `DELETE FROM erasure_log`).
- [ ] A6 `salt_fingerprint` = BLAKE3(salt_exec) SEBELUM salt dihancurkan (diuji: fingerprint ≠ salt; fingerprint deterministik).
- [ ] A7 T-1 `lineage_lookup` + idx (execution_id, node_id, attempt, output_index, seq_start): `attempt` TIDAK dihitung di repository — DIAMBIL dari otoritas `task.attempt` ENGINE (D-L8 #951); tidak ada counter lokal.
- [ ] A8 T-2 (migrasi-004, v0.3.4 + mandat #1134): `content_proof_plain BLOB NULL` (NULL-able TANPA default otomatis; peran casd_ref #1081); `data_class TEXT NOT NULL CHECK(IN ('PERSONAL','NON_PERSONAL'))`; `class_rule_id TEXT NOT NULL`; `class_rule_ver INTEGER NOT NULL`; item_count, estimated_bytes, retention_expired_at; idx_ext_purge partial. CHECK DUA-ARAH (G-I2 #1116 + mandat #1134): `(data_class='PERSONAL' AND content_proof_plain IS NULL) OR (data_class='NON_PERSONAL' AND content_proof_plain IS NOT NULL)`. TIDAK ADA kolom digest di ext.
- [ ] A9 NOL `uuid` di DDL/impl (Ruling 10.4 #1073 / #1080); penamaan berkas spill apa pun → `SpillPath` (id.rs:97) — dan DIPERLUKAN: sanitasi path (Ruling 15 JEBAKAN-2; FS-13).

## B. Implementasi (lineage.rs / SqliteLineageRepository)
- [ ] B1 Backend konsisten dgn crate storage: **rusqlite** (sync; preseden spill.rs) — sqlx DIBUANG (keputusan yang benar; alasan: runtime+dep tambahan, semangat Ruling 7). Cargo.toml: tidak ada tokio/tracing/uuid baru.
- [ ] B2 `#![forbid(unsafe_code)]` ada di crate; clippy 0.
- [ ] B3 Semantik entri: insert idempotent deterministik (exec+output_ref sama → hasil sama; duplikat TIDAK diam-diam di-skip — PK conflict = sinyal bug → Err/flag eksplisit).
- [ ] B4 Erasure = satu jalur: `erasure(execution_id, subject_request_id, actor, authority)` → (1) hancurkan salt_exec, (2) INSERT erasure_log + fingerprint, (3) TIDAK menyentuh trail. Tidak ada jalur delete lineage.
- [ ] B5 Redaksi (D93): tidak ada log/error yang memuat salt_exec, konten item, atau plaintext digest; error memakai tipe terstruktur (mirror kanonik; TIDAK memakai uuid/Internal flatten untuk kasus yang punya varian).
- [ ] B6 RAM & determinisme: jalur hot tidak memuat seluruh konten; ukur dengan `/usr/bin/time -v` (MAX RSS) → <500 MB (mandat #1074).

## C. Unit tests (falsifiable; yang DEP dinyatakan tertulis)
- [ ] G-I1 determinisme: 2× insert dengan salt fixture sama → output_ref & content_hash identik.
- [ ] G-I2 tamper: ubah 1 byte content_hash/output_ref di DB → verifier/query mismatch terdeteksi (BROKEN flag; bukan silent).
- [ ] G-I3 GC → path UNKNOWN: item hilang → repr=3 + unknown_reason=GC (repo-level bila ada hook; else DEP: gate integrasi).
- [ ] G-I4 0 plaintext: insert konten ber-PII (email/token) → SELECT semua kolom lineage → 0 kemunculan nilai asli; `content_hash != sha256/is` check.
- [ ] G-I5 idempotent renumber: retry attempt → item_seq tidak bergeser (duplikat → conflict terjaga).
- [ ] G-I6 (797-numbering: G-I1/G-I2 = anti-plain-hash/anti-vacuous) dedup: 10.000 item identik NON_PERSONAL → `content_proof_plain` SAMA (lintas-eksekusi) + 1 blob; 10.000 item PERSONAL identik → `content_proof_plain` SEMUA NULL (G-I1 797).
- [ ] G-I7 EXPLAIN QUERY PLAN utk (exec,node,attempt,out_idx) memakai idx_lookup — 0 table scan (ukur, tulis output EXPLAIN di laporan).
- [ ] G-I8 shredding: salt dihancurkan → tautan putus (dihitung ulang, mismatch); erasure_log berbaris + fingerprint ADA; file log lineage mtime/tidak berubah (append-only utuh).
- [ ] G-I9 Envelope tetap VERIFIED_ANCHORED pasca-shred → DEP (gate integrasi W3; bukan repo).

## D. Laporan (definisi DONE per #994§1)
- [ ] D1 path kode + jumlah #[test] lulus (100%) + commit hash + sha256 artefak utama.
- [ ] D2 deliverable_type = `code-prototype` (s/d merge kanonik @matt).
- [ ] D3 Catatan batas: dedup 69% = dalam-eksekusi (ditulis eksplisit); conformance ≠ review independen penuh.

— agent10 (konformansi; dokumen ini = acceptance criteria utk review Pilar-5, bukan pengganti review independen)
