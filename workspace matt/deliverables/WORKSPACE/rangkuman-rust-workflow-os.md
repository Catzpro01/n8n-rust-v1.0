# 📋 Rangkuman Percakapan — "Rust Workflow OS" (n8n-compatible)

**Sumber:** [ChatGPT shared chat — "Simulasi Trading AI"](https://chatgpt.com/share/6aa0a890-8d84-83ec-b976-9005a6d20aae)
**Tanggal rangkuman:** 2026-09-09
**Isi:** Poin penting · Detail rancangan · Keputusan grill-me (D1–D100) · Big plan roadmap

---

## 0. TL;DR

Percakapan ini **berubah arah secara total**. Dimulai dari pertanyaan "cara dapat uang dari AI trading pakai MiroFish", lalu berevolusi menjadi:

> **Membangun ulang n8n dari nol dalam bahasa Rust** — sebuah *Rust-native, n8n-compatible workflow automation engine* yang jauh lebih hemat RAM/CPU, dirancang khusus untuk jalan di **VPS 2 CPU / 2 GB RAM / 50 GB disk**, dengan **UI visual bergaya n8n** dan **Arena AI bertindak sebagai developer (bukan runtime)**.

Proyek ini sudah melewati **100 keputusan arsitektur (D1–D100)** lewat metode `grill-me` (Matt Pocock), dan ditutup dengan **blueprint teknis siap-eksekusi untuk 3 agent Arena yang bekerja paralel**.

**Hukum tertinggi proyek (D100):**
> **Correctness → Measurable Performance → Compatibility**

---

## 1. Alur Evolusi Ide

| Tahap | Topik | Hasil / Keputusan |
|---|---|---|
| 1 | Trading AI dengan **MiroFish** | ❌ Ditolak/ditinggalkan — user di bawah 18 tahun, hanya boleh paper-trading/simulasi |
| 2 | Cara kerja MiroFish | Dipahami: multi-agent simulation (Seed → Knowledge Graph → Persona → OASIS → Memory → Report) |
| 3 | MiroFish di VPS 2GB via API | Bisa, tapi berat & **boros token** (~500rb–1jt token/simulasi, <40 ronde) |
| 4 | **OpenRouter** free tier | 50 req/hari (gratis) → 1.000 req/hari (setelah top-up $10). Kuota **per akun**, bukan per model |
| 5 | **n8n + MiroFish + OpenRouter** | n8n = orchestrator, bukan mesin simulasi |
| 6 | n8n murni (tanpa MiroFish) | Alur: Data → Multi-analyst AI → Evidence Aggregator → Decision → Risk Engine → **Paper Trade** → Evaluasi |
| 7 | n8n **Skills / Agent / MCP** | Bisa. Workflow bisa jadi tool. Fitur masih status *Preview* |
| 8 | Arena AI ↔ n8n | Arena = **AI developer** yang membuat workflow JSON, n8n = execution engine |
| 9 | Data "sekelas Bloomberg" | Bangun **data layer sendiri** (SEC EDGAR, FRED, dsb). **AI bukan sumber data.** Wajib simpan timestamp ketersediaan (hindari look-ahead bias) |
| 10 | **"Buat n8n sendiri dalam Rust?"** | ⭐ Titik balik. Jawaban: sangat realistis & lebih cocok untuk VPS kecil |
| 11 | Arena hanya bantu develop, bukan ikut runtime | ✅ Dikunci — production tidak bergantung pada sesi chat Arena |
| 12 | Buat ulang n8n **lengkap + community nodes** | Diubah jadi *clean-room, n8n-compatible* (bukan clone), karena masalah lisensi & skala |
| 13 | **Mode "Matt" + grill-me** | ChatGPT jadi *strategic orchestrator*, menggali 100 keputusan satu per satu |
| 14 | **Blueprint teknis v1.0** | Output final: blueprint 34 bagian untuk 3 agent Arena |

---

## 2. Poin Penting

### 2.1 Constraint lingkungan (dijadikan *benchmark environment*, bukan sekadar batasan)

```
VPS: 2 CPU core · 2 GB RAM · 50 GB disk
```
> Kalau engine bisa bekerja sangat baik di situ, desainnya **dipaksa efisien sejak awal**.

### 2.2 Prinsip utama

- **Arena AI TIDAK menjadi bagian runtime production.** Arena = development/maintenance workforce saja. Kalau Arena mati, sistem tetap jalan.
- **LLM inference selalu eksternal** (API). VPS hanya orchestration + database + queue + simulator.
- **Clean-room implementation** — jangan salin source code n8n, asset proprietary, trademark, atau kode community node. Tiru **perilaku/interface publik**, bukan implementasi.
- **Jangan mengklaim performa tanpa benchmark.** Jangan mengklaim kompatibilitas n8n tanpa compatibility test.
- **Jangan mengimplementasikan fitur hanya karena "n8n punya fitur itu"** — pahami dulu *mengapa* dan apa *semantics*-nya.

### 2.3 Hukum engineering utama (D4 — "Resource Efficiency Absolut")

> **Jangan menggunakan RAM, CPU, proses, thread, disk I/O, atau network bandwidth lebih banyak daripada yang diperlukan untuk menghasilkan hasil yang benar.**

### 2.4 North Star (final)

> **"Build a Rust-native, n8n-compatible workflow platform whose primary engineering advantage is dramatically lower resource overhead and superior execution efficiency, while progressively approaching feature and behavior parity with n8n."**

### 2.5 Larangan keras (dari "Final Command for Arena Agents")

- ❌ Menambah dependency berat tanpa alasan
- ❌ Memakai Redis / PostgreSQL / distributed workers untuk masalah yang masih bisa diselesaikan SQLite + Tokio
- ❌ Menjadikan UI sebagai sumber execution logic
- ❌ Concurrency statis sebagai solusi utama
- ❌ Menyimpan large payload tanpa batas di RAM
- ❌ Menganggap swap sebagai memory management utama
- ❌ "Temporary duplicate implementation" yang jadi architecture debt

---

## 3. Detail Rancangan (Arsitektur)

### 3.1 Arsitektur target

```
                    WEB UI
                 n8n-style Editor
                       │
                 REST/WebSocket
                       │
                ┌──────▼──────┐
                │  API SERVER │
                │    Axum     │
                └──────┬──────┘
                       │
                ┌──────▼──────┐
                │   RUST CORE │
                │ Workflow    │
                │ DAG/Event   │
                │ Scheduler   │
                │ Runtime     │
                │ State       │
                └──────┬──────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
    RAM Fast       SQLite         File/Blob
      Queue         Durable         Store
         │            State            │
         └────────────┬───────────────┘
                      ▼
              Adaptive Data Plane
                      │
                      ▼
              Native Rust Nodes
                      │
              Optional JS Adapter
```

### 3.2 Core Execution Model — DAG + Event-Driven (Hybrid)

> **Graph menentukan APA yang boleh dijalankan. Event menentukan KAPAN & BAGAIMANA execution bergerak.**

```
Workflow → Dependency Engine → READY TASK → Adaptive Scheduler
   → Node → Execution Event → State + Checkpoint → Dependency Engine → Next Task
```

Contoh event stream:
```
ExecutionCreated → NodeReady(Webhook) → NodeStarted(Webhook)
→ NodeCompleted(Webhook) → NodeReady(HTTP) → ...
```
State bisa direkonstruksi dari event/checkpoint → inilah jalan menuju **resume setelah crash**.

**Wajib mendukung:** branching, parallel execution, retry, timeout, cancellation, waiting, webhook resume, scheduled execution, long-running workflow, pause/resume, crash recovery, subworkflow, human approval, loop safeguards, queue, distributed workers (nanti).

### 3.3 Unit Eksekusi (3 lapisan)

```
WORKFLOW EXECUTION   ← root/container ("Workflow #100 sedang jalan")
      ↓
NODE EXECUTION       ← domain execution ("HTTP Node #5 sedang jalan")
      ↓
TASK                 ← scheduling primitive / atomic work unit scheduler
```

Field Task: `id, workflow_id, execution_id, node_id, attempt, priority, status, payload, created_at, started_at, finished_at`

**Keuntungan:** core tidak terikat jumlah worker. Hari ini 2 CPU/1 process → besok 16 CPU/8 worker → lusa cluster 100 worker. Model eksekusi tetap sama.

### 3.4 Resource Model — Adaptive Parallel Execution

> **"Jalankan sebanyak mungkin yang menguntungkan tanpa melewati budget resource."**
> (Bukan "selalu jalankan sebanyak mungkin", bukan `max_concurrency = 10` statis.)

Scheduler mempertimbangkan: CPU · RAM · I/O · payload size · node resource hints · system pressure saat ini · workflow limits.

Contoh di 2 CPU:
```
B = CPU-heavy  → RUN (CPU 1)
C = network I/O → RUN
D = network I/O → RUN
E = CPU-heavy  → QUEUE
```

### 3.5 Memory Architecture — 3 Lapisan Perlindungan + Memory Governor

```
🥇 Layer 1 → RAM
🥈 Layer 2 → Application Streaming / Spill
🥉 Layer 3 → OS Swap   (emergency safety layer SAJA)
```

> **Swap adalah sabuk pengaman terakhir, bukan tempat penyimpanan workflow.**
> Swap dikendalikan OS → bisa menyebabkan **thrashing** (RAM→SWAP→RAM→SWAP) dan performa jatuh.
> Spill yang kita desain sendiri → deterministik: payload besar → file sementara → RAM hanya simpan reference.

**Memory Governor:**
| State | Aksi |
|---|---|
| 🟢 NORMAL | RAM execution |
| 🟡 PRESSURE | reduce concurrency, stream/spill, large payload → streaming |
| 🔴 CRITICAL | stop new tasks, flush state, protect active executions |

Budget RAM 2 GB dialokasikan ke: OS + services · Rust Core · SQLite · execution memory · safety reserve. **Angka final tidak boleh ditebak — harus diukur dengan benchmark & memory profiling.**

### 3.6 Data Plane — Adaptive Typed Item Model

```
Item
├── JSON metadata
├── Binary references
├── Execution metadata
└── Payload reference (content_id)
```

| Ukuran | Tempat |
|---|---|
| 1 KB – 100 KB JSON | RAM |
| 50 MB payload | streaming / disk |
| 1 GB file | streaming |

> Core **tidak boleh memaksa seluruh payload besar berada di RAM**. Node tetap "merasa" menerima data, tapi core hanya membawa reference.

Fondasi untuk: binary files, streaming, large API responses, file processing, AI responses besar, database results besar, workflow jutaan item.

### 3.7 Streaming — Item-by-Item (streaming-first)

```
Producer
  ├── item 1 → Consumer
  ├── item 2 → Consumer
  ├── item 3 → Consumer
  └── ...
```
- RAM jauh lebih rendah, latency lebih rendah
- Backpressure bisa diterapkan saat consumer lebih lambat
- **Konsekuensi:** tidak semua node bisa benar-benar streaming (mis. agregasi/sort butuh seluruh input) → core harus mendukung node yang **menyatakan kebutuhan buffering secara eksplisit**

### 3.8 Queue — Hybrid

```
TASK CREATED → SQLite (durable source) → RAM Queue (fast path)
             → Adaptive Scheduler → Worker / Worker / Worker
```
- **RAM queue:** bounded, cepat, backpressure
- **SQLite:** durable, recovery source, task state, **lease/claim**
- Crash → RAM Queue ❌, SQLite ✅ → Recovery → RAM Queue dibangun kembali
- **Redis tidak diperlukan untuk MVP**

### 3.9 State & Recovery — Hybrid Event Log + Checkpoint

```
Execution ├── Event Log  └── Checkpoint → SQLite
```
Skenario: `A✓ B✓ C✓ D RUNNING` → 💥 VPS crash → restart → `A✓ B✓ C✓`, D retry/resume, E wait. **Tidak mengulang A/B/C.**

Status state machine: `PENDING → READY → RUNNING → WAITING → SUCCESS / FAILED / CANCELLED / RETRYING` (+ paused)

### 3.10 Node API — Async Node + Streaming Interface

```
Node
├── input stream
├── output stream
├── context
├── cancellation
├── checkpoint support
└── resource hints   (CPU: HIGH · RAM: LOW · I/O: HIGH · Streaming: YES)
```

Konsep Rust:
```rust
#[async_trait]
pub trait Node: Send + Sync {
    async fn execute(
        &self,
        input: NodeInput,
        context: ExecutionContext,
    ) -> Result<NodeOutput, NodeError>;
}
```

**Node = plugin terhadap core. Core TIDAK boleh tahu implementasi internal node.**
Lifecycle: `load → validate → execute → cancel → cleanup`

### 3.11 Node Runtime — Compatibility Tiers

```
Native Rust Nodes (jalur utama)
      └── Optional isolated JS compatibility layer (hanya bila kompatibilitas menuntut)
```

| Level | Isi |
|---|---|
| A | Native Rust: HTTP, Postgres, Redis, JSON, Transform |
| B | OpenAPI-generated nodes: GitHub, Stripe, Notion, dll |
| C | Compatibility runtime: JS community node → Sandbox → Rust Adapter → Core |

**Ide menarik:** `OpenAPI spec → Rust Node Generator → Node definition → Rust connector → Workflow` — satu spesifikasi API menghasilkan node otomatis.

### 3.12 Storage — SQLite-first

Konfigurasi: WAL · prepared statements (SQLx) · transactions · migrations · indexes · compact execution history · configurable retention.

Tabel: `workflows, workflow_versions, executions, execution_events, checkpoints, tasks, leases, credentials, audit_events, metadata`

> Large payload **tidak dipaksa masuk SQLite**.

### 3.13 File/Blob Store

Untuk: binary data, large payload, execution artifacts, spilled data, large logs.
Lifecycle: `CREATE → REFERENCE → EXECUTION → RETENTION → GC` (garbage collector membersihkan artifact yang tidak direferensikan lagi).

### 3.14 API — Axum (REST + WebSocket)

- **REST:** workflow CRUD, execution control, credentials, configuration, health, admin
- **WebSocket:** realtime execution, node status, logs, progress, events
- **Health:** `/health`, `/ready`
- Contoh: `POST /workflows`, `GET /workflows/:id`, `POST /workflows/:id/execute`, `GET /executions/:id`, `POST /executions/:id/cancel`

### 3.15 Workflow Format — Versioned JSON Schema + n8n Importer

Mendukung: workflow version, node type, node version, connections, parameters, credential references, settings, metadata.

```
n8n workflow → Parser → Converter → Compatibility Validator → Internal Workflow
```
Export ke format kompatibel n8n **jika memungkinkan**.

### 3.16 Expression Engine (subsystem tersendiri, sering diremehkan)

Harus mendukung: `{{ $json.name }}`, `{{ $json.items[0] }}`, `{{ $execution.id }}`, `{{ $node["HTTP"].json }}`
Syarat: deterministic · sandboxed · fast · memory-efficient · tidak memberi akses sistem bebas.

### 3.17 Error Model

```
Retry → Exponential Backoff → Jitter → Retry limit → Dead/Quarantine
```
Support: node error, workflow error, timeout, cancellation, resource exhaustion, dependency failure.

### 3.18 Wait / Schedule / Webhook

- **Wait:** jangan tidur pakai RAM → `persist state → release resources → WAKE EVENT → resume`
- **Schedule:** lightweight scheduler
- **Webhook:** `HTTP Request → Async Event Ingestion → Durable Event → Workflow Execution` + **idempotency** untuk cegah duplicate execution

### 3.19 Security

- Credentials: **encrypted at rest**, node hanya menerima credential yang diperlukan
- Logging: **automatic secret redaction**
- Network: configurable egress policy
- Resource abuse: quota per workflow / node / execution
- Auth: **JWT + API Key** · Authorization: **RBAC** · Multi-user: **workspace/project isolation**

### 3.20 Versioning

Workflow versioned · Explicit node versions · Backward-compatible schema evolution · **Immutable execution record** · Concurrent editing: optimistic versioning + conflict detection

### 3.21 Observability (tidak boleh ditambahkan belakangan)

Structured logs · Metrics (Prometheus-compatible) · Tracing (OpenTelemetry-compatible) · Append-only audit events · Webhook alerting

Metrics penting: `execution_duration, node_duration, CPU_usage, RAM_usage, peak_RAM, queue_depth, spill_bytes, active_tasks, failed_tasks, retry_count`

Contoh tampilan execution timeline:
```
Execution #4821
HTTP Request       182ms
Transform           4ms
IF                  1ms
Postgres            21ms
------------------------
Total              208ms
```

### 3.22 Testing (wajib)

`Unit · Integration · Property · Compatibility · Crash Recovery · Load · Memory · Performance Benchmarks`

### 3.23 Deployment

**Single binary** · satu core process · Tokio async tasks sebagai worker model · Docker **opsional** · external dependency minimal · **tanpa Redis untuk MVP** · PostgreSQL = future adapter.

### 3.24 UI — n8n-style Visual Editor

> **Ya, tampilan node di website tetap seperti n8n.** Yang diubah adalah mesin di belakang layar, bukan pengalaman visualnya.
> User tetap: **drag → connect → configure → execute**.

Node punya: icon, nama, input, output, connection, parameter panel, credential selector, execution status, error indicator.
Status visual: `🟢 Webhook → 🟢 HTTP Request → 🟡 IF ↙🟢 DB ↘🔴 Slack`
Klik node → `Status / Duration / Input / Output / Memory / Retry`

**Nilai tambah UI kita — "Resource Intelligence":**
```
Workflow: Order Processing
Nodes: 187      Execution: 2.4 s     Peak RAM: 184 MB
Spilled: 32 MB  CPU: 0.71 core       Tasks: 193

Memory Governor: 🟢 NORMAL
RAM Budget: 512 MB · Used: 184 MB · Spill: 32 MB
```
Frontend: **TypeScript + Vue**. UI **tidak boleh mengandung execution semantics** — backend tetap source of truth.

### 3.25 Struktur Project

```
rust-n8n-core/
├── crates/
│   ├── workflow/          ├── node-sdk/         ├── credentials/
│   ├── execution/         ├── node-runtime/     ├── storage/
│   ├── scheduler/         ├── data-plane/       ├── api-server/
│   ├── queue/             ├── resource-governor/├── compatibility/
│   ├── state/             ├── expression/       └── observability/
│   └── runtime/
├── nodes/  (http, webhook, transform, database, logic, ...)
├── frontend/
├── migrations/
├── tests/ (unit, integration, compatibility, crash-recovery, performance, memory)
├── benchmarks/
├── docs/
├── Cargo.toml
└── README.md
```

---

## 4. Keputusan grill-me — D1 s/d D100 (Decision Register)

### 🔒 Keputusan yang digali satu-per-satu (D1–D12)

| # | Topik | Keputusan |
|---|---|---|
| **D1** | Compatibility target | **Maximum practical n8n compatibility.** MVP = Rust-native core dulu; compatibility layer bertahap; community nodes fase tersendiri; kompatibilitas diukur dengan **test suite, bukan klaim** |
| **D2** | Bentuk core | **Headless / API-first.** Tidak ada UI logic di execution engine |
| **D3** | Execution model | **Hybrid: DAG + Event-Driven** (bukan pure DAG, bukan pure event) |
| **D4** | Hukum engineering | **Resource efficiency absolut** |
| **D5** | Data plane | **Adaptive Data Plane + Memory Budget + Streaming/Spill** · D5+: Layer1 RAM, Layer2 App Spill/Streaming, Layer3 OS Swap, + **Memory Governor** |
| **D6** | Durability | **Hybrid Event Log + Checkpoint** (di SQLite) |
| **D7** | Parallelism | **Adaptive Parallel Execution + Resource-Aware Concurrency** |
| **D8** | Task queue | **Hybrid: RAM fast queue + SQLite durable queue** (tanpa Redis) |
| **D9** | Data model eksekusi | **Adaptive Data Model** (kecil→RAM, besar→streaming, sangat besar→spill, metadata/state→SQLite) |
| **D10** | Model data antar-node | **Hybrid Typed Item Model** (json metadata + binary refs + execution metadata + payload reference) |
| **D11** | Execution semantics | **Item-by-Item Streaming** (streaming-first; node boleh declare buffering eksplisit) |
| **D12** | Model node | **Async Node + Streaming Interface** (bukan trait sederhana, bukan proses terpisah) |

### D13–D22 — Execution

| # | Keputusan | Pilihan |
|---|---|---|
| D13 | Node API | B — Context + Cancellation + Resource Hints |
| D14 | Error handling | C — Retry + Backoff + Dead-letter |
| D15 | Workflow state | C — State machine persisted |
| D16 | Trigger | C — Event-driven unified trigger |
| D17 | Wait/Sleep | B — Persist lalu wake-up |
| D18 | Cron/Schedule | B — Lightweight scheduler |
| D19 | Subworkflow | B — Native nested execution |
| D20 | Cancellation | B — Cooperative cancellation |
| D21 | Node isolation | B — Native Rust + optional sandbox |
| D22 | Plugin/community node | C — Native Rust + isolated JS compatibility layer |

### D23–D32 — API / Storage / Compatibility

| # | Keputusan | Pilihan |
|---|---|---|
| D23 | API | B — REST + WebSocket |
| D24 | Database | B — SQLite WAL + migration |
| D25 | Secrets | C — Encrypted credential store |
| D26 | Observability | C — Lightweight logs + metrics + tracing |
| D27 | Webhook | B — Async event ingestion |
| D28 | HTTP client | B — Connection pooling + streaming |
| D29 | Memory pressure | C — Governor + backpressure + spill |
| D30 | Large files | B — Streaming file/blob store |
| D31 | Workflow format | C — Versioned JSON schema + n8n importer |
| D32 | Compatibility | C — Automated behavioral test suite |

### D33–D42 — Runtime / Tech Stack

| # | Keputusan | Pilihan |
|---|---|---|
| D33 | Runtime async | **B — Tokio** |
| D34 | HTTP server | **B — Axum** |
| D35 | Serialization | **B — Serde + JSON** |
| D36 | Internal events | B — Typed Rust events |
| D37 | Concurrency | C — Bounded adaptive concurrency |
| D38 | SQLite access | **B — SQLx + prepared queries** |
| D39 | File cleanup | C — Lifecycle + garbage collector |
| D40 | Crash recovery | C — Automatic reconciliation |
| D41 | Config | B — TOML + environment override |
| D42 | Testing | C — Unit + integration + property + compatibility |

### D43–D52 — UI

| # | Keputusan | Pilihan |
|---|---|---|
| D43 | UI | B — n8n-style visual editor |
| D44 | Frontend | **B — TypeScript + Vue** |
| D45 | UI/Core komunikasi | B — REST + WebSocket |
| D46 | Execution view | C — Realtime event stream |
| D47 | Debugging | C — Per-node input/output inspection |
| D48 | Workflow versioning | C — Immutable execution + versioned workflow |
| D49 | Import n8n | C — Converter + compatibility validator |
| D50 | Export | B — Own format + n8n-compatible export where possible |
| D51 | Community nodes | C — Rust native + isolated JS adapter |
| D52 | Performance target | **C — Benchmark gate; setiap optimasi wajib terukur** |

### D53–D62 — Security / Reliability

| # | Keputusan | Pilihan |
|---|---|---|
| D53 | Auth | B — JWT + API key |
| D54 | Permissions | C — RBAC |
| D55 | Multi-user | B — Workspace/project isolation |
| D56 | Queue priority | C — Adaptive priority |
| D57 | Retry | C — Exponential backoff + jitter |
| D58 | Timeout | B — Per-node configurable timeout |
| D59 | Rate limit | C — Adaptive per-node/provider |
| D60 | Idempotency | C — Execution/node idempotency keys |
| D61 | Transactions | B — SQLite transaction boundaries |
| D62 | Backup | C — Incremental/consistent SQLite backup |

### D63–D72 — Observability / Resource Control

| # | Keputusan | Pilihan |
|---|---|---|
| D63 | Logging | B — Structured logs |
| D64 | Metrics | B — Prometheus-compatible |
| D65 | Tracing | B — OpenTelemetry-compatible |
| D66 | Health check | B — `/health` + `/ready` |
| D67 | Audit | C — Append-only audit events |
| D68 | Alerting | B — Webhook-based |
| D69 | Dead tasks | C — Automatic recovery + quarantine |
| D70 | Worker model | B — Async Tokio tasks |
| D71 | CPU control | C — Adaptive CPU budget |
| D72 | RAM control | C — Hard memory budget + governor |

### D73–D82 — Deployment

| # | Keputusan | Pilihan |
|---|---|---|
| D73 | Deployment | B — Single binary |
| D74 | Runtime dependency | C — Minimal external dependencies |
| D75 | Container | B — Optional Docker |
| D76 | Process model | B — One core process |
| D77 | Worker scaling | C — Adaptive internal workers |
| D78 | Distributed mode | B — Future extension, bukan MVP |
| D79 | Message broker | B — **Tidak perlu Redis untuk MVP** |
| D80 | Database scaling | B — SQLite dahulu, PostgreSQL adapter nanti |
| D81 | Plugin loading | C — Dynamic registry + isolated adapters |
| D82 | Installation | B — Single binary + simple config |

### D83–D100 — Advanced Reliability / Compatibility

| # | Keputusan | Pilihan |
|---|---|---|
| D83 | Workflow locking | C — Optimistic versioning |
| D84 | Concurrent editing | B — Conflict detection |
| D85 | Execution history | C — Compact + configurable retention |
| D86 | Large execution logs | B — Stream/spill to disk |
| D87 | Database migration | C — Versioned automatic migration |
| D88 | Schema evolution | C — Backward-compatible versioning |
| D89 | Node versioning | C — Explicit node versions |
| D90 | Workflow compatibility | C — Versioned compatibility layer |
| D91 | Expressions | B — Dedicated lightweight expression engine |
| D92 | Credentials | C — Encrypted at rest + never exposed unnecessarily |
| D93 | Secrets in logs | C — Automatic redaction |
| D94 | Network security | C — Configurable egress policies |
| D95 | Resource abuse | C — Per-workflow/node quotas |
| D96 | Infinite loops | C — Execution limits + cycle safeguards |
| D97 | Human approval | B — Persisted wait/resume state |
| D98 | Webhook reliability | C — Durable ingestion + idempotency |
| D99 | Upgrade strategy | C — Backward-compatible migrations + rollback |
| **D100** | **Architecture law** | **C — Correctness first → measurable performance → compatibility** |

### Fondasi terkunci (ringkas)

```
Rust Core
├── DAG + Event Engine
├── Adaptive Scheduler
├── Resource Governor
├── Fast RAM Queue
├── SQLite Durable Queue
├── Streaming-first Data Plane
├── Hybrid Typed Items
├── Event Log + Checkpoint
├── Persistent State Machine
├── Unified Event Triggers
├── Native Rust Nodes
└── Optional JS Compatibility Layer
```

---

## 5. Big Plan Roadmap

### 5.1 Strategi "Matt" (metode kerja)

```
NORTH STAR → CAPABILITY MAP → ARCHITECTURE → DEPENDENCY GRAPH
→ MILESTONES → SMALL TASKS → IMPLEMENT → TEST → VERIFY → NEXT TASK
```

Setiap keputusan besar melewati: **Challenge → Recommendation → Decision → ADR → Implementation**
(sejalan dengan `grill-with-docs`: keputusan & glossary disimpan agar tidak hilang setelah sesi selesai).

**Aturan main Matt sebagai gatekeeper arsitektur:**
1. Jangan terlalu cepat coding — kalau arsitektur belum jelas: **STOP → GRILL**
2. **Satu keputusan penting pada satu waktu** (bukan 50 pertanyaan sekaligus)
3. Setiap keputusan besar punya: `Decision · Why · Alternatives · Trade-offs · Consequence`
4. Setiap milestone punya **acceptance criteria**
5. Arena **tidak boleh mengarang progress** — kalau belum diuji: `NOT DONE`, bukan "seharusnya sudah selesai"
6. Worker **tidak boleh memperluas scope sendiri** — perubahan arsitektur harus kembali ke Matt/Orchestrator

### 5.2 BIG ROADMAP — Capability (PHASE 0 → 17)

| Phase | Nama | Isi | Output/Kriteria |
|---|---|---|---|
| **0** | **Project Constitution** | `PROJECT.md, VISION.md, ARCHITECTURE.md, DOMAIN.md, ROADMAP.md, CONTEXT.md, docs/adr/` + aturan: Rust-first, Async-first, Modular, Testable, Observable, Crash-resilient, Backward-compatible, No premature optimization, No feature without acceptance criteria | Sebelum kode |
| **1** | **Workflow Domain Model** | Workflow/Node/Connection/Input/Output/Parameter/Credential ref/Metadata · schema · node identity · expressions · validation · serialization · versioning. **Belum ada DB besar, belum ada UI, belum ada AI** | Fondasi paling bawah |
| **2** | **Execution Engine** ⭐ *jantung proyek* | Planner → Execution Context → Scheduler → Node Executor → Output → Next Node. Sequential, branching, multi-input/output, cancellation, timeout, error propagation, partial execution | Acceptance test: `A→B→C`, lalu `A→{B,C}→D`, lalu `A→B→{C,D,E}`. **Kalau graph semantics belum benar, JANGAN LANJUT** |
| **3** | **State Engine** | Status: pending/running/waiting/success/failed/cancelled/paused. SQLite: workflows, executions, execution_events, node_state, checkpoints | Kalau proses mati, workflow tidak kehilangan keadaan |
| **4** | **Queue + Scheduler** | Priority, concurrency limits, retry, exponential backoff, timeout, cancellation, backpressure, starvation prevention, graceful shutdown. **Bounded concurrency dari awal** | Setelah execution semantics stabil |
| **5** | **Node SDK** | `trait Node` + lifecycle `load/validate/execute/cancel/cleanup`. Node = plugin. `node-http, node-if, node-switch, node-transform, node-code, node-merge, node-loop, node-delay` | Core tidak boleh tahu internal node |
| **6** | **Essential Nodes** | **Tier 1:** Manual Trigger, Schedule Trigger, Webhook, HTTP Request, Set, Edit Fields, IF, Switch, Merge, Code/Transform, Wait, NoOp · **Tier 2:** Loop, Split, Aggregate, Filter, Sort, Limit, Execute Workflow, Subworkflow · **Tier 3 (DB):** PostgreSQL, SQLite, MySQL, Redis | — |
| **7** | **Expression Engine** | `{{ $json.name }}`, `{{ $execution.id }}`, `{{ $node["HTTP"].json }}` — aman & deterministik, **subsystem tersendiri** | Sering diremehkan |
| **8** | **Credentials & Secrets** | Node → Credential Reference → Secret Manager → Runtime. **Tidak ada secret plaintext di workflow** | — |
| **9** | **Trigger System** | HTTP Webhook, Cron, Interval, Event, Queue, DB event, External webhook → `Trigger → Create Execution → Queue → Scheduler → Workflow` | — |
| **10** | **API** | REST/Webhook/Execution/Workflow/Credential/Node/Health/Metrics API | — |
| **11** | **Observability** | Tracing, logs, metrics, execution timeline, node timing, error context, resource usage | **Tidak boleh ditambahkan belakangan** |
| **12** | **Web UI** | Workflow editor (canvas). **UI bukan tempat workflow semantics berada — backend tetap source of truth** | Setelah semantics engine stabil |
| **13** | **AI + MCP** | LLM Node, AI Agent, Tool, Memory, MCP. **MCP = integration/tool protocol, bukan fondasi engine** | Setelah core automation matang |
| **14** | **Community Node Compatibility** ⚠️ *tersulit* | Node Registry → {Native Rust · HTTP/OpenAPI · JS Adapter}. Level A: HTTP/Postgres/Redis/JSON/Transform. Level B: OpenAPI-generated (GitHub/Stripe/Notion). Level C: JS community node → Sandbox → Rust Adapter → Core | **Jangan menulis ulang ribuan node** |
| **15** | **n8n Compatibility Layer** | Compatibility Test Suite: `input sama → semantics sama → output sama → error behavior sama` | Bukan "kodenya mirip" |
| **16** | **Scale & Hardening** | SQLite→Postgres · Single process→Worker architecture · Local queue→Distributed queue — **hanya kalau memang dibutuhkan** | Jangan distributed terlalu dini |
| **17** | **Production Grade** | Rust Workflow OS = Workflow Engine + Node Ecosystem + Platform → Production Runtime | Target akhir |

### 5.3 ROADMAP IMPLEMENTASI v1.0 (Phase 0 → 10)

| Phase | Implement | Output |
|---|---|---|
| **0 — Foundation** | Cargo workspace, crate boundaries, CI, error model, configuration, logging, basic database, migration system | Project dapat compile & test |
| **1 — Workflow Core** | Workflow schema, nodes, connections, DAG, validation, workflow versioning | `Workflow → Validated DAG` |
| **2 — Execution Engine** | Execution state machine, dependency engine, node lifecycle, async node API, streaming interface, cancellation, timeout | `A → B → C` dieksekusi benar |
| **3 — Queue + Scheduler** | SQLite durable queue, RAM fast queue, task lease, task claim, adaptive scheduler, priority, concurrency limits | Parallel execution yang resource-aware |
| **4 — Data Plane** | Typed Item, JSON, binary references, streaming, spill, backpressure, memory governor, file/blob lifecycle | Workflow besar tidak memakan seluruh RAM |
| **5 — Recovery** | Event log, checkpoint, crash reconciliation, retry, dead/quarantine, persisted wait, resume | `CRASH → RECOVER → CONTINUE` |
| **6 — Essential Nodes** | HTTP Request, Webhook, Set/Transform, IF, Switch, Code/Expression, Database, Merge, Loop, Wait, Schedule — tiap node support resource hints & streaming | — |
| **7 — API** | REST, WebSocket, authentication, RBAC, execution control, workflow CRUD, realtime events | — |
| **8 — UI** | Canvas, Nodes, Connections, Node Config, Execution View, Logs, Input/Output, Workflow Versions | UI **tidak boleh** mengandung execution semantics |
| **9 — n8n Compatibility** | `n8n JSON → Parser → Converter → Compatibility Validator → Internal Workflow` + compatibility test corpus | — |
| **10 — Performance Engineering** | Benchmark 10 / 100 / 1.000 / 10.000 nodes. Ukur: CPU, RAM, latency, throughput, queue depth, spill, recovery, parallelism | **Tidak hanya cepat di workflow kecil, tapi tetap terkendali saat workflow & data membesar** |

### 5.4 Pembagian 3 Agent Arena (Ownership Boundary)

> **Jangan biarkan tiga agent mengedit area yang sama secara bebas.**

#### 🤖 AGENT 1 — CORE EXECUTION
**Ownership:** `crates/workflow/`, `crates/execution/`, `crates/scheduler/`, `crates/queue/`, `crates/state/`, `crates/runtime/`, `crates/node-sdk/`

**Tugas:** ① Workflow graph ② DAG engine ③ Execution state machine ④ Async node interface ⑤ Streaming execution ⑥ SQLite queue ⑦ RAM queue ⑧ Adaptive scheduler ⑨ Retry ⑩ Timeout ⑪ Cancellation ⑫ Checkpoint/event log ⑬ Crash recovery

**Deliverable:** `Rust Core + Execution Engine + Scheduler + Queue + Recovery`

#### 🤖 AGENT 2 — DATA / RESOURCE / NODE PLATFORM
**Ownership:** `crates/data-plane/`, `crates/resource-governor/`, `crates/storage/`, `crates/expression/`, `crates/credentials/`, `nodes/`, `crates/compatibility/`

**Tugas:** ① Typed Item ② Streaming ③ Binary reference ④ Spill ⑤ File/blob store ⑥ Memory Governor ⑦ CPU budget ⑧ Backpressure ⑨ Expression engine ⑩ Credential encryption ⑪ Secret redaction ⑫ Native nodes ⑬ JS compatibility boundary ⑭ n8n workflow conversion

**Deliverable:** `Adaptive Data Plane + Resource Control + Node Platform + Compatibility Layer`

#### 🤖 AGENT 3 — API / UI / TEST / OBSERVABILITY
**Ownership:** `crates/api-server/`, `crates/observability/`, `frontend/`, `tests/`, `benchmarks/`, `docs/`

**Tugas:** ① Axum API ② REST ③ WebSocket ④ Authentication ⑤ RBAC ⑥ UI visual editor ⑦ Execution monitoring ⑧ Realtime events ⑨ Structured logs ⑩ Metrics ⑪ Tracing ⑫ Compatibility tests ⑬ Integration tests ⑭ Crash tests ⑮ Performance benchmarks ⑯ Documentation

**Deliverable:** `API + UI + Observability + Test/Benchmark System`

### 5.5 Agent Coordination & Integration Contract

```
        Agent 1
       ┌───┴───┐
       ▼       ▼
    Agent 2  Agent 3
       └───┬───┘
           ▼
      Integration → v1.0
```
Development berjalan paralel menggunakan **stable interfaces**.

Agent **tidak boleh** mengubah public interface agent lain tanpa: ① document change ② update interface ③ update tests ④ notify dependent agent.

Interface = contract: `Node, ExecutionContext, Item, Task, ExecutionEvent, Checkpoint, ResourceHint`. Breaking change harus dicatat sebelum merge.

### 5.6 Definition of Done

```
Code + Unit Test + Integration Test + Error Handling
+ Resource Accounting + Documentation + Benchmark (bila performance-sensitive)
```
Untuk execution-critical feature, tambahan wajib: **Crash Test + Recovery Test**.
> Feature **belum** dianggap selesai hanya karena compile.

### 5.7 Performance Law

Setiap optimasi wajib menjawab:
```
RAM turun berapa? · CPU turun/naik berapa? · Latency berubah berapa?
Throughput berubah berapa? · Correctness tetap?
```
**Benchmark sebelum dan sesudah. Jangan optimasi berdasarkan asumsi.**

### 5.8 MVP Acceptance Test (v1.0)

```
Create workflow → Connect nodes → Execute → Stream items → Parallelize
→ Handle error → Retry → Persist state → Crash → Recover → Continue
```
UI harus bisa: `Create · Edit · Save · Run · Stop · Inspect · Debug · View execution`

### 5.9 Prioritas pengerjaan (Final Command)

```
1. Correct execution semantics
2. Stable interfaces
3. Resource safety
4. Crash recovery
5. Performance
6. n8n compatibility
7. Feature expansion
```

### 5.10 Definisi "v1.0 Selesai"

```
Rust Core + Workflow Engine + Adaptive Scheduler + Hybrid Queue
+ Streaming Data Plane + Memory Governor + Crash Recovery
+ Essential Nodes + REST/WebSocket API + n8n-style UI
+ n8n Import + Compatibility Tests + Performance Benchmarks
+ Security + Observability
```
…berjalan sebagai sistem yang **benar, recoverable, resource-efficient, terukur, dan siap dikembangkan lebih jauh.**

---

## 6. Konteks Pendukung (Bagian Trading — sudah tidak dipakai, tapi berguna)

### 6.1 Kenapa MiroFish ditinggalkan
- Boros token: ~**500rb–1jt token** per simulasi awal (30 agent × 30 ronde × banyak interaksi)
- Rekomendasi fork: mulai **<40 ronde**, **10–20 agent** dulu, naik bertahap
- Hasilnya **simulasi skenario, bukan jaminan prediksi**
- Riset: kemampuan prediksi AI **tidak otomatis** = performa trading bagus di kondisi nyata

### 6.2 Batas OpenRouter free model
| Kondisi | Kuota |
|---|---|
| Tanpa kredit | **50 request/hari** |
| Setelah top-up ≥ $10 | **1.000 request/hari** |

> Berlaku **per akun secara keseluruhan**, bukan 1.000 × jumlah model. Model gratis juga punya rate limit provider masing-masing. `openrouter/free` memilih model gratis otomatis; OpenRouter mendukung **model fallback** saat rate limit.

### 6.3 Arsitektur paper-trading n8n (kalau nanti dibutuhkan)
```
Scheduler → Market Data Layer → Data Normalizer
→ {Technical Analyst ∥ Fundamental Analyst} → News/Macro AI
→ Evidence Aggregator (+confidence score) → Decision Analyst
→ Risk Engine (rules-based) → PAPER TRADE ONLY
→ Performance DB → Evaluation Engine (accuracy/P&L/drawdown/bias) → Telegram
```
**Jangan:** `Data → AI → BUY/SELL`. **AI Research Loop:** analysis → paper signal → outcome → evaluation → AI Reviewer ("why wrong?") → update rules → next session.

### 6.4 "Bloomberg-like Data Layer"
- **Jalur resmi:** Bloomberg Data License (pricing, reference, fundamentals, estimates, historical, risk, ESG) via REST/SFTP/Cloud; real-time = B-PIPE. Tunduk pada lisensi.
- **Jalur bangun-sendiri (untuk riset):** Market data (pricing) + Fundamentals (**SEC EDGAR API** — submissions & XBRL JSON) + Macro (**FRED API** — mendukung *real-time periods* untuk rekonstruksi data historis → **menghindari look-ahead bias**) → n8n ETL → Unified DB → AI Analyst
- ⚠️ **Setiap data wajib disimpan bersama timestamp kapan informasi itu tersedia**
- **AI tidak boleh menjadi sumber data** — AI hanya reasoning atas data yang sudah dikumpulkan

---

## 7. Yang Masih Terbuka / Next Action

- [ ] **Angka budget RAM final** — sengaja tidak ditebak; harus diukur dengan benchmark & memory profiling
- [ ] **Phase 0: Project Constitution** — buat `PROJECT.md`, `VISION.md`, `ARCHITECTURE.md`, `DOMAIN.md`, `ROADMAP.md`, `CONTEXT.md`, `docs/adr/`
- [ ] Tulis **ADR** untuk D1–D100 agar keputusan tidak hilang antar-sesi
- [ ] Definisikan **stable interfaces** (`Node`, `ExecutionContext`, `Item`, `Task`, `ExecutionEvent`, `Checkpoint`, `ResourceHint`) **sebelum** 3 agent mulai paralel
- [ ] Siapkan **compatibility test corpus** (workflow n8n asli + expected behavior)
- [ ] Siapkan **benchmark harness** (10 / 100 / 1.000 / 10.000 nodes) sejak awal, bukan di akhir
- [ ] Verifikasi **lisensi n8n & community nodes** sebelum menulis compatibility layer (clean-room wajib)

---

*Dokumen ini disusun dari 20 chunk percakapan lengkap. Bagian blueprint akhir (§3, §5.3–5.10) adalah output final yang dimaksudkan untuk langsung diberikan ke 3 agent Arena.*
