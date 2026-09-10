> [!WARNING]
> **DOKUMEN INI RESMI SUPERSEDED (USANG)** oleh PRD-3-CANONICAL-APPROVED.md dan PRD-WORKFLOW-HUB-STANDALONE.md per Putusan Lead Architect @matt (#1000) dan Juru Bicara @fern (#994). JANGAN jadikan acuan untuk implementasi kode aktif atau tipe data kanonik.

# PRD-1 — ANALISIS LENGKAP n8n ASLI

**Dokumen:** Analisis teknis menyeluruh n8n (referensi untuk reimplementasi Rust)
**Versi:** 1.0
**Tanggal:** 2026-09-09
**Versi n8n yang dianalisis:** 2.39.0 (master, commit `3afdf4a`, 9 Sep 2026)
**Tujuan:** Memetakan *apa yang sebenarnya dilakukan n8n*, supaya PRD-2 (versi Rust) bisa memutuskan dengan sadar apa yang ditiru, apa yang diubah, dan apa yang dibuang.

> **Metode & batas kejujuran.** Dokumen ini disusun dari sumber publik: repositori
> GitHub n8n-io/n8n, dokumentasi resmi, DeepWiki/ZRead (analisis pihak ketiga atas
> repo), dan materi pemasaran. Saya **tidak** membaca seluruh 24.019 commit atau
> menjalankan n8n untuk memverifikasi setiap klaim.
>
> Angka yang sumbernya saling bertentangan saya tandai **⚠️ VERIFIKASI** alih-alih
> memilih satu dan menyajikannya sebagai fakta. Bagian §18 mengumpulkan semuanya.

---

## 1. Identitas Proyek

| | |
|---|---|
| Nama | n8n (dibaca "n-eight-n", dari "nodemation") |
| Pemilik | n8n GmbH, Berlin |
| Lisensi | **Sustainable Use License** (fair-code) + n8n Enterprise License untuk sebagian kode |
| Bintang GitHub | ~204.000 |
| Fork | ~60.600 |
| Commit | 24.019 |
| Branch / tag | 1.597 / 2.148 |
| Versi saat analisis | 2.39.0 |
| Runtime minimum | Node.js ≥ 22.16, pnpm ≥ 10.22 |
| Valuasi (dilaporkan) | ~$2,5 miliar |

### 1.1 Lisensi — poin yang paling sering disalahpahami

n8n **bukan open source** dalam arti OSI. Ia *fair-code* / *source-available*:

| Boleh | Tidak boleh |
|---|---|
| Self-host untuk keperluan sendiri | Menyediakan n8n ke pihak ketiga sebagai layanan hosted |
| Modifikasi untuk penggunaan internal | White-labeling |
| Penggunaan komersial internal | Menghapus/mengaburkan fitur berlisensi |

**Implikasi untuk kita:** kita tidak menyalin kode, jadi SUL tidak mengikat produk kita. Tapi kita juga tidak boleh memakai nama/merek "n8n" (merek dagang n8n GmbH), dan klaim kompatibilitas harus hati-hati. Detail di PRD-2 §11.

---

## 2. Arsitektur Monorepo

n8n adalah monorepo pnpm + Turborepo. Ini peta paketnya, dan **ini bagian terpenting dokumen ini** — karena struktur paket n8n pada dasarnya adalah daftar komponen yang harus kita bangun ulang.

### 2.1 Paket fondasi

| Paket | Path | Peran |
|---|---|---|
| `n8n-workflow` | `packages/workflow` | Model domain: interface inti (`IWorkflowBase`, `INode`, `INodeType`, `ICredentialType`), engine ekspresi, graph traversal, validasi. Dual ESM/CJS. **Dipakai frontend DAN backend.** |
| `n8n-core` | `packages/core` | Engine eksekusi: `WorkflowExecute`, node execution contexts, node loader, binary data manager, enkripsi AES-256, resolusi kredensial |
| `n8n` (CLI) | `packages/cli` | Server Express, REST controller, auth middleware, webhook handler, scaling/queue mode, perintah CLI (`start`, `worker`, `webhook`), WebSocket push |

### 2.2 Paket node

| Paket | Peran |
|---|---|
| `n8n-nodes-base` | Node integrasi bawaan. **Terverifikasi: 558 node dalam 308 direktori**, plus 407 credential type. Lihat §18. |
| `@n8n/nodes-langchain` | Node AI/LLM berbasis LangChain: agents, LLM, vector store, embeddings, MCP tools, memory, document loader |
| `node-dev` | CLI untuk membuat node & kredensial kustom (template scaffolding) |

### 2.3 Paket infrastruktur `@n8n/*`

| Paket | Peran | Catatan untuk kita |
|---|---|---|
| `@n8n/config` | 50+ modul konfigurasi bertipe, skema divalidasi Zod | Kita butuh padanan; Rust: `serde` + validasi |
| `@n8n/di` | IoC container berbasis `reflect-metadata`, registrasi service via decorator | **Tidak perlu ditiru.** Rust tidak butuh DI runtime — trait object & constructor injection cukup |
| `@n8n/db` | Entity TypeORM, migrasi, repository (SQLite + PostgreSQL) | Padanan: `sqlx` atau `diesel` + migrasi |
| `@n8n/api-types` | Interface TypeScript yang dibagi FE↔BE untuk API type-safe | Padanan: crate `api-types` + codegen OpenAPI/TS |
| `@n8n/errors` | Hierarki error: `UserError`, `OperationalError`, `UnexpectedError` | Sudah ada padanan di kernel kita (`NodeError`, `KernelError`) |
| `@n8n/expression-runtime` | Sandbox evaluasi ekspresi `{{ $json.field }}` | **Kritis.** Padanan: QuickJS (lihat PRD-2) |
| `@n8n/task-runner` | Eksekusi ter-sandbox untuk kode tak tepercaya (Code node JS/Python) | Kritis untuk keamanan |
| `@n8n/agents` | Runtime agent AI, orkestrasi skill, tool generation | Di luar scope MVP kita |
| `@n8n/instance-ai` | Deep agent orchestrator, autonomous tools | Di luar scope |
| `@n8n/design-system` | Komponen Vue reusable + design token | Relevan hanya jika kita bangun UI sendiri |

### 2.4 Paket frontend & lain

| Paket | Peran |
|---|---|
| `frontend/editor-ui` | SPA Vue 3: canvas, panel konfigurasi node, execution viewer |
| `frontend/*` lain | Module frontend (mis. `modules/insights`) |
| `extensions/insights` | Dashboard penggunaan/insights |
| `testing` | Infrastruktur test (Playwright, dsb.) |
| `@n8n/engine` | **Engine v2 (inkubasi)** — engine eksekusi generasi berikutnya, saat ini masih scaffold, belum dipakai produksi. Lihat §17.4 — ini sinyal penting. |

### 2.5 Properti graf paket

Sumber pihak ketiga menyatakan graf dependency n8n **acyclic**, dengan `n8n-workflow`, `@n8n/config`, `@n8n/di` sebagai fondasi berdependensi minimal.

> **Catatan untuk kita:** ini memvalidasi keputusan kernel kita. Kita sudah membuktikan hal yang sama secara mekanis — `kernel` punya 0 dependency internal, dan `scripts/check-freeze.sh` memverifikasi tidak ada cycle. Bedanya: n8n mencapai ini lewat disiplin konvensi, kita lewat enforcement kompilator + CI.

---

## 3. Tech Stack per Layer

| Layer | Teknologi n8n | Versi |
|---|---|---|
| Frontend framework | Vue 3 (Composition API) | 3.5+ |
| Build tool | Vite | — |
| State management | Pinia | 2.2+ |
| Canvas / graph | **Vue Flow** | — |
| UI component base | Element Plus | — |
| Styling | Tailwind CSS | — |
| Expression editor / Code node | CodeMirror 6 | — |
| Component dev | Storybook | — |
| Backend runtime | Node.js | ≥ 22.16 |
| Bahasa | TypeScript | 5.9+ |
| HTTP framework | Express | 5.1.0 |
| Security headers | Helmet | — |
| ORM | TypeORM | 0.3.20+ |
| Database | SQLite (default/dev), PostgreSQL (produksi) | — |
| Job queue | `bull` (Redis-backed) — **bukan** BullMQ | 4.16.4 (terverifikasi) |
| AI/LLM | LangChain, `@langchain/openai`, `@langchain/anthropic`, MCP SDK, LangGraph | — |
| Package manager | pnpm workspaces | 10.22 → 12 |
| Monorepo build | Turborepo, esbuild, tsdown, tsc + tsc-alias | — |
| Test | Vitest, Jest, Playwright, nock | — |
| Lint/format | Biome, ESLint 9, Stylelint, Lefthook | — |
| DI | `@n8n/di` (reflect-metadata) | custom |
| Deploy | Docker multi-stage, Alpine base | — |

**Pola arsitektur backend:** Controller → Service → Repository (MVC-like), Dependency Injection via `@n8n/di`, internal event bus untuk komunikasi ter-decouple, dan **context-based execution** (tipe node berbeda menerima context berbeda).

---

## 4. Model Data Inti

Ini jantung kompatibilitas. Kalau kita salah di sini, tidak ada workflow n8n yang bisa diimpor.

### 4.1 Item — unit data

```typescript
interface INodeExecutionData {
  json: IDataObject;                    // payload utama, selalu ada
  binary?: { [key: string]: IBinaryData };  // file/blob, opsional
  pairedItem?: PairedItem | PairedItem[];   // lineage ke item input
  error?: INodeExecutionError;
}
```

**Aturan semantik yang wajib ditiru:**
- Setiap node menerima **array of items** dan menghasilkan **array of items**
- Node "execute once" melihat semua item sekaligus; node per-item dipanggil sekali per item
- `json` adalah objek JSON biasa — bukan skema tetap
- `binary` dipisah dari `json` supaya payload besar tidak ikut terkopi saat transformasi ringan

### 4.2 Node

```typescript
interface INode {
  id: string;              // uuid, stabil
  name: string;            // nama tampilan, UNIK dalam satu workflow — dipakai expression $('Name')
  type: string;            // mis. 'n8n-nodes-base.httpRequest'
  typeVersion: number;     // versi node, di-pin per workflow
  position: [number, number];  // koordinat canvas — dipakai ExecutionOrder v1
  parameters: IDataObject;
  credentials?: { [type: string]: INodeCredentialsDetails };
  disabled?: boolean;
  executeOnce?: boolean;
  retryOnFail?: boolean;
  maxTries?: number;
  waitBetweenTries?: number;
  onError?: 'stopWorkflow' | 'continueRegularOutput' | 'continueErrorOutput';
  notes?: string;
  alwaysOutputData?: boolean;
}
```

> **Detail yang mudah terlewat:** `name` adalah kunci yang dipakai expression untuk
> mereferensikan node lain (`$('HTTP Request').item.json`). Artinya **rename node
> memutus expression** yang menunjuk ke nama lama. n8n menangani ini dengan
> update reference saat rename. Kita harus meniru perilaku ini atau workflow
> impor akan rusak diam-diam.

### 4.3 Connections

```typescript
interface IConnections {
  [sourceNodeName: string]: {
    main?: INodeConnections[][];     // output branch → target list
    [otherType: string]: INodeConnections[][] | undefined;  // mis. 'ai_tool', 'ai_memory'
  };
}
```

Poin penting:
- Koneksi diidentifikasi by **node name**, bukan node id
- `main` adalah jalur data utama; AI node memakai tipe koneksi lain (`ai_tool`, `ai_model`, `ai_memory`, `ai_vectorStore`, dsb.) — **ini yang membuat node AI bisa dipasang sebagai sub-node**
- Satu output bisa punya banyak branch (IF punya 2, Switch punya N)

### 4.4 Workflow

```typescript
interface IWorkflowBase {
  id?: string;
  name: string;
  active: boolean;
  nodes: INode[];
  connections: IConnections;
  settings?: IWorkflowSettings;   // executionOrder, timezone, errorWorkflow, saveData*, dll.
  staticData?: IDataObject;      // persisten lintas eksekusi
  versionId?: string;
  meta?: { templateId, instanceId };
  tags?: ITag[];
  pinData?: { [nodeName: string]: INodeExecutionData[] };  // data pin untuk testing
}
```

`settings.executionOrder`: `'v0'` (legacy, breadth-first) atau `'v1'` (per-branch, default n8n ≥1.0). **Kita harus mendukung keduanya** — workflow lama masih memakai v0.

### 4.5 Execution

Satu eksekusi menghasilkan `IRun`:
- `data.resultData.runData[nodeName]` → array `ITaskData` per node (input, output, error, startTime, executionTime)
- `data.executionData` → state mesin eksekusi (waiting nodes, run order)
- Disimpan sebagai **JSON blob besar** di kolom `data` tabel `execution_entity`

> **Ini akar masalah memorinya.** Seluruh hasil eksekusi — semua item dari semua
> node — diserialisasi jadi satu blob. Untuk workflow dengan output besar, blob
> ini bisa ratusan MB, dan ia hidup di heap JS selama eksekusi. Lihat §17.1.

---

## 5. Cara Kerja Execution Engine

### 5.1 Alur eksekusi regular mode

```
Trigger (webhook/cron/manual)
   ↓
WorkflowExecute.run()
   ↓
Hitung node mana yang siap (semua dependency selesai)
   ↓
Untuk tiap node:
   1. Resolusi parameter → evaluasi expression {{ }} per item
   2. Ambil kredensial (decrypt)
   3. Buat execution context sesuai tipe node
      (NodeExecutionContext / TriggerExecutionContext / WebhookExecutionContext)
   4. Panggil node.execute() / node.trigger() / node.webhook()
   5. Kumpulkan INodeExecutionData[]
   6. Simpan ke runData
   ↓
Ulangi sampai tidak ada node siap
   ↓
Tulis execution entity ke DB
```

### 5.2 Context-based execution

Pola kunci: **tipe node berbeda menerima context dengan kemampuan berbeda.**
`packages/core/src/execution-engine/node-execution-context/` mengimplementasikan varian-varian ini.

Ini penting untuk keamanan dan untuk kejelasan API: node webhook tidak butuh akses ke `putExecutionToWait`, node trigger tidak butuh `helpers.httpRequest` yang sama dengan node regular.

> **Sudah kita tiru di kernel:** `NodeContext<'a>` kita adalah padanan langsung, dan kita memperkuatnya dengan `ResourceHint` + `SideEffect` yang tidak dimiliki n8n.

### 5.3 Execution order

| Mode | Perilaku |
|---|---|
| `v0` (legacy) | Breadth-first: node pertama semua branch, lalu node kedua semua branch |
| `v1` (default ≥1.0) | Selesaikan satu branch penuh sebelum branch berikutnya, urut posisi canvas (atas→bawah, lalu kiri→kanan) |

`v0` sudah dihapus di n8n 1.0 tapi setting-nya masih dibaca untuk kompatibilitas.

### 5.4 Error handling

Tiga level:
1. **Per-node**: `retryOnFail` + `maxTries` + `waitBetweenTries`; `onError` menentukan apakah workflow berhenti, lanjut dengan output reguler, atau lanjut lewat output error
2. **Error Workflow**: workflow terpisah yang dipicu saat workflow lain gagal (Error Trigger node)
3. **Per-execution**: status `error`, `success`, `waiting`, `running`, `canceled`

> **Celah yang kita perbaiki:** n8n tidak membedakan "side effect sudah terjadi tapi proses mati" dari "side effect belum terjadi". Retry pada node non-idempoten bisa menduplikasi transfer/email. Kita menambah `SideEffect` + `TaskStatus::InDoubt` + `NEEDS_REVIEW`. Lihat PRD-2.

### 5.5 Waiting & resume

Node `Wait` dan webhook-resume menandai eksekusi sebagai `waiting`, **menyimpan state ke DB**, dan melepaskan proses. Saat timer/webhook datang, eksekusi dimuat ulang dan dilanjutkan.

Ini fitur penting: artinya n8n sudah punya konsep *persisted wait*. Kita menirunya lewat `TaskStatus::Waiting` + `Checkpoint` + `wake_at`.

---

## 6. Mode Eksekusi & Topologi Deployment

### 6.1 Regular mode (default)

Satu proses `n8n start` melakukan semuanya: UI, REST API, webhook, scheduler, dan eksekusi workflow.

| Aspek | Nilai |
|---|---|
| Scaling | Vertikal saja |
| DB | SQLite (default) atau PostgreSQL |
| Redis | Tidak perlu |
| Persistensi job | Tidak ada — sinkron |
| Locking | File atau database lock |
| Batas praktis | UI melambat dan webhook timeout saat beban eksekusi berat |

### 6.2 Queue mode

Tiga peran proses terpisah:

```
┌─────────────┐     ┌───────┐     ┌──────────┐     ┌────────────┐
│ n8n main    │────▶│ Redis │◀────│ worker×N │────▶│ PostgreSQL │
│ UI/API/sched│     │ Bull  │     │ eksekusi │     │            │
└─────────────┘     └───────┘     └──────────┘     └────────────┘
       │                                                    
┌──────▼──────┐                                             
│ n8n webhook │  (opsional, proses khusus webhook produksi)  
└─────────────┘                                             
```

| Aspek | Nilai |
|---|---|
| Perintah CLI | `n8n start`, `n8n worker --concurrency=N`, `n8n webhook` |
| DB | **PostgreSQL wajib** (SQLite tidak mendukung akses konkuren multi-proses) |
| Redis | Wajib (Bull queue + pub/sub untuk orkestrasi) |
| Scaling | Horizontal — tambah replika worker |
| HA | `multiMain` — **Enterprise only** |

**Environment variable kunci:**
```
EXECUTIONS_MODE=queue
QUEUE_BULL_REDIS_HOST / _PORT / _PASSWORD / _DB
QUEUE_BULL_QUEUE_NAME=n8n_executions_queue
DB_TYPE=postgresdb
N8N_CONCURRENCY_PRODUCTION_LIMIT      # atau --concurrency
OFFLOAD_MANUAL_EXECUTIONS_TO_WORKERS=true
N8N_DISABLE_PRODUCTION_MAIN_PROCESS=true
N8N_ENCRYPTION_KEY                    # HARUS sama di semua proses
WEBHOOK_URL
EXECUTIONS_DATA_PRUNE=true
EXECUTIONS_DATA_MAX_AGE=168           # jam
```

**Siklus hidup job:**
1. Main menerima trigger → buat execution record (belum dijalankan)
2. Main enqueue job (execution ID + payload) ke Redis
3. Worker melakukan atomic pop dari Redis → tidak ada dua worker dapat job sama
4. Worker baca workflow dari Postgres → eksekusi
5. Worker tulis hasil ke Postgres → ack job di Redis
6. Main merefleksikan status ke UI/API

**Panduan konkurensi worker** (dari sumber komunitas):
- I/O-heavy (HTTP, DB, webhook): 10–20 per worker
- CPU-heavy (image processing, transformasi besar, custom JS): 2–5 per worker, tambah worker bukan tambah konkurensi
- Mixed: mulai 10, pantau CPU

### 6.3 Konsekuensi yang relevan untuk kita

**Queue mode n8n butuh 5 service** (main, worker, webhook, PostgreSQL, Redis). Itu lompatan kompleksitas operasional yang besar dari regular mode, dan alasan banyak pengguna tetap di regular mode sampai mereka terpaksa.

> **Peluang kita:** karena spill-to-disk membuat satu proses jauh lebih hemat memori, kita bisa menunda kebutuhan queue mode jauh lebih lama. Satu binary Rust + SQLite bisa menangani beban yang di n8n sudah memaksa orang pindah ke topologi 5-service. **Ini diferensiator operasional yang nyata**, bukan cuma "lebih cepat".

---

## 7. Sistem Node

### 7.1 Anatomi node

```typescript
interface INodeType {
  description: INodeTypeDescription;
  execute?(this: IExecuteFunctions): Promise<INodeExecutionData[][]>;
  trigger?(this: ITriggerFunctions): Promise<ITriggerResponse>;
  webhook?(this: IWebhookFunctions): Promise<IWebhookResponse>;
  methods?: { loadOptions?, resourceMapping?, listSearch? };
}

interface INodeTypeDescription {
  displayName: string;
  name: string;                    // mis. 'httpRequest'
  group: string[];                // mis. ['trigger'] atau ['action']
  version: number | number[];
  subtitle?: string;
  description?: string;
  defaults: { name: string; color?: string };
  inputs: string[] | INodeInputConfiguration[];
  outputs: string[] | INodeOutputConfiguration[];
  properties: INodeProperties[];  // ← definisi parameter, menggerakkan form UI
  credentials?: INodeCredentialType[];
  icon?: string;
  badges?: string[];
  usableAsTool?: boolean;         // bisa dipakai AI agent sebagai tool
  hidden?: boolean;
  codex?: { categories?, subcategories?, resources?, alias?, primaryDocumentation? };
  hooks?: INodeHooks;
  requestDefaults?: IRequestDefaults;
  mockManualExecution?: boolean;
}
```

### 7.2 `INodeProperties` — satu sumber untuk validasi DAN UI

```typescript
interface INodeProperties {
  displayName: string;
  name: string;
  type: INodePropertiesType;   // 'string'|'number'|'boolean'|'options'|'multiOptions'
                               // |'dateTime'|'json'|'collection'|'fixedCollection'
                               // |'resourceLocator'|'credentials'|'notice'|'button'
                               // |'stringArray'|'object'|'objectArray'|'filter'
  default: NodeParameterValueType;
  required?: boolean;
  description?: string;
  displayOptions?: { show?, hide?, values? };  // conditional visibility
  options?: INodePropertyOptions[];
  typeOptions?: { multipleValues?, minValue?, maxValue?, codeEditor?, rows? };
  placeholder?: string;
  loadOptionsMethod?: string;      // dropdown dinamis dari API
  modes?: Array<{ name; type; initCode?; autocompleteFunction? }>;
  routing?: IRequestItemS;         // declarative: mapping parameter → HTTP request
}
```

> **Ini temuan desain terpenting untuk kita.** n8n sudah punya skema deklaratif yang
> menggerakkan form UI *dan* validasi dari satu definisi. Kernel kita punya
> `ParameterSchema` + `ParameterField` + `DisplayCondition` yang merupakan padanan
> langsung, dan test `parameter_schema_validation_and_visibility` sudah
> membuktikan conditional visibility bekerja.
>
> Konsekuensi: **kita tidak perlu menemukan desain form dari nol.** Kita perlu
> memetakan `INodeProperties` → `ParameterSchema` dengan setia.

### 7.3 Declarative HTTP routing

Node modern n8n memakai `routing` untuk memetakan parameter ke HTTP request secara deklaratif, tanpa menulis kode:

```typescript
{
  displayName: 'User ID',
  name: 'userId',
  type: 'string',
  default: '',
  routing: { send: { type: 'query', property: 'user_id' } }
}
```

**Ini penting** karena menjelaskan bagaimana n8n punya banyak integrasi dengan usaha rendah — dan itu persis jalur yang bisa kita otomatisasi lebih jauh lewat codegen OpenAPI (PRD-2 §8.4).

### 7.4 Kategori node

| Kategori | Contoh |
|---|---|
| **Trigger** | Manual, Schedule (cron), Webhook, Chat, Form, RSS Read, Email (IMAP), Error, Execute Workflow Trigger, n8n Form Trigger |
| **Flow control** | IF, Switch, Merge, Loop Over Items (SplitInBatches), Wait, Filter, Limit, Remove Duplicates, NoOp, Execute Workflow (sub-workflow), Stop and Error |
| **Transform** | Set (Edit Fields), Code, Split Out, Aggregate, Sort, Summarize, Rename Keys, Convert to File, Extract from File, HTML, XML, Markdown, Date & Time, Crypto |
| **Action / integrasi** | HTTP Request, GraphQL, SSH, FTP, Email (SMTP), Slack, Gmail, Google Sheets, Notion, Airtable, Salesforce, HubSpot, GitHub, GitLab, Jira, Discord, Telegram, WhatsApp, Stripe, PayPal, Twilio, OpenAI, Anthropic, dst. |
| **Database** | PostgreSQL, MySQL, SQLite, MongoDB, Redis, ElasticSearch, ClickHouse, Snowflake, BigQuery, Supabase, MS SQL |
| **AI / LangChain** | AI Agent, Chat Model (OpenAI/Anthropic/Gemini/Ollama/Bedrock/…), Memory (Simple/Redis/Postgres/Zep), Vector Store (Pinecone/Qdrant/pgvector/Supabase/In-Memory), Embeddings, Document Loader, Text Splitter, Tool (Calculator, Code, SerpAPI, VectorStore, **Workflow Tool**), Output Parser, MCP Client/Server, Human-in-the-loop |
| **Utility** | Respond to Webhook, NoOp, Set, Code, Compare Datasets, Edit Image, Read/Write Files from Disk |

### 7.5 Credential types

Kredensial adalah tipe terpisah dari node, didefinisikan via `ICredentialType`:

```typescript
interface ICredentialType {
  name: string;
  displayName: string;
  extends?: string[];
  properties: INodeProperties[];
  authenticate?: IAuthenticateGeneric | IAuthenticate[];
  preAuthentication?: () => Promise<INodeExecutionData>;
  test?: ICredentialTestRequest;
  docsUrl?: string;
}
```

Autentikasi bisa dideklarasikan generik (`authenticate: { type: 'generic', properties: { headers: {...} } }`) atau kustom (OAuth1/OAuth2 flow lengkap).

**OAuth2 adalah bagian termahal dari setiap integrasi.** Node dengan OAuth butuh: authorization URL, token URL, scope, client ID/secret, refresh token flow, dan penyimpanan token yang aman. Ini alasan utama "menambah integrasi" jauh lebih mahal dari yang terlihat.

---

## 8. Expression System

### 8.1 Bentuk

`{{ <JavaScript expression> }}` di dalam nilai parameter mana pun. Dievaluasi per-item.

### 8.2 Permukaan yang tersedia

| Variabel | Arti |
|---|---|
| `$json` | Payload item saat ini |
| `$input.all()` / `$input.first()` / `$input.item` | Akses input node saat ini |
| `$binary` | Binary data item saat ini |
| `$('Node Name')` / `$('Node Name').item` / `.all()` / `.first()` | **Akses acak output node lain** — inilah yang membuat streaming murni mustahil |
| `$items('Node Name')` | Alias lama |
| `$node['Name'].json` | Alias lebih lama lagi |
| `$now`, `$today` | Luxon DateTime, sudah dalam timezone workflow |
| `$execution.id`, `$execution.mode`, `$execution.resumeUrl` | Metadata eksekusi |
| `$workflow.id`, `$workflow.name`, `$workflow.active` | Metadata workflow |
| `$itemIndex` | Indeks item dalam batch |
| `$vars.*` | Variables global (level instance) |
| `$env.*` | Environment variable — **di-gate allowlist** |
| `$secrets.*` | External secrets (Enterprise) |
| `$runIndex` | Indeks run ke-n dari node |
| `$prevNode` | Node sebelumnya |
| `$fromAI('key','desc','type')` | Ekstraksi terstruktur untuk AI |

### 8.3 Helper bawaan

n8n menyuntikkan helper ke scope ekspresi, antara lain: `$jmespath`, `$parseJson`, `$toDateTime`, `$convertDateTime`, `$extractDomainParts`, `$isEmailValid`, `$isValidJSON`, `$base64Encode/Decode`, `$urlEncode/Decode`, `$hash`, `$roundTo`, `$randomInt`, `$uuid`, `$randItem`, `$shuffle`, `$difference`, `$intersection`, `$merge`, `$splitInBatches`, `$ifEmpty`, `$ifNot`, `$isEmpty`, `$not`.

Plus seluruh Luxon (DateTime) dan fungsi bawaan JS.

### 8.4 Sandbox

`@n8n/expression-runtime` menyediakan sandbox. Di self-hosted, `NODE_FUNCTION_ALLOW_EXTERNAL` mengizinkan modul npm tertentu di Code node (mis. `ajv,ajv-formats,puppeteer`).

### 8.5 Kenapa ini bagian tersulit dari kompatibilitas

1. **Ini JavaScript nyata**, bukan DSL kecil. Setiap perilaku JS adalah kontrak: coercion, `null` vs `undefined`, `Date` formatting, regex flavor, `Array.prototype` methods.
2. **Helper tidak terdokumentasi lengkap.** Perilaku edge hanya bisa diketahui dari kode sumber atau eksperimen.
3. **Akses acak lintas node** (`$('Node')`) menuntut semua output node sebelumnya tetap *addressable* selama eksekusi.
4. **Evaluasi per-item** — untuk 1 juta item, ekspresi dievaluasi 1 juta kali. Biaya setup context JS harus diamortisasi.

> **Konsekuensi arsitektur:** inilah alasan kita butuh QuickJS (bukan menulis DSL sendiri) dan butuh `PriorOutputs` di kernel. Dan inilah alasan `eval_batch` ada di trait kita — setup context QuickJS ratusan mikrodetik, tidak boleh dibayar per ekspresi per item.

---

## 9. Inventaris Fitur Lengkap

### 9.1 Core — tersedia di semua tier termasuk Community/self-hosted

| Fitur | Detail |
|---|---|
| Visual workflow builder | Canvas node-based, drag-drop, branching, looping, merging |
| Real-time output preview | Lihat data antar node saat build |
| Pin data | Bekukan output node untuk testing tanpa memanggil API nyata |
| Partial execution | Jalankan hanya sampai node tertentu |
| Execution logs & history | Daftar eksekusi, filter, cari, lihat per-node |
| Debugging | Step-through data, error stack, retry from node |
| Error workflows | Workflow khusus penangan error |
| Sub-workflows | Execute Workflow node — modularity |
| Code nodes | JavaScript **dan Python**, dengan import library di self-hosted |
| HTTP Request node | Mendukung GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS + PROPFIND/MKCOL/MOVE/COPY/REPORT |
| Webhook node | Endpoint HTTP inbound, test & production URL terpisah |
| Schedule trigger | Cron, interval, specific times |
| Manual trigger | Eksekusi dari UI/CLI/API |
| Form trigger | Form builder + submission capture |
| Chat trigger | Embeddable chat widget |
| RSS trigger | Poll feed |
| Email trigger (IMAP) | Trigger dari email masuk |
| Credentials | Enkripsi AES-256 at-rest, injeksi runtime, sharing via project |
| Variables | Key-value global level instance |
| Static data | State persisten per-workflow/per-node lintas eksekusi |
| Binary data | File handling: filesystem atau S3-compatible |
| Execution data pruning | `EXECUTIONS_DATA_PRUNE`, `EXECUTIONS_DATA_MAX_AGE` |
| Templates | diklaim 1.700+ — **tidak terverifikasi**, tidak ada di monorepo (§18.3) |
| Public REST API | Kelola workflow, execution, credential, user, audit |
| CLI | `n8n start/worker/webhook/export/import/update` |
| Community nodes | Install node pihak ketiga via npm |
| Self-hosting | Docker, npm/npx, bare metal |
| Queue mode | Tersedia di Business/Enterprise (self-hosted) |

### 9.2 AI / Agent

| Fitur | Detail |
|---|---|
| AI Agent node | Multi-step reasoning, tool calling |
| Multi-agent systems | Koordinasi beberapa agent spesialis |
| RAG pipelines | Document loader → text splitter → embeddings → vector store → retriever |
| Vector stores | Pinecone, Qdrant, pgvector, Supabase, In-Memory, dll. |
| LLM providers | OpenAI, Anthropic, Google Gemini, Mistral, DeepSeek, Perplexity, Groq, Cohere, Azure OpenAI, AWS Bedrock, Hugging Face, Ollama (lokal) |
| Memory | Simple (session), Redis, Postgres, Zep + Chat Memory Manager |
| Tools | Calculator, Code Tool, SerpAPI, Vector Store Tool, Think Tool, **Workflow Tool** |
| **Workflow Tool** | Memanggil workflow n8n lain sebagai tool — mengubah seluruh katalog integrasi jadi kemampuan agent tanpa menulis tool code |
| MCP | Client **dan** Server node |
| Human-in-the-loop | Checkpoint approval di titik mana pun, termasuk sebelum tool call agent |
| Output parsers | Structured output dari LLM |
| AI Evaluations | Uji workflow AI dengan data nyata; metrik string similarity, exact match, LLM-as-a-Judge |
| Chat Hub | Satu antarmuka untuk banyak model + workflow agentik (rilis Jan 2026) |
| AI Workflow Builder | Deskripsi bahasa natural → workflow jadi. Multi-agent LangGraph (Supervisor → Planner → Responder → Parameter Updater), Claude Sonnet 4.5. **Cloud-only, berbasis kredit** |
| Instance AI | Autonomous agent orchestrator (`@n8n/instance-ai`) |

### 9.3 Enterprise / Business tier

| Fitur | Tier |
|---|---|
| SSO SAML 2.0 / OIDC (Okta, Azure AD) | Business+ |
| LDAP / Active Directory | Business+ |
| Enforced 2FA (instance-wide) | Enterprise |
| RBAC: viewer/editor/admin per project | Business+ |
| **Custom roles** (granular) | Enterprise |
| Instance admins (terpisah dari project admin) | Enterprise |
| Git source control | Business+ |
| Environments (dev/staging/prod) | Business+ |
| Workflow diffs | Business+ |
| Audit logging | Enterprise |
| Log streaming ke SIEM (Datadog, dll.) | Enterprise |
| External secrets (HashiCorp Vault, AWS Secrets Manager, Azure Key Vault) | Enterprise |
| White-labeling | Enterprise |
| Air-gapped deployment (tanpa telemetry) | Enterprise |
| Queue mode + multi-main HA | Business+ / Enterprise |
| Insights (dashboard penggunaan) | semua, retensi berbeda: 7/7/30/365 hari |
| Extended execution log retention | Enterprise: unlimited |

### 9.4 Pricing tiers (cloud)

| | Starter | Pro | Business | Enterprise |
|---|---|---|---|---|
| Harga (annual) | €20/bln | €50/bln | €667/bln | Custom |
| Eksekusi/bln | 2.500 | 10.000 | 40.000 | Custom |
| Hosting | Cloud | Cloud | **Self-hosted** | Keduanya |
| Shared projects | 1 | 3 | 6 | Unlimited |
| Concurrent executions | 5 | 20 | — | 200+ |
| Insights retention | 7 hari | 7 hari | 30 hari | 365 hari |
| Max saved executions | 2.500 | 25.000 | 25.000 | 50.000 |

**Model harga: per-execution, bukan per-step.** Satu workflow dengan 50 node = 1 eksekusi. Ini penting — artinya n8n tidak menghukum workflow kompleks, berbeda dari Zapier (per-task) dan Make (per-operation).

---

## 10. Arsitektur Frontend

### 10.1 Struktur

```
packages/frontend/editor-ui/src/
├── app/
│   ├── init.ts              # bootstrap, registrasi Pinia store (baris 45-217)
│   ├── stores/              # Pinia: workflows, NDV, UI, execution, history
│   ├── views/NodeView.vue   # tampilan utama: canvas + sidebar konfigurasi
│   └── composables/
│       ├── useCanvasOperations.ts   # add/move/connect node → command ke store
│       ├── useCanvasNode.ts
│       ├── useWorkflowHelpers.ts
│       └── useKeybindings.ts
├── features/                # modul fitur: AI, credentials, execution, workflows
└── experiments/             # feature flags
```

### 10.2 Sub-sistem UI

| Komponen | Fungsi |
|---|---|
| **Canvas** | Vue Flow. Drag-drop node, koneksi bezier, pan/zoom, minimap |
| **Node Detail View (NDV)** | Modal konfigurasi properti node — tempat `INodeProperties` dirender jadi form |
| **Node Creator** | Browse & tambah node |
| **Parameter Inputs** | Form field per tipe properti |
| **Resource Locator** | Pilih resource dari API eksternal (dengan `loadOptionsMethod`) |
| **Expression Editor** | CodeMirror 6, autocomplete `$json`, `$('Node')`, preview hasil |
| **Execution Viewer** | Visualisasi run per-node dengan data |
| **Canvas Chat** | Antarmuka AI assistant untuk build workflow |
| **Undo/Redo** | History store untuk operasi canvas |

### 10.3 Store Pinia utama

`workflows.store`, `ndv.store`, `ui.store`, `execution.store`, `history.store`, `workflowDocumentStore` (canvas state), `credentials.store`, `settings.store`, `users.store`, `projects.store`.

### 10.4 Real-time

Push connection via **WebSocket** — server mendorong update status eksekusi ke UI. Tanpa ini, execution viewer tidak bisa live.

### 10.5 Estimasi effort frontend

> **⚠️ Ini penilaian saya, bukan data n8n.**

Canvas editor dengan Vue Flow, NDV yang merender ~15 tipe parameter termasuk `fixedCollection` bersarang dan `resourceLocator` dinamis, expression editor dengan autocomplete, execution viewer dengan data diffing, plus i18n — ini **produk frontend tersendiri**. Wajar kalau porsinya ~30% dari total effort reimplementation.

---

## 11. Arsitektur Backend

### 11.1 Layering

```
Controller (REST / Public API)
    ↓
Service (business logic)
    ↓
Repository (@n8n/db, TypeORM)
    ↓
SQLite / PostgreSQL
```

Cross-cutting: `@n8n/di` (IoC), internal event bus, `@n8n/errors` (hierarki), `@n8n/config` (50+ modul config tervalidasi Zod).

### 11.2 Direktori `packages/cli/src/`

| Direktori | Isi |
|---|---|
| `commands/` | CLI: `start`, `worker`, `webhook`, `export`, `import`, `update` |
| `controllers/` | Endpoint REST API |
| `webhooks/` | Handler webhook |
| `execution-lifecycle/` | Hook pre/post execution |
| `node-execution/` | Koordinasi runtime node |
| `scaling/` | Logika queue mode, Bull setup, job processor, Redis pub/sub orchestration |
| `eventbus/` | Message event bus, log streaming relay |

### 11.3 Auth

- Session-based (cookie) untuk UI
- API key untuk Public API
- Basic auth opsional (`N8N_BASIC_AUTH_*`)
- SSO SAML/OIDC, LDAP (tier atas)
- `N8N_SECURE_COOKIE`, `N8N_ENCRYPTION_KEY`

### 11.4 Keamanan data

- Kredensial dienkripsi **AES-256** at-rest memakai `N8N_ENCRYPTION_KEY`
- Key **harus identik** di semua proses (main/worker/webhook) — mismatch menyebabkan gagal decrypt
- Ada guardrail penggunaan enkripsi (`feat(core): Add encryption usage guardrails`, commit `44d9d90`) dan perbaikan format key legacy (`fix(core): Repair legacy-format data-encryption keys during bootstrap`, commit `4b36c51`) — indikasi bahwa **rotasi/migrasi key adalah area yang sudah pernah menyakitkan**
- `N8N_ENFORCE_SETTINGS_FILE_PERMISSIONS=true` untuk hardening

---

## 12. Storage & Persistensi

### 12.1 Database

| | SQLite | PostgreSQL |
|---|---|---|
| Peran | Default, dev, single-process | Produksi, queue mode wajib |
| Konkurensi | Single writer | Multi-process |
| Migrasi | TypeORM | TypeORM |

MySQL juga didukung historisnya — **⚠️ VERIFIKASI** status dukungan MySQL/MariaDB di 2.x.

### 12.2 Entity utama (perkiraan dari pola TypeORM)

`workflow_entity`, `execution_entity`, `credentials_entity`, `user`, `project`, `role`, `webhook_entity`, `tag`, `variables`, `settings`, `audit_event`, `insights_*`, `shared_workflow`, `shared_credentials`.

> **⚠️ VERIFIKASI:** daftar entity persis perlu dibaca dari `packages/@n8n/db/src/entities/`.

### 12.3 Binary data

Tiga mode:
- **default/filesystem** — di `~/.n8n/binaryData`
- **database** — dalam kolom DB
- **S3** — direkomendasikan untuk queue mode / persistensi

### 12.4 Execution data

Blob JSON besar di `execution_entity.data`. Pruning via `EXECUTIONS_DATA_PRUNE` + `EXECUTIONS_DATA_MAX_AGE`.

**Ini sumber bloat Postgres yang paling sering dikeluhkan pengguna.** Rekomendasi komunitas: `EXECUTIONS_DATA_MAX_AGE=168` (7 hari).

---

## 13. Observability

| Kemampuan | Detail |
|---|---|
| Metrics | Prometheus-compatible endpoint (`N8N_METRICS=true`), counter eksekusi, rate, failure, webhook count, scheduler event |
| Log | Structured logging, `N8N_LOG_LEVEL` |
| Insights | Dashboard penggunaan dalam UI; retensi 7/30/365 hari per tier |
| Audit log | Enterprise: siapa buat/ubah/hapus/jalankan workflow |
| Log streaming | Enterprise: kirim event eksekusi/error/sistem ke SIEM (Datadog, webhook generic) |
| Health check | Endpoint untuk orchestrator |

**Queue depth** (`bull:default:wait`, `bull:jobs:active`) dipakai untuk autoscaling via HPA/KEDA.

---

## 14. Deployment

| Metode | Detail |
|---|---|
| Docker | Image Alpine multi-stage, `n8nio/n8n:latest` |
| npm/npx | `npx n8n` |
| Bare metal | Node.js ≥ 22.16 |
| Kubernetes | Helm chart resmi (`charts/n8n`), HPA/KEDA untuk worker |
| Cloud | n8n Cloud (EU/Frankfurt), managed |

**Kebutuhan resource** (dari riset sebelumnya, terverifikasi multi-sumber):

| Beban | Kebutuhan |
|---|---|
| Minimum | 1 vCPU / 1 GB RAM / 10 GB storage + SQLite — 50-100 workflow ringan |
| Direkomendasikan | 2 vCPU / 2-4 GB RAM / 20 GB SSD |
| Berat | 4+ vCPU / 8-16 GB + PostgreSQL + Redis queue mode |

Karakter: **memory-bound, bukan CPU-bound.** Idle ~100-500 MB, ~50 MB per workflow aktif.

---

## 15. Ekosistem

| Aspek | Detail |
|---|---|
| Community nodes | Node pihak ketiga via npm, di-install dari UI |
| Templates | diklaim 1.700+ — **tidak terverifikasi** (§18.3) |
| Forum | community.n8n.io — aktif |
| Dokumentasi | docs.n8n.io — matang dan luas |
| Kursus | n8n Academy, sertifikasi |
| Kontribusi | 24.019 commit; repo sangat aktif (commit dalam 42 menit terakhir saat dokumen ini ditulis) |

> **Ini moat terbesar n8n dan paling sulit ditiru.** Bukan kodenya — ekosistemnya.
> Ribuan template, node komunitas, jawaban forum, dan dokumentasi yang
> terakumulasi bertahun-tahun. Produk baru dengan kode lebih baik tetap kalah
> dari sisi ini selama bertahun-tahun.

---

## 16. Yang Dilakukan n8n dengan Baik (harus ditiru)

1. **`INodeProperties` sebagai satu sumber untuk UI + validasi.** Desain ini yang membuat 694 node mungkin tanpa 694 form kustom.
2. **Declarative HTTP routing.** Node integrasi bisa dibuat tanpa menulis kode request.
3. **Context-based execution.** Tiap tipe node dapat kemampuan yang sesuai, tidak lebih.
4. **Semantik item array yang konsisten.** Mudah dipahami, mudah diprediksi.
5. **Pin data & partial execution.** Debugging experience yang sangat baik.
6. **Workflow Tool untuk AI agent.** Mengubah seluruh katalog integrasi jadi kemampuan agent — leverage arsitektural yang cerdas.
7. **Persisted wait.** Node Wait tidak menahan proses.
8. **Harga per-execution, bukan per-step.** Tidak menghukum workflow kompleks.
9. **Graf paket acyclic.** Disiplin arsitektur yang dijaga.
10. **Dokumentasi dan template.** Moat nyata.

---

## 17. Keterbatasan Struktural (peluang kita)

### 17.1 Memory-bound by design

Seluruh array item hidup di heap JS selama eksekusi, dan seluruh hasil eksekusi diserialisasi jadi satu blob JSON. Tidak ada mekanisme spill.

**Bukti terukur kita** (BENCH-A03.md, mesin 2 GB / 2 CPU / tanpa swap):

| Item | Payload | Spill (kita) | Inline (cara n8n) |
|---|---|---|---|
| 200.000 | 29,4 MB | 14,0 MB | 200,1 MB |
| 1.000.000 | 148,0 MB | 20,0 MB | 987,5 MB |
| 5.000.000 | 752,9 MB | **50,4 MB** | **OOM-killed pada 1,68 GB** |

Ini bukan masalah tuning — ini konsekuensi arsitektur. Node.js GC + heap JS + blob JSON tidak punya jalur keluar.

### 17.2 Queue mode adalah lompatan kompleksitas

Untuk lolos dari batas single-process, pengguna harus pindah dari 1 container ke **5 service** (main, worker, webhook, PostgreSQL, Redis) + manajemen `N8N_ENCRYPTION_KEY` yang identik + tuning konkurensi.

Banyak pengguna menunda ini sampai mereka terkena masalah di produksi.

### 17.3 Execution data bloat

Blob JSON di `execution_entity` membengkakkan Postgres. Solusinya pruning — yaitu **membuang data**, bukan menyimpannya secara efisien.

### 17.4 Engine v2 sedang dibangun ulang — oleh n8n sendiri

Ada paket `@n8n/engine` yang dideskripsikan sebagai *"Next-generation workflow execution engine — currently scaffolded, not yet wired into production"*.

> **Sinyal penting:** n8n sendiri menilai engine eksekusinya perlu ditulis ulang. Ini validasi bahwa ada keterbatasan arsitektural di engine v1, sekaligus peringatan bahwa **rewrite engine itu sulit bahkan untuk tim yang memiliki kode tersebut sepenuhnya.** Kita berencana melakukan hal yang sama dari nol, dalam bahasa berbeda, sendirian.

### 17.5 Tidak ada pembedaan side-effect untuk retry

Retry pada node non-idempoten bisa menduplikasi efek. Crash di tengah side-effect tidak punya status "tidak diketahui".

### 17.6 Single-threaded CPU

Satu proses Node.js = satu thread untuk JS. CPU-bound work memblokir event loop. Solusinya worker terpisah, bukan paralelisme dalam proses.

### 17.7 Learning curve

Sumber ulasan independen menyebut lompatan dari "connect Slack to Google Sheets" ke "build AI agent with custom tool calling" itu curam, dan fitur lanjutan (queue mode, worker, custom node, LangChain) butuh pengetahuan JavaScript/Node.js yang solid.

---

## 18. Angka Terverifikasi dari Repo (menggantikan estimasi)

**Status: SELESAI diverifikasi 2026-09-09** langsung dari `github.com/n8n-io/n8n`,
branch `master`, versi `n8n-monorepo 2.39.0`.

Metode: GitHub Git Trees API `recursive=1` — satu snapshot penuh repo.
Respons `truncated=false`, 35.072 entri, jadi **hitungan ini lengkap, bukan sampel**.

Perintah reproduksi (sudah diuji — menghasilkan persis angka di §18.1):
```bash
curl -sS "https://api.github.com/repos/n8n-io/n8n/git/trees/master?recursive=1" -o tree.json
cat > verify.py <<'PY'
import json, re, sys
t = json.load(open('tree.json'))['tree']
assert not t or True
d = json.load(open('tree.json'))
if d.get('truncated'): sys.exit('FATAL: tree terpotong, hitungan tidak lengkap')
blobs = [e['path'] for e in d['tree'] if e['type'] == 'blob']

def istest(p): return re.search(r'\.(test|spec)\.ts$', p) is not None

NB = 'packages/nodes-base/'
LC = 'packages/@n8n/nodes-langchain/'

paket = [p for p in blobs if p.endswith('/package.json')]
node  = [p for p in blobs if p.endswith('.node.ts') and not istest(p)
         and (p.startswith(NB + 'nodes/') or p.startswith(LC + 'nodes/'))]
cred  = [p for p in blobs if p.endswith('.ts') and not istest(p)
         and re.match(re.escape(NB) + r'credentials/[^/]+\.ts$', p)
         or (p.endswith('.ts') and not istest(p)
             and re.match(re.escape(LC) + r'credentials/[^/]+\.ts$', p))]
ent   = [p for p in blobs if p.startswith('packages/@n8n/db/src/entities/')
         and p.endswith('.ts') and not istest(p) and not p.endswith('/index.ts')]
nbdir = sorted(set(p.split('/')[3] for p in node if p.startswith(NB)))

print('truncated            :', d.get('truncated'))
print('paket (package.json) :', len(paket))
print('node  (*.node.ts)    :', len(node), '= nodes-base',
      sum(1 for p in node if p.startswith(NB)), '+ langchain',
      sum(1 for p in node if p.startswith(LC)))
print('direktori node NB    :', len(nbdir))
print('credential type      :', len(cred))
print('entity DB            :', len(ent), '(enterprise .ee:',
      sum(1 for p in ent if '.ee.' in p), ')')
PY
python3 verify.py
```

Output yang dihasilkan:
```
truncated            : False
paket (package.json) : 92
node  (*.node.ts)    : 694 = nodes-base 558 + langchain 136
direktori node NB    : 308
credential type      : 445
entity DB            : 63 (enterprise .ee: 18 )
```

> **Catatan:** draf pertama bagian ini memuat one-liner Python yang salah —
> `and` mengikat lebih kuat daripada `or`, sehingga ia menghitung *semua* file
> di bawah `nodes-langchain/nodes/` dan *semua* direktori `credentials/` di
> seluruh repo, menghasilkan 1.374 node dan 630 credential. Angka itu
> **bertentangan dengan tabel di bawahnya**. Snippet di atas sudah diperbaiki
> dan diuji. Ini contoh konkret kenapa setiap angka wajib punya perintah
> reproduksi yang benar-benar dijalankan, bukan yang terlihat benar.

---

### 18.1 Hasil

| Yang dihitung | Angka terverifikasi | Cara menghitung |
|---|---|---|
| **Paket monorepo** | **92** | direktori berisi `package.json` (91 di `packages/`, 1 di `.github/scripts`) |
| **Node nyata** (`*.node.ts`) | **694** | 558 di `nodes-base` + 136 di `nodes-langchain`, tanpa file test |
| Direktori node `nodes-base` | **308** | satu direktori bisa berisi >1 node (mis. node + trigger terpisah) |
| **Credential type** | **445** | 407 di `nodes-base` + 38 di `nodes-langchain` (top-level `.ts`, non-test) |
| Entity database | **63** | `@n8n/db/src/entities`, non-test non-index: **45 community + 18 `.ee` enterprise** |
| Library queue | **`bull` 4.16.4** | dikonfirmasi di `packages/cli/package.json` |
| Klien Redis | **`ioredis` 5.3.2** | + `ioredis-mock ^8.8.1` untuk test |

Rincian distribusi 92 paket: `@n8n/*` 61, `frontend/*` 14, `testing/*` 7,
`modules/*` 3, dan masing-masing 1 untuk `cli`, `core`, `extensions`,
`node-dev`, `nodes-base`, `workflow`.

### 18.2 Klaim lama yang TERNYATA SALAH

| Klaim yang beredar | Kenyataan di repo | Verdict |
|---|---|---|
| "~20 paket" (analisis pihak ketiga, dan **saya tulis di draf awal dokumen ini**) | **92 paket** | ❌ Salah besar — kurang ~4,6x |
| "250+ direktori node" | **308 direktori**, **558 file node** | ❌ Kurang; dan salah satuan (direktori ≠ node) |
| "400+ integrations" (README resmi n8n) | **694 file `.node.ts`** | ⚠️ README menghitung *aplikasi/integrasi*, bukan node. Angka 400+ masuk akal sebagai jumlah aplikasi, tapi **bukan** jumlah node |
| "1.385+ native nodes" | **694** | ❌ Salah ~2x. Tidak ada 1.385 node di repo |
| "1.900+ integrations" | 694 node / 445 credential | ❌ Tidak bisa direproduksi dari repo |
| "BullMQ" (banyak artikel) | **`bull` 4.16.4** | ❌ Salah — Bull dan BullMQ adalah library berbeda |

**Pelajaran dari selisih ini:** hampir semua angka pemasaran — termasuk dari
situs ulasan dan dari README n8n sendiri — menghitung satuan yang berbeda
(aplikasi vs node vs operasi vs credential). 694 node + 445 credential = 1.139,
yang mungkin asal muasal klaim "1.385+". **Angka apa pun tanpa satuan yang
didefinisikan adalah angka yang tidak berguna.**

### 18.3 Yang masih BELUM terverifikasi

| Klaim | Status | Kenapa |
|---|---|---|
| Jumlah template (600+ / 1.700+) | ❌ **TIDAK BISA diverifikasi dari monorepo** | Template workflow tidak disimpan di repo `n8n-io/n8n` — mereka ada di layanan/registry terpisah. Perlu sumber lain, dan sejauh ini hanya klaim pemasaran |
| "500+" / "600+" integrasi | ❌ Tidak direproduksi | Kemungkinan snapshot versi lama atau hitungan aplikasi |

### 18.4 Angka yang kita pakai mulai sekarang

Untuk semua dokumen dan materi publik proyek ini:

> **n8n 2.39.0 berisi 694 node bawaan (558 umum + 136 AI/LangChain),
> 445 credential type, dalam 92 paket monorepo.**
> Sumber: dihitung langsung dari tree repo, 2026-09-09.

Implikasi untuk PRD-2: MVP kita ~26 node = **3,7% dari 694**, bukan 6,5% dari
400. **Gap-nya hampir dua kali lebih besar dari yang saya tulis sebelumnya.**

> **Aturan yang tetap berlaku:** setiap angka yang dikutip harus punya satuan
> jelas dan perintah reproduksi. Bagian ini ada karena draf pertama dokumen ini
> sendiri memuat angka yang salah.

---

## 19. Ringkasan untuk PRD-2

| Aspek n8n | Keputusan yang harus dibuat di PRD-2 |
|---|---|
| Semantik item array | **Tiru persis** — sudah terbukti di kernel kita |
| `INodeProperties` | **Tiru** — sudah ada padanan `ParameterSchema` |
| Declarative HTTP routing | **Tiru dan perluas** via codegen OpenAPI |
| Expression JS | **Tiru via QuickJS** — tidak ada alternatif realistis |
| Context-based execution | **Tiru** — sudah ada `NodeContext` |
| Vue Flow canvas | **Putuskan:** bangun sendiri, pakai library, atau tunda (headless dulu) |
| TypeORM + SQLite/Postgres | **Ganti** sqlx/diesel |
| Express | **Ganti** axum |
| Bull + Redis | **Tunda** — single binary dulu, queue nanti jika perlu |
| `@n8n/di` IoC | **Buang** — Rust tidak butuh |
| Eksekusi single-thread | **Perbaiki** — tokio multi-thread |
| Semua item di heap | **Perbaiki** — spill-to-disk (sudah terbukti) |
| Retry tanpa side-effect awareness | **Perbaiki** — `SideEffect` + `InDoubt` |
| Execution blob JSON | **Perbaiki** — event log + checkpoint terpisah dari payload |
| 694 node | **Jangan ditiru langsung** — subset + codegen |
| Community node npm | **Buang** — menghancurkan keunggulan memori |
| Ekosistem template/forum/docs | **Tidak bisa ditiru cepat** — akui sebagai kelemahan |

---

*Dokumen ini adalah input untuk PRD-2 (`PRD-2-RUST.md`). Setiap klaim bertanda ⚠️ VERIFIKASI harus dikonfirmasi langsung dari repo n8n sebelum dipakai sebagai dasar keputusan atau materi publik.*
