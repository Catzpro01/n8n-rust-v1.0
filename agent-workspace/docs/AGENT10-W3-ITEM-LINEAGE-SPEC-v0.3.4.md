# AGENT10 — W3-ITEM-LINEAGE: Rencana Implementasi (di atas Norm v1.0)
## [RFC] spec v0.3.3 — jalur konsensus #922C (RFC → ≥2 peer review → [CONSENSUS-REACHED] → kode)

| | |
|---|---|
| **Task** | `W3-ITEM-LINEAGE` (Wave 3, P1, ROLE_COMPLIANCE — IN_PROGRESS agent10; klaim = reservasi anti-tabrakan; NOL kode sebelum konsensus) |
| **Norm (BINDING)** | **`docs/AGENT10-ITEM-LINEAGE-SPEC.md` v1.0** (sha256 `eae5cbc254df569e57d95022f60005aabc58997a6849003d32ca908ccaef7750`). Dokumen ini **TIDAK menggantikan**; ia rencana implementasi yang merujuk v1.0. |
| **v0.3.4 delta** | SINKRON DENGAN MANDAT #1134 @matt + temuan konformansi mandiri saya: (1) kolom digest ter-salt TETAP SATU = `lineage_edge.content_hash` (v1.0 §7) — lineage_ext TIDAK punya kolom digest (kolom `content_digest` di v0.3.3 DIHAPUS; duplikat peran + membuka pintu salt ketiga); (2) kolom plain = `content_proof_plain` (peran `casd_ref` #1081; SATU kolom, Ruling 18 aturan 2); (3) klasifikasi: `data_class TEXT NOT NULL` + `class_rule_id TEXT NOT NULL` + `class_rule_ver INTEGER NOT NULL` (mandat #1134: class_rule_id TEXT, bukan INTEGER); (4) CHECK DUA-ARAH `(PERSONAL AND content_proof_plain IS NULL) OR (NON_PERSONAL AND content_proof_plain IS NOT NULL)` = semantik G-I2 #1116 (catatan: butir saya di #1141 "dilarang CHECK NON_PERSONAL→NOT NULL" SAYA TARIK — mandat #1134 "NULL kecuali terklasifikasi NON-PERSONAL" lebih baru dan lebih kuat); (5) gate §5 di-rename G-I# → L-G# (kills twin-numbering dgn checklist 797b42d5). | (v0.3.3 delta sebelumnya: | Ruling 18 #1116 (matt TARIK Ruling 14): OPSI **(c)** BERLAKU — content_digest BLAKE3-256(salt_exec) WAJIB tiap baris; `content_proof_plain`/casd_ref Satu kolom, SHA-256 polos HANYA baris NON-PERSONAL (personal→NULL); klasifikasi TEREKAM (data_class+class_rule_id+class_rule_ver); ambang 1 KiB DIBUANG (proxy buruk; JSON 5KB bisa memuat NIK). R19: #1082/#1095 agent2 DIREKLASIFIKASI (catatan implementor, BUKAN peer-1); reviewer kepatuhan = @agent10; independent storage = @agent3 (#1117). R20: RFC §2 v1.2.1 (gate store_ref/Arc). (v0.3.2 delta sebelumnya: | SERAP RULING 13-17 #1102: (R14) amandemen content_hash DIPUTUSKAN = salted-only (suara (b) saya DITARIK; alasan diterima — plain hash = brute-force PII; Art.15 dari payload bukan hash); casd_ref NULLABLE + ambang ≥1 KiB; (R15) penamaan spill = `SpillPath` per-execution BUKAN content-addressed (A-21); (R16) rename RecordBody/SpillTag DIADOSI — JANGAN revert; (R17) `SpillIo` WAJIB bawa `kind` (ENOSPC→ResourceExhausted / EACCES-EPERM→Permanent / EINTR-EAGAIN-EMFILE→Transient) — utk RFC §3.4. Peer-1: @agent2 #1095 APPROVE (3 catatan diterima: phasing 004/005; dual-hash=R14; index G-I7). (v0.3.1 delta sebelumnya: | KOREKSI TIPE (temuan #1078 §4): `execution_id` = `ExecutionId` (numeric_id! id.rs:35) — BUKAN `ContentId` (id.rs:41, identitas KONTEN). Keduanya u64 → compiler diam; salah pakai = query erasure salah baris. Update D-L8. (v0.3 delta sebelumnya: | Rekonsiliasi DUA dokumen lineage (sebelumnya dua sumber — penyakit twin-doc, Ruling 3(d) #1000): (1) DDL draft v0.2 saya DITARIK; baseline = v1.0 §7; (2) D-L# di-remap ke §v1.0; (3) amandemen `content_hash` ter-salt (R-1 #940) diusulkan; (4) CASD dual-hash (#994§4.3) = ekstensi terpisah; (5) query per-(node,attempt) butuh tabel lookup (gap di v1.0 — diusulkan di sini). v0.2/v0.2.1 = SUPERSEDED. |
| **Kontrak hulu** | W2-STORAGE-L0 DONE (agent2); W3-TIMELINE-REPLAY #910 (agent1); W2-DETERM-ENFORCE DONE (agent10); tipe = kernel/src NYATA (Ruling 3(c) #1000); §2 BAHASA-BERSAMA root = NON-BINDING (Ruling 3(a)); BAHASA-BERSAMA-v1.1-matt.md = register temuan, bukan kamus (Ruling 3(d)). |

## 1. Tujuan & batas (dari v1.0 §0-§1; tidak diulang di sini)

- **APA:** jalankan desain v1.0 (ItemRef pseudonim BLAKE3-128 + LineageSet repr 0-3 + crypto-shredding Art.17 + Envelope satu-trail-dua-kunci) di W3: DDL final (agent2), index untuk query audit instan, integrasi CASD dual-hash, gates implementasi, kontrak lintas agen.
- **BUKAN:** (a) replay engine (#910); (b) RecordSet DETERM; (c) checksum berkas (SHA-256 G-C7); (d) klaim GDPR-compliant (v1.0 §4.3: dilarang; yang boleh "pseudonymised by construction" + review hukum; gate LIN-3).
- **C-06 tetap:** lineage = metadata non-PII + digest ber-salt; PII tinggal di payload/event_json.

## 2. Pemetaan keputusan D-L# → v1.0 (norm), bukan spek baru

| ID (dari #1016) | Keputusan | Pangkalan di v1.0 |
|---|---|---|
| D-L1 | Row per **rentang** seq (hemat ~10×) | = repr **RANGE** (v1.0 §3.2, LineageSet repr=1) + `inputs_range{node_id,seq_base,count,hash_root}`; UNKNOWN (repr=3) utk GC/gap |
| D-L2 | Sparse-stabil; retry TIDAK menggeser item_seq | v1.0 §2.1 (ItemRef deterministik, tanpa wall-clock) + repr=3 UNKNOWN eksplisit (C-08 kegagalan-1) |
| D-L3 | Salt per eksekusi (`salt_exec`, keyring) | v1.0 §2.1 ItemRef = BLAKE3-128(`salt_exec`‖node_id‖seq‖content_hash), framing u32-BE (C-02) |
| D-L4 | Cross-link dua arah event↔lineage | v1.0 §5 (satu trail, dua kunci: chain_key Envelope + salt_exec) |
| D-L5 | Erasure = keputusan manusia; append-only | v1.0 §4.2 (hancurkan salt_exec + erasure_log{subject_request_id, actor, authority, salt_fingerprint}) — **erasure_log mine v0.2 DITARIK; pakai v1.0 (lebih kuat: fingerprint)** |
| D-L6 | CASD dual-hash (dedup+identity) | **EKSTENSI** (lihat §4) — #980+#994§4.3. Ruling 18: content_digest salted WAJIB; plain = SATU kolom (casd_ref) NON-PERSONAL saja; klasifikasi terekam (3 kolom) |
| D-L7 | Retensi terpisah (lineage panjang / payload pendek) | v1.0 §4.3 butir 2 (payload = retensi terpisah PRD-2 §5.3) + kolom ekstensi `retention_expired_at` |
| D-L8 | `attempt` = otoritas ENGINE; id kernel | v1.0 §7 + **id.rs:35** (`ExecutionId` newtype u64 — kolom `execution_id`); **id.rs:41** (`ContentId` — identitas KONTEN, dipakai utk nama berkas spill per Ruling 10.4 matt, BUKAN sebagai execution_id); node_id → via lookup-table §3 |

**RESOLUSI AMANDEMEN (Ruling 18 #1116 — FINAL, OPSI (c)):** Ruling 14 (opsi (a) salted-only) DITARIK oleh matt; (c) DIADOSI apa adanya (3 aturan):
  1. `content_digest` = BLAKE3-256(salt_exec ‖ kanon(is)) — WAJIB di SETIAP baris (jangkar tautan + bukti tamper selama retensi; hancurkan salt_exec → tautan putus, Art.17 utuh).
  2. `content_proof_plain` = SHA-256 polos — SATU kolom (dipakai bersama pola `casd_ref` per #1081 agent3); HANYA baris terklasifikasi NON-PERSONAL; personal → NULL. TIDAK ada kolom hash polos kedua di mana pun.
  3. Klasifikasi WAJIB TEREKAM: `data_class` + `class_rule_id` + `class_rule_ver` (audit: baris mana yang boleh punya hash polos dan mengapa). NULL-able TANPA default mengisi otomatis (default mengisi = jalur senyap → hash polos atas PII).
  Alasan (b) ditolak: hash polos ke SETIAP baris = membangun vektor brute-force yang R-1 tutup. Art.15 dilayani SEBELUM shredding saat salt MASIH ada → hash ter-salt cukup. Argument tunggal dedup: 69% (#964) hanya untuk konten NON-personal — (c) menyelamatkannya, (a) mematikannya. Ambang 1 KiB (R14) = proxy buruk → dibuang (klasifikasi, bukan ukuran). Ratifikasi @agent5+@fern (crypto/kepatuhan) sebelum merge.

## 3. Skema (BASELINE v1.0 §7 — kutipan normatif; usulan teknis hanya di bawahnya)

```sql
-- DARI v1.0 §7 (kolom wajib; DDL final = @agent2) — TIDAK saya ubah:
lineage_edge(execution_id, output_ref BLOB16, repr 0..3, inputs_exact|inputs_range|inputs_digest BLOB,
             unknown_reason INTEGER, rule_id, rule_ver, content_hash BLOB32, PK(execution_id, output_ref));
erasure_log(id, subject_request_id, execution_id, destroyed_at, actor, authority, salt_fingerprint BLOB);
```

**USULAN TEKNIS (tambahan; kalau ditolak, tanpa ini pun v1.0 tetap jalan):**
```sql
-- (T-1) QUERY AUDIT per (node,attempt,output_index): ItemRef BLOB16 = opaque → v1.0 TIDAK punya
-- jalur lookup node/attempt tanpa scan. Tambahan non-normatif:
CREATE TABLE lineage_lookup (
  execution_id INTEGER NOT NULL, node_id TEXT NOT NULL, attempt INTEGER NOT NULL,
  output_index INTEGER NOT NULL DEFAULT 0, seq_start INTEGER NOT NULL, seq_end INTEGER NOT NULL,
  output_ref BLOB NOT NULL, PRIMARY KEY (execution_id, output_ref));
CREATE INDEX idx_lookup ON lineage_lookup(execution_id, node_id, attempt, output_index, seq_start);
-- bentuk index = turunan migrasi-003 #982 (compat; saya tidak menulis ke tabel #910).
-- (T-2) EKSTENSI CASD + retensi (terpisah, tidak menyentuh v1.0):  -- execution_id bertipe ExecutionId (id.rs:35), bukan ContentId

CREATE TABLE lineage_ext (
  execution_id INTEGER NOT NULL, output_ref BLOB NOT NULL,
  content_proof_plain BLOB,            -- SHA-256 polos kanon(is). SATU kolom (peran casd_ref #1081; Ruling 18 aturan 2).
                                       -- NULL-able TANPA default; NON_PERSONAL = WAJIB terisi; PERSONAL = WAJIB NULL (CHECK bawah).
  data_class TEXT NOT NULL CHECK (data_class IN ('PERSONAL','NON_PERSONAL')),   -- aturan 3: WAJIB TEREKAM
  class_rule_id TEXT NOT NULL,         -- id aturan klasifikasi (TEXT: bisa non-numerik; mandat #1134)
  class_rule_ver INTEGER NOT NULL,     -- versi aturan klasifikasi
  item_count INTEGER NOT NULL DEFAULT 1, estimated_bytes INTEGER,
  retention_expired_at INTEGER, PRIMARY KEY (execution_id, output_ref),
  -- DUA-ARAH (semantik G-I2 #1116): PERSONAL ⇒ NULL; NON_PERSONAL ⇒ terisi. Arah berbahaya diblokir oleh DB.
  CHECK ((data_class = 'PERSONAL' AND content_proof_plain IS NULL)
      OR (data_class = 'NON_PERSONAL' AND content_proof_plain IS NOT NULL)));
CREATE INDEX idx_ext_purge ON lineage_ext(execution_id) WHERE retention_expired_at IS NOT NULL;
CREATE INDEX idx_ext_proof ON lineage_ext(content_proof_plain)
  WHERE data_class = 'NON_PERSONAL' AND content_proof_plain IS NOT NULL;
```

## 4. CASD dual-hash (Ruling 18 #1116 — (c)) — EKSTENSI, di lineage_ext (v0.3.4)
- **Digest ter-salt = SATU, dan ia SUDAH ADA:** `lineage_edge.content_hash` BLOB NOT NULL = BLAKE3-256(salt_exec ‖ kanon(is)) — Ruling 18 aturan 1 dipenuhi oleh v1.0 §7; lineage_ext TIDAK menambah kolom digest (v0.3.3 `content_digest` DIHAPUS: duplikat peran; salt `salt_instance` = salt KETIGA — v1.0 §4.2/§5 hanya chain_key + salt_exec).
- `content_proof_plain` (= peran `casd_ref`, pola #1081 agent3; SATU kolom — Ruling 18 aturan 2) = SHA-256 polos kanon(is) — **HANYA baris NON_PERSONAL**; PERSONAL → NULL. NULL-able TANPA default otomatis. **LINTAS-instance** (pola module_sha256 WCB e2e2587e) — jalur dedup lintas-eksekusi Non-personal (69% #964). Ambang ukuran (1 KiB, R14) = DIBUANG — klasifikasi, bukan panjang byte (JSON 5KB bisa memuat 1 NIK).
- **Klasifikasi TEREKAM** (aturan 3): `data_class` + `class_rule_id (TEXT)` + `class_rule_ver (INTEGER)` — auditor bisa menjelaskan mengapa baris X punya hash polos; tanpa ini, keputusan kepatuhan tidak bisa dipertanggungjawabkan. Klasifikasi diam-diam per-baris saat runtime = DILARANG (unwrap_or/fallback = cacat, bukan fitur).
- CHECK DUA-ARAH (bukan satu-arah — mandat #1134 "NULL kecuali terklasifikasi NON-PERSONAL" + G-I2 #1116): `(PERSONAL AND proof IS NULL) OR (NON_PERSONAL AND proof IS NOT NULL)`.
- Dedup **baris lineage = NON-GOAL** (baris = bukti; jawaban #980§3).
- Kunci pemisahan: `chain_key` (integritas, KMS) ≠ `salt_exec` (penautan/shredding) — v1.0 §4.2/§5; TIDAK ada salt ketiga.

## 5. Gates implementasi L-G# (falsifiable; perilaku v1.0 — di-rename dari G-I# v0.3.0-0.3.3 utk mengakhiri tabrakan penomoran dgn checklist 797b42d5, yg G-I1 = anti-plain-hash, G-I2 = anti-vacuous)
| Gate | Uji (buatan saya, menjalankan KONTRAK v1.0) | Lulus bila |
|---|---|---|
| L-G{n} | Replay deterministik: 2× eksekusi seed sama → ItemRef identik (repr RANGE benar) | ref sama; repr & inputs_range konsisten (v1.0 §2.2) |
| L-G{n} | Tamper 1 byte `output_ref`/`content_hash` → verifier TOLAK (bukan silent) | verdict BROKEN/mismatch terdeteksi |
| L-G{n} | Item di-GC → referensi → status **UNKNOWN** eksplisit (repr=3, unknown_reason=GC) | tidak ada ref "sah" menunjuk item hilang (v1.0 §2.1) |
| L-G{n} | Scan lineage eksekusi ber-PII (email/token) | 0 plaintext; hanya ItemRef+digest ter-salt + tabel lookup metadata |
| L-G{n} | Retry/edge → seq TIDAK bergeser; renumber = event | idempotent index; UNKNOWN bila gap |
| L-G{n} | 10.000 item identik → dedup aktif → 1 blob; digest ter-salt ≠ digest polos luar instance | dedup jalan + R-1 terpenuhi |
| L-G{n} | Query (exec,node,attempt,out_idx) → EXPLAIN QUERY PLAN pakai idx_lookup | 0 table scan (ukur bukan duga) |
| L-G{n} | Shredding: hancurkan salt_exec → refs tak-tertautkan; log TIDAK ditulis ulang; erasure_log berbaris + `salt_fingerprint` ADA | Art.17 + append-only utuh + fingerprint terbukti (v1.0 §4.2/§7) |
| L-G{n} | Envelope tereksekusi tetap `VERIFIED_ANCHORED` SETELAH shredding (chain_key ≠ salt_exec) | v1.0 §4.2 tabel: verifikasi integritas TETAP ✅ |

## 6. Integrasi & kontrak lintas agen
- **@agent2 (implementor):** migrasi 004 = v1.0 §7 baseline + T-1 (lookup) + T-2 (ext) sesuai v0.3.4 §3/§4 — veredik konformansi saya: 5 temuan terkirim [CONFORMANCE] (content_digest duplikat+dibuang, naming #1134, komentar Ruling, fallback klasifikasi, SHAs dokumen).
- **@agent1 (peer-2):** sinkron #910 (event ↔ lineage cross-link; `attempt` = otoritas ENGINE #951; index shape #982). Review v0.2 saya Anda TUNDA karena dua-dokumen — rekonsiliasi v0.3 ini MENUTUP alasan itu.
- **@agent3:** CASD = T-2 (content_proof_plain, peran casd_ref #1081) + klasifikasi; dedup baris = non-goal (§4).
- **@agent4:** Rosetta → rule_id/rule_ver mapping (v1.0 §2.1) — tidak mengubah skema manifest.
- **@agent6:** content_proof_plain pola NC-1 e2e2587e (identity lintas-instance).
- **@matt/@fern:** endorse → [CONSENSUS-REACHED] → kode di shadow → patch → merge matt.

## 7. Definisi DONE (disiplin #960/#994§1)
Kode shadow + patch → review + konsensus → merge @matt. Laporan wajib: (a) path, (b) jumlah #[test] lulus, (c) commit hash. Deliverable_type = `code-prototype` hingga merge kanonik.

## 8. Batas jujur (tetap, v1.0 §2.3/§4.3 + catatan)
Pseudonimisasi ≠ anonimisasi (LIN-3); analisis frekuensi digest = sisa risiko tercatat (bounded, bukan klaim nol); 16-byte ItemRef cukup utk tautan internal, 32-byte content_hash = jangkar bukti (v1.0 §2.3); amandemen salt-content_hash menambah +32B/baris bila opsi (b).

— agent10 (ROLE_COMPLIANCE / Plt. ROLE_SECURITY_QA), sesi lanjutan
