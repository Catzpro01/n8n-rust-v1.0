# AGENT10 — W3-ITEM-LINEAGE: Rencana Implementasi (di atas Norm v1.0)
## [RFC] spec v0.3.1 — jalur konsensus #922C (RFC → ≥2 peer review → [CONSENSUS-REACHED] → kode)

| | |
|---|---|
| **Task** | `W3-ITEM-LINEAGE` (Wave 3, P1, ROLE_COMPLIANCE — IN_PROGRESS agent10; klaim = reservasi anti-tabrakan; NOL kode sebelum konsensus) |
| **Norm (BINDING)** | **`docs/AGENT10-ITEM-LINEAGE-SPEC.md` v1.0** (sha256 `eae5cbc254df569e57d95022f60005aabc58997a6849003d32ca908ccaef7750`). Dokumen ini **TIDAK menggantikan**; ia rencana implementasi yang merujuk v1.0. |
| **v0.3.1 delta** | KOREKSI TIPE (temuan #1078 §4): `execution_id` = `ExecutionId` (numeric_id! id.rs:35) — BUKAN `ContentId` (id.rs:41, identitas KONTEN). Keduanya u64 → compiler diam; salah pakai = query erasure salah baris. Update D-L8. (v0.3 delta sebelumnya: | Rekonsiliasi DUA dokumen lineage (sebelumnya dua sumber — penyakit twin-doc, Ruling 3(d) #1000): (1) DDL draft v0.2 saya DITARIK; baseline = v1.0 §7; (2) D-L# di-remap ke §v1.0; (3) amandemen `content_hash` ter-salt (R-1 #940) diusulkan; (4) CASD dual-hash (#994§4.3) = ekstensi terpisah; (5) query per-(node,attempt) butuh tabel lookup (gap di v1.0 — diusulkan di sini). v0.2/v0.2.1 = SUPERSEDED. |
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
| D-L6 | CASD dual-hash (dedup+identity) | **EKSTENSI** (lihat §4) — bukan di v1.0; #980+#994§4.3 |
| D-L7 | Retensi terpisah (lineage panjang / payload pendek) | v1.0 §4.3 butir 2 (payload = retensi terpisah PRD-2 §5.3) + kolom ekstensi `retention_expired_at` |
| D-L8 | `attempt` = otoritas ENGINE; id kernel | v1.0 §7 + **id.rs:35** (`ExecutionId` newtype u64 — kolom `execution_id`); **id.rs:41** (`ContentId` — identitas KONTEN, dipakai utk nama berkas spill per Ruling 10.4 matt, BUKAN sebagai execution_id); node_id → via lookup-table §3 |

**AMANDEMEN YANG DIUSULKAN (1 butir, wajib disetujui peer sebelum kode):** v1.0 §7 `content_hash BLOB 32B = BLAKE3-256` atas isi **polos**. Konflik dengan R-1 (#940): digest isi ter-salt (pre-image PII entropi rendah). Usul: `content_hash = BLAKE3-256(salt_exec ‖ kanon(is))` — efek: setelah shredding, jangkar isi ikut putus (KONSISTEN dengan v1.0 §4.2: tautan mati, bentuk graf tetap ada; Envelope tetap utuh via chain_key). Efek samping: bukti Art.15 atas isi ikut hilang pasca-shred → **keputusan ini bukan saya; apakah boleh trade-off?** Alternatif (b): simpan KEDUA (plain `content_hash` utk bukti verifikatif + salted `content_digest` utk tautan) — biaya +32B/baris. Suara saya: (b) — bukti integritas dan pseudonim adalah dua kebutuhan berbeda (v1.0 §2.3 sendiri membedakan ancaman/hash), tapi keputusan di peer + @matt/@fern.

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
  content_digest TEXT,        -- blake3_hex(salt_instance ‖ kanon(is))  (D-L6; R-1)
  casd_ref TEXT,              -- rujukan CASD (opaque) + SHA-256 identity HANYA utk konten NON-PII (WCB NC-1 #972);
                              -- PII → NULL (R-1; jangan hash polos konten PII)
  item_count INTEGER NOT NULL DEFAULT 1, estimated_bytes INTEGER,
  retention_expired_at INTEGER, PRIMARY KEY (execution_id, output_ref));
CREATE INDEX idx_ext_purge ON lineage_ext(execution_id) WHERE retention_expired_at IS NOT NULL;
```

## 4. CASD dual-hash (#994§4.3, #980) — EKSTENSI, di lineage_ext
- `content_digest` = BLAKE3 ter-salt instance (dedup dalam-instance; 69%-claime #964 hanya berlaku setelah gate G-I6 lokal) — dedup **baris lineage = NON-GOAL** (baris = bukti; jawaban #980§3).
- `casd_ref` = rujukan CASD + SHA-256 identity untuk modul/konten non-PII (pola module_sha256 e2e2587e). Konten ber-PII → NULL (amandemen §2 + R-1).
- Kunci pemisahan tetap: `chain_key` (integritas, KMS) ≠ `salt_exec` (penautan) ≠ `salt_instance` (dedup) — v1.0 §4.2/§5.

## 5. Gates implementasi G-I# (falsifiable; perilaku v1.0, di shadow pra-merge)
| Gate | Uji (buatan saya, menjalankan KONTRAK v1.0) | Lulus bila |
|---|---|---|
| G-I1 | Replay deterministik: 2× eksekusi seed sama → ItemRef identik (repr RANGE benar) | ref sama; repr & inputs_range konsisten (v1.0 §2.2) |
| G-I2 | Tamper 1 byte `output_ref`/`content_hash` → verifier TOLAK (bukan silent) | verdict BROKEN/mismatch terdeteksi |
| G-I3 | Item di-GC → referensi → status **UNKNOWN** eksplisit (repr=3, unknown_reason=GC) | tidak ada ref "sah" menunjuk item hilang (v1.0 §2.1) |
| G-I4 | Scan lineage eksekusi ber-PII (email/token) | 0 plaintext; hanya ItemRef+digest ter-salt + tabel lookup metadata |
| G-I5 | Retry/edge → seq TIDAK bergeser; renumber = event | idempotent index; UNKNOWN bila gap |
| G-I6 | 10.000 item identik → dedup aktif → 1 blob; digest ter-salt ≠ digest polos luar instance | dedup jalan + R-1 terpenuhi |
| G-I7 | Query (exec,node,attempt,out_idx) → EXPLAIN QUERY PLAN pakai idx_lookup | 0 table scan (ukur bukan duga) |
| G-I8 | Shredding: hancurkan salt_exec → refs tak-tertautkan; log TIDAK ditulis ulang; erasure_log berbaris + `salt_fingerprint` ADA | Art.17 + append-only utuh + fingerprint terbukti (v1.0 §4.2/§7) |
| G-I9 | Envelope tereksekusi tetap `VERIFIED_ANCHORED` SETELAH shredding (chain_key ≠ salt_exec) | v1.0 §4.2 tabel: verifikasi integritas TETAP ✅ |

## 6. Integrasi & kontrak lintas agen
- **@agent2 (peer-1, DDL owner):** finalisasi migrasi 004 = v1.0 §7 baseline + T-1 (lookup) + T-2 (ext) — ya/delta? (Keputusan terpenting: amandemen content_hash §2 — (a) salted-only atau (b) dual; suara saya (b).)
- **@agent1 (peer-2):** sinkron #910 (event ↔ lineage cross-link; `attempt` = otoritas ENGINE #951; index shape #982). Review v0.2 saya Anda TUNDA karena dua-dokumen — rekonsiliasi v0.3 ini MENUTUP alasan itu.
- **@agent3:** CASD = T-2 (content_digest+casd_ref); dedup baris = non-goal (§4).
- **@agent4:** Rosetta → rule_id/rule_ver mapping (v1.0 §2.1) — tidak mengubah skema manifest.
- **@agent6:** casd_ref pola NC-1 e2e2587e.
- **@matt/@fern:** endorse → [CONSENSUS-REACHED] → kode di shadow → patch → merge matt.

## 7. Definisi DONE (disiplin #960/#994§1)
Kode shadow + patch → review + konsensus → merge @matt. Laporan wajib: (a) path, (b) jumlah #[test] lulus, (c) commit hash. Deliverable_type = `code-prototype` hingga merge kanonik.

## 8. Batas jujur (tetap, v1.0 §2.3/§4.3 + catatan)
Pseudonimisasi ≠ anonimisasi (LIN-3); analisis frekuensi digest = sisa risiko tercatat (bounded, bukan klaim nol); 16-byte ItemRef cukup utk tautan internal, 32-byte content_hash = jangkar bukti (v1.0 §2.3); amandemen salt-content_hash menambah +32B/baris bila opsi (b).

— agent10 (ROLE_COMPLIANCE / Plt. ROLE_SECURITY_QA), sesi lanjutan
