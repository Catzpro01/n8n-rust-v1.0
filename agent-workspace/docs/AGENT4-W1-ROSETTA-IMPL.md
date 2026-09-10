# AGENT4-W1-ROSETTA-IMPL — Rencana Implementasi Rosetta Compiler (crates/rosetta/) — shadow v0.1

**Pemilik:** agent4 (ROLE_SCHEMA) · **2026-09-09** · **Status: SHADOW-PLAN** (nol kode — menunggu entri queue + keputusan #1005 butir 2; SOP #864 Pilar 1 & 4)
**Dasar:** DIRECTIVE #994 §4.1 (implementasi Rosetta di crates/rosetta/), W1-ROSETTA-PARSE-PLAN (DONE-DOC), AGENT4-WORKFLOW-ROSETTA v1.1 (19fad5ed), AGENT4_SCHEMA_COMPILER_SPEC v0.2, AGENT4-NORMALIZER-DESIGN (N-01..N-22), AGENT4-NODE-ALIAS-DEPRECATION (V-1..V-3), WCB-NODE-MANIFEST v0.3 (e2e2587e, CONSENSUS-REACHED #994 §3), PRD-3-CANONICAL L1/L3, BAHASA-BERSAMA (ground truth tipe kernel #924/#929).

## 1. Tujuan & batas

Kode nyata utk compiler Rosetta: parse workflow JSON (era n8n mana pun) → grafik kanon + migrasi rule berversi → receipt + normalisasi — mencapai paritas skema 694 node (K-3 Opsi A: Rosetta = jalur resmi). DONE-CODE criteria (#994 §1): kode di crates/rosetta/ + suite #[test] hijau + commit hash.

**Batas:** TIDAK menyentuh crate lain (kernel/executor/workflow milik agent1/agent4-doc; storage agent2). Rosetta = konsumen `ParameterSchema`/`NodeDescriptor` kernel (tipe NYATA), bukan pembuat tipe baru. Parser TIDAK mengeksekusi workflow — hanya struktur + migrasi (execution = agent1).

## 2. Keputusan terbuka sblm kode (utk @matt/@fern/@agent1)

1. **Lokasi & kepemilikan:** `crates/rosetta/` baru di kernel-asli-d3bcff0 — pemilik agent4 (usul #1005-2c; konsisten crates/workflow = agent4; rosetta = compiler skema, bukan runtime).
2. **Cargo deps:** kernel sengaja 4-dep (serde, serde_json, async-trait, thiserror — gate #924). Rosetta butuh parsing JSON → serde_json; usul: ikut disiplin sama (serde/serde_json/thiserror saja; tanpa tokio — compiler = pure function, tidak I/O async). Bila butuh uuid → TIDAK (ContentId u64; konsisten #924/#929).
3. **Posisi di workspace:** anggota workspace kanonik vs shadow crate dulu? Usul: shadow di /home/agent4/w1-rosetta-impl/ sampai 100% test hijau + review agent1/agent5/agent6 → merge via matt (Pilar 4 #865; pola agent6 Slice-1, agent10 determ).
4. **Status W4 & cap:** lihat #1005-2b (keputusan fern).

## 3. Struktur crate (mengikuti plan W1 §3 — dipertegas utk kode)

```
crates/rosetta/
  Cargo.toml            # serde, serde_json, thiserror (usul)
  src/lib.rs            # API publik: parse_workflow(), migrate(), receipt(); #![forbid(unsafe_code)]
  src/parse/            # workflow JSON -> grafik kanon (node, connection, posisi utk stickyNote)
  src/registry/         # aturan migrasi berversi: Rule {rule_version, applies(), apply()}
  src/migrate/          # penerap aturan -> workflow bersih + Receipt {rules[], assumptions[], deviations[]}
  src/binding/          # note-binding geometri (cover-test deterministik) + override manifest
  src/kanon/            # normalisasi utk diff: N-01..N-22 (fungsi bernama + daftar field berversi)
  src/opaque/           # node tak dikenal -> OpaqueNode + receipt (TIDAK gagalkan impor)
  src/manifest_wcb/     # resolve node komunitas WCB via AGENT6-WCB-NODE-MANIFEST (module_sha256 = identitas tipe)
  src/receipt.rs        # skema receipt (JSON kecil; deviation_id dari DEVIATION-CATALOG — no local id)
tests/
  fixtures/             # korpus 171 (salinan utk diff; hash asli diverifikasi tdk termutasi)
  diffgate/             # asli-vs-migrasi, banding kanon (mode deterministik; seed sama)
  wcb_fixtures/         # manifest node WCB contoh (MV-2: params ↔ ParameterSchema)
```

## 4. Milestone implementasi (urutan; tiap milestone = test hijau + heartbeat)

| M | Isi | Gate keluar |
|---|---|---|
| M1 | crate skeleton + parser JSON → grafik kanon + OpaqueNode | parse 171/171 tanpa gagal; 0 file termutasi (hash utuh) = R-1 |
| M2 | registry v1: rule cron-v1→scheduleTrigger, function/functionItem→Code, alias deprecated; rule_version + fixture | R-4: tiap rule punya fixture korpus + diff-test lolos |
| M3 | migrate + receipt {rules, assumptions, deviations: deviation_id eksternal} | R-2 (opaque → receipt 100%), R-5 (deviation_id DEVIATION-CATALOG) |
| M4 | binding stickyNote: cover-test deterministik (2× hitung identik) + override | R-3 (HUB-7) |
| M5 | kanon N-rules + diffgate harness (asli vs migrasi; matriks L3) | N-01..N-22 teruji; seed sama = hasil sama |
| M6 | manifest_wcb: resolve node komunitas + type-check params | MV-2 (params ↔ ParameterSchema) + integrasi receipt |

## 5. Acceptance (dari plan W1 §4 — R-1..R-5, diperluas utk kode)

- R-1: parse 171/171 file korpus tanpa gagal; 0 file termutasi (hash asli utuh) — terukur sha256 per file sblm/sesudah.
- R-2: tipe tak dikenal = opaque + receipt; 100% kasus tidak hentikan impor.
- R-3: stickyNote binding deterministik (2× hitung = identik); coverage per template tercatat (HUB-7).
- R-4: tiap rule v1 punya rule_version + fixture korpus + diff-test lolos (cron: tpl-159,175,199,1471,1771,1841; function: korpus 12+9).
- R-5: receipt memakai deviation_id dari DEVIATION-CATALOG (agent5); nol ID lokal.
- R-6 (baru): 694-node catalog: tiap tipe dikenal → mapping ke ParameterSchema kernel; tak dikenal → opaque + tercatat di coverage (penerimaan katalog = OPEN #418 — jangan tulis "selesai").
- R-7 (baru, WCB): node komunitas via manifest WCB resolve + type-check (MV-2) — konsisten kontrak v0.3.
- R-8 (baru): determinisme — 2× run migrate input sama = receipt & output byte-identik.

## 6. Kait lintas-agent (review + verifikator)

- agent1 (kernel): interface NodeDescriptor/ParameterSchema — review interface M1; penerimaan di engine.
- agent5 (QA): DEVIATION-CATALOG id; diff-test determinisme = verifikator gate L1 (recommender ≠ approver — saya tak sahkan gate artefak sendiri, #635).
- agent2 (storage): penyimpanan workflow + receipt (identitas = hash ContentId).
- agent6 (WASM): hook manifest WCB (M6) — konsumen kontrak v0.3.
- agent9/agent7: impor template Hub = jalur sama; resource receipt (n8n://templates/{id}/receipt).
- matt: merge ke kanonik (Pilar 4).

## 7. Risiko & mitigasi

- Corpus 198 vs 171: gate L1 wajib sebut himpunan (kanonik §5 L1) — fixtures = 171 datar + subset rekursif terpisah.
- Normalisasi berubah (N-rules naik): versi DI DALAM kunci kanon (#988/#990 konsensus tech-debate) — input_canon_key sha256(norm_version ∥ norm_fn_id ∥ canon_fields ∥ payload), framing u32-BE.
- Durasi/wall-clock: TIDAK pernah masuk kanon/digest (metadata only — C-04, #981).
- Katalog 694: tipe langka tanpa fixture korpus → unit test sintetis + tanda [VERIFIKASI-UPSTREAM] (pola V-1..V-3) — bukan klaim palsu.

## 7a. Data fixture TERUKUR (scan korpus live 11:28, read-only)

| Node type | Instance | File | Utk rule/milestone |
|---|---|---|---|
| stickyNote | 450 | 171 (multi per file) | M4 binding (R-3) |
| cron (v1) | 13 | 13 | M2 rule cron→schedule (R-4) |
| function | 22 | 22 | M2 rule function→Code |
| functionItem | 11 | 11 | M2 rule functionItem→Code |
| scheduleTrigger | 25 | 25 | M2 fixture deterministik |
| code | 92 | 92 | M2 baseline Code v2 |
| @blotato/blotato | 36 | — | opaque R-2 + node komunitas (WCB future) |
| @n8n/langchain (lmChat, agent) | 33+31 | — | opaque R-2 + kait WCB |

Baseline hash 171 JSON: /home/agent4/corpus-baseline-1128.sha256 (8970ba3c…) — bahan bukti R-1 (0 mutasi).

## 8. Riwayat

v0.3 2026-09-09 (implementasi): #1022 directive fern — W1-ROSETTA-IMPL IN_PROGRESS, M1 langsung di crates/rosetta/ (rust-engine). M1–M5-inti SELESAI: 37 test hijau, clippy 0, 2.486 baris, 10 modul (binding/catalog/error/kanon/migrate/model/parse/receipt/registry). Gate: 171/171 parse 0-mutasi; migrasi 35 (function 22 + cron 13) di 20 file unik; 11 functionItem unresolved (V-3b); binding 450 sticky (918 cover, 16 ambigu); kanon digest stabil + diffgate-L1 (20/20 file termigrasi berubah digest). Bukti tree@sha: M1 485a321d → M1-3 0410af8b → M1-4 fabf7a3d → M5-inti 6afbb8c7 (file .rs+Cargo.toml; fixtures = salinan korpus tak di-hash). Keputusan terresolve: lokasi rust-engine/crates/rosetta (#1022); dep serde/serde_json/thiserror/sha2=0.10.8 (RULING 7); W4=DONE-DOC (cap bebas). Target scheduleTrigger 1.3 (koreksi katalog; amend 974aef88). M6 (manifest WCB) SELESAI per arahan matt RULING 21 "Lanjut M6": src/manifest_wcb.rs (model RFC v0.3 e2e2587e + validate MV-1 + type_check_params MV-2; n8n_type opsional = usul jembatan resolver amend RFC v0.4 utk agent6) + tests/manifest_fixtures/. Total: 11 modul src, 44 test hijau (serap syarat amend v0.4 agent6: n8n_type non-kosong), clippy 0, tree@sha fde68375. Sisa: review agent1/5/9 (#1062), verifikasi independen + matt utk DONE-CODE.

v0.2 2026-09-09: data fixture terukur §7a (scan korpus live) — da82d1dd.
v0.1 2026-09-09: shadow plan — menunggu entri queue W1-ROSETTA-IMPL (permintaan #1005) + keputusan lokasi/dep (#2).

---
## Append 2026-09-09 (pasca #1408–#1433) — RULING 46/47, Jalur B batch-1/2, HOLD
- #1408 RULING 46: anchor n8n@2.38.5 DISETUJUI; 46a tiga kategori provenance (upstream-verified / upstream-tool-generated / corpus-induced); __schema__ diutamakan; temuan *Tool = varian digenerate → retroaktif membenarkan RULING 35.
- #1415 jawaban-pengukuran: irisan diff 6 tool = ∅ → rekonstruksi per-tipe (bukan template injeksi tunggal).
- Batch-1 (box/coda/github) + batch-2 (airtableTrigger/autopilotTrigger/awsTranscribe/compareDatasets/elasticsearch/mailchimp) = 9/50 upstream-verified; data sha 0b16dd27; Temuan 7 (pollTimes legacy).
- #1419 (agent10): param_schemas_jalur_b.json untracked → DI-COMMIT 892f920; kriteria acceptance run-50 dicatat.
- #1396 RULING 45 dibaca PENUH (via sqlite): 45a pecah required → required_corpus (observasi)/required_ui (kontrak UI); 45b required_corpus bukan kendala validasi; penolakan absen sah hanya required_ui=true tanpa default; kunci asing + tipe tetap enforceable.
- #1420 RULING 47 §5b: agent4 TUNGGU review adversarial agent3 atas RULING 45 sebelum 47 tipe → HOLD sisa 41 tipe; posisi diumumkan #1433; review agent3 BELUM terbit per poll #1433 (scan 62 pesan).
- Data Jalur B tracked di commit 892f920 (crates/rosetta/data/param_schemas_jalur_b.json); skema A/multi belum dimigrasi ke 45a/45b — sengaja menunggu bentuk final hasil review.
