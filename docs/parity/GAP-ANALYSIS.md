# GAP ANALYSIS — n8n asli vs n8n-rust v1.1

> **Tujuan:** memetakan **apa yang kurang** agar `n8n-rust` setara dengan n8n asli —
> per **halaman**, **fitur**, **fungsi**, **node**, **API**, dan **perilaku engine**.
> **Sumber primer:** repo `n8n-io/n8n` (master @ 2026-09-11), `docs.n8n.io/llms.txt`, runtime probe repo ini.
> **Tanggal audit:** 2026-09-11 · **Status:** hasil pemeriksaan (bukan opini) — semua angka bisa diverifikasi ulang lewat `docs/parity/verify-parity.sh`.

---

## 0. Ringkasan eksekutif

| | n8n asli | n8n-rust (repo ini) | Selisih |
|---|---|---|---|
| Versi | **2.38.6** (rilis 2026-09-10) | runtime melaporkan `0.9.0`, README menulis `v1.1.0` | tertinggal **~1 major + 38 minor** |
| Halaman/tab UI | **~60 route** + 52 feature-view | **1 halaman** (`/`), 6 item sidebar yang mayoritas toast | **~98% halaman hilang** |
| Node type terdaftar | **701** implementasi (558 `nodes-base` + 143 LangChain) | **19** di mock preview, **46** di server Rust (19 core + 27 "extended") | ~6% |
| Integrasi (direktori) | **308** | 27 "extended" (mayoritas mock/HTTP tipis) | ~9% |
| Credential type | **456** file credential | **12–13** | ~3% |
| Public REST API | **90 path** (OpenAPI v1) | **25 route** (mayoritas non-API: `/`, `/health`, `/ws/*`) | ~10% |
| Tabel database | **141** tabel SQLite (142 Postgres) | **0** — file JSON per entitas | tidak ada layer data |
| Test file | **6.911** | **95** unit test | ~1,4% |
| Komponen UI (Vue) | **1.069** | 1 file HTML (1.287 baris, 129 KB) | monolit vs design-system |
| Auth/RBAC | API key, user, MFA, SSO, LDAP, role+scope | **tidak ada** (0 referensi auth di server) | tidak ada |
| AI/LangChain | 143 node (agent, chain, memory, vector store, MCP, guardrails) | **0** (`langchain` = 0 match) | tidak ada |

**Lima temuan terbesar (urutan dampak):**

1. **Tidak ada layer halaman/routing.** `app.html` tidak punya router sama sekali (`history.pushState`/`location.hash` = 0 match).
   Item sidebar Projects/Templates/Variables/Settings hanya memunculkan toast. URL tidak bisa di-share, tombol back browser mati, deep-link mati.
2. **Tidak ada layer data & identitas.** n8n berjalan di atas 141 tabel + RBAC + API key + MFA; n8n-rust menyimpan JSON di folder, tanpa login. Ini yang memblokir *semua* fitur multi-user (Projects, Credentials sharing, Source control, Audit, Insights).
3. **Node & credential bukan "jumlah kecil", tapi beda orde besar.** Target realistis bukan mengejar 701 node, tapi menutup **daftar core node resmi (~62 node non-AI + 5 core AI)**; 41 di antaranya belum ada sama sekali.
4. **Semantik eksekusi belum n8n.** Tidak ada: error workflow, retry, sub-workflow (`Execute Sub-workflow`), pinning data, partial execution, binary data, queue mode/worker, resume `Wait` yang benar-benar menahan eksekusi.
5. **Klaim dokumen > realita.** `README.md` menulis "v1.1 Premium", badge "19 node types", `ACCURACY_COMPARISON.md` mengklaim "99% akurasi" dan UI "SAMA PERSIS"; hasil probe menunjukkan sebaliknya (mis. `/templates`, `/variables`, `/settings`, `/signin` → **404**).

**Estimasi realistis:** untuk "terasa seperti n8n" (bukan 1:1) perlu **3 fase** — lihat §10. Fase 0–1 mengubah repo dari *demo API* menjadi *aplikasi*.

---

## 1. Metodologi & bukti

| Sumber | Yang diambil | Cara |
|---|---|---|
| `gh api repos/n8n-io/n8n/git/trees/master?recursive=1` (9,4 MB, 28.518 blob) | jumlah node, credential, test, tabel, view | enumerasi `*.node.ts`, `*.credentials.ts`, `docs/generated/*-schema` |
| `packages/frontend/editor-ui/src/app/router.ts` + `features/**/views/*.vue` | daftar halaman | grep `path:` / `name: VIEWS.*` |
| `packages/cli/src/public-api/v1/openapi*.yml` | 90 path REST API | regex path/verb |
| `docs.n8n.io/llms.txt` + `.../core-nodes.md` | daftar resmi **core node** | index dokumentasi |
| Runtime probe lokal | perilaku nyata repo ini | `curl /health`, `/api/nodes`, 6 rute halaman, `grep` kapabilitas |

Reproduksi: `bash docs/parity/verify-parity.sh` (lihat lampiran §12).

---

## 2. HALAMAN (pages & route) — selisih terbesar

Legenda: ✅ setara · 🟡 sebagian/versi mini · 🟠 stub/mock · ❌ tidak ada

### 2.1 Editor & workflow

| Halaman n8n | Route n8n | n8n-rust | Catatan |
|---|---|---|---|
| Daftar workflow (grid/list, filter, sort, arsip, folder, tag, owner, last run) | `/` (WorkflowsView) | 🟡 | hanya list di sidebar kiri, tanpa grid/tab/sort/search/pagination |
| Editor canvas | `/workflow/new`, `/workflow/:id/:nodeId?` | ✅ | canvas, pan/zoom, minimap, edge bezier — ini bagian terkuat repo |
| Node Details View (NDV) | panel `/workflow/:id/:nodeId` | 🟡 | inspector inline 420px, bukan NDV modal ala n8n |
| Debug eksekusi per node | `/workflow/:id/debug/:executionId` | ❌ | tidak ada mode debug |
| Riwayat versi workflow + diff | `/workflow/:id/history/:versionId?` | ❌ | tidak ada |
| Template: cari/koleksi/detail/setup | `/templates`, `/collections/:id`, `/templates/:id`, `/templates/:id/setup` | ❌ | toast "Templates" saja |
| Resource center | `/resource-center` | ❌ | |
| Demo & demo diff | `/workflows/demo`, `/workflows/demo/diff` | ❌ | |
| Onboarding workflow | `/workflows/onboarding/:id` | ❌ | |
| Review workflow (workflow reviews) | `workflow-reviews/views/*` | ❌ | fitur baru 2.x |

### 2.2 Eksekusi, evaluasi, insight

| Halaman | Route | n8n-rust | Catatan |
|---|---|---|---|
| Daftar eksekusi global | `/executions` (ExecutionsView) | 🟡 | hanya list di sidebar, tanpa tabel kolom/status filter/retry/stop |
| Eksekusi per workflow | `/workflow/:id/executions` | 🟡 | tab header "Executions" → panel bawah, data dari `/api/executions` |
| Detail eksekusi | `/workflow/:id/executions/:executionId/:nodeId?` | ❌ | tidak ada drill-down, output per node tidak bisa dibuka ulang |
| Evaluations (test runs, compare collection) | `/workflow/:id/evaluation`, `.../test-runs/:runId`, `.../collections/:id/compare` | ❌ | tab "Evaluations" hanya label kosong |
| Insights (dashboard waktu tersimpan, run, failure rate) | `/settings/usage` + Insights | ❌ | |
| Logs | tab bawah | 🟡 | log WS sendiri, bukan log eksekusi resmi |

### 2.3 Aset & kolaborasi

| Halaman | n8n-rust | Catatan |
|---|---|---|
| Credentials (list, modal create/edit, sharing, test, transfer) | 🟡 (list + modal create; tanpa test/sharing/transfer) |
| Variables | ❌ (toast) |
| Projects (list, settings, members, variables, roles, project-role) | ❌ (toast) |
| Folders (nested, per project) | ❌ |
| Tags (filter, bulk) | ❌ |
| Collaborators & komentar/annotation tags | ❌ |
| Data Tables (tabel CRUD + grid) | ❌ |

### 2.4 AI (jalur baru n8n 2.x)

| Halaman | n8n-rust |
|---|---|
| Chat Hub | ❌ |
| Agents (list, builder, session, timeline) | ❌ |
| AI Assistant / Instance AI | ❌ |
| MCP Access (server triger + klien + registry) | ❌ |
| AI Gateway / connect | ❌ |

### 2.5 Settings & akun

| Group | Halaman n8n | n8n-rust |
|---|---|---|
| Personal | `/settings/personal`, `/settings/security`, `/settings/mfa` | ❌ |
| Instance | users, API keys, roles (instance + project), SSO (SAML/OIDC), LDAP, environments (source control git), external secrets, secrets providers, log streaming, encryption keys, community nodes, usage & plan, AI settings, n8n connect/gateway, migration report (5 view) | ❌ semua |
| Auth | `/signin`, `/signup`, `/signout`, `/setup` (owner), `/forgot-password`, `/change-password`, `/oauth/consent` | ❌ semua (server tidak punya konsep user) |
| Error/utility | `EntityNotFound`, `EntityUnAuthorised`, `ErrorView`, `LoadingView` | ❌ |

**Ringkas:** dari ~60 route UI n8n, repo ini punya **1** (`/`) + sebagian kecil sidebar. Tab header (Editor/Executions/Evaluations/Logs) ada, tapi 2 dari 4 kosong.

---

## 3. FITUR PLATFORM

| Fitur | n8n | n8n-rust | Gap yang harus ditutup |
|---|---|---|---|
| Penyimpanan | DB relasional (SQLite/Postgres, 141 tabel, migrasi terurut) | JSON file (`data/`) | layer DB + migrasi + indeks + transaksi |
| Auth user | email/password, sesi, cookie, MFA (TOTP + recovery), invite | ❌ | user store, hashing (argon2), sesi, middleware |
| API key (`X-N8N-API-KEY`) | ada, scoped | ❌ | tabel + middleware + UI |
| RBAC | role global + role per project + scope granular (dekorator `@ProjectScope`) | ❌ | model permission, enforcement per endpoint |
| Multi-user/project | personal + team project, transfer, ownership | ❌ | |
| Webhook produksi vs test | 2 URL (`/webhook/...`, `/webhook-test/...`), auto-registrasi saat aktif | 🟡 (hanya `/hook/:path` manual, tanpa konsep aktif/nonaktif) | registrasi otomatis dari workflow aktif |
| Aktifasi & polling trigger | scheduler + polling (`interval`) + aktivasi per workflow | 🟠 (`scheduleTrigger`/`cron` = echo/mock) | scheduler nyata + state aktivasi + trigger registry |
| Queue mode (Redis/Bull) | worker terpisah, `EXECUTIONS_MODE=queue` | ❌ | worker, queue, scaling |
| Task runner (JS/Python terpisah) | proses sandbox, `n8n-nodes-base.code` pakai runner | ❌ (Code = Rhai in-process) | sandbox + runner protocol |
| `N8N_RUNNERS_*`, concurrency | concurrency limit global/per-worker | ❌ | |
| Binary data | mode memory/filesystem/S3, `$binary`, node file | ❌ | model binary + storage + HTTP stream |
| Error workflow | workflow terpisah dipanggil saat gagal + `Error Trigger` | ❌ | |
| Retry on fail | retry per node + backoff | ❌ (0 match "retries") | |
| Sub-workflow | `Execute Sub-workflow` + trigger + mode wait | ❌ (0 match) | |
| Pin data | pin output node untuk dev | ❌ | |
| Partial execution | jalankan dari node terpilih | 🟡 (run penuh saja) | |
| Execution order | `v1` (default di 2.x) + warisan `v0` | 🟡 (topological + paralel rayon) | kompatibilitas urutan & merge input |
| Manual execution dari editor | "Execute workflow" + "Execute node" | 🟡 (run penuh) | eksekusi per-node |
| Webhook `Respond to Webhook` | mode first/streaming/last node | 🟡 | |
| Form & Form Trigger | form builder + hosted page | ❌ | |
| Chat Trigger (hosted chat) | ada + Chat Hub | ❌ | |
| MCP server trigger / client | ada (registry + akses) | ❌ | |
| Community nodes | install dari npm + verifikasi + UI | ❌ | |
| Custom nodes (module npm) | loader dinamis | ❌ | |
| Source control (git, environments) | EE: commit/pull/push per environment | ❌ | |
| External secrets | Vault, AWS SM, Infisical, 1Password, Azure | ❌ | |
| Log streaming | webhook/syslog/Sentry/Datadog/... | ❌ | |
| Audit log | `/audit` + UI | ❌ | |
| Insights | agregasi waktu tersimpan, run, failure | ❌ | |
| SSO SAML/OIDC + provisioning | EE | ❌ | |
| LDAP | EE | ❌ | |
| License/plan gating | `license.ts`, EE flag | ❌ | |
| Observability | Prometheus metrics, OTel traces, `/metrics` | 🟡 (`/api/metrics` JSON sederhana) | |
| i18n | ~18 bahasa (package `@n8n/i18n`) | ❌ (hardcode Inggris) | |
| Telemetry & PostHog | ada (bisa dimatikan) | ❌ (bagus: tanpa telemetri) | opsional |

---

## 4. NODE — dari 19 ke daftar resmi core

### 4.1 Angka

| | n8n | n8n-rust |
|---|---|---|
| Total implementasi node | 701 (558 base + 143 LangChain) | 19 core + 27 extended = 46 (mock preview hanya 19) |
| Direktori integrasi | 308 | 27, mayoritas **mock** (`extended.rs`: "mock fallback for tests") |
| Node LangChain/AI | 143 | 0 |
| Credential type | 456 | 12–13 |

### 4.2 Core node resmi (docs.n8n.io) vs repo — **hanya 18/63 yang setara**

✅ **Sudah ada & layak:** Code (+Function lama), Date & Time, Edit Fields (Set), Execute Command, Filter, HTTP Request, If, Limit, Manual Trigger, Merge, No Operation, Respond to Webhook, Schedule Trigger, Sort, Stop And Error, Switch, Wait, Webhook.

❌ **Hilang (prioritas, dari daftar core resmi):**
`Activation Trigger` · `Aggregate` · `AI Transform` · `Compare Datasets` · `Compression` · `Convert to File` ·
`Data Table` · `Debug Helper` · `Edit Image` · `Email Trigger (IMAP)` · `Error Trigger` · `Evaluation` ·
`Evaluation Trigger` · `Execute Sub-workflow` · `Execute Sub-workflow Trigger` · `Execution Data` · `Extract From File` ·
`FTP` · `Git` · `GraphQL` · `JWT` · `LDAP` · `Local File Trigger` · `Markdown` · `n8n` · `n8n Form` · `n8n Form Trigger` ·
`n8n Trigger` · `Read/Write Files from Disk` · `Remove Duplicates` · `Rename Keys` · `RSS Feed Trigger` · `Send Email` (ada versi mock) ·
`Split Out` · `SSE Trigger` · `SSH` · `Summarize` · `TOTP` · `Workflow Trigger` · `Chat` · `Chat Trigger` ·
`MCP Client` · `MCP Server Trigger` · `Guardrails`

🟠 **Ada tapi mock/tipis:** `Crypto`, `HTML`, `XML`, `RSS Read`, `Split in Batches`, `Slack`, `Discord`, `Telegram`, `Gmail`,
`Google Sheets`, `Notion`, `Airtable`, `Postgres`, `MySQL`, `Redis`, `S3`, `GitHub`, `GitLab`, `Jira`, `Trello`, `Stripe`, `Twilio`, `OpenAI`, `Send Email`.

> Catatan: node "extended" memakai HTTP jika memungkinkan, **fallback mock** untuk test — jadi tidak bisa disebut paritas fungsional (mis. `SlackNode` test memeriksa `out[0][0]["slack"]["sent"] == true`).

### 4.3 Integrasi paling sering dipakai (rekomendasi urutan implementasi nyata)

Gelombang 1 (10 node): `Slack`, `Gmail`, `Google Sheets`, `HTTP Request` (perkaya), `Postgres`, `MySQL`, `Notion`, `Airtable`, `OpenAI`, `Webhook` produksi.
Gelombang 2: `Google Drive`, `Discord`, `Telegram`, `Microsoft Teams/Outlook/Excel`, `Jira`, `GitHub`, `Airtable`, `Redis`, `S3`, `Supabase`, `MongoDB`, `Stripe`, `HubSpot`, `Salesforce`.
Gelombang 3: sisanya per kategori (CRM, marketing, DB, storage, komunikasi, devops).

---

## 5. PUBLIC API

n8n: **90 path** di OpenAPI v1, termasuk module yang belum ada di repo:
`credentials(+schema/test/transfer)`, `executions(+retry/stop/tags)`, `workflows(+archive/unarchive/publish/unpublish/transfer/history/versions/tags/test-runs)`,
`tags`, `variables`, `users(+role)`, `projects(+users/folders)`, `roles`, `role-mapping-rules`,
`data-tables(+columns/rows CRUD)`, `insights/summary`, `audit`, `community-packages`, `discover`, `n8n-packages(import/export)`,
`settings/{ldap,sso/saml,sso/oidc,otel,security-policy,log-streaming/*}`, `source-control/{status,push,pull}`, `promotions/*`.

n8n-rust saat ini (25 route):

```
GET  /                       POST /api/validate        GET  /api/runs
GET  /health                 POST /api/explain         GET  /api/runs/:id (+DELETE)
GET  /api/nodes              POST /api/run             GET|POST /api/workflows
GET  /api/metrics            GET  /api/executions      GET|PUT|DELETE /api/workflows/:id (+activate/deactivate)
GET  /api/openapi.json       POST /api/wait/resume/:id GET  /api/credentials (+POST, GET/:id, DELETE)
GET  /api/hook... /api/hooks (+POST, DELETE /:path)    GET  /api/credential-types
GET  /ws/logs  /ws/executions  GET|POST|PUT|PATCH|DELETE /hook/:path
```

**Yang kurang di API (fungsional, bukan kosmetik):**
- Autentikasi: `X-N8N-API-KEY` (semua endpoint n8n butuh ini) — repo terbuka tanpa auth.
- Pagination `limit`/`cursor`, filter (`active`, `tags`, `projectId`), sorting, `includeData`.
- Bentuk respons n8n (`{data: [...], nextCursor}`), bukan array telanjang.
- Endpoint `retry`/`stop` eksekusi; `test-runs` (evaluasi); `versions`/`history` workflow; `variables`; `tags`; `projects`; `users`; `audit`; `insights`.
- OpenAPI spec: repo hanya mengembalikan **ringkasan** `{"/api/nodes":{get:{summary:...}}}`, bukan spec OpenAPI 3 valid (tidak ada `components`, `parameters`, `responses`).

---

## 6. ENGINE / SEMANTIK EKSEKUSI

| Perilaku | n8n | n8n-rust | Bukti |
|---|---|---|---|
| Execution order v1 | item-based, "range" antar node | 🟡 topologi + rayon paralel | `n8n-engine/src/lib.rs` |
| Paired item / item linking | `$input.item`, `pairedItem` | ❌ | |
| `$json`, `$node`, `$items`, `$runIndex` | lengkap | 🟡 `$json`, `$node`, `$()`, `$input`… subset | `expr.rs` 1.248 baris |
| `$fromAI()` | ada | ❌ | |
| Error workflow + `Error Trigger` | ada | ❌ (0 match) | |
| `continueOnFail` / `onError` | 3 mode (stop/continueErrorOutput/continueRegularOutput) | 🟡 2 mode | grep `onError`: 2 |
| Retry on fail + `maxTries`/`waitBetweenTries` | ada | ❌ | grep `retries`: 0 |
| Binary data & file nodes | mode memory/fs/S3 + 20 node file | ❌ (0 match `BinaryData`) | |
| Pinning / partial execution | ada | ❌ | |
| Wait: resume webhook + resumeAt | menahan eksekusi, simpan state, resume | 🟠 marker `__waitResume` + `/api/wait/resume/:id` mock (tidak benar-benar menahan) | `api_wait_resume` = "simulate" |
| Sub-workflow | `executeWorkflow` + wait | ❌ | |
| Streaming ke UI (push) | Socket.IO per execution, node-by-node | 🟡 WS log global | |
| Manual/partial/triggered mode | `execution.mode` + retry/test | 🟡 sebagian | |
| Concurrency & queue | limit + worker | ❌ | |
| Eksekusi terjadwal & polling | scheduler cron + interval node | 🟠 mock | |

---

## 7. UI / UX — detail yang membuat "terasa n8n"

| Kapabilitas | n8n | n8n-rust |
|---|---|---|
| Canvas, pan/zoom, minimap, edge bezier, badge urutan | ✅ | ✅ (kekuatan repo) |
| Node creator (tab Trigger/Core/App/AI + search + docs) | ✅ | 🟡 (search + 4 tab, hanya node lokal) |
| NDV: Parameters/Input/Output/Settings/Docs + drag-drop data ke parameter | ✅ | 🟡 (tabs ada; drag-drop data ❌) |
| Expression editor (`={{ }}`) + autocomplete + preview hasil | ✅ | 🟡 (input JSON mentah + `$json` terbatas) |
| Schema view output & tabel input | ✅ | 🟡 (JSON mentah) |
| Sticky notes, group, align/distribute, node naming | ✅ | 🟡 (notes ada, group/align ❌) |
| Copy/paste, duplicate, multi-select (rubber band), Ctrl+drag | ✅ | 🟡 (duplicate per node) |
| Undo/redo persisten lintas reload | ✅ | 🟡 (history in-memory 60 langkah) |
| Executions tab dengan tabel & filter status | ✅ | 🟡 list sederhana |
| Logs panel (log, chat, exec, table) | ✅ | 🟡 log sendiri |
| Workflow diff & riwayat versi | ✅ | ❌ |
| Command palette ⌘K | ✅ | ✅ |
| Keyboard shortcuts lengkap | ✅ (docs: keyboard-shortcuts) | 🟡 (N, Del, Esc, ⌘⏎, ⌘K, ⌘Z, ⌘B/⌘I) |
| Dark/light + design tokens | ✅ | ✅ |
| i18n | ✅ 18+ bahasa | ❌ |
| Accessibility (axe, fokus, ARIA) | ✅ | ❌ belum diuji |
| Mobile/responsive | ✅ sebagian | 🟡 satu breakpoint 1200px |
| URL state (node terpilih, tab, zoom) | ✅ | ❌ |

---

## 8. DATA, KEAMANAN, KUALITAS

| Aspek | n8n | n8n-rust | Gap |
|---|---|---|---|
| Skema data | 141 tabel (workflow_entity, execution_entity, credentials_entity, project, user, role, tag, variables, data_table, chat_hub_*, insights_*, oauth_*, webhook_entity, test_run, ...) | 0 | rancang skema inti ±20 tabel dulu |
| Migrasi | migrasi bernomor + CLI `db:revert` | ❌ | |
| Enkripsi credential | AES-CBC + key dari `N8N_ENCRYPTION_KEY`, rotasi key | 🟡 AES-256-GCM + fallback base64 | rotasi key + sumber key dari env |
| Secret scanning CI | ✅ | ❌ | |
| Auth endpoint | guard semua route non-publik | ❌ **semua endpoint terbuka** | P0 keamanan |
| Eksekusi shell (`executeCommand`) | ada, tapi gated & didokumentasikan risikonya | ✅ tanpa gate | tambahkan allowlist/flag |
| Rate limit | ada (config) | 🟡 middleware ada | |
| Security headers/CORS/request-id | ✅ | ✅ | — |
| Test | 6.911 file test (unit + e2e Playwright + benchmark) | 95 unit test (`#[test]`), 0 e2e | butuh e2e UI + contract test terhadap API n8n |
| Docs | docs.n8n.io lengkap + directory node docs | README + 3 laporan | docs fitur per halaman |

---

## 9. Klaim dokumen vs realita (koreksi jujur)

| Klaim di repo | Realita terverifikasi | Bukti |
|---|---|---|
| `README.md`: badge **v1.1.0**, "19 node types", "Premium Quality" | `/health` → `"version":"0.9.0"`, `"nodes":19` (mock). Server Rust mendaftarkan 46. | `curl /health` |
| `README.md`: "UI premium single-file 1500+ lines, command palette, minimap, undo/redo" | benar (1.287 baris, 129 KB) — tapi **hanya 1 halaman** tanpa router | grep router = 0 |
| `ACCURACY_COMPARISON.md`: "UI 98% akurasi … SAMA PERSIS", "Backend API 99%", "46 types" | UI: 1 route vs ~60 (≈2%); API: 25 route vs 90 (≈10%); node: 46 klaim vs 701 (≈6%). Angka 98–99% **tidak punya bukti** di repo. | tabel §2–§5 |
| `ACCURACY_COMPARISON.md`: "Nodes 46 total (19 core + 27 extended)" | benar sebagai *jumlah tipe terdaftar*, tetapi 27 extended = mock/tipis | `extended.rs:2` "mock fallback for tests" |
| `ACCURACY_COMPARISON.md`: "Wait resume … 95%", "WS logs 96%" | Wait tidak benar-benar menahan eksekusi; resume berlabel `"mode":"mock"` | `api_wait_resume` |
| `UPGRADE_REPORT_v0.9.md`: "99% n8n asli" | tidak terverifikasi; n8n kini 2.x dengan banyak permukaan baru (AI, Chat Hub, Data Tables, Evaluations, Insights, workflow reviews) yang **sama sekali belum ada** | §2.4, §3 |
| Dockerfile/docker-compose: port 3000, healthcheck | konsisten | — |

**Konsekuensi:** sebelum mengejar fitur baru, klaim di README/ACCURACY_COMPARISON perlu dinormalkan (mis. "target: subset fungsional n8n, ~10% permukaan UI") supaya roadmap bisa diukur.

---

## 10. PETA JALAN — menutup gap dengan urutan yang benar

Prinsip: **jangan kejar jumlah node dulu**. n8n terasa n8n karena *halaman + identitas + eksekusi yang benar*, bukan karena 300 integrasi.

### Fase 0 — Fondasi aplikasi (P0, wajib sebelum fitur apa pun)
1. **Router + halaman** (`/workflows`, `/workflow/:id`, `/executions`, `/executions/:id`, `/credentials`, `/settings/*`, `/signin`, `/404`) — shell sidebar+header yang sudah ada dipakai ulang.
2. **DB SQLite + migrasi** (minimal 20 tabel inti) menggantikan file JSON; repository layer.
3. **Auth**: user + sesi + API key (`X-N8N-API-KEY`) + guard semua `/api/*`.
4. **Contract test** API terhadap spec n8n (bentuk respons `{data, nextCursor}`) — supaya tidak perlu refactor dua kali.
5. **Normalkan klaim dokumen** (README/ACCURACY_COMPARISON) agar progres terukur.

### Fase 1 — Editor & eksekusi setara (P1)
6. Daftar workflow bergaya n8n (grid/list, search, tag, arsip) + tab Executions/Evaluations/Logs berfungsi.
7. Halaman detail eksekusi + NDV output per node + tombol retry/stop.
8. Riwayat versi workflow + diff.
9. Semantik eksekusi: **error workflow, retry, pinning, partial/per-node execution, item linking, binary data**.
10. **Sub-workflow** + Wait yang benar-benar menahan (state tersimpan, resume via webhook).
11. Credentials lengkap: test, sharing, transfer, per-project; credential type mengikuti daftar n8n.
12. Node creator + expression editor (autocomplete `$json/$node/$items`, preview).
13. Form Trigger + halaman form; `n8n Trigger`; SSE/Interval/Local File trigger; registrasi webhook produksi vs test.

### Fase 2 — Permukaan produk penuh (P2)
14. Node core yang hilang (§4.2) — **core dulu, baru integrasi**.
15. Variables, Tags, Projects, Folders, RBAC + role; Data Tables; Insights; Audit; Evaluations.
16. Queue mode + worker + concurrency; task runner untuk Code (sandbox).
17. Community nodes + custom node loader.
18. AI: Chat Trigger/Chat Hub, MCP server & client, node LangChain inti (agent, model, memory, tool, vector store, embeddings, retriever, splitter, parser, guardrails), `$fromAI()`.
19. Source control (git), external secrets, log streaming, SSO/LDAP (bila multi-user jadi target).
20. i18n, a11y, e2e Playwright, benchmark.

**Definition of done setiap item:** ada halaman/endpoint, ada test otomatis, ada entri parity matrix yang berubah status, dan bisa diverifikasi lewat `verify-parity.sh`.

---

## 11. Lampiran A — daftar core node n8n (untuk checklist implementasi)

```
Activation Trigger · Aggregate · AI Transform · Code · Compare Datasets · Compression · Chat Trigger* ·
Convert to File · Crypto · Data Table · Date & Time · Debug Helper · Edit Fields (Set) · Edit Image ·
Email Trigger (IMAP) · Error Trigger · Evaluation · Evaluation Trigger · Execute Command · Execute Sub-workflow ·
Execute Sub-workflow Trigger · Execution Data · Extract From File · Filter · FTP · Git · GraphQL · Guardrails* ·
HTML · HTTP Request · If · JWT · LDAP · Limit · Local File Trigger · Loop Over Items (Split in Batches) ·
Manual Trigger · Markdown · MCP Client* · MCP Server Trigger* · Merge · n8n · n8n Form · n8n Form Trigger ·
n8n Trigger · No Operation · Read/Write Files from Disk · Remove Duplicates · Rename Keys · Chat* · Respond to Webhook ·
RSS Read · RSS Feed Trigger · Schedule Trigger · Send Email · Sort · Split Out · SSE Trigger · SSH · Stop And Error ·
Summarize · Switch · TOTP · Wait · Webhook · Workflow Trigger · XML
```
`*` = node paket LangChain (AI).

## 12. Lampiran B — cara memverifikasi ulang

```bash
# 1) angka dari sumber primer n8n (butuh gh auth)
gh api repos/n8n-io/n8n/git/trees/master?recursive=1 > /tmp/n8n-tree.json
python3 -c "import json;t=json.load(open('/tmp/n8n-tree.json'));p=[e['path'] for e in t['tree']];print('node files',len([x for x in p if x.endswith('.node.ts')]))"

# 2) runtime repo ini
bash docs/parity/verify-parity.sh
```

Lihat juga `docs/parity/PARITY-MATRIX.json` (matriks mesin-terbaca untuk tracking) dan
`docs/parity/SKILLS-GITHUB.md` (skill GitHub yang dipakai untuk mempercepat penutupan gap).
