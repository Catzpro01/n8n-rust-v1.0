# ROSETTA-API-FOR-WORKFLOW — Inventaris & peta konsumsi crates/rosetta untuk C-6 / TB-02+

- Izin: RULING-66 §4 (matt #1696) — riset read-only disetujui; deliverable laporan + sha.
- Pelaksana: agent4 (ROLE_SCHEMA) — 2026-09-10. Kategori riset (R64 Aturan 3).
- Batas: baca sumber + grep; TIDAK ada eksekusi runtime baru; NOL commit, NOL perubahan pohon.
- Anchor: rust-engine HEAD 6c02bd2; kernel-asli-d3bcff0 (workspace kernel path-dep).

## 1. Posisi rosetta dalam Tahap 1 (konteks dokumen kanonik)
- REPLAN-BACKEND-MATANG.md:45 — rosetta 4.653 baris, "parser/penggolong JSON workflow n8n" = MODAL NYATA.
- REPLAN:98, :184-185 — C-6 (crate `crates/workflow`) **memakai ulang rosetta yang sudah bisa parse**; deliverable muat/simpan/validasi workflow JSON n8n 2.39.0 + publish/unpublish.
- RULING-62 §3 (@agent4): "yang dibutuhkan sekarang rosetta bisa **dimakan** oleh crate workflow".
- RULING-66 §4: C-6 mengonsumsi parse rosetta / adapter di atasnya, **BUKAN parser baru** (agent3 dicatat).
- Dokumen kanonik: docs/spec/PRD-FRONTEND-RUST.md (sha 997cd83271ff; daftar kanonik kosakata domain = tipe kernel: Item, ItemList, ParameterSchema, NodeDescriptor, … — nama rosetta TIDAK masuk daftar itu), TICKETS-TRACER-BULLET.md (b9d18f654fb8; TB-02/03/04/10), CLI-PARITY-SPEC.md, REPLAN-BACKEND-MATANG.md.

## 2. Inventaris API publik crates/rosetta (verified file:line; src/lib.rs:21-39)
Modul publik: binding, catalog, error, kanon, kindmap, manifest_wcb, migrate, model,
param_schema, param_schema_b, parse, receipt, registry. Re-export lib.rs:35-39:
`bind_notes/BindingReport/NoteBinding/NoteRect`, `RosettaError`, `CanonNode/NodeOrigin/
OpaqueNote/TypeVersion/Workflow`, `parse_workflow_bytes/parse_workflow_path/
parse_workflow_str`, `build_receipt/receipt_to_json/Receipt`.

| Modul | API publik (pilihan) | Fungsi utk C-6/TB-02 |
|---|---|---|
| parse.rs | `parse_workflow_bytes/str/path` (parse.rs:22/41/46); `MAX_JSON_DEPTH=64` (:19); guard kernel::json_depth_exceeded (:24-31) | **MUAT**: file/bytes/str workflow → Workflow; fail-loud struktur rusak; opaque-safe |
| model.rs | `Workflow` (:194: name, nodes urut-sumber, connections Value utuh, extra preserved, opaque_nodes); `CanonNode` (:167: name, type_full, type_version Option<TypeVersion>, origin, position Option<[f64;2]>, parameters Value UTUH, extra preserved); `TypeVersion` (:11, from_f64/parse_str fail-closed minor≥2 digit); `NodeOrigin` (:131); `OpaqueNote` (:152) | **Model grafik kanonik M1**; TANPA kehilangan info (kunci tak dikenal → extra) |
| kindmap.rs | `kind_for_type` (:58, tabel 147 tipe base RATIFIED #1241+R35); `version_u16` (:72, MAJOR*10+minor → u16 kernel) | type_full n8n → NodeKind kernel + version u16 (deskriptor node) |
| param_schema.rs | `all_schemas` (:94, 96 = 22 A + 74 multi); `schema_for` (:101); `validate` (:128, semantik 54e); `validate_notes` (:171) | validasi bentuk-dini param node (tipe berskema korpus) |
| param_schema_b.rs | `jalur_b_entries` (:91, 50); `validate_jalur_b` (:123); `tripwire_violations` (:102) | validasi 50 tipe Jalur B upstream-verified (46a) |
| migrate.rs | `migrate_workflow` (:79; rules function→code @2.0, cron→scheduleTrigger @1.3); `MigrationReport` | era lama → versi modern (import lintas era) |
| kanon.rs | `canonicalize` (:285), `canonical_json_bytes` (:328), `canonical_digest` (:334, sha256), `KANON_VERSION` | digest deterministik (diff/identitas; jalur receipt) |
| receipt.rs | `build_receipt` (:71), `receipt_to_json` (:89) | jejak opaque + migrasi |
| registry.rs | `cron_params_to_rule` (:146), `function_params_to_code` (:176), dst. | util migrasi spesifik |
| binding.rs / catalog.rs | `bind_notes` (sticky note), `origin_of` | pelengkap (sticky note / klasifikasi origin) |
| manifest_wcb.rs | WcbManifest dll. | RANAH WCB agent6 — JANGAN tercampur ke jalur workflow |

## 3. Fakta integrasi (verified)
1. **rosetta → kernel sudah menjadi dep**: Cargo.toml rosetta `kernel = { workspace = true }`; dipakai nyata di parse.rs:24 (`kernel::json_depth_exceeded`, RULING 50b/50c). Tidak ada cycle (kernel bebas dep internal, D115).
2. **TIDAK ADA konsumen rosetta di luar** (grep workspace: rosetta:: hanya dipakai internal + tests) → rosetta = pustaka siap-konsumsi, belum dikonsumsi. Konsumen pertama = crates/workflow (C-6).
3. **TIDAK ADA serializer publik** (tidak ada to_json/workflow export). Yang ada hanya kanon (normalisasi+digest). Catatan: model Workflow/CanonNode derive Serialize — `serde_json::to_value` bisa menyusun ulang; TAPI serde_json Map tanpa fitur `preserve_order` = BTreeMap → **urutan kunci JSON ≠ urutan sumber**. n8n toleran thd urutan kunci (round-trip FUNGSIONAL aman); kesetiaan byte TIDAK dijamin. C-6 perlu serializer tersendiri (atau fitur preserve_order) utk ekspor.
4. **parameters dipertahankan UTUH sebagai Value** (CanonNode.parameters; parse.rs:88-92; kunci lain → extra). Artinya tidak ada kehilangan bentuk param n8n — termasuk bentuk ganda Set (legacy fields.values[] vs kanonik assignments[], RULING-66 §3) — karena rosetta tidak menafsirkan isi parameters, hanya menyimpan.
5. **kindmap mencakup SEMUA node inti Tahap 1** (verified): set kindmap.rs:215, if :169, merge :186, httpRequest :164, manualTrigger :180, scheduleTrigger :212, webhook :236, switch :226, code :115, errorTrigger :134, executeWorkflow :136, respondToWebhook :209, noOp :196, cron :120. Konvensi kernel konsisten (http.request dll).
6. **version_u16** = major*10+minor → `NodeDescriptor.version: u16` kernel (D89). TypeVersion dari parse (float 1.2 → 1/2) → u16 12; minor 2 digit ditolak fail-closed (model.rs:34-66).
7. **Dua model schema** (gap nyata): kernel `ParameterSchema` (kernel params.rs:22; dipakai NodeDescriptor utk validator/UI A-30) vs rosetta `SchemaField` (param_schema.rs:35; 146 tipe korpus+upstream, semantik 54e). Belum ada konverter. Jalur runtime (deskriptor kernel) dan jalur verifikasi (rosetta) tidak tersambung.
8. **httpRequest TIDAK terskema param di rosetta** (deferred; R-6 menunggu node inti jalan, R62 §3) — padahal TB-03 memakai node HTTP Request. Untuk TB-03, validasi param httpRequest harus via skema manual/deskriptor (atau tunda sampai R-6 lanjut).

## 4. Peta konsumsi (siapa memakai apa)
- **C-6 muat (import)**: `parse_workflow_path/bytes` → `Workflow` — LANGSUNG. Migrasi era lama via `migrate_workflow` bila perlu. Validasi struktural = parse (fail-loud); validasi per-tipe = `param_schema::schema_for`+`validate` (96) / `param_schema_b::validate_jalur_b` (50) — dengan catatan: skema korpus A/multi menolak kunci asing; keputusan kebijakan (tolak vs catat) di tangan C-6.
- **TEMUAN (verified, data 2026-09-10):** node `set` terskema di multi (n=169, provenance corpus-induced-union majors=[1,2,3], shapes=10; fields: assignments, fields, include, includeOtherFields, jsonOutput, keepOnlySet, mode, options, values — MENCANGKUP bentuk legacy DAN kanonik). Namun `duplicateItem` (dipakai fixture kanonik 2.39.0 agent9, RULING-66 §3) TIDAK ada di fields skema → `validate` multi thd workflow 2.39.0 ber-`duplicateItem` = error "parameter tak dikenal". Artinya: skema korpus = snapshot scan 2026-09-09; ekspor 2.39.0 memuat bentuk lebih baru yang belum terekam. KONSEKUENSI utk C-6/TB-02: jangan jadikan `validate` rosetta sbg gate keras impor workflow 2.39.0 — pakai sbg laporan/peringatan, dan catat pembaruan data skema (re-scan thd ekspor 2.39.0 / fixture agent9) sbg tugas lanjut sebelum validasi keras diaktifkan. Konsisten dgn prinsip RULING-66 §3 (beberapa bentuk per typeVersion; menolak bentuk sah = cacat).
- **C-6 simpan/ekspor**: `Workflow` (Serialize) → simpan ke storage (SQLite) sbg dokumen+model; ekspor → serializer C-6 (di atas model rosetta; lihat §3.3). Round-trip "bisa diimpor n8n asli" (TB-02) = muat → simpan → serialize: AMAN fungsional krn parse preserved + Value utuh.
- **C-3 executor / TB-02 execute**: per node — `kind_for_type(type_full)` → NodeKind; `version_u16(type_version)` → version u16; parameters = `CanonNode.parameters` (Value utuh). **Ini data kunci utk keputusan kanal params (Opsi-1 vs Opsi-2):** executor MEMEGANG parameters lengkap per node sejak parse; gap hanya cara meneruskannya ke `Node::execute` (NodeContext tanpa kanal params — audit PREFACTOR-4 T-1).
- **TB-03 (HTTP Request + If/Merge + ekspresi)**: kindmap siap; validasi param httpRequest TIDAK tersedia (gap §3.8); ekspresi = ranah expr/expr-quickjs (di luar rosetta); `executionOrder` = ranah executor.
- **TB-10 publish/unpublish**: status → storage; rosetta tidak terlibat (model tak punya status).
- **Determinisme/diff (jalur lama W1)**: kanon + receipt — tetap utk receipt/digest; bukan jalur Tahap-1.

## 5. Rekomendasi (data utk keputusan yang ditunda matt — RULING-66 §4)
**Keputusan Opsi-1 vs Opsi-2 (kanal parameter instance).** Data riset:
- parameters tersedia UTUH di `CanonNode.parameters` (Value) sejak parse; executor per node tinggal meneruskan.
- TB-01 (agent1) membuktikan pola **params-on-self** (=Opsi-2) JALAN thd kernel tanpa ubah kontrak (6c02bd2, nodes-core SetNode).
- Kernel freeze (D115/A-05) + nol konsumen di luar → mengubah kontrak (Opsi-1) murah sekarang, tapi menyentuh kernel yang baru saja dipakai TB-01/TB-02.
- A-30 "satu sumber schema+validasi+form" paling tepat dilayani Opsi-1 (kanal params + validasi terpusat), tapi prasyaratnya konverter dua model schema (§3.7) — pekerjaan ekstra.
**Rekomendasi berbasis data: Opsi-2 utk Tahap-1** — factory/registry pola `kind → fn(&Value) → Box<dyn Node>` di executor (params-on-self, konsisten TB-01); kernel tidak disentuh; TB-02/03 tidak terblokir. **Opsi-1** dipertimbangkan kembali SEBELUM node library besar (pasca-TB-10), saat konverter schema + A-30 jadi kebutuhan nyata. Matt memutuskan; riset hanya menyediakan data.
**Rekomendasi lain:** (a) konsumsi: crates/workflow langsung depend `rosetta` (sudah workspace member) — adapter tipis (bukan crate baru) utk muat/validasi/serialize; (b) serializer: bangun di crates/workflow, putuskan kesetiaan byte (preserve_order) vs fungsional — rekomendasi: fungsional utk TB-02 (kriteria "bisa diimpor kembali"), catat kesetiaan byte sbg utang; (c) jalur validasi: mulai dari parse (struktural) + deskriptor kernel (runtime); integrasikan rosetta param_schema A/B bertahap — utk TB-02: validasi bentuk dini via rosetta HANYA sbg laporan (lihat temuan §4: duplicateItem vs skema set), bukan gate keras; re-scan data skema thd ekspor 2.39.0 = tugas lanjut; (d) httpRequest schema: tetap deferred (R62) — TB-03 memakai deskriptor manual; (e) manifest_wcb TIDAK dipakai jalur ini.

## 6. Kepatuhan & artefak
- Read-only: nol commit, nol perubahan pohon kanonik (rust-engine & kernel). Seluruh perintah = baca + grep.
- Laporan ini: docs/ROSETTA-API-FOR-WORKFLOW.md (sha256 lihat pesan forum). File lokal sumber ada di workspace agent4.
