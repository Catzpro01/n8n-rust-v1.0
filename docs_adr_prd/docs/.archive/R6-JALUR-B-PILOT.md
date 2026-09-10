# R-6 JALUR B — PILOT verifikasi-upstream INodeProperties (3 tipe)

**2026-09-09 · agent4 · RULING 38 §7/38b (#1291) + arahan #1317 ("Jalur B via upstream INodeProperties"). PILOT membuktikan metode; 47 tipe lain menunggu keputusan anchor versi (#1349 — rekomendasi di bawah).**

## Anchor upstream
- Repo: n8n-io/n8n master, **HEAD fcf21f5efc63**, release **n8n@2.38.5** (rilis 2026-09-09 — hari ini). Sparse clone di /tmp/n8n-upstream (VPS; blob:none).
- Path: packages/nodes-base/nodes/<…>; node bisa `VersionedNodeType` (subfolder v1/v2/v3…) — VERIFIKASI WAJIB per versi major yang teramati di korpus (bukan file utama saja).

## Hasil pilot (draft — PROVISIONAL-ANCHOR, belum final)

### 1. calendlyTrigger (korpus n=1, typeVersion **1**)
File: Calendly/v1/CalendlyTriggerV1.node.ts (base CalendlyTrigger.node.ts defaultVersion=2; versi 1 di v1/).
- properties v1: `authentication` (options, default apiKey), `scope` (options, default user, required), `events` (multiOptions, default [], **required**), `pollTimes` (?) — periksa lanjut bila final.
- credentials: calendlyApi | calendlyOAuth2Api.
- KORPUS node berisi {events} saja → **required INodeProperties ≠ dijamin hadir di JSON** (default implisit tidak diserialkan). Temuan semantik, lihat bawah.

### 2. compareDatasets (korpus n=1, typeVersion **2.3**)
File: CompareDatasets/CompareDatasets.node.ts.
- properties: `mergeByFields` (fixedCollection, default {values:[{field1:'',field2:''}]}), `resolve` (options, default 'preferInput2'), + konfigurasi lanjut utk perbedaan (input yang dipakai: 'Use Input A/B/Mix/Include Both').
- KORPUS keys [mergeByFields, options, resolve] — 'options' menampung opsi tambahan (mis. fuzzyCompare?) — konfirmasi saat final.

### 3. awsRekognition (korpus n=1, typeVersion **1**)
File: Aws/Rekognition/AwsRekognition.node.ts (NON-versioned) + **__schema__/v1.0.0/image/analyze.json** (n8n mulai memakai schema-driven terpisah — perhatikan utk tipe Aws lain).
- KORPUS keys [additionalFields, binaryData, type].

## Temuan desain (penting utk model schema konsumen)
1. **required (INodeProperties) = keharusan UI, BUKAN jaminan kehadiran di JSON**: field `required:true` + `default` sering TIDAK diserialkan saat = default. Skema Jalur B utk validasi korpus TIDAK boleh menolak ketiadaan field ber-default; `required` dipertahankan sbg METADATA upstream (label `required_ui`).
2. **VersionedNodeType**: periksa per (tipe, major) teramati; file versi = sumber kebenaran, bukan file utama (defaultVersion bisa beda dari major korpus).
3. **n8n bergerak ke schema-driven** (`__schema__/v1.0.0/…json`) — tipe baru mungkin lebih mudah diverifikasi dari JSON skema daripada parsing .ts.
4. Provenance Jalur B: `upstream-verified (INodeProperties @ n8n 2.38.5 fcf21f5e)`.

## Rekomendasi anchor (menunggu matt/fern)
**Anchor tunggal n8n@2.38.5** (rilis hari ini, mayor 2 = mayor produk terbaru; node typeVersion mayor korpus 1–4 DIPERIKSA per file versi masing-masing — anchor produk ≠ typeVersion node). Alternatif: anchor = versi n8n saat fixture dihasilkan (tak diketahui). Mohon konfirmasi; tanpa arahan, 47 tipe tersisa TIDAK dieksekusi.

## Deliverable saat final nanti
- data/param_schemas_jalur_b.json (GENERATED: 50 skema + provenance anchor) + gate korpus (validate: field yang ADA dicek jenis; tiada penolakan field ber-default hilang).

---
## TEMUAN 4 (pilot-2, 2026-09-09) — keluarga *Tool TIDAK punya INodeProperties mandiri
- n8n@2.38.5 master: `cryptoTool`/`dateTimeTool`/`httpRequestTool`/`gmailTool` dll **TIDAK ADA** sebagai node/file deskripsi (git grep 0 hit; dir HttpRequest@1.65.0..2.10.0 hanya base node + V1..V3).
- Kesimpulan: *Tool = **varian tool yang DIGENERATE n8n** dari node base + parameter tool yang disuntikkan (toolDescription, descriptionType, outputFieldName, dataPropertyName, stringLength, ...) — direkam di JSON workflow saat node dipakai sbg "tool" AI Agent. Bukan sumber INodeProperties terpisah; `required_ui`/default = milik NODE BASE (mis. dateTimeTool → DateTime v2).
- Korpus 147 base: **9 tipe Tool** (6 di Jalur B: cryptoTool, dateTimeTool, googleSheetsTool, httpRequestTool, rssFeedReadTool, telegramTool; 3 lain sudah ber-skema korpus A/multi). Konsistensi: keys korpus dateTimeTool [toolDescription, descriptionType, endDate, operation, options, outputFieldName, startDate] = tool-injection + DateTime v2 operation/options — cocok hipotesis.
- IMPLIKASI ANCHOR: anchor tunggal n8n@2.38.5 valid utk 44 tipe non-Tool Jalur B; 6 tipe Tool diverifikasi via **base node (per major) + aturan injeksi tool**, provenance: `upstream-tool-generated (base @ n8n 2.38.5 fcf21f5e)` — keputusan tetap di @matt/@fern (pertanyaan #1387 diperkaya).

---
## TEMUAN 5 + 6 (2026-09-09, probe korpus & tree 2.38.5)
5. **KELAYAKAN ANCHOR 2.38.5 TERUKUR**: 43/44 tipe non-Tool Jalur B ADA di tree 2.38.5 (basename *.node.ts/.json case-insensitive; verifikasi per file versi). 1 missing: `sendInBlue` → **rebrand Brevo** di n8n modern (folder Brevo/, name 'brevo') — tipe korpus lama; verifikasi wajib dari tag n8n lama (anchor kedua utk 1 tipe) atau catat `upstream-removed (rebrand brevo)`. Probe tag: file SendInBlue.node.ts ADA di n8n@1.0.0, 404 di 1.1.0..2.38.5 → anchor utk tipe ini = **n8n@1.0.0**.
6. **POLA FIELD TOOL TIDAK SERAGAM** (scan korpus 11 tipe *Tool, incl. komunitas): `toolDescription` ADA di cryptoTool/dateTimeTool/httpRequestTool/rssFeedReadTool TAPI TIDAK di googleSheetsTool ([columns,documentId,operation,options,sheetName]) / telegramTool ([additionalFields,chatId,text]); komunitas blotatoTool/firecrawlTool = struktur sendiri. → TIDAK ada 1 template injeksi; verifikasi tool per (base, varian): keys = campuran base operation/options/resource + field tool-spesifik; per tipe Tool n=1 korpus → skema dari base + keys korpus, provenance `upstream-tool-verified (base @ n8n 2.38.5 fcf21f5e)`.
