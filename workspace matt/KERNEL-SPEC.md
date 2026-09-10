# 🧊 KERNEL SPEC — `crates/kernel`

**Dokumen:** D115 (Kernel Freeze) — prasyarat sebelum crate apa pun ditulis
**Menyelesaikan temuan audit:** A-01, A-03, A-05, A-10, A-16, A-30, D101, D102, D105, D106, D107, D109
**Status:** ⬜ Menunggu review → setelah disetujui, **beku** (perubahan wajib lewat ADR)

---

## 0. Kenapa dokumen ini ada

Tiga temuan CRITICAL audit semuanya berujung ke **satu pertanyaan yang belum terjawab**:

> *Apa bentuk data yang mengalir antar-node?*

- **A-01** — ekspresi n8n `$('Node').item.json` butuh random access ke output node sebelumnya
- **A-03** — D11 (streaming-first) membalik semantik array n8n dan merusak `Merge`/`Sort`/`pairedItem`
- **A-05** — tujuh tipe inti tidak terdefinisi dan crate-nya depend melingkar

Semuanya selesai begitu `Item` dan `ItemList` didefinisikan dengan benar. Itu isi dokumen ini.

---

## 1. Resolusi A-03 — wawasan kuncinya

D11 memilih *"item-by-item streaming"* demi menghemat RAM. Audit menemukan ini merusak kompatibilitas, karena n8n mengoper **array of items** dan banyak node butuh seluruh array.

**Resolusinya bukan memilih antara streaming dan array. Resolusinya menyadari keduanya bukan lawan.**

> ### *Addressable* ≠ *resident in RAM*
>
> Sebuah list bisa **sepenuhnya ter-index dan bisa diakses acak** (semantik n8n terjaga)
> sementara **item-nya sendiri tidak sedang berada di RAM** (efisiensi terjaga).

```
                    n8n (Node.js)                 Engine kita (Rust)
                    ─────────────                 ──────────────────
1.000.000 item  →   semua jadi objek JS      →    index offset di RAM (~8 MB)
                    di RAM sekaligus               item di spill file (disk)
                    ~ beberapa GB                  dibaca & di-deserialize
                    → OOM di 2 GB                  satu per satu saat dibutuhkan
                                                   → RAM tetap < 100 MB
```

**Ini seluruh tesis efisiensi proyekmu, dalam satu kalimat.** Bukan "kurangi fitur", tapi "pindahkan resident set ke disk sambil mempertahankan semantik." Node.js tidak bisa melakukan ini dengan mudah karena GC dan object overhead; Rust bisa, karena kamu mengontrol layout dan lifetime.

Konsekuensinya: **D11 direvisi.** Streaming bukan *default semantics*, melainkan *optimasi internal* di balik antarmuka yang tetap array-based.

---

## 2. Aturan dependency (resolusi A-05)

```
kernel  →  TIDAK depend ke crate internal mana pun
  ↑
semua crate lain depend ke kernel
```

Graf crate wajib **DAG**. Yang lama melingkar (`node-sdk` butuh `Item` dari `data-plane`, `data-plane` butuh `Node` dari `node-sdk`). Diperbaiki dengan menaruh semua tipe bersama di `kernel`.

```
                          kernel
        ┌──────────┬────────┴───────┬──────────┬──────────┐
        ↓          ↓                ↓          ↓          ↓
  expr-quickjs  storage        data-plane   workflow   observability
        │          │                │          │
        └────┬─────┴────────┬───────┘          │
             ↓              ↓                  │
         execution  ←────  queue               │
             ↓                                 │
         scheduler ←───────────────────────────┘
             ↓
           nodes
             ↓
        node-codegen   ← OpenAPI → Rust (lihat §7)
             ↓
        api-server
             ↓
        frontend (Vue)
```

**Dependency eksternal `kernel` dibatasi keras:**
```
serde · serde_json · async-trait · tokio-util (CancellationToken)
time / chrono-tz · thiserror · bytes
```
Tidak ada `reqwest`, tidak ada `sqlx`, tidak ada `axum`, **tidak ada JS engine**. Kernel adalah tipe + trait murni.

---

## 3. `item.rs` — Item & Binary

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Satu item — unit data n8n. Bentuknya sengaja identik dengan
/// `INodeExecutionData` supaya serialisasi n8n JSON langsung cocok.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Payload utama. WAJIB `serde_json::Value` — bukan generic —
    /// karena ekspresi `$json.x` harus bekerja tanpa monomorphisasi per-node.
    pub json: Value,

    /// Properti `binary` n8n (D105).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary: Option<BinaryContainer>,

    /// Lineage tracking n8n. TANPA ini, fitur "execute previous node again"
    /// dan debugging per-item di editor tidak mungkin. Jangan dihapus demi
    /// "efisiensi" — ukurannya 8 byte per item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paired_item: Option<PairedItem>,
}

/// n8n menamai binary attachment per-key: `data`, `data1`, `thumbnail`, ...
pub type BinaryContainer = BTreeMap<String, BinaryData>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryData {
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// Di mana byte-nya berada. TIDAK PERNAH keduanya.
    pub location: BinaryLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryLocation {
    /// Kecil → base64 inline. Cocok dengan perilaku default n8n
    /// (`N8N_DEFAULT_BINARY_DATA_MODE=database`).
    Inline(String),
    /// Besar → reference ke blob store (D30). Inilah penghematan RAM-nya.
    /// Threshold diatur config, bukan hardcoded (mis. 256 KB).
    Ref { content_id: ContentId, size_bytes: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId(pub u64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PairedItem {
    pub item_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_node_index: Option<u8>,
}
```

---

## 4. `item.rs` — ItemList (inti dari semuanya)

```rust
/// List item yang mengalir antar-node.
///
/// KEPUTUSAN PENTING (resolusi A-01 + A-03):
/// Semantik yang terlihat oleh node adalah ARRAY — bisa di-index, di-iterate,
/// dihitung panjangnya, persis seperti n8n. Efisiensi didapat dari fakta bahwa
/// backing store-nya boleh di disk.
pub enum ItemList {
    /// List kecil: seluruhnya di RAM. Zero overhead.
    Inline(Vec<Item>),

    /// List besar: file spill + index offset di RAM.
    /// Random access O(1) lewat index; item di-deserialize saat diminta.
    Spilled(SpillHandle),
}

/// Handle ke list yang di-spill. Yang resident di RAM HANYA index.
pub struct SpillHandle {
    /// Offset & panjang tiap item di dalam spill file.
    /// 12 byte/item → 1 juta item = ~12 MB RAM. Masih aman di 2 GB.
    /// Untuk list sangat besar, index sendiri bisa di-mmap.
    index: IndexLocation,
    path: SpillPath,
    len: u32,
    /// Format serialisasi item di disk. Postcard/MessagePack, BUKAN JSON —
    /// ~3x lebih kecil dan ~5x lebih cepat di-deserialize.
    codec: SpillCodec,
}

enum IndexLocation {
    Ram(Vec<(u64, u32)>),   // (offset, len)
    Mmap(MmapIndex),        // untuk list > ~5 juta item
}

impl ItemList {
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    /// Materialisasi SATU item ke RAM. Inilah yang membuat `$('Node').item`
    /// tetap bekerja walau list-nya di disk.
    pub fn get(&self, i: usize) -> Result<Item, KernelError>;

    /// Iterasi streaming — item dibaca satu per satu, tidak pernah seluruhnya
    /// resident. Dipakai oleh node ber-`BufferingMode::Stream`.
    pub fn iter(&self) -> ItemIter<'_>;

    /// Untuk node ber-`BufferingMode::Batch`. Engine boleh menolak kalau
    /// Memory Governor sedang CRITICAL (D72) — node harus handle error ini.
    pub fn materialize_all(&self) -> Result<Vec<Item>, KernelError>;

    /// Di-spill atau tidak. Dipakai Governor untuk akuntansi RAM nyata.
    pub fn ram_footprint(&self) -> usize;
    pub fn is_spilled(&self) -> bool;

    /// Konstruksi. Threshold diambil dari config; engine yang memutuskan, bukan node.
    pub fn from_vec(items: Vec<Item>, policy: &SpillPolicy) -> Result<Self, KernelError>;
}

pub struct SpillPolicy {
    /// Di atas jumlah item ini → spill.
    pub max_inline_items: u32,          // default: 1_000
    /// Di atas total byte ini → spill.
    pub max_inline_bytes: u64,          // default: 4 MB
}
```

**Kenapa ini menyelesaikan A-01:** `PriorOutputs` (§6) menyimpan `ItemList`, bukan `Vec<Item>`. Karena `ItemList` bisa spilled, seluruh output node tetap **addressable** selama execution hidup tanpa menghabiskan RAM. Ekspresi `$('HTTP Request').item.json` bekerja. `$items()` bekerja. `pairedItem` bekerja.

**Kenapa ini menyelesaikan A-03:** node tetap menerima array. `Merge`, `Sort`, `Aggregate`, `RemoveDuplicates` semua bisa `materialize_all()` (atau lebih baik: algoritma eksternal merge-sort di atas spill file). Semantik n8n utuh 100%.

---

## 5. `node.rs` — Node, Descriptor, ResourceHint

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Node: Send + Sync {
    /// Metadata statis. Dipanggil sekali saat registrasi, bukan per-execute.
    fn descriptor(&self) -> &NodeDescriptor;

    /// Eksekusi. SEMUA akses input lewat `ctx` — engine yang mengontrol
    /// apa yang masuk RAM dan kapan. Node tidak boleh memegang referensi
    /// ke payload mentah.
    async fn execute(&self, ctx: &mut NodeContext<'_>) -> Result<NodeOutput, NodeError>;
}

#[derive(Debug, Clone)]
pub struct NodeDescriptor {
    /// Identifier stabil: "http.request", "if", "merge", "code.javascript"
    pub kind: NodeKind,
    /// D89 — explicit node version. Wajib, karena parameter schema berevolusi.
    pub version: u16,
    pub display_name: String,
    pub group: NodeGroup,               // Trigger / Action / Flow / Data / Code
    pub hints: ResourceHint,
    /// SATU sumber untuk VALIDASI dan RENDERING UI (resolusi A-30).
    /// Editor Vue membaca ini untuk membangkitkan form — tidak hardcode per node.
    pub params: ParameterSchema,
    pub credentials: Vec<CredentialSpec>,
    /// Jumlah input/output. Merge punya 2 input; IF punya 2 output; dst.
    pub inputs: u8,
    pub outputs: u8,
    /// n8n: "Execute Once" vs per-item.
    pub execute_once: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceHint {
    pub cpu: Weight,
    pub io: Weight,
    pub memory: Weight,
    /// RESOLUSI A-03: default adalah BATCH. Ini semantik n8n yang sebenarnya.
    /// Stream adalah opt-in untuk source/sink dan transformasi pass-through.
    pub buffering: BufferingMode,
    /// RESOLUSI A-10: tanpa ini, crash reconciliation TIDAK BISA benar.
    pub side_effect: SideEffect,
    /// Node boleh di-cache kalau input identik? (mis. HTTP GET)
    pub cacheable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight { None, Low, Medium, High }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferingMode {
    /// Butuh seluruh input list. DEFAULT.
    /// Contoh: Sort, Merge, Aggregate, Limit, RemoveDuplicates, Summarize.
    Batch,
    /// Bisa menghasilkan output per item input.
    /// Contoh: Set, Edit Fields, HTTP download→file, transformasi 1:1.
    Stream,
    /// Butuh seluruh input DAN menghasilkan banyak output sekaligus.
    /// Contoh: SplitOut, Compare Datasets.
    BatchInOut,
}

/// RESOLUSI A-10 / D107.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffect {
    /// Komputasi murni. Selalu aman di-retry.
    None,
    /// Aman di-retry ASAL membawa idempotency key (D60).
    /// Contoh: PUT, Stripe dengan Idempotency-Key header.
    Idempotent,
    /// TIDAK BOLEH auto-retry. Crash saat node ini berjalan →
    /// status INDOUBT, masuk quarantine (D69), execution NEEDS_REVIEW.
    /// Contoh: POST /transfer, kirim email, SMS.
    NonIdempotent,
}

impl Default for ResourceHint {
    fn default() -> Self {
        Self {
            cpu: Weight::Low,
            io: Weight::Medium,
            memory: Weight::Low,
            buffering: BufferingMode::Batch,   // ← default n8n
            side_effect: SideEffect::None,      // ← aman secara default
            cacheable: false,
        }
    }
}
```

### NodeOutput

```rust
pub struct NodeOutput {
    /// Satu entry per output branch. IF → 2 branch. Merge → 1 branch.
    pub branches: Vec<ItemList>,
    /// Artifact besar yang dihasilkan (file, dsb) — sudah di blob store.
    pub blobs: Vec<ContentId>,
}
```

---

## 6. `context.rs` — apa yang dilihat node

```rust
pub struct NodeContext<'a> {
    // ── Identitas & lingkungan ────────────────────────────────
    pub execution: &'a ExecutionMeta,     // id, mode, workflow_id, started_at
    pub node: &'a NodeMeta,               // id, name, attempt, input_index

    // ── Input ────────────────────────────────────────────────
    pub input: InputAccess<'a>,           // per-input-index → &ItemList

    // ── RESOLUSI A-01: output node sebelumnya ────────────────
    /// Untuk $('Node Name') dan $items(). Wajib ada, karena ekspresi n8n
    /// bergantung padanya. Memori aman karena ItemList bisa spilled.
    pub prior: &'a dyn PriorOutputs,

    // ── Kredensial (D92) ─────────────────────────────────────
    /// HANYA kredensial yang dideklarasikan di descriptor.credentials.
    /// Node tidak bisa meminta lebih. Redaction otomatis di log (D93).
    pub credentials: &'a CredentialProvider,

    // ── D101: static data ────────────────────────────────────
    /// State persisten per (workflow, node) antar-eksekusi.
    /// ⚠️ Hanya di-flush pada PRODUCTION execution (D106) — meniru n8n.
    pub static_data: &'a mut StaticData,

    // ── D109: expression engine ──────────────────────────────
    /// Kernel hanya mendefinisikan TRAIT. Implementasi QuickJS ada di
    /// crate `expr-quickjs`. Ini yang membuat kernel bebas dependency JS.
    pub expr: &'a dyn ExpressionEngine,

    // ── Infrastruktur ────────────────────────────────────────
    pub http: &'a dyn HttpClient,         // D28 pooled; D117 size-gated
    pub blobs: &'a dyn BlobStore,         // D30
    pub spill: &'a dyn SpillStore,        // D5/D111
    pub log: &'a Logger,                  // D63 + auto-redaction D93

    // ── Kontrol ──────────────────────────────────────────────
    pub cancel: CancellationToken,        // D20 cooperative
    pub deadline: Instant,                // D58 per-node timeout
}

pub struct ExecutionMeta {
    pub id: ExecutionId,
    /// D106: menentukan apakah static_data di-flush dan apakah data disimpan.
    pub mode: ExecutionMode,
    pub workflow_id: WorkflowId,
    pub workflow_version: u32,
    /// D102: cron, $now, $today semuanya bergantung ini.
    pub timezone: Timezone,
    /// D104: 'v0' | 'v1' — mengubah urutan eksekusi branch. WAJIB dihormati.
    pub execution_order: ExecutionOrder,
    pub started_at: Timestamp,
}

pub enum ExecutionMode {
    Production,   // static_data di-flush, retry aktif, error workflow aktif
    Manual,       // dari editor: static_data TIDAK di-flush
    Test,         // pinData aktif, tidak ada side effect nyata
    Webhook,      // dipicu webhook
}

pub enum ExecutionOrder { V0Legacy, V1 }   // D104
```

### PriorOutputs — kunci resolusi A-01

```rust
/// Semua output node yang sudah selesai pada execution ini.
/// TETAP ADDRESSABLE sampai execution selesai — ini syarat mutlak
/// kompatibilitas ekspresi n8n.
///
/// Hemat memori karena mengembalikan handle, bukan data:
///   addressable ≠ resident
pub trait PriorOutputs: Send + Sync {
    fn by_name(&self, node_name: &str) -> Option<NodeOutputRef>;
    fn by_id(&self, node_id: &str) -> Option<NodeOutputRef>;

    /// Item ke-`i` dari output node tertentu — yang dipakai $('Node').item
    fn item(&self, node: &str, input_index: u8, i: usize) -> Result<Item, KernelError>;

    /// Seluruh list — yang dipakai $items(). Mengembalikan handle ringan.
    fn items(&self, node: &str, input_index: u8) -> Result<ItemListRef<'_>, KernelError>;

    /// Node mana saja yang sudah selesai (untuk validasi ekspresi).
    fn available_nodes(&self) -> Vec<&str>;
}
```

### ExpressionEngine (D109)

```rust
/// RESOLUSI A-01: ekspresi n8n ADALAH JavaScript.
/// Kernel tidak menanam JS engine — kernel mendefinisikan kontraknya.
/// Implementasi (QuickJS via rquickjs) ada di crate terpisah.
pub trait ExpressionEngine: Send + Sync {
    fn eval(&self, expr: &str, scope: &ExpressionScope<'_>) -> Result<Value, ExprError>;

    /// Evaluasi banyak ekspresi dalam satu JS context yang sudah hangat.
    /// Penting untuk performa: setup context QuickJS ~ratusan µs,
    /// jadi jangan bayar per-ekspresi.
    fn eval_batch(&self, exprs: &[&str], scope: &ExpressionScope<'_>)
        -> Result<Vec<Value>, ExprError>;
}

/// Permukaan yang harus cocok dengan n8n. Kalau ada yang kurang di sini,
/// compatibility test (D32) akan gagal.
pub struct ExpressionScope<'a> {
    pub json: &'a Value,                 // $json
    pub input: &'a ItemList,             // $input
    pub prior: &'a dyn PriorOutputs,     // $('Node') · $items() · $node[]
    pub execution: &'a ExecutionMeta,    // $execution
    pub workflow: &'a WorkflowMeta,      // $workflow
    pub now: DateTime<Tz>,               // $now · $today     (D102)
    pub vars: &'a Vars,                  // $vars
    pub env: &'a EnvAccess,              // $env — DIBUTUHKAN whitelist
}
```

---

## 7. `task.rs` · `event.rs` · `checkpoint.rs`

```rust
// ── task.rs ──────────────────────────────────────────────────
pub struct Task {
    pub id: TaskId,
    pub execution_id: ExecutionId,
    pub node_id: NodeId,
    pub input_index: u8,
    pub attempt: u16,               // D14/D57 retry
    pub priority: i8,               // D56
    pub status: TaskStatus,
    pub lease: Option<Lease>,       // D8: claim + timeout
    pub hints: ResourceHint,        // disalin dari descriptor saat planning
    pub created_at: Timestamp,
}

pub enum TaskStatus {
    Pending,     // dependency belum terpenuhi
    Ready,       // boleh dijadwalkan
    Running,     // ada lease aktif
    Waiting,     // D17: persist & lepas resource (wait/sleep/human approval)
    Success,
    Failed,
    /// RESOLUSI A-10: crash terjadi saat node NonIdempotent sedang berjalan.
    /// Tidak boleh auto-retry. Butuh keputusan manusia.
    InDoubt,
    Cancelled,   // D20
    Quarantined, // D69: melebihi retry limit
}

pub struct Lease { pub holder: WorkerId, pub expires_at: Timestamp }

// ── event.rs (resolusi A-16) ─────────────────────────────────
/// Typed untuk internal (cepat), tapi JUGA harus bisa dipersist & direplay.
/// Karena itu: setiap record membawa schema_version.
pub struct EventRecord {
    pub schema_version: u16,        // ← WAJIB. Tanpa ini D99 rollback mustahil.
    pub seq: u64,
    pub ts: Timestamp,
    pub event: ExecutionEvent,
}

pub enum ExecutionEvent {
    ExecutionCreated { .. },
    TaskReady { .. },
    TaskStarted { .. },
    TaskCompleted { .. },
    TaskFailed { error_code: ErrorCode, retryable: bool },
    TaskInDoubt { .. },             // A-10
    TaskRetrying { attempt: u16, backoff_ms: u32 },
    CheckpointWritten { .. },
    ExecutionCompleted { .. },
    ExecutionFailed { .. },
    ExecutionCancelled { .. },
    ResourcePressure { level: GovernorLevel },   // D72
    Spilled { bytes: u64 },
}

// Aturan: varian enum boleh DITAMBAH. Tidak boleh DIUBAH atau DIHAPUS
// tanpa menaikkan schema_version + menulis migrator replay.

// ── checkpoint.rs ────────────────────────────────────────────
pub struct Checkpoint {
    pub id: CheckpointId,
    pub execution_id: ExecutionId,
    pub schema_version: u16,
    /// Posisi recovery: task mana yang sudah aman.
    pub completed: RoaringBitmap,   // node index → compact, bukan Vec
    /// Task yang harus di-reset ke Ready saat recovery.
    pub in_flight: Vec<TaskId>,
    /// TIDAK menyimpan payload. Payload sudah di spill/blob store,
    /// direferensikan lewat ContentId di ItemList.
    pub created_at: Timestamp,
}
```

> **Catatan A-06:** checkpoint **tidak** ditulis setiap node. Hanya di *recovery point* — node ber-`side_effect != None`, boundary branch/merge, atau tiap N task. Ini yang membuat beban write SQLite terkendali.

---

## 8. `error.rs`

```rust
pub enum NodeError {
    /// Bisa di-retry (network, 429, 5xx)
    Transient { retry_after: Option<Duration>, source: Box<dyn Error> },
    /// Tidak bisa di-retry (4xx, validasi, auth)
    Permanent { code: ErrorCode, message: String },
    /// D58: melewati deadline
    Timeout { elapsed: Duration },
    /// D20: dibatalkan
    Cancelled,
    /// D72: Governor menolak alokasi — node harus handle ini, bukan panic
    ResourceExhausted { resource: Resource, requested: u64, available: u64 },
    /// Ekspresi gagal
    Expression { expr: String, reason: ExprError },
    /// Bug di node — harus jadi test case baru
    Internal { source: Box<dyn Error> },
}

/// Penting untuk A-07: ResourceExhausted adalah error NORMAL yang harus
/// ditangani, bukan kondisi fatal. Ini satu-satunya cara "RAM budget"
/// bisa benar-benar ditegakkan dari dalam proses.
```

---

## 9. `ParameterSchema` (resolusi A-30)

Ini yang paling sering diremehkan dan paling mahal kalau salah. Editor Vue **harus membangkitkan form dari schema ini**, bukan hardcode per node. Kalau tidak, setiap node = pekerjaan UI manual, dan 400 node jadi mustahil.

```rust
pub struct ParameterSchema {
    pub fields: Vec<ParameterField>,
}

pub struct ParameterField {
    pub name: String,
    pub display_name: String,
    pub kind: ParameterKind,        // String, Number, Boolean, Options, Json,
                                    // Code, DateTime, ResourceLocator, FixedCollection
    pub default: Option<Value>,
    pub required: bool,
    /// n8n `displayOptions`: show/hide bersyarat berdasarkan field lain.
    /// Contoh: field "body" hanya muncul kalau method == "POST".
    pub display_if: Option<DisplayCondition>,
    /// n8n `options` / resource-operation cascade
    pub options: Option<Vec<ParameterValue>>,
    /// Ekspresi boleh di sini? ({{ }})
    pub supports_expression: bool,
    pub description: Option<String>,
    pub placeholder: Option<String>,
}

pub enum DisplayCondition {
    Show { field: String, equals: Value },
    Hide { field: String, equals: Value },
    Any(Vec<DisplayCondition>),
    All(Vec<DisplayCondition>),
}
```

**Konsekuensi penting:** schema ini harus bisa **dihasilkan** dari spec OpenAPI (§10). Kalau tidak, codegen hanya setengah berguna — kamu dapat connector tapi tidak dapat form UI-nya.

---

## 10. Dampaknya pada roadmap — OpenAPI codegen naik kelas

Dengan `ParameterSchema` dan `NodeDescriptor` sebagai output codegen, strateginya jadi:

```
OpenAPI spec (yaml/json)
      │
      ├─→ paths + operations      → NodeDescriptor.kind, credentials, side_effect
      ├─→ parameters + requestBody → ParameterSchema  (→ form UI otomatis)
      ├─→ responses                → output shape hints
      └─→ securitySchemes          → CredentialSpec
      │
      ▼
codegen  →  crate `nodes/generated/<service>/`  (Rust asli, reqwest + serde)
      │
      ▼
registry →  tersedia di engine, tanpa JS runtime apa pun
```

**Perubahan prioritas roadmap:**

| | Roadmap lama | Roadmap baru |
|---|---|---|
| Essential nodes (12) | PHASE 6 | Phase 6 — **tetap** |
| **OpenAPI node generator** | tidak ada (hanya disebut sekilas) | **Phase 7** ⭐ naik drastis |
| API server | PHASE 10 | Phase 8 |
| Web UI | PHASE 12 | Phase 9 — **baca ParameterSchema, jangan hardcode** |
| Community node JS | PHASE 14 ("tersulit") | **pasca-v1.0, opt-in, butuh Node.js terpisah** |
| n8n compat layer | PHASE 15 | Phase 10 |

**Target cakupan koneksi:**

```
Tier 1  Native hand-written   ~15 node   (HTTP, IF, Merge, Code, DB, Schedule, ...)
Tier 2  OpenAPI generated     ratusan    ← INI jawaban "koneksi penuh"
Tier 3  JS community adapter  opsional   ← pasca-v1.0, jangan dijanjikan sekarang
```

Tier 2 adalah keunggulan kompetitifmu, bukan bebanmu. n8n menulis ~400 integrasi dengan tangan. Kamu bisa menghasilkan ribuan dari spec, secara native, tanpa JS runtime. **Itu klaim "performa maksimal + koneksi penuh" yang benar-benar bisa ditepati.**

---

## 11. Checklist freeze

Kernel dianggap **beku** kalau semua ini benar:

```
□ crates/kernel/ compile dengan dependency list §2 saja (tidak lebih)
□ cargo build hijau untuk SEMUA crate lain terhadap kernel
□ graf crate terverifikasi DAG (script CI: `cargo metadata` + cek siklus)
□ ItemList::Inline dan ItemList::Spilled lolos test round-trip identik
    → property test: get(i) dari Spilled == get(i) dari Inline, ∀i
□ BufferingMode default = Batch (bukan Stream)
□ SideEffect default = None
□ ResourceExhausted bisa dikembalikan node dan ditangani scheduler
□ EventRecord punya schema_version; test replay lintas versi lolos
□ Checkpoint TIDAK menyimpan payload (assert di test)
□ ExpressionScope mencakup SEMUA helper n8n yang di-list di §6
□ ParameterSchema bisa membangkitkan form (proof: 1 node end-to-end)
□ Definisi ketujuh tipe tercantum di ARCHITECTURE.md
□ D101 static_data + D102 timezone + D104 executionOrder ada di ExecutionMeta
```

**Setelah checklist ini hijau, tidak ada yang boleh mengubah `kernel/` tanpa ADR tertulis.** Setiap perubahan adalah breaking change untuk seluruh codebase.

---

## 12. Yang sengaja TIDAK diputuskan di sini

Supaya kernel tidak membengkak:

- Algoritma spill (format file, kompresi) → crate `data-plane`
- Implementasi QuickJS → crate `expr-quickjs`
- Skema SQLite → crate `storage`
- Kebijakan governor → crate `resource-governor`
- Format workflow JSON → crate `workflow`

Kernel hanya **kontrak**. Semua kebijakan hidup di crate implementasi.

---

*Dokumen ini adalah resolusi konkret untuk temuan audit A-01, A-03, A-05, A-10, A-16, A-30 dan keputusan hilang D101, D102, D104, D105, D106, D107, D109, D115. Setelah kamu setujui, langkah berikutnya adalah materialisasi sebagai kode Rust sungguhan.*

---

## 13. DELTA IMPLEMENTASI — spec vs kode nyata

Ditambahkan setelah `crates/kernel/` benar-benar dikompilasi dan diuji.
Spec di atas (§1–12) tidak diubah; deviasi dicatat di sini supaya dokumen
ini tidak berbohong tentang apa yang sebenarnya dibangun.

**Status:** build bersih (0 warning), clippy bersih (0 warning), 19/19 test hijau,
`scripts/check-freeze.sh` PASSED.

### 13.1 Lubang desain yang ditemukan — dan ini yang paling penting

**`SpillStore::write(&[Item])` sendirian membuat seluruh desain spill tidak berguna.**

Spec §4 mendefinisikan hanya:

```rust
async fn write(&self, items: &[Item]) -> Result<SpilledList, KernelError>;
```

Perhatikan tipe argumennya: `&[Item]`. Untuk memanggilnya, pemanggil **harus
sudah menahan seluruh item di RAM**. Jadi pada saat tekanan memori paling
tinggi — saat item sedang diproduksi — spilling menghemat **nol byte**.
Spill baru terjadi setelah puncaknya lewat.

Ini persis terbalik dari maksudnya, dan tidak terlihat di atas kertas karena
signature-nya tampak wajar. Baru ketahuan saat menulis benchmark: peak RSS
kedua skenario identik, karena keduanya menahan `Vec<Item>` penuh sebelum spill.

**Perbaikan — ditambahkan ke trait:**

```rust
/// Buka writer inkremental.
async fn writer(&self) -> Result<Box<dyn SpillWriter>, KernelError>;

#[async_trait]
pub trait SpillWriter: Send {
    async fn push(&mut self, items: &[Item]) -> Result<(), KernelError>;
    async fn finish(self: Box<Self>) -> Result<SpilledList, KernelError>;
    fn pushed(&self) -> u32;   // supaya Governor bisa memantau mid-write (D72)
}

pub async fn spill_from_iter(
    batch: usize,
    store: &dyn SpillStore,
    items: impl IntoIterator<Item = Item>,
) -> Result<ItemList, KernelError>;
```

Sekarang trigger nyata — cursor row Postgres, body webhook besar, node HTTP
berpaginasi — bisa spill **saat item tiba**. Peak RAM jadi satu batch, bukan
seluruh result set.

**Konsekuensi:** klaim D3 ("jalan di VPS 2 GB") hanya benar *setelah* perbaikan
ini. Tanpa `SpillWriter`, klaim itu palsu untuk semua kasus yang menarik.

Bukti terukur ada di `/home/user/BENCH-A03.md`: 5 juta item / 753 MB payload
diproses dalam **50,4 MB peak RSS**, sementara jalur inline **di-OOM-kill
kernel pada 1,68 GB**.

### 13.2 Kontrak tambahan yang dipaksa oleh `SpillWriter`

Trait baru butuh aturan baru, semuanya sudah di-assert oleh test:

- `finish(self: Box<Self>)` mengonsumsi writer → `push` setelah `finish`
  mustahil secara tipe, bukan cuma dicek runtime.
- **Drop tanpa `finish` WAJIB menghapus file parsial.** Ini jalur abort:
  execution dibatalkan saat trigger masih memproduksi item. Bocor di sini
  adalah cara disk VPS penuh setelah berbulan-bulan. (audit A-21)
  Test: `unfinished_writer_leaves_nothing_behind`.
- Iterator kosong → `ItemList::empty()`, **tanpa** file tersisa.
  Test: `spill_from_iter_empty_leaves_no_file`.
- Index bersifat **global, bukan per-batch**: `read_at(i)` mengembalikan item
  ke-i lintas semua push. Batch hanya hint efisiensi I/O, bukan batas semantik.
  Test: `spill_from_iter_handles_all_batch_boundaries` (batch 1, 3, 7, 50, 99,
  100, 101, 10.000).

### 13.3 Deviasi lain dari spec

| Spec mengatakan | Kode melakukan | Alasan |
|---|---|---|
| `tokio_util::sync::CancellationToken` | trait `CancellationToken` sendiri | kernel tidak boleh menarik tokio; runtime adalah kebijakan |
| `chrono::DateTime<Utc>` | `i64` unix millis | hindari dependency; parsing timezone diserahkan ke crate `expr` |
| Timezone sebagai tipe terurai | string IANA di kernel | validasi tz butuh tabel IANA — itu kebijakan, bukan kontrak |
| `SpillHandle` | struct `SpilledList` | `len` harus tersimpan agar `len()` tidak pernah menyentuh store |
| Offset index di kernel | index hidup di impl `SpillStore` | index adalah detail format file = data-plane |
| `Stream<Item>` untuk iterasi | `ItemCursor` + `next_chunk()` | menghindari dependency `futures`; chunk lebih mudah di-budget Governor |

### 13.4 Yang TIDAK berubah

Semua resolusi audit tetap seperti di spec: `ItemList::Inline`/`Spilled`
(A-03), `PriorOutputs` (A-01), `SideEffect` + `InDoubt` (A-10),
`SCHEMA_VERSION` (A-16), `ParameterSchema` (A-30), dan kernel tetap **0
dependency internal + 4 dependency eksternal** (A-05).

### 13.5 Checklist §11 — status aktual

| Item | Status |
|---|---|
| Ketujuh tipe terdefinisi & compile | ✅ 10 file src (2.644 baris) + 833 baris test + 653 baris benchmark |
| `ItemList::Inline` ≡ `ItemList::Spilled` round-trip | ✅ test `spilled_and_inline_are_observationally_identical` |
| `BufferingMode` default = `Batch` | ✅ test `resource_hints_default_to_safe_values` |
| `SideEffect` punya jalur `InDoubt` | ✅ test `recovery_action_respects_side_effects` |
| Event & checkpoint punya `schema_version` | ✅ test `event_records_are_versioned` |
| D101/D102/D104 ada di `ExecutionMeta` | ✅ + test `static_data_flushes_only_in_production` (D106) |
| `ExpressionScope` mencakup helper n8n §6 | ⚠️ field ada; **belum** diverifikasi terhadap D32 |
| `ParameterSchema` bisa membangkitkan form | ⚠️ validasi + visibility teruji; **belum** ada 1 node end-to-end |
| Graf crate terverifikasi DAG | ✅ `scripts/check-freeze.sh` §4 (otomatis, bisa diulang) |
| Definisi tipe tercantum di ARCHITECTURE.md | ❌ ARCHITECTURE.md belum ada |

Dua ⚠️ dan satu ❌ itu **sengaja** belum ditutup: butuh crate `expr-quickjs`
dan satu node nyata, yang di luar scope freeze kernel.

### 13.6 Cara mereproduksi

```bash
cd rust-n8n-core
./scripts/check-freeze.sh                                  # semua guard
cargo run --release --example spill_bench -- compare 1000000   # angka RAM
```
