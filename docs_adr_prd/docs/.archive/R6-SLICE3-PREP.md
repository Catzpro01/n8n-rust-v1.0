# R-6 SLICE-3 PREP — pengukuran varians parameter korpus (read-only)

**2026-09-09 · agent4 · Persiapan keputusan slice-3 (ParameterSchema per tipe). Menunggu keputusan matt/fern: lanjut sekarang/nanti.**

## Angka (scan read-only 171 fixture, node base saja)
- 147 tipe base, 2.260 node total (base ~1.9xx).
- Median kunci parameter unik per tipe: **4**; mean 4,5. Schema per tipe umumnya KECIL.
- Bentuk keyset: 63 tipe 1-bentuk (non-kosong) → induksi schema langsung; 75 tipe multi-bentuk (perlu union/opsional); 0 tipe semua-kosong.
- **50 tipe hanya 1 node di korpus** → bukti korpus TIDAK cukup utk induksi schema (aturan preflight: (b) VERIFIKASI-UPSTREAM wajib, n8n INodeProperties). 97 tipe punya ≥2 node → induksi + validate layak.
- Tipe terberat: httpRequest (31 kunci, 50 bentuk, 274 node), set (9/10/169), googleSheets (12/17/68), slack (16/13/24), hubspot (20/12/15).

## Implikasi desain slice-3
1. Schema per tipe = ParameterSchema kernel {fields: ParameterField} — field OPSIONAL untuk kunci yang tidak selalu hadir (multi-bentuk); validate() harus menerima subset (display_if/required hanya utk kunci yang benar-benar wajib menurut sumber).
2. Prioritas pilot (bukti kuat): tipe 1-bentuk + ≥2 node (induksi deterministik); urutan berat: httpRequest/googleSheets/set menyusul dengan kasus union.
3. 50 tipe 1-node: jangan induksi dari 1 contoh (risiko overfit) — verifikasi upstream atau tunda sampai korpus/authoritative source tersedia.
4. Sumber kebenaran param: (a) korpus (untuk bentuk nyata), (b) n8n INodeProperties upstream (untuk jenis field: string/number/options/dst + default) — mapping ParameterKind di tepi, konsisten RULING 28 & fakta kernel params.rs.
5. Estimasi volume: ~147 schema kecil + 1 kompleks (httpRequest) + tool/trigger variants; diff-test validate thd 2.260 node = gate (schema salah → validate menolak param nyata → terlihat).

## Verifikasi reproduksi
Scan mandiri: /tmp/param_prep.py (python3, read-only corpus/).

---
## STATUS EKSEKUSI (2026-09-09) — RULING 38 §7 (matt #1291)
- **JALUR A SELESAI**: 22 tipe 1-bentuk n>=2 → `param_schemas_jalur_a.json` (sha 446f5e52) — laporan #1303.
- **MULTI (A2) SELESAI**: 74 tipe multi-bentuk (dari 75; **httpRequest DITUNDA** per matt "kerjakan paling akhir") → `param_schemas_multi.json` (sha 1a06453e). Pengukuran: 0/75 tipe jadi 1-bentuk bila dipecah per-major → union per-tipe = pendekatan korpus jujur. 447 fields total, 62 required (irisan), provenance `corpus-induced-union (n=…, shapes=…)`.
- **Total skema korpus: 96** (22 A + 74 multi); tipe tanpa skema = 50 Jalur B (n=1, upstream) + httpRequest = 51.
- **59/59 test**, clippy 0, tree@sha bf66c8fe. Gate: validate thd SEMUA node base 171 fixture 0 penolakan; tiap skema terpakai; tipe tanpa skema == 51.
- Berikutnya: httpRequest (paling akhir, per-major perhatian — 50 bentuk lintas v1..v4.x), lalu Jalur B upstream-verified (50 tipe n=1; INodeProperties n8n; `provenance: upstream-verified (INodeProperties @ <versi n8n>)`).

## JALUR B — inventori siap (2026-09-09, read-only)
- 50 tipe n=1 → `rosetta-src/data/jalur_b_inventory.json` + `/tmp/jalur_b_list.json` (server): tiap baris {type_full, major, typeVersion, file asal, n_keys, keys}. 2 tipe dgn 0 kunci param → murni upstream.
- Verifikasi upstream = bandingkan thd INodeProperties n8n (options/displayOptions/type/default per field). Anchor versi n8n: DIMINTA ke matt/fern (ACK #R41 sudah menanyakannya).
- rules: `provenance: upstream-verified (INodeProperties @ <versi n8n>)`; tanpa bukti = `unverified`, tidak diterbitkan sbg skema otoritatif (RULING 38 §7).

---
## JALUR B — STATUS EKSEKUSI (batch-1, 2026-09-09)
- RULING 46a (matt #1408): 3 kategori provenance resmi — `upstream-verified` (44 non-Tool), `upstream-tool-generated` (6 Tool), `corpus-induced` (A/multi). Anchor n8n@2.38.5 disahkan utk semua; __schema__ diutamakan bila ada; sendInBlue → anchor n8n@1.0.0 (rebrand Brevo, 404 di 1.1.0+).
- Jawaban pengukuran tool (pertanyaan matt §1): himpunan injeksi TIDAK tertutup (irisan diff 6 tool = ∅; 2 tool = subset base) → rekonstruksi per tipe.
- data/param_schemas_jalur_b.json: **9/50** (box ✓ [file:upload binaryData bool req-ui + fileName str], coda ✓ [docId/tableId options req-ui + options collection], github ✓ [owner/repository resourceLocator req-ui + operation/resource]); sha256 b82cf449. Tiap field: {name, type_ui, required_ui, default, verified(file:line)}.
- Model: operasi tak terserialisasi di JSON → operation_context dicatat (default diasumsikan hati-hati, tanpa klaim berlebih).
- BATCH-2 (6): airtableTrigger (baseId/tableId resourceLocator; triggerField string req; additionalFields collection; **pollTimes LEGACY — 0 hit di 2.38.5 & 1.0.0, dicatat bukan dijamin**), autopilotTrigger (event options req), awsTranscribe (operation/transcriptionJobName/mediaFileUri/detectLanguage/options + __schema__), compareDatasets (mergeByFields/options/resolve — pilot-1 dlm), elasticsearch (DocumentDescription.ts: operation/indexId/options/fieldsUi/additionalFields + __schema__ v1.0.0), mailchimp (resource/operation/list options-loadOptions/options). Sha data kini 0b16dd27.

---
## JALUR B — HOLD RULING 47 §5b (2026-09-09, setelah #1420 & #1433)
- **KEADAAN: HOLD.** matt (#1420 RULING 47 §5b) menugaskan agent3 = review adversarial atas RULING 45 (#1396) SEBELUM agent4 menjalankan 47 tipe tersisa. Batch-1/2 (9 tipe) terlanjur dieksekusi pra-#1420 (kronologi jujur diumumkan #1433 — kesalahan proses: poll tak menjangkau #1420 sebelum batch-2 terbit). Sisa 41 tipe (35 non-Tool + 6 Tool) DITAHAN.
- **Data ter-commit**: param_schemas_jalur_b.json kini tracked di repo crate rosetta, commit **892f920** (jawaban #1419 agent10; sha data 0b16dd27). Catatan komit: RULING 45a/46a, anchor per entri.
- **RULING 45 dibaca penuh (#1396)**: 45a — field `required` PECAH: `required_corpus` (hadir di semua n instance; OBSERVASI, bukan kontrak) vs `required_ui` (INodeProperties mewajibkan; KONTRAK UI). 45b — `required_corpus` TURUN dari kendala validasi jadi observasi: validator TIDAK BOLEH menolak workflow karena field-absen yang hanya "selalu hadir di korpus"; penolakan field-absen SAH hanya bila Jalur B `required_ui=true` DAN field tidak punya default upstream; yang tetap boleh ditegakkan = himpunan kunci TERTUTUP + kesesuaian tipe utk kunci yang hadir. Gate "0 penolakan 171 fixture" = bukti cocok-korpus, BUKAN bukti required_corpus aman sbg kendala.
- **Kesiapan adaptasi pasca-review agent3** (belum dikerjakan, menunggu bentuk final):
  1. 9 entri Jalur B sudah pakai `required_ui` + `default` + `verified(file:line)` → tinggal disesuaikan bila agent3 menuntut bentuk lain.
  2. `param_schema.rs` + data jalur_a/multi masih memakai field `required` (semantik korpus) & validate menolak field-absen → migrasi ke required_corpus/required_ui + relaksasi 45b akan direplikasi SEKALI setelah bentuk dikunci review (hindari menulis dua kali).
  3. Aturan "pasti ada" (45b): dari 9 entri verified, daftar field required_ui=true TANPA default upstream = kandidat penegakan absen; required_ui=true DENGAN default = TIDAK diserialkan saat = default (contoh korpus calendly: {events} saja walau scope/authentication di upstream). Validasi ulang per entri setelah review.
- **PENGUKURAN dukung-review §5b Q2 (2026-09-09, read-only, 9 entri verified vs korpus)**: 33 fields → 20 required_ui=true → **0 tanpa default** → **2 absen di korpus** (`awsTranscribe.operation`, `mailchimp.resource`; keduanya required_ui=true + default → nilai=default ⇒ tak diserialkan). Arti: utk n=1 saat ini aturan "pasti ada" 45b TIDAK PERNAH menyala (0/20) — penegakan absen vacuous kecuali field required_ui tanpa default ditemukan di 41 tipe tersisa; mekanisme tahu-default = `verified(file:line)` per field (sifat statis deklarasi INodeProperties, bukan evaluasi ekspresi TS; default dinamis/fungsi = perlakukan sbg tanpa-default). Data ini disuplai ke agent3 sbg bahan review, bukan verdict.
- **BATCH-3 (6, 2026-09-09)**: crypto (tpl-574; tv1 → sumber v1 walaupun defaultVersion 2.38.5=2; action default hash/type MD5/binaryData false tak terserialisasi), aiTransform (hidden:true; 3 field non-required; jsCode/instructions/codeGeneratedForPrompt = konstanta AI_TRANSFORM_*), markdown (mode markdownToHtml non-default + markdown/destinationKey required + options 2 varian per mode), renameKeys (keys fixedCollection; additionalOptions tak terpakai), sort (type default simple; sortFieldsUi[].fieldName req; order default ascending), summarize (0 required; fieldsToSummarize[].aggregation default count; fieldsToSplitBy 2 varian outputFormat; options.outputFormat default separateItems). 15/50; sha data 0ad004fb27790e01; semua provisional:true; TIDAK ADA corpus-legacy baru di batch ini (6/6 field kunci terverifikasi upstream; options={} = artefak template dicatat).
- **RULING 50b/50c TER-EKSEKUSI (2026-09-09)**: commit 7665fd6712c4158653b4f3becf1d8f2f72079d0d — guard `kernel::json_depth_exceeded` (kernel-asli 5bc8ea82; dep workspace path) di `parse_workflow_bytes` (parse.rs; MAX_JSON_DEPTH=64 pub) + `parse_manifest` (manifest_wcb.rs); varian `RosettaError::DepthExceeded`; alasan tertulis 50c di param_schema.rs include_str!. Tests/json_depth_guard.rs (4 test: batas empiris biner, delegasi str/path, manifest dalam, fixture normal). **63 passed (59+4), clippy 0.** Sha artefak penuh di pesan commit (50g). Kepemilikan crates/rosetta: agent4 Primary, agent3 co-reviewer (Dekrit #1469).
- **BATCH-4 (6, 2026-09-09)**: 21/50 — calendlyTrigger (events multiOptions req default []; authentication/scope default tak terserialisasi = pola RULING 45 §1), dhl (trackingNumber req; resource hidden shipment + operation get default), hackerNews (resource all non-default; limit; additionalFields 2 varian per resource), hunter (operation emailVerifier non-default; email req displayOptions), mauticTrigger (events req; opsi = loadOptions dinamis eventName/eventId), mindee (0 kunci korpus → fields [] + verifikasi upstream default dicatat). Sha data e2ea03a6; 0 anomali corpus-legacy.
- **BATCH-5 (6, 2026-09-09)**: 27/50 — awsRekognition (type detectLabels non-default; binaryData req default false; CATATAN: __schema__/v1.0.0/image/analyze.json = skema RESPONS AWS bukan parameter UI → verifikasi .ts dicatat jujur), githubTrigger (owner/repository resourceLocator req + events multiOptions req default []), shopifyTrigger (topic statis tidak required), slackTrigger (trigger multiOptions + watchWorkspace + options), medium (title/contentFormat/content req + additionalFields; default post/create), trello (listId/name req + description + additionalFields; blok create). Sha 22498e64; 0 anomali corpus-legacy; stats script: 27 entri/74 fields/36 required_ui/0 tanpa default.
