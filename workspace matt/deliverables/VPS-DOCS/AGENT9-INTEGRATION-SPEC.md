# AGENT9-INTEGRATION-SPEC — Codegen OpenAPI (arah B), Webhook Engine, & Integrasi Katalog Node

**Penulis:** agent9 (Integration & Node Ecosystem Converter, ROLE_INTEGRATION)
**Tanggal:** 2026-09-09
**Versi:** v0.5 (DRAFT untuk review matt/fern/agent4/agent5 — bukan keputusan)
**Riwayat:** v0.1 awal. v0.2 (2026-09-09): verifikasi upstream docs.n8n.io (Webhook node) + statistik korpus 171 template (26 file webhook) + §2.4.1 race-resume & aktivasi E-INGRESS-COLLISION (RE: agent1 #515) + gate INTEG-06 + §7 tabel hasil verifikasi. v0.3 (2026-09-09): verifikasi SOURCE upstream `packages/nodes-base/nodes/Webhook/Webhook.node.ts` (version [1,1.1,2,2.1] default 2.1 = cocok korpus; responseMode default onReceived; IP-allowlist 403 eksak; path-param → webhookId prepend; auth ke-5 n8nOAuth2; sensitiveOutputFields) + dokumen turunan baru: `hub-intel-types.json` (katalog sumber Hub) & `AGENT9-W3-HUB-INGRESS.md`. v0.4 (2026-09-09): prefiks endpoint terverifikasi (env `N8N_ENDPOINT_WEBHOOK/WEBHOOK_TEST` + temuan `WEBHOOK_WAIT=webhook-waiting` utk resume) + kebijakan kolisi registrasi n8n terverifikasi (konsisten E-INGRESS-COLLISION) + katalog naik schema_version 2 (adopsi SEC-HUB-01 agent10: pin.expected_sha256 + verified-tofu/anchored).
**Mandat:** Pengumuman fern #452 (ROLE_INTEGRATION) + Core Memory L0: *"Spesialis generator OpenAPI, konverter node community, deklarasi skema n8n, dan webhook ingestion. Aturan wajib: paritas 100% skema n8n, opaque node parsing, zero credential leaks."*
**Dokumen acuan:**
- PRD-1-N8N-ANALYSIS.md §7.2 (INodeProperties), §7.3 (Declarative HTTP routing), §7.4 (kategori node), §7.5 (Credential types), §18.4 (694 node / 445 credential type terverifikasi)
- PRD-2-RUST.md §3.2 (struktur crate), §7.2 (ResourceHint), §7.4 (Codegen OpenAPI), §10.1 (tiga tingkat kompatibilitas), §12 FASE 2 (prototipe codegen, gate keluar)
- AGENT4_SCHEMA_COMPILER_SPEC.md §1 (tabel pemisahan arah A/B), §2 (tabel tipe IR), §4 (aturan mapping), §9 (default SideEffect — F-C2), §10 (registry per (kind,typeVersion))
- AGENT4-ADVERSARIAL-REVIEW.md F-C2 (translasi skema & SideEffect), F-C5 (kredensial node hasil generate)
- AGENT1_CORE_ENGINE_SPEC.md §4 (state machine task), §6.2.1 (determinism record), §6.3 (tabel wait_state)
- AGENT2_STORAGE_SPEC.md (kolom mode eksekusi `webhook`)
- AGENT4-NODE-ALIAS-DEPRECATION.md (registry alias)
- ANALISIS-KORPUS-NODE.md (metrik coverage jujur), DEVIATION-CATALOG.md (disiplin deviasi)

**Status kepatuhan WORK FREEZE (#386):** dokumen ini TIDAK memuat satu baris kode Rust. Semua artefak implementasi menunggu keputusan K-1/K-2/K-3 pemilik proyek.

---

## 0. Posisi & batas kepemilikan (mengisi slot yang kosong)

AGENT4_SCHEMA_COMPILER_SPEC §1 menetapkan dua arah codegen yang tidak boleh dicampur, dengan baris arah B **tanpa pemilik** ("Fase 2 prototipe, PRD-2"). Dokumen ini mengisi slot itu dan memetakan batas wilayah agar tidak ada dual-ownership:

| Wilayah | Pemilik | Catatan |
|---|---|---|
| Arah A: `INodeProperties`/`ParameterSchema` → validator/OpenAPI-doc/form | agent4 | AGENT4_SCHEMA_COMPILER_SPEC — tidak diubah dokumen ini |
| **Arah B: OpenAPI spec publik → NodeDescriptor + eksekusi HTTP deklaratif** | **agent9 (dokumen ini)** | crate `openapi-codegen` (generator) + `nodes-openapi` (hasil) — nama & posisi sudah dikunci PRD-2 §3.2 |
| Webhook node & respondToWebhook: **semantik eksekusi** (Waiting→Ready, wait_state, resume) | agent1 | AGENT1 §4, §6.3 — dokumen ini KONSUMSI, tidak mendefinisikan ulang |
| Webhook node & respondToWebhook: **ingress HTTP** (route registry, request masuk, plumbing respons) | **agent9 (dokumen ini)** | lapisan `api`/axum Fase 2 |
| Skema penyimpanan (DDL tabel) | agent2 | dokumen ini mengusulkan kebutuhan, bukan DDL final |
| Gate QA/keamanan lintas proyek | agent5 | gate INTEG-xx di §4 diajukan masuk kerangka QA agent5 |
| Semantik node AI/LangChain (136 dari 694) | agent7 (ROLE_AI_MCP) | dokumen ini hanya menyediakan jalur konversi, bukan semantik |

Konflik yurisdiksi → fern memutuskan (prosedur #aturan Pasal 5.1).

---

## 1. Codegen arah B: `openapi-codegen`

### 1.1 Pipeline lima tahap (build-time, bukan runtime — PRD-2 §3.2)

```
(1) INGEST   : OpenAPI 3.0.x / 3.1.x (JSON/YAML) dari registry spec ber-provenansi
(2) VALIDATE : validasi struktural + lint; gagal cepat dengan kode CG-E-xxx (§1.6)
(3) TRIAGE   : operasi HTTP → kandidat node (pengelompokan per tag/path-prefix → "resource")
(4) TRANSLATE: skema parameter (§1.2) + serialisasi (§1.3) + ResourceHint (§1.4)
                + kredensial (§1.5)
(5) EMIT     : sumber crate nodes-openapi + manifest verifikasi (§1.7);
               operasi ambigu DITOLAK — tidak pernah menghasilkan node rusak (R8)
```

Prinsip: **compiler adalah satu-satunya jalan** (agent4 §6: no hand-edit artefak; CI membangun ulang & diff). Node hasil generate = kode Rust terkompilasi (keunggulan vs n8n, PRD-2 §7.4), tapi **generate ≠ verified** (§1.7).

### 1.2 Tabel translasi skema OpenAPI → IR ParameterSchema

Menjawab F-C2 butir (1) — transformasi tipe yang belum terdefinisi di PRD-2 §7.4. Format mengikuti konvensi tabel agent4 §2 (status eksak, alasan eksplisit, tanpa TBD). IR = `ParameterSchema`/`ParameterField` kernel (padanan langsung `INodeProperties`, PRD-1 §7.2).

| # | Bentuk OpenAPI 3.x | IR tujuan | Status v1 | Catatan |
|---|---|---|---|---|
| 1 | `type: string` (tanpa enum/format) | `StringField` | ✅ SUPPORTED | `description` → description; `example` → placeholder |
| 2 | `type: string` + `enum` | `OptionsField(oneOf)` | ✅ SUPPORTED | `enum` boleh string; default dari `default` |
| 3 | `type: string` + `format: date-time` | `DateTimeField` | ✅ SUPPORTED | validasi parse ISO-8601 ketat (agent4 §2 #6) |
| 4 | `type: string` + format lain (`uuid`/`email`/`uri`/…) | `StringField` | ✅ SUPPORTED | format hanya jadi *hint* dokumentasi; validasi format DEFER (butuh verifikasi paritas perilaku n8n — catat di katalog deviasi saat tersentuh) |
| 5 | `type: integer` / `number` | `NumberField` | ✅ SUPPORTED | `minimum`/`maximum` (+`exclusive*`) → minValue/maxValue; NaN/Infinity dalam default → TOLAK |
| 6 | `type: boolean` | `BooleanField` | ✅ SUPPORTED | |
| 7 | `type: array`, `items: {type: string}` | `JsonField` (opaque) | ✅ SUPPORTED (opaque) | N1 #1038: ParameterKind belum punya varian string-array → sementara Json-opaque; varian kernel = TBD (agent1+matt) |
| 8 | `type: array`, `items: {type: number/integer/boolean}` | `JsonField` OPAQUE (bukan `MultiOptionsField` — ini array nilai, bukan pilihan berulang) | ✅ SUPPORTED (opaque) | sesuai aturan L0 *opaque node parsing*: array non-string dilewatkan apa adanya sebagai JSON |
| 9 | `type: array`, `items: {type: object}` | `JsonField` OPAQUE | ✅ SUPPORTED (opaque) | TIDAK digenerate `fixedCollection` (risiko semantik bersarang — agent4 §6); pengguna menulis JSON manual |
| 10 | `type: object` | `JsonField` OPAQUE | ✅ SUPPORTED (opaque) | idem; `additionalProperties` tidak diterjemahkan |
| 11 | `oneOf`/`anyOf` — SEMUA varian skalar-dengan-enum | `OptionsField` | ✅ SUPPORTED | union nilai enum; label = kombinasi |
| 12 | `oneOf`/`anyOf` — varian campuran/non-scalar | — | ❌ REJECT `CG-E-201` | kurasi manual, bukan node rusak |
| 13 | `allOf` | merge tanpa konflik | 🟡 CONDITIONAL | hanya bila hasil merge bebas konflik field; konflik → REJECT `CG-E-201` |
| 14 | `nullable: true` | field opsional | ✅ SUPPORTED | tidak mengubah tipe dasar |
| 15 | `$ref` | resolusi rekursif | ✅ SUPPORTED | `$ref` siklik → REJECT `CG-E-202`; `$ref` lintas-file → hanya jika file ada di registry provenansi |
| 16 | `required: [f]` di skema | `required: true` IR | ✅ SUPPORTED | required TANPA default → error saat GENERATE (bukan import; agent4 §2 meng-error-kan di import — untuk node hasil generate, error harus lebih dini: di build) |
| 17 | keyword tak dikenal / `x-*` (non-rate-limit) | — | ⚠️ WARNING | **anti silent-drop** (agent4 §4.1): dicatat di laporan build + entri katalog deviasi bila memengaruhi perilaku |

**Aturan lintas-baris** (mengadopsi agent4 §2, tanpa modifikasi semantik):
- `default` wajib tipe-konsisten dengan field → pelanggaran = gagal build.
- Nilai parameter boleh expression `{{...}}` — validator hanya cek bentuk; evaluasi = domain `expr-quickjs`.
- `displayOptions.show/hide` TIDAK di-generate dari OpenAPI (OpenAPI tidak punya kondisi tampilan); visibilitas bersyarat hanya muncul di node tulisan-tangan.

### 1.3 Aturan serialisasi parameter → request (F-C2 butir 2)

Output translasi = spesifikasi request deklaratif bentuk `routing.send.{type: query/body/header/path}` (PRD-1 §7.3) — satu-satunya jembatan arah A↔B (agent4 §4.3).

| Lokasi OpenAPI | `style` | Status v1 | Perilaku eksak |
|---|---|---|---|
| query | `form` (default) | ✅ | `explode=true`: `?a=1&a=2`; `explode=false`: `?a=1,2` |
| query | `spaceDelimited` | ✅ | `?a=1%202%203` |
| query | `pipeDelimited` | ✅ | `?a=1\|2\|3` |
| query | `deepObject` | ❌ REJECT `CG-E-203` | tidak ada padanan routing n8n; kurasi manual |
| path | `simple` (default) | ✅ | `/users/{id}` → substitusi eksak; label wajib cocok template |
| header | `simple` (default) | ✅ | |
| cookie | (semua) | ❌ REJECT `CG-E-204` | n8n routing tidak mengirim cookie — entri katalog deviasi saat tersentuh korpus |
| requestBody `application/json` | — | ✅ | body JSON; `Content-Type` eksak |
| requestBody `text/plain`, `application/x-www-form-urlencoded` | — | ✅ | |
| requestBody `multipart/form-data`, `image/*`, dll. binary | — | ❌ REJECT `CG-E-205` | butuh dukungan binary data-plane; DEFER sampai binary path didefinisikan (PRD-1 §8.2 `$binary`) |

Parameter path yang tidak terisi saat runtime → gagal **sebelum** request dikirim, error `E_PATH_PARAM_INCOMPLETE` (konsisten: SideEffect PUT/DELETE butuh path lengkap, §1.4).

### 1.4 ResourceHint & SideEffect (adopsi penuh agent4 §9 — tidak diduplikasi)

Tabel default SideEffect per metode HTTP di AGENT4_SCHEMA_COMPILER_SPEC §9 diadopsi **sebagai-isi** sebagai sumber kebenaran tunggal untuk arah B (status dokumen itu: keputusan matt — lihat OPEN-INTEG-2):

- GET/HEAD/OPTIONS → `Idempotent`
- PUT/DELETE → `Idempotent` **syarat seluruh parameter path terisi** — diverifikasi per-operasi saat build; syarat tak terpenuhi → turun ke `NonIdempotent` + warning
- POST/PATCH → `NonIdempotent`
- ambigu → **build GAGAL**, kurasi manual

Tambahan khusus arah B (bukan tumpang-tindih, melengkapi):
- `weight`: heuristik — response `application/json` → `Batch`; response binary/streaming → `Streaming`; tak diketahui → `Batch` + catatan.
- `max_concurrency`: dari ekstensi `x-ratelimit-*`/`x-rate-limit-*` bila ada (nilai paling ketat); tidak ada → `None` (default engine).

### 1.5 Kredensial node hasil generate (menjawab F-C5 — belum terjawab di dokumen mana pun)

**Kontrak (mengadopsi usul agent4 F-C5 verbatim, diperketat):**
1. GeneratedNode WAJIB memakai kredensial via `NodeDescriptor.credentials` — tidak ada pengecualian.
2. Refresh token hanya via crate `auth`/kernel — trait `CredentialProvider` (kernel `context.rs:389`, TERVERIFIKASI #1038; sebelumnya tertulis `CredentialRefresher` = fiktif, DICABUT) — DILARANG menyimpan/menyegarkan token di dalam kode node hasil generate.
3. Pelanggaran 1-2 = **kegagalan build** (lint pada artefak generate), bukan review manual.
4. Segala rahasia (key, token) tidak pernah boleh muncul di URL, log, pesan error, atau snapshot differential — aturan L0 *zero credential leaks*; diuji gate INTEG-05 (§4).

**Pemetaan `securitySchemes` OpenAPI → tipe kredensial:**

| `securitySchemes` | Tindakan | Status v1 |
|---|---|---|
| `type: apiKey` (`in: header`) | petakan ke tipe kredensial generik header (padanan n8n `httpHeaderAuth`) | ✅ auto |
| `type: apiKey` (`in: query`) | petakan ke tipe generik query (padanan n8n `httpQueryAuth`) | ✅ auto |
| `type: http`, `scheme: basic` | padanan n8n `httpBasicAuth` | ✅ auto |
| `type: http`, `scheme: bearer` | padanan n8n `httpHeaderAuth` (header `Authorization: Bearer <token>`) | ✅ auto |
| `type: http`, `scheme: digest` | padanan n8n `httpDigestAuth` | 🟡 CONDITIONAL — [VERIFIKASI-UPSTREAM: dukungan digest di routing deklaratif n8n] |
| `type: oauth2` / `openIdConnect` / `mutualTLS` | **TIDAK di-generate otomatis** → daftar "perlu kurasi manual" (spec tidak memuat alur OAuth — PRD-2 §7.4: sisi credential tidak bisa digenerate) | ❌ auto, ✅ kurasi |
| operasi TANPA `security` (publik) | node tanpa kredensial | ✅ auto |

Nama tipe generik mengikuti nama *predefined credential types* n8n demi paritas impor workflow yang memakai kredensial generik — `[VERIFIKASI-UPSTREAM: daftar nama persis dari repo n8n]` sebelum implementasi (prinsip paritas 100% skema, L0).

Spesifikasi OpenAPI yang mensyaratkan OAuth2 pada operasi yang ingin dipakai → operasi masuk daftar kurasi; node tetap bisa di-generate untuk operasi lain (keputusan per-operasi, dicatat di manifest §1.7).

### 1.6 Rejection rules — kode error build (R8: node rusak lebih buruk dari tidak ada)

Semua kegagalan = **gagal build** dengan pesan `spec:<file> operation:<operationId> kode:<CG-E-xxx> alasan:<eksak>`. Kode terdaftar (dapat bertambah, tidak boleh digunakan ulang):

| Kode | Kondisi |
|---|---|
| `CG-E-101` | dokumen bukan OpenAPI 3.0.x/3.1.x valid (parse/struktural) |
| `CG-E-102` | `servers` ber-template variable tanpa `default`/`enum` |
| `CG-E-103` | `operationId` hilang/duplikat dalam scope node yang sama dan tidak dapat diderivasi deterministik dari method+path |
| `CG-E-201` | skema parameter tidak dapat dipetakan eksak (§1.2 baris 12-13) |
| `CG-E-202` | `$ref` siklik / lintas-file tak terdaftar |
| `CG-E-203`–`205` | serialisasi tidak didukung (§1.3) |
| `CG-E-301` | `securityScheme` yang dipakai operasi tidak terpetakan (§1.5) |
| `CG-E-302` | default bernilai non-finit / tak konsisten tipe |

**Statistik kejujuran wajib di output build:** `ops_total, ops_emitted, ops_rejected{kode}`, `nodes_emitted`. Klaim publik hanya boleh memuat `nodes_emitted` **yang verified** (§1.7). Mengutip angka generate tanpa konteks reject = pelanggaran disiplin PRD-1 §18.4 ("setiap angka punya satuan + perintah reproduksi").

### 1.7 Verified nodes (PRD-2 §7.4 mitigasi 3-4: daftar "verified", jangan klaim jumlah)

- Direktori `verifications/<node>.json`: `{spec_sha256, operations, test: {mode: mock|live, cases, output_snapshots}, verified_by, verified_at}`.
- `verified_by` manusia atau agent dengan bukti reproduksi — atribusi tool memakai identitas akun (catatan ERR-011: sampai fix `getpass`, atribusi tidak bukti otentik — verified wajib disertai perintah reproduksi yang bisa dijalankan siapa pun).
- Node ter-generate tapi belum verified: tersedia dengan penanda status terpisah; TIDAK dihitung dalam klaim jumlah.
- Mulai dari 5-10 spec berkualitas tinggi (Stripe/GitHub/Slack — PRD-2 §7.4 mitigasi 1; daftar final = keputusan OPEN-INTEG-5).

---

## 2. Webhook engine (ingress HTTP → eksekusi)

### 2.1 Antarmuka terhadap spec agent1 (tidak mendefinisikan ulang)

Dokumen ini hanya memiliki **lapisan ingress**: menerima request HTTP, mencocokkan rute, membuat eksekusi, dan mem-lumbing respons `respondToWebhook`. Semua semantik eksekusi mengikuti agent1: transisi `Running → Waiting` / `Waiting → Ready` (AGENT1 §4), tabel `wait_state(execution_id, node_id, resume_key, spill_path)` (§6.3; rename `payload_ref`→`spill_path` di-mirror dari spec inti agent1 2b10c733), determinism record berisi `webhook_urls` (§6.2.1), mode eksekusi `webhook` di storage agent2 (kolom `mode`).

### 2.2 Model rute

Kebutuhan (DDL final = wilayah agent2; ini usulan kebutuhan, bukan keputusan skema):

```
webhook_routes(workflow_id, node_id, method, path, auth_type,
               active, PRIMARY KEY (workflow_id, node_id, method))
path_uniq UNIQUE (method, path) WHERE active   -- kebijakan kolisi §2.3
```

- **Bentuk URL — dua URL per node (TERVERIFIKASI v0.2, docs.n8n.io Webhook node + Workflow development):** sistem test/produksi: URL **test** ter-register saat "Listen for Test Event" dan **aktif 120 detik**; URL **produksi** ter-register saat workflow di-publish. **[v0.4 — docs "Endpoints env vars"]:** prefiks eksak TERVERIFIKASI: `N8N_ENDPOINT_WEBHOOK` default **`webhook`**, `N8N_ENDPOINT_WEBHOOK_TEST` default **`webhook-test`** → `/webhook/<path>` & `/webhook-test/<path>`; keduanya configurable (padanan env kita = keputusan implementasi). **Temuan baru: `N8N_ENDPOINT_WEBHOOK_WAIT` default `webhook-waiting`** — endpoint KETIGA khusus resume Wait-node (`$execution.resumeUrl`); konsekuensi utk §2.4.1: resume punya namespace rute sendiri, terpisah dari rute webhook biasa (desain kita: `resume_key` di bawah prefiks serupa).
- **Path (TERVERIFIKASI v0.2):** default n8n = **path acak** per node workflow (docs: "randomly generated webhook URL path"); pengguna bisa menamai path manual, termasuk **path parameter** `:variable` dengan format `/:variable`, `/path/:variable`, `/:variable/path`, `/:v1/path/:v2`, `/:v1/:v2` → nilai tersedia di `params`. **[v0.3 — SOURCE Webhook.node.ts]:** jika path memakai segmen dinamis, n8n me-**prepend `webhookId` ke depan path** ("If dynamic values are set 'webhookId' would be prepended to path") — padanan kita: prefix deterministik `(workflow_id, node_id)` berperan sebagai webhookId; termasuk DEV-A9-1. Kita menurunkan path **deterministik** dari `(workflow_id, node_id)` + path eksplisit bila pengguna menamainya. Alasan: `webhook_urls` masuk determinism record (agent1 §6.2.1) — path acak merusak replay deterministik. Konsekuensi: URL hasil re-import workflow n8n berbeda → **kandidat deviasi DEV-A9-1** (dampak L3: `$execution.resumeUrl`). Bukti korpus: 21/27 node webhook punya `webhookId`, 27/27 path diset (contoh nama manusiawi: `support-chat`, `steam`, `scrape-agent`, `fromsyncro`). Reproduksi: `python3 /opt/agent-workspace/qa/webhook_corpus_stats.py`.
- **Method (TERVERIFIKASI v0.2):** dukungan n8n = DELETE, GET, HEAD, PATCH, POST, PUT. Korpus: POST 19/27 eksplisit, 8/27 absen (= default node). Method tak terdaftar → 404 (bukan 405) demi paritas `[VERIFIKASI-UPSTREAM: perilaku eksak n8n method-tak-terdaftar]`.

### 2.3 Siklus hidup rute & kebijakan kolisi

- Aktivasi workflow → register semua rute node Webhook aktif; deaktivasi → unregister (idempotent).
- Dua workflow aktif mengklaim `(method, path)` sama → **aktivasi yang kedua GAGAL** dengan error eksak menyebut pemilik rute pertama. Tiebreak deterministik (urutan aktivasi), tidak ada silent override.
- Register ulang rute yang sama oleh workflow yang sama = no-op.

### 2.4 Request → eksekusi

1. Cocokkan `(method, path)` → rute aktif; tak ada → 404.
2. Batas ukuran payload: **default 16 MB — paritas n8n TERVERIFIKASI v0.2** (docs Webhook node; di n8n konfigurabel via env `N8N_PAYLOAD_SIZE_MAX`; padanan env kita = keputusan implementasi).
3. Autentikasi rute: `none | basic | header | jwt` — **TERVERIFIKASI v0.2** (docs: Basic auth, Header auth, JWT auth, None). **[v0.3 — SOURCE]** ada opsi ke-5 **`n8nOAuth2`** (Webhook v2.1+; dua-langkah: bearer token → resolve ke user n8n → eksekusi atas nama user itu, termasuk merge kredensial privatnya) — DI LUAR cakupan v1 kita (butuh subsistem user-auth engine); korpus 0 pemakaian; bila tersentuh → kandidat deviasi eksplisit. Korpus: none 23, headerAuth 3, basicAuth 1, JWT 0, n8nOAuth2 0. Kegagalan → 401 tanpa membocorkan nilai kredensial; perbandingan waktu-konstan.
4. Buat eksekusi mode `webhook` (agent2 `mode` enum) dengan payload `{body, headers, query, params}` tersedia ke ekspresi node — **bentuk kunci TERVERIFIKASI v0.2** (docs "Only Run If": `$json` = `{ body, headers, params, query }`).
5. `responseMode` (paritas n8n — **TERVERIFIKASI v0.2**, 4 nilai): `onReceived` (Immediately; default UI) → respons langsung; `lastNode` (When Last Node Finishes) → respons = output node terakhir; `responseNode` (Using 'Respond to Webhook' Node) → menunggu `respondToWebhook`; `streaming` (Streaming response) → streaming real-time, butuh node pendukung streaming. Korpus: absen/default 15, responseNode 7, lastNode 5, streaming 0. Timeout `responseNode`/`lastNode` → respons gagal standar `[VERIFIKASI-UPSTREAM: nilai timeout persis n8n]`.
6. Resume (Wait node mode webhook / webhook callback): request masuk cocok `resume_key = (execution_id, node_id)` → transisi `Waiting → Ready` via jalur agent1; payload disimpan lewat data-plane (`spill_path`, rename mirror agent1 2b10c733), tidak dibawa di kolom JSON.

#### 2.4.1 Semantik resume: single-use, first-wins (RE: agent1 #515 — keputusan semantik agent9)

- `resume_key` bersifat **single-use (exactly-once)**: klaim = atomic `UPDATE task SET status='Ready' WHERE (execution_id, node_id) AND status='Waiting'`; rowcount=1 = pemenang.
- Request kedua — kalah race ATAU datang setelah `Waiting` berakhir (konsumsi/timer) → **404 + metric `ingress_resume_duplicate_total`**; TIDAK error engine, TIDAK pernah dua transisi `Waiting → Ready` (transisi ilegal §4 agent1 tetap mustahil secara konstruksi).
- Kolisi aktivasi rute → kode **`E-INGRESS-COLLISION`** (slot RESERVED agent1 — AKTIF, lihat #558): kegagalan di layer API (HTTP 409, bukan error task), menyebut `(method, path)` + pemilik rute pertama; register-ulang idempotent oleh workflow sama = no-op. **[v0.4 — docs "Common issues" Webhook node]:** kebijakan n8n TERVERIFIKASI: *"n8n only permits registering one webhook for each path and HTTP method combination"* — penolakan terjadi SAAT REGISTRASI dengan pesan "path and method already in use" (bukan runtime) — konsisten penuh dengan semantik E-INGRESS-COLLISION kita. Data point tambahan: n8n Cloud (Cloudflare) memutus respons >100 detik dengan 524 — angka acuan timeout respons `responseNode`/`lastNode`.
- `[VERIFIKASI-UPSTREAM: perilaku n8n saat webhook-resume duplikat & method tak terdaftar (kode status 404 vs 405)]` — 2 marker tersisa; namespace endpoint `webhook-waiting` (v0.4) sudah teridentifikasi, tinggal perilaku eksaknya di kode cli.

#### 2.4.2 Opsi node — daftar paritas (TERVERIFIKASI v0.2 dari docs; cakupan fase ditentukan roadmap)

Allowed Origins/CORS (default `*`) · Binary Property (hanya POST/PATCH/PUT) · Ignore Bots · IP(s) Allowlist (pelanggaran → **403**) · Only Run If (false → **200 tanpa eksekusi**; gagal evaluasi → warning + lolos) · Raw Body · Response Code/Content-Type/Data/Headers · Property Name (hanya lastNode + First Entry JSON) · No Response Body. Keamanan tambahan n8n ≥1.103.0: respons HTML dibungkus `<iframe>` sandbox — diadopsi sebagai paritas keamanan (kandidat gate INTEG-05).

### 2.5 Keamanan (selaras agent5)

- **Zero credential leak:** nilai auth rute tidak pernah masuk log/error/metrics; gate INTEG-05.
- Path: segmen tetap **+ segmen parameter `:variable`** (paritas n8n — TERVERIFIKASI v0.2, format `/:variable`, `/path/:variable`, dst.); dinormalisasi, tanpa glob/traversal; pencocokan: exact-match segmen demi segmen, nilai `:variable` → `params`. Interaksi dua rute beririsan (mis. `/a/b` vs `/a/:x`) → exact menang, deterministik `[VERIFIKASI-UPSTREAM: aturan konflik n8n utk path berparameter]`.
- Rate limit per rute (padanan `max_concurrency` di tingkat ingress); 429 saat terlampaui.
- Eksekusi webhook TIDAK mewarisi kredensial workflow lain; lingkup kredensial = `NodeDescriptor.credentials` node itu saja (PRD-2 §11).
- Payload disimpan sebagai data execution — retensi mengikuti PRD-2 §5.3 (bukan keputusan dokumen ini).

### 2.6 Deviasi & determinisme yang didaftarkan

| ID kandidat | Isu | Dampak |
|---|---|---|
| DEV-A9-1 | Path webhook deterministik vs `webhookId` acak n8n | L3: `$execution.resumeUrl` beda saat re-import; L1/L2 tidak terdampak |
| DEV-A9-2 | Cookie/header tertentu di-drop? — belum ada bukti; hanya akan didaftarkan BILA uji 3× menunjukkan perbedaan | — |

Semua entri final wajib mengikuti DEVIATION-CATALOG §2 (deterministik 3×, `case_id`, versi korpus, hash build, perintah reproduksi; entri tanpa itu DITOLAK QA).

---

## 3. Integrasi katalog 694 node (Node Ecosystem Converter)

### 3.1 Angka yang dipakai (PRD-1 §18.4 — tidak mengutip ulang tanpa sumber)

> n8n 2.39.0: **694 node bawaan (558 umum + 136 AI/LangChain), 445 credential type, 92 paket.** MVP 26 node = 3,7% katalog. Setiap pengutipan angka wajib menyertai sumber + perintah reproduksi.

### 3.2 Katalog = data ber-provenansi, bukan klaim

`catalog.json` (di-generate dari repo upstream n8n; provenansi: commit sha + perintah reproduksi; konvensi sama dengan corpus README-CORPUS):
```
{ "schema_version": 1, "source_commit": "<sha>", "nodes": [
  { "type": "n8n-nodes-base.webhook", "typeVersions": [1,2], "category": "trigger",
    "declarative": false, "credentials": ["httpBasicAuth", ...],
    "deprecated": false, "alias_of": null } ] }
```
Fungsi katalog: **pengenalan L1** (import 100% parse — PRD-2 §10.1) + peta kerja konversi. Katalog TIDAK menjanjikan eksekusi — pemisahan "kompatibilitas impor vs kapabilitas eksekusi" PRD-2 §7.3.4 diadopsi penuh.

### 3.3 Lima jalur konversi (triage 694)

| # | Segmen | Jalur | Pemilik |
|---|---|---|---|
| 1 | Node deprecated (function/cron/…) | registry alias agent4 (tidak diduplikasi) | agent4 |
| 2 | Node deklaratif (routing-based) | arah A: kompilasi deklarasi → ParameterSchema | agent4 |
| 3 | Node programmatic yang API-nya punya OpenAPI spec publik berkualitas | **arah B: codegen (dokumen ini §1)** | agent9 |
| 4 | Node programmatic tanpa spec | antrean port manual, diprioritasi frekuensi korpus (ANALISIS-KORPUS-NODE) | executor/node-writer (Fase 1+) |
| 5 | 136 node AI/LangChain | semantik = agent7; saya hanya menyediakan antarmuka konversi | agent7 |

Node komunitas (ekstensi `@n8n/…`/community) masuk jalur yang sama via manifest paket + wajib status `verified` terpisah sebelum tersedia default (keamanan ekosistem — selaras PRD-2 §11).

### 3.4 Metrik kejujuran (mengadopsi ANALISIS-KORPUS-NODE temuan 2, mengikat)

1. **Dilarang** mengutip coverage instance node ("63% node didukung") sebagai indikator kesiapan — satu-satunya metrik produk = **% workflow korpus yang jalan penuh**.
2. Coverage import (L1) dan coverage eksekusi (L2/L3) **selalu dilaporkan terpisah**.
3. Angka katalog bersifat snapshot bertanggal + sumber commit; "n8n terus menambah node, targetnya bergerak" (PRD-2 §7.3.1) — katalog meregenerasi via CI, diff di-review.

---

## 4. Gate verifikasi (falsifiable — diajukan masuk kerangka QA agent5)

Semua gate menunggu WORK FREEZE dicabut; definisi sekarang agar desain bisa diserang sebelum dikode (murah sekarang, mahal kemudian — agent4 F-C4).

| Gate | Kriteria PASS (falsifiable) | Reproduksi |
|---|---|---|
| INTEG-01 golden specs | ≥5 spec berkualitas (daftar OPEN-INTEG-5) → snapshot `NodeDescriptor` ter-generate; CI rebuild + diff = 0 beda | `cargo run -p openapi-codegen -- --spec tests/specs/<x>.json --emit /tmp/out && diff -ru tests/golden/<x> /tmp/out` |
| INTEG-02 rejection corpus | ≥20 kasus spec rusak/ambigu → SEMUA gagal build dengan kode CG-E eksak; 0 node ter-emit | `cargo test -p openapi-codegen rejection_corpus` (kasus di-repo, bukan hardcode hasil) |
| INTEG-03 translation pairs | tiap baris §1.2 SUPPORTED punya pasangan terima/tolak + `output_eq` byte-identical (3 level gate agent1 #445 + agent4 #448: reject_accept, output_eq) | `cargo test -p openapi-codegen translation_pairs` |
| INTEG-04 webhook e2e | register rute → POST → eksekusi → respons `respondToWebhook` **byte-identical vs n8n asli** pada workflow webhook deterministik korpus — tersedia **26 file (15% korpus, 20 node respondToWebhook)**; angka terukur v0.2 via `python3 /opt/agent-workspace/qa/webhook_corpus_stats.py` | skrip differential di `scripts/compat-corpus/` (PRD-2 §3.2) |
| INTEG-06 race-resume (RE: agent1 #515) | 2 request paralel cocok `resume_key` sama → **tepat SATU** transisi `Waiting → Ready`; yang kalah menerima 404; metric `ingress_resume_duplicate_total` **+1** (hanya duplikat pihak-kalah yang tercatat; pemenang bukan duplikat — klarifikasi #1049) | uji beban paralel (2×curl serentak) pada skrip differential + inspeksi tabel task |
| INTEG-05 zero-credential-leak | (a) grep artefak generate: 0 literal rahasia; (b) runtime: request/log/metrics di-scan pola nilai kredensial yang ditanam = 0 kebocoran (selaras SEC-MCP-01 agent5) | `scripts/check-secrets.sh` + differential harness |

Prasyarat lingkungan (pelajaran agent5 #461): build hanya di direktori milik akun runner; toolchain rustup per-akun; jangan build besar sampai vdb ter-mount (#455).

---

## 5. Keputusan yang diminta (OPEN-INTEG-1..6 — untuk matt/fern, rekomendasi disertakan)

1. **Kepemilikan**: konfirmasi agent9 = arah B + webhook ingress (§0). *Rekomendasi: setuju, sesuai ROLE_INTEGRATION.*
2. **Tabel SideEffect §9 agent4**: jadikan mengikat untuk codegen B. *Rekomendasi: setuju (sudah menjawab F-C2 dengan syarat PUT/DELETE diverifikasi).*
3. **Nama tipe kredensial generik**: pakai nama predefined n8n (`httpHeaderAuth` dll.) demi paritas impor — atau nama sendiri. *Rekomendasi: nama n8n + verifikasi upstream daftar persisnya.*
4. **Determinisme path webhook** (DEV-A9-1): path deterministik `(workflow_id,node_id)` vs acak-per-node ala n8n. *Rekomendasi: deterministik + entri deviasi.*
5. **Daftar 5-10 spec golden pertama** (INTEG-01). *Rekomendasi: Stripe, GitHub, Slack + 2-7 ditentukan berdasarkan (a) kualitas spec, (b) frekuensi node padanan di korpus.*
6. **Cakupan REJECT v1** (§1.3: deepObject/cookie/multipart). *Rekomendasi: REJECT semua + katalog deviasi saat tersentuh — jangan hasilkan node setengah-benar.*

---

## 6. Kepatuhan freeze & langkah berikutnya

- Dokumen ini: 0 baris Rust, 0 perubahan file sistem, 0 perubahan file agent lain. Home sendiri di-chmod 0700 (rekomendasi agent5 #462).
- Setelah K-1/K-2/K-3 diputuskan pemilik & freeze dicabut: urutan kerja = (1) tuntaskan sisa tanda `[VERIFIKASI-UPSTREAM]` (5/8 selesai di v0.2 — lihat §7), (2) prototipe Fase 2 sesuai gate PRD-2 §12 ("≥5 node lolos test API nyata"), (3) gate INTEG-01..06 dijalankan berurutan.
- Checkpoint working memory dicatat di L1; temuan desain yang terbukti akan masuk L2 (`mem learn`).

---

## 7. Hasil verifikasi v0.2 (tabel bukti — ZERO FAKE WORK)

Sumber: docs.n8n.io halaman "Webhook node" + "Workflow development" (diakses 2026-09-09) dan statistik korpus via `python3 /opt/agent-workspace/qa/webhook_corpus_stats.py` (171 file, read-only).

| Klaim v0.1 | Hasil | Bukti |
|---|---|---|
| Dua URL test/produksi | ✅ TERVERIFIKASI + test aktif **120 detik** | docs Webhook node + Workflow development |
| Payload default 16 MiB (usulan) | ✅ jadi PARITAS: n8n default **16 MB**, env `N8N_PAYLOAD_SIZE_MAX` | docs Webhook node ("Webhook max payload") |
| Auth `none/basic/header` | ✅ + **JWT** (4 opsi) | docs "Supported authentication methods" |
| Struktur `$json` webhook | ✅ `{body, headers, params, query}` | docs opsi "Only Run If" |
| responseMode 3 nilai | ✅ 4 nilai: + **streaming response** (butuh node pendukung) | docs "Respond" |
| Path acak default n8n | ✅ "randomly generated" + dukungan path param `:variable` | docs "Path"; korpus 21/27 `webhookId` |
| Opsi node (CORS, Ignore Bots, dll.) | ✅ daftar lengkap §2.4.2 (IP Allowlist→403; Only-Run-If→200-skip; HTML→iframe sandbox ≥1.103.0) | docs "Node options" |
| Prefiks `/webhook/` & `/webhook-test/` eksak, 404-vs-405, prioritas konflik path-param, duplikat-resume n8n | ✅ **prefiks TERVERIFIKASI v0.4** (env `N8N_ENDPOINT_WEBHOOK=webhook`, `WEBHOOK_TEST=webhook-test`, + `WEBHOOK_WAIT=webhook-waiting` utk resume Wait-node); ⏳ 3 tersisa: 404-vs-405, prioritas path-param, dup-resume | docs "Endpoints" + "Common issues" |
| **[v0.3 SOURCE]** `version: [1, 1.1, 2, 2.1]`, `defaultVersion: 2.1` | ✅ cocok persis sebaran korpus (1:15, 2:6, 2.1:3, 1.1:3) — F-C3 | Webhook.node.ts |
| **[v0.3 SOURCE]** `responseMode` default = `onReceived` | ✅ (`getNodeParameter('responseMode','onReceived')`) | Webhook.node.ts |
| **[v0.3 SOURCE]** IP-allowlist → 403 `'IP is not allowed to access the webhook!'`; ignoreBots → 403 | ✅ eksak (teks respons tercatat) | Webhook.node.ts |
| **[v0.3 SOURCE]** auth `n8nOAuth2` (v2.1+) ada di source, belum di docs | ✅ dicatat — di luar cakupan v1 | Webhook.node.ts |
| **[v0.3 SOURCE]** `sensitiveOutputFields: ['headers.authorization','headers.cookie']` (redaksi bawaan n8n) | ✅ memperkuat INTEG-05 | Webhook.node.ts |

**Statistik korpus (26 file / 15%):** 27 node webhook — typeVersion 1:15, 2:6, 2.1:3, 1.1:3 (konfirmasi dimensi F-C3); httpMethod POST 19 / absen 8; responseMode absen 15 / responseNode 7 / lastNode 5; auth none 23 / header 3 / basic 1; respondToWebhook: 20 node di 8 file.

*Dokumen ini dapat diserang; itu tujuannya. — agent9*
e.ts |

**Statistik korpus (26 file / 15%):** 27 node webhook — typeVersion 1:15, 2:6, 2.1:3, 1.1:3 (konfirmasi dimensi F-C3); httpMethod POST 19 / absen 8; responseMode absen 15 / responseNode 7 / lastNode 5; auth none 23 / header 3 / basic 1; respondToWebhook: 20 node di 8 file.

*Dokumen ini dapat diserang; itu tujuannya. — agent9*

## §1.6 Addendum konsensus RFC #936 (serapan review agent4 #1017 — v0.5.1)

1. **[R-A DITUTUP — batas IR vs tipe kernel, EKSPLISIT]:** IR openapi-codegen = model internal **build-time (private)**, bentuk bebas, TIDAK pernah dibaca runtime. **EMIT final = tipe kernel NYATA**: seluruh kode hasil-generate di `crates/nodes-openapi/` memakai `NodeDescriptor` + `ParameterSchema` kernel (dari sisi agent1). **TIDAK ada vocab tipe paralel yang bocor ke runtime.**
2. **[R-B DIKONFIRMASI — pemetaan SideEffect default per-metode (agent4 §9):]** GET/HEAD/OPTIONS → `Idempotent`; PUT/DELETE → `Idempotent` (bersyarat, path-lengkap); POST/PATCH → `NonIdempotent`; override per-node via file `verifications/<node>.json`.
3. **[NIT DITERIMA — manifest verifikasi diperluas]:** `{spec_sha256, hash_domain: "raw-bytes"|"canon-json"+versi, ops_emitted, ops_rejected{kode}}` — `hash_domain` eksplisit agar golden-diff INTEG-01 tak flaky (S4 #713); `spec_sha256` disimpan untuk audit-ulang (pola pin S3).

## §1.7 Addendum serapan review agent1 #1038 (CONSENSUS-ACK 2/2 — v0.5.2)

1. **[R-C DITUTUP]** `CredentialRefresher` (fiktif, 0-hit workspace) → diganti `CredentialProvider` (kernel `context.rs:389`, nyata, ada CT-suite; diverifikasi ulang oleh agent9 di pohon kanonik).
2. **[N1]** `StringArrayField` dicabup → baris §1.2#7 kini `JsonField` (opaque); varian kernel string-array = TBD berpemilik (agent1+matt).
3. **[N2] Legenda IR⇒ParameterKind eksplisit (dikoreksi #1049):** `StringField⇒kind::String`, `NumberField⇒kind::Number` (tak ada `IntegerField` di §1.2 — dicabut), `BooleanField⇒kind::Boolean`, `EnumField⇒kind::Options` (bukan `kind::Enum` — fiktif; `Options` = single-choice, `params.rs:194`), `JsonField⇒kind::Json` (termasuk array-of-string sampai N1 diputuskan).
4. **[N3]** `payload_ref` → `spill_path` (menemani `SpilledList.path`): spec inti agent1 SUDAH turun (2b10c733) — mirror SELESAI di dokumen ini (§2.1/§2.4) pada v0.5.3.
5. **[N4]** INTEG-06 dihitung **+1** — semantik race-resume: dari 2 request paralel, tepat 1 pemenang melanjutkan transisi `Waiting→Ready` (bukan duplikat) dan 1 duplikat pihak-kalah yang dihitung metric (klarifikasi #1049).
6. **[ENDORSE balik OPEN-INTEG-4 diterima]:** array `urls[]`/parameter-list DI-SORT kanonik sebelum masuk digest & identitas replay (order acak meracuni identitas replay — agent1 #1038).

## §1.8 Addendum mikro v0.5.3 (serapan verifikasi agent1 #1049)

1. N2-legenda dikoreksi: `EnumField⇒kind::Options` (`kind::Enum` fiktif — tidak ada di `ParameterKind`, `params.rs:188-200`); `IntegerField` dicabut (tak ada di §1.2).
2. N4-dirumuskan ulang + tabel §4 INTEG-06 diselaraskan: metric `ingress_resume_duplicate_total` = **+1** (duplikat pihak-kalah saja; pemenang bukan duplikat).
3. N3-mirror dieksekusi: `payload_ref`→`spill_path` di §2.1/§2.4 (spec inti agent1 2b10c733 sudah turun).
