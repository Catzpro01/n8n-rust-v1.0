# NODE-PARAM-FORMS-TB03 — Bentuk parameter per typeVersion: If, Merge, HTTP Request
- Kategori: B (pencarian fakta, RULING-67 §1B) — disusun utk TB-03 (Ekspresi `{{ }}` + node HTTP Request) & prinsip RULING-66 §3: node n8n nyata punya beberapa bentuk parameter per typeVersion; bentuk sah yang belum didukung harus error menyebut nama, JANGAN diabaikan diam-diam.
- Sumber: upstream n8n 2.39.0 (clone kanonik /opt/agent-workspace/upstream/n8n-2.39.0). Ekstraksi statis dari source TS (nama param level-atas properties; tidak ada eksekusi n8n — tanpa node_modules). Nama path di bawah relatif thd `packages/` kecuali dinyatakan lain.
- Tanggal: 2026-09-10; pelaksana: agent4; read-only; anchor: rust-engine HEAD 6c02bd2.

## 0. Ringkasan versi (verified file:line)
| Node | Versi lama | Versi modern | defaultVersion | type (kindmap) |
|---|---|---|---|---|
| If | V1 (v1) | V2 (v2, 2.1, 2.2, 2.3) | 2.3 (If/If.node.ts:16) | n8n-nodes-base.if |
| Merge | v1 (v1), v2 (v2, 2.1) | v3 (v3, 3.1, 3.2) | 3.2 (Merge/Merge.node.ts:18) | n8n-nodes-base.merge |
| HTTP Request | V1 (v1), V2 (v2) | V3 (v3, 4, 4.1–4.5) | 4.5 (HttpRequest/HttpRequest.node.ts:18) | n8n-nodes-base.httpRequest |

## 1. If (`nodes-base/nodes/If/`)
- `If.node.ts:16` defaultVersion 2.3; `V1/IfV1.node.ts:29` version 1; `V2/IfV2.node.ts:23` version [2, 2.1, 2.2, 2.3].
- Parameter level-atas:
  - v1: `conditions`, `combineOperation`
  - v2: `conditions`, `options` (collection; contoh isi verified: `ignoreCase` — IfV2.node.ts:69-77)
- **Temuan halus**: IfV2.node.ts:47 — versi *parameter kondisi* dikendalikan ekspresi UI `={{ $nodeVersion >= 2.3 ? 3 : $nodeVersion >= 2.2 ? 2 : 1 }}`. Artinya bentuk UI kondisi berubah **di dalam satu versi file** (typeVersion 2.0/2.1 → bentuk 1; 2.2 → bentuk 2; ≥2.3 → bentuk 3). Peringatan utk penerap: membaca `conditions[].type` apa adanya + menghormati `typeVersion` node, bukan hard-code bentuk tunggal.
- Implikasi: If v1 (combineOperation) vs v2 (options) — dua bentuk param berbeda yang harus diterima (atau error eksplisit), konsisten RULING-66 §3.

## 2. Merge (`nodes-base/nodes/Merge/`)
- `Merge.node.ts:18` defaultVersion 3.2; `v1/MergeV1.node.ts:23` version 1; `v2/MergeV2.node.ts:38` version [2, 2.1]; v3 dirakit dari `v3/actions/versionDescription.ts:7-12` (displayName Merge, name merge, version [3, 3.1, 3.2]) + router (pola deklaratif action-router).
- Parameter level-atas:
  - v1: `mode`, `join`, `propertyName1`, `propertyName2`, `output`, `overwrite`
  - v2: `mode`, `combinationMode`, `mergeByFields`, `joinMode`, `outputDataFrom`, `chooseBranchMode`, `output` (outputDataFrom muncul di >1 sub-bentuk — lihat file utk rincian display)
  - v3: deklaratif action-router → daftar mode/param hidup di `v3/actions/` (router.ts, helpers/, dst.); penerap TB-03 harus membaca folder actions/ utk bentuk v3, JANGAN menebak dari v2.
- Implikasi: mode v1 (join+overwrite), v2 (combinationMode/mergeByFields/chooseBranchMode), v3 (actions) — tiga generasi bentuk.

## 3. HTTP Request (`nodes-base/nodes/HttpRequest/`)
- `HttpRequest.node.ts:18` defaultVersion 4.5; `V1/HttpRequestV1.node.ts:46` version 1; `V2/HttpRequestV2.node.ts:54` version 2; `V3/HttpRequestV3.node.ts:79` version [3, 4, 4.1, 4.2, 4.3, 4.4, 4.5] dan `:106` memakai `properties: mainProperties` dari `V3/Description.ts:33`.
- Parameter level-atas:
  - v1 (21): authentication, requestMethod, url, allowUnauthorizedCerts, responseFormat, dataPropertyName, jsonParameters, options, sendBinaryData, binaryPropertyName, bodyParametersJson, bodyParametersUi, headerParametersJson, headerParametersUi, queryParametersJson, queryParametersUi, infoMessage
  - v2 (v1 + 2): menambah `nodeCredentialType`, `genericAuthType` (prefill kredensial; bentuk transisi ke prefill system)
  - v3–4.5 (32 nama dari mainProperties, V3/Description.ts): curlImport, method, url, authentication, nodeCredentialType, googleApiWarning, genericAuthType, provideSslCertificates(+Notice, sslCertificate), sendQuery/specifyQuery/queryParameters/jsonQuery, sendHeaders/specifyHeaders/headerParameters/jsonHeaders, sendBody/contentType/specifyBody/bodyParameters/jsonBody/inputDataFieldName/rawContentType/body, options, infoMessage
- **Implikasi utama TB-03**: ekspor/import workflow 2.39.0 yang relevan = bentuk modern v4.x (`method`/`url`/`sendHeaders`/`sendQuery`/`sendBody`…). Bentuk v1/v2 (requestMethod/jsonParameters/…Ui) hanya muncul di workflow era lama — tetap harus dimuat tanpa tolak diam-diam (error eksplisit kalau belum didukung). Catatan: jsonParameters U/I era lama vs jsonQuery/jsonBody era baru = perbedaan pola serialisasi param yang nyata.

## 4. executionOrder — fakta engine (utk kriteria TB-03 "per-branch, bukan breadth-first v0")
Sumber: `packages/core/src/execution-engine/workflow-execute.ts` + `requests-response.ts` (n8n 2.39.0).
- `workflow-execute.ts:209-210` — `isLegacyExecutionOrder(w) = w.settings.executionOrder !== 'v1'`: **hanya `'v1'` yang memicu jalur modern (per-branch)**.
- `workflow-execute.ts:1511` — komentar: legacy = `executionOrder: 'v0'`.
- Perbedaan perilaku eksplisit utk 'v1' vs lainnya: antrean node (:451 unshift vs push), pemaksaan eksekusi node input (:987), logika wire/indeks & cabang (:2520, :2541, :2609), urutan request balik (:requests-response.ts:271-272).
- Generator internal n8n menulis **`executionOrder: 'v1'` eksplisit** saat membuat workflow modern: `packages/cli/src/modules/mcp/tools/workflow-builder/create-workflow-from-code.tool.ts:316`, `packages/cli/src/modules/instance-ai/instance-ai.adapter.service.ts:1416`, `packages/cli/src/modules/chat-hub/chat-hub-workflow.service.ts:161,238,1883`.
- **Implikasi TB-03**: (a) target "per-branch" = jalur `'v1'`; (b) mesin eksekusi 2.39.0 memperlakukan nilai selain `'v1'` (termasuk settings kosong/`undefined` thd workflow JSON mentah) sbg legacy breadth-first — jadi paritas membutuhkan keputusan eksplisit di sisi impor/kanonik kita (default-kanonik `'v1'` vs hormati ketiadaan sbg legacy). Fakta di sini hanya perilaku mesin + praktik generator; normalisasi sisi save/import n8n (editor) belum diverifikasi (butuh telusur cli/editor-ui lebih lanjut bila kriteria menuntut).

## 5. Sumber & batas metode
- Semua path relatif thd `packages/` dalam clone kanonik n8n-2.39.0. Baris: file:line persis sesuai ekstraksi 2026-09-10.
- Daftar param = nama level-atas properti (string literal); duplikat nama = varian display sub-bentuk. Struktur dalam (opsi collection/fixedCollection, tipe kondisi If, mode Merge v3/HTTP v4) belum dirinci di sini — rincian dibaca langsung di file yang dirujuk saat implementasi.
- Tidak ada verifikasi runtime (tanpa node_modules, n8n tidak dijalankan).
- Konsumen: TB-03 (node If/Merge/HTTP Request + executionOrder), juga koreksi data param_schema rosetta bila re-scan dilakukan (setelah izin/tiket).
