# 🔍 AUDIT ARSITEKTUR — Decision Register D1–D100

**Objek audit:** `rangkuman-rust-workflow-os.md` §4 (100 keputusan arsitektur)
**Proyek:** Rust-native, n8n-compatible workflow engine
**Constraint:** VPS 2 CPU / 2 GB RAM / 50 GB disk
**Auditor:** Arena.ai Agent Mode
**Tanggal:** 2026-09-09
**Verdict:** 🔴 **JANGAN mulai coding dulu.** 5 temuan CRITICAL, 9 HIGH.

---

## 0. Ringkasan Eksekutif

Register D1–D100 **terdengar sangat solid saat dibaca berurutan**, karena disajikan sebagai daftar keputusan yang masing-masing masuk akal. Masalahnya muncul saat keputusan-keputusan itu **diadu satu sama lain**.

Temuan inti audit ini:

> **D12 keputusan pertama (D1–D12) digali satu-per-satu dengan alternatif & trade-off. 88 keputusan sisanya (D13–D100) disetujui dalam batch 10-an hanya dengan menjawab "SETUJU" — tanpa Why, tanpa Alternatives, tanpa Trade-offs.**
>
> Ini **melanggar aturan governance proyek sendiri** (Strategi Matt aturan #3: *"Setiap keputusan besar punya alasan: Decision · Why · Alternatives · Trade-offs · Consequence"*).
>
> Akibatnya terlihat: register mengandung **duplikat, tumpang tindih, dan kontradiksi** yang tidak terdeteksi karena tidak pernah di-cross-check.

### Scorecard

| Kategori | Jumlah | Keterangan |
|---|---|---|
| ✅ Sound — pertahankan apa adanya | ~40 | Mayoritas pilihan stack (Tokio/Axum/Serde/SQLx/SQLite) sudah tepat |
| ⚠️ Sound tapi under-specified | ~35 | Arahnya benar, detail implementasi belum diputuskan |
| 🔶 Konflik — wajib diamandemen | 13 | Saling bertentangan dengan keputusan lain |
| ❌ Tidak feasible seperti tertulis | 6 | Melanggar constraint fisik (RAM/binary/lisensi) |
| ⛔ **Missing — belum ada keputusannya** | **18** | Termasuk 2 yang **memblokir** seluruh proyek |

### 3 hal yang paling penting dari audit ini

1. **🔴 A-01 — Expression engine n8n itu JavaScript.** `{{ $('Node').item.json }}` butuh random-access ke output node sebelumnya, dan sintaksnya JS penuh. "Lightweight expression engine" (D91) yang kompatibel n8n **mustahil tanpa JS runtime** — dan JS runtime bertabrakan dengan D73/D74/D76. **Ini kontradiksi terdalam di seluruh register dan tidak terdeteksi sama sekali.**

2. **🔴 A-03 — D11 (item-by-item streaming) membalik semantik n8n.** n8n mengoper data sebagai **array of items** antar node. Streaming-first merusak `Merge`, `Sort`, `Aggregate`, `pairedItem`, `$items()`, dan `$('Node')`. D11 dipilih karena terdengar hemat RAM, tapi biayanya adalah D1 (kompatibilitas) — yaitu *seluruh alasan proyek ini ada*.

3. **🔴 A-05 — "Integration Contract" itu kosong.** Blueprint §29 mendaftar tujuh tipe inti (`Node`, `ExecutionContext`, `Item`, `Task`, `ExecutionEvent`, `Checkpoint`, `ResourceHint`) sebagai "contract" — **tanpa satu pun definisi**. Dan struktur crate di §25 membuat ketujuhnya saling tergantung melingkar. Di Rust itu artinya `cargo build` gagal di menit pertama.

---

## 1. Temuan CRITICAL

### 🔴 A-01 — Expression engine = JavaScript (D91 ⊥ D73/D74/D76, dan ⊥ D11)

**Keputusan terkait:** D91, D1, D11, D31, D73, D74, D76

**Masalah.**
D91 memutuskan *"dedicated lightweight expression engine"* dengan syarat: deterministic, sandboxed, fast, memory-efficient.

Tapi ekspresi n8n **adalah JavaScript**. Yang harus didukung agar kompatibel (D1):

```js
{{ $json.nama }}
{{ $json.items[0].price * 1.1 }}
{{ $('HTTP Request').item.json.id }}        // ← random access ke node LAIN
{{ $items('Node').map(i => i.json.x) }}     // ← akses seluruh item node lain
{{ $now.minus({days: 1}).toISO() }}         // ← luxon DateTime
{{ $execution.id }}  {{ $workflow.id }}  {{ $node["X"].json }}
{{ Math.max(...$json.arr) }}  {{ JSON.stringify(x) }}
```

Konsekuensinya dua:

**(a) Butuh evaluator JS.** Anda tidak bisa "lightweight" dan sekaligus kompatibel-JS. Pilihannya:
- Tulis interpreter JS sendiri → tidak realistis
- Embed **QuickJS** (`rquickjs`/`quickjs-ng`) → ~1–3 MB binary, RSS ratusan KB–beberapa MB. **Feasible.**
- Embed **V8** (`rusty_v8`/`deno_core`) → binary membengkak puluhan MB, dep tree besar. **Melanggar D73/D74.**

**(b) `$('Node')` menghancurkan D11.** Ekspresi `$('HTTP Request').item.json` mengharuskan engine **menyimpan dan meng-index output setiap node sebelumnya** selama eksekusi berjalan. Ini bertentangan langsung dengan:
- D11 (item-by-item streaming — data sudah lewat, tidak bisa diambil lagi)
- D5/D9 (release memory setelah selesai)
- D85 (compact execution history)

**Impact.** Kalau ini tidak diselesaikan di awal, seluruh desain data plane harus dirombak setelah ribuan baris kode ditulis.

**Rekomendasi.**
```
D91 (REVISI) → Expression Runtime = embedded QuickJS (rquickjs),
               bukan "lightweight engine" bikinan sendiri.
               - Context per-execution, di-reuse, hard timeout 50ms/ekspresi
               - Tanpa akses I/O, tanpa require(), tanpa fs/net
               - API host: $json, $input, $items(), $(), $now, $execution, $workflow

D11 (REVISI) → lihat A-03. Output node HARUS addressable (retained/indexed)
               selama eksekusi hidup, bukan pure streaming.
```
Lihat **D109** dan **D105** di §5.

---

### 🔴 A-02 — "Isolated JS compatibility layer" untuk community node tidak feasible (D22/D51/D81 ⊥ D72/D73/D74/D76)

**Keputusan terkait:** D22, D51, D81, D21, D73, D74, D76, D72

**Masalah.**
Empat keputusan berbeda menjanjikan hal yang sama — *"Native Rust + isolated JS compatibility layer"* — tapi **tidak satu pun memilih runtime-nya**. Ini bukan detail, ini penentu feasibility:

| Opsi | Biaya | Vonis |
|---|---|---|
| Embed V8 / `deno_core` | Binary +puluhan MB, RSS +50–150 MB, build kompleks | ❌ Melanggar D72 (hard RAM budget), D73, D74 |
| QuickJS / WASM sandbox | Ringan, **tapi tidak ada Node.js API** | ❌ Community node n8n adalah **paket npm** yang memakai `require()`, `axios`, `fs`, `crypto`, npm module resolution. Tidak akan jalan tanpa menulis ulang sebagian besar Node API |
| Spawn proses Node.js eksternal | +40–80 MB RSS per proses | ❌ Melanggar D76 (one core process), dan di 2 GB RAM praktis tidak ada headroom |
| `libloading` / dynamic `.so` | Arbitrary code execution **tanpa sandbox**, build tidak reproducible | ⚠️ Ambiguitas D81 "dynamic registry" — berbahaya jika ini yang dimaksud |

**Catatan tambahan:** community node n8n berlisensi beragam (banyak MIT, sebagian Sustainable Use License). Menyalin definisi node mereka = risiko lisensi, bukan cuma risiko teknis.

**Impact.** Ini adalah **lubang teknis terbesar** di register. Target awal user di percakapan bahkan eksplisit: *"buat ulang n8n dengan segala koneksinya... mencakup node community juga."* Target itu **tidak tercapai dalam constraint 2 GB / single binary.**

**Rekomendasi.**
```
D22/D51 (REVISI) → Pecah masalah JS jadi DUA, jangan disatukan:

  (1) EXPRESSION  → QuickJS embedded, IN-PROCESS, selalu aktif.
                    Kecil, aman, cukup untuk kompatibilitas ekspresi. (lihat A-01)

  (2) COMMUNITY NODE → Node.js worker OUT-OF-PROCESS, OPT-IN via config,
                    TIDAK di MVP, TIDAK di single binary default.
                    Feature flag: --enable-js-nodes
                    Butuh Node.js terinstal terpisah.

D81 (REVISI) → "Dynamic registry" = registry DEKLARATIF (manifest node Rust
               yang di-compile-in + node OpenAPI-generated), BUKAN dynamic
               library loading. Larang eksplisit libloading/.so di v1.0.
```
Lihat **D110** di §5.

---

### 🔴 A-03 — D11 streaming-first membalik semantik n8n (D11 ⊥ D1, D32, D49)

**Keputusan terkait:** D11, D1, D9, D10, D32, D49, D91

**Masalah.**
Model data n8n yang sebenarnya:

```ts
// n8n: node menerima dan mengembalikan ARRAY OF ITEMS
execute(items: INodeExecutionData[][]): INodeExecutionData[][]
// INodeExecutionData = { json, binary?, pairedItem? }
```

n8n itu **batch/item-list oriented**, bukan streaming. Konsekuensi D11 (streaming-first):

| Fitur n8n | Rusak oleh streaming-first? |
|---|---|
| `Merge` (combine 2 input) | ❌ Butuh kedua input lengkap |
| `Sort`, `Limit`, `Remove Duplicates`, `Aggregate`, `Summarize`, `Compare Datasets` | ❌ Butuh seluruh item |
| `SplitInBatches` / Loop | ❌ Butuh boundary batch eksplisit |
| `pairedItem` tracking (item lineage) | ❌ Butuh output node sebelumnya addressable |
| `$items()`, `$('Node').item` | ❌ Lihat A-01 |
| `Execute Once` vs per-item flag | ⚠️ Semantik harus dipertahankan per-node |

Di percakapan, downside D11 hanya diakui sekilas: *"tidak semua node bisa benar-benar streaming... node boleh declare buffering eksplisit."* **Itu terbalik.** Di n8n, **buffering adalah default** dan streaming adalah pengecualian. Membaliknya berarti setiap node compat harus opt-in ke semantik yang salah.

**Impact.** D32 (automated behavioral compatibility tests) akan gagal massal begitu corpus n8n asli dimasukkan. Dan D1 — alasan proyek ini ada — jadi tidak tercapai.

**Rekomendasi.**
```
D11 (REVISI) → "n8n-compatible item-list semantics by DEFAULT,
                streaming sebagai OPT-IN per-node capability."

  Default : node menerima & mengembalikan Vec<Item>  (kompatibel n8n)
  Opt-in  : node mendeklarasikan  streaming: true  pada resource hints
            → hanya untuk SOURCE (HTTP download, DB cursor, file read)
              dan SINK (file write, upload) serta pass-through transform

  Hasil   : 80% penghematan RAM tetap didapat DI TEMPAT YANG PENTING
            (payload besar masuk/keluar), tanpa merusak semantik di tengah graph.

  Output node TETAP di-retain (dengan spill bila besar) selama eksekusi hidup,
  karena dibutuhkan oleh ekspresi $() dan pairedItem.
```

---

### 🔴 A-04 — Governance: 88 dari 100 keputusan melanggar aturan proyek sendiri

**Keputusan terkait:** D13–D100 (semua)

**Masalah.**
Strategi Matt menetapkan 6 aturan. Aturan #3:

> *"Setiap keputusan besar punya alasan: Decision · Why · Alternatives · Trade-offs · Consequence"*

Yang benar-benar melewati proses itu: **D1–D12 saja** (12 keputusan).
**D13–D100 (88 keputusan)** disetujui dalam 9 batch, masing-masing cuma tabel `# | Keputusan | Pilihan terbaik` dijawab **"Setuju"**. Tidak ada Why, tidak ada Alternatives, tidak ada Trade-offs, tidak ada Consequence.

Alasan batching ini eksplisit di percakapan: *"buat dalam kalimat yang sedikit agar token chat ini tak habis banyak."* Trade-off yang wajar untuk konteks chat — **tapi tidak wajar sebagai baseline arsitektur proyek multi-bulan.**

**Bukti nyata kerusakannya — duplikasi & tumpang tindih:**

| Kelompok tumpang tindih | Keputusan | Masalah |
|---|---|---|
| Data & Memory subsystem | **D5, D9, D10, D29, D30** | 5 keputusan untuk 1 subsystem. Batas antar-kelimanya tidak jelas → tiap crate akan mengimplement versinya sendiri |
| Observability | **D26, D63, D64, D65** | D26 sudah memutuskan "logs+metrics+tracing", lalu D63/64/65 memutuskan hal yang sama lagi |
| Credentials | **D25, D92** | Duplikat persis ("encrypted credential store") |
| Error/retry | **D14, D57, D69** | Retry+backoff+dead-letter dipecah jadi 3 keputusan terpisah |
| Resource control | **D4, D29, D37, D71, D72** | 5 keputusan untuk "hemat resource" |
| Deployment | **D73, D82** | Duplikat persis ("single binary") |
| Community nodes | **D21, D22, D51, D81** | 4 keputusan untuk 1 masalah yang belum terjawab (lihat A-02) |

**Rekomendasi.**
1. **Konsolidasi** register 100 → ~60 keputusan efektif, dengan pengelompokan subsystem.
2. Untuk setiap keputusan **CRITICAL/HIGH path** (D1–D12 + yang diamandemen audit ini), tulis **ADR sungguhan** dengan Why/Alternatives/Trade-offs/Consequence.
3. Untuk keputusan LOW-risk (D41 TOML, D66 /health, dsb), cukup satu baris — tidak perlu ADR.
4. Tambahkan **mechanism tie-breaker** eksplisit (lihat A-26).

---

### 🔴 A-05 — "Integration Contract" hanya daftar nama tipe, tanpa definisi

**Keputusan terkait:** Blueprint §29, D10, D12, D13, D36, D15

**Masalah.**
Blueprint §29 menyatakan bahwa interface harus diperlakukan sebagai *contract*, lalu mendaftar:

```
Node · ExecutionContext · Item · Task · ExecutionEvent · Checkpoint · ResourceHint
```

**Tujuh nama. Nol definisi.** Tidak ada field, tidak ada signature, tidak ada invariant, tidak ada siapa yang boleh mengubahnya. Contract tanpa isi bukan contract — itu daftar belanja.

Dan struktur crate di §25 **membuat ketujuh tipe itu saling tergantung lintas-crate**:

| Tipe | Dipakai oleh | Direncanakan di crate | Masalah |
|---|---|---|---|
| `Item` | execution, scheduler, semua node, expression | `data-plane/` | `node-sdk` harus depend ke `data-plane` |
| `Node` trait | execution engine, semua node | `node-sdk/` | signature-nya memakai `Item` → depend silang |
| `ResourceHint` | scheduler (baca), node (declare) | **tidak jelas** | belum ada pemiliknya sama sekali |
| `ExecutionContext` | execution, node, expression, credentials | `execution/` | node-sdk depend ke execution? atau sebaliknya? |
| `ExecutionEvent` | state, observability, recovery | `state/` | D36 (typed events) vs D6 (persisted) — lihat A-16 |
| `Checkpoint` | state, recovery | `state/` | — |
| `Task` | queue, scheduler | `queue/` | scheduler depend queue, queue depend state |

Hasilnya **dependency graph antar-crate berbentuk lingkaran**, bukan DAG. Di Rust ini berarti `cargo` menolak compile — atau Anda dipaksa menggabungkan semuanya jadi satu crate, yang membuat pemisahan §25 jadi fiktif.

**Impact.** Ini akan terdeteksi di **menit pertama** `cargo build`, bukan bulan ketiga. Tapi karena belum ada yang menuliskan definisi ketujuh tipe itu, belum ada yang menyadari.

**Rekomendasi → D115.**
```
Tambah milestone "KERNEL FREEZE" — dikerjakan PALING AWAL, serial, sebelum
crate lain apa pun ditulis:

  crates/kernel/   ← satu crate, tanpa dependency internal
    ├── item.rs        Item, ItemList, BinaryRef, PayloadRef, PairedItem
    ├── node.rs        trait Node, NodeKind, ResourceHint, BufferingMode
    ├── context.rs     ExecutionContext, NodeContext, CredentialRef
    ├── task.rs        Task, TaskStatus, Lease
    ├── event.rs       ExecutionEvent (typed + versioned, lihat A-16)
    ├── checkpoint.rs  Checkpoint, RecoveryPoint
    ├── error.rs       NodeError, ExecutionError, RetryPolicy
    └── expr.rs        trait ExpressionEngine (impl QuickJS di crate terpisah)

  Aturan dependency: SEMUA crate lain depend ke kernel/. kernel/ depend ke
  tidak ada crate internal. Ini memaksa graph jadi DAG.

  Aturan perubahan: kernel/ hanya boleh diubah lewat ADR tertulis.
  Setiap perubahan = breaking change untuk seluruh codebase.

  Selesai ketika: cargo build hijau + definisi ketujuh tipe terkunci
                  di ARCHITECTURE.md, SEBELUM kode engine apa pun ditulis.
```

---

## 2. Temuan HIGH

### 🟠 A-06 — SQLite single-writer adalah bottleneck tersembunyi (D8, D24, D61, D6 ⊥ D4, D7)

**Masalah.** WAL mode mengizinkan banyak reader, tapi **hanya SATU writer pada satu waktu**. Dengan desain ini, SEMUA hal berikut melewati satu writer:

```
task status transition (D8)  ·  event log append (D6)  ·  checkpoint write (D6)
audit event (D67)            ·  lease claim/renew (D8)  ·  execution state (D15)
```

Phase 10 menargetkan benchmark **10.000 node**. Kalau tiap node menghasilkan ~3 event + 1 checkpoint + 1 task update = **~50.000 write transaction terserialisasi**. Dengan `synchronous=FULL` di atas fsync, itu bisa puluhan detik — murni overhead bookkeeping, bukan pekerjaan berguna.

**Ini langsung menggugurkan D4 (resource efficiency) dan target throughput.** Tidak terdeteksi di percakapan.

**Rekomendasi → D108.**
```
- synchronous = NORMAL (aman dengan WAL; hanya kehilangan transaksi terakhir saat power loss)
- Group commit: coalesce event log writes dalam window 5–20 ms atau N=64 events
- Checkpoint HANYA di node boundary yang ditandai "recovery point", bukan tiap node
- Audit event → buffer in-memory, flush periodik (bukan per-event)
- Lease renewal → batch, bukan per-task
- Single dedicated writer task (actor pattern) — semua write lewat satu channel
```

---

### 🟠 A-07 — "Hard RAM budget" tidak bisa di-enforce; Serde mem-buffer semuanya (D72 ⊥ D35, D28, D11)

**Masalah (a).** D72 menjanjikan *"hard memory budget + governor"*. Rust **tidak bisa** hard-cap RSS tanpa custom `GlobalAlloc`. Governor bisa *mengestimasi* dan *menahan task baru*, tapi tidak bisa menghentikan alokasi yang terjadi **di dalam** node.

**Masalah (b).** D35 = Serde + JSON. Pola default `serde_json::from_slice(&body)` **mem-buffer seluruh body ke RAM**. Demikian juga `reqwest::Response::json()`. Jadi:

```
HTTP response 1.5 GB  →  serde_json::from_slice  →  RAM 2 GB  →  OOM kill
                            ↑ governor tidak sempat bereaksi
```

Ini **membatalkan D28 (streaming HTTP), D11 (streaming), D5/D9 (adaptive data), dan D30** sekaligus — dan semuanya sudah "dikunci".

**Rekomendasi → D117.**
```
- Larang serde_json::from_slice untuk body eksternal. Wajib:
    reqwest .bytes_stream() → serde_json::StreamDeserializer / simd-json streaming
- Content-Length / Transfer-Encoding gating di boundary:
    > soft_limit (mis. 8 MB)  → otomatis jadi PayloadRef ke blob store (D30)
    > hard_limit (mis. 512 MB)→ tolak dengan error eksplisit
- Implement custom GlobalAlloc yang MENGHITUNG alokasi (accounting),
  bukan membatasi — feeding angka nyata ke Memory Governor (bukan estimasi)
- Node TIDAK boleh memegang &[u8] penuh; wajib ByteRef / stream handle
```

---

### 🟠 A-08 — Enam control loop adaptif saling berinteraksi → risiko osilasi & priority inversion (D29, D37, D56, D59, D71, D72)

**Masalah.** Register memutuskan **enam** mekanisme adaptif independen:

| # | Mekanisme | Mengontrol |
|---|---|---|
| D29 | Governor + backpressure + spill | Memory |
| D37 | Bounded adaptive concurrency | Slot |
| D56 | Adaptive priority | Urutan |
| D59 | Adaptive per-node/provider rate limit | Laju API |
| D71 | Adaptive CPU budget | CPU |
| D72 | Hard RAM budget + governor | Memory (lagi) |

Enam feedback controller pada mesin 2-core, tanpa spesifikasi siapa yang menang → **osilasi** (concurrency naik-turun), **priority inversion**, dan perilaku yang sangat sulit di-debug.

**Masalah tambahan — D71 kemungkinan tidak bisa diimplementasikan.** "CPU budget" di userspace Linux tanpa cgroups hanya bisa berupa heuristik sampling `/proc/self/stat`. Anda tidak bisa benar-benar *membatasi* CPU dari dalam proses. cgroups butuh root/container → bertentangan dengan D73 (single binary di VPS polos).

**Rekomendasi.**
```
- Konsolidasi D29+D37+D71+D72 → SATU "Resource Governor" dengan SATU policy
  dan SATU setpoint. Sub-mekanisme lain (D56 priority, D59 rate limit)
  adalah INPUT ke governor, bukan controller terpisah.
- D71 (REVISI) → ganti dari "CPU budget" (tidak enforceable) menjadi
  "CPU-aware admission control": ukur load average + run-queue,
  batasi jumlah task CPU-heavy yang admitted bersamaan = n_cpu.
- Tambahkan: governor harus DAPAT DIMATIKAN (mode deterministic/fixed)
  untuk benchmarking. Tanpa ini, D52 benchmark gate tidak reproducible.
```

---

### 🟠 A-09 — Nested subworkflow + bounded pool = worker starvation deadlock (D19 ⊥ D37, D70, D77)

**Masalah.** Pola klasik:

```
Pool = 2 slot (2 core)
Workflow A occupy slot 1 → panggil Subworkflow B → BLOKIR menunggu B selesai
Workflow C occupy slot 2 → panggil Subworkflow D → BLOKIR menunggu D selesai
B dan D butuh slot. Tidak ada slot tersisa.
→ DEADLOCK permanen. Tidak ada timeout yang menolong karena semua node "sedang berjalan".
```

D19 memilih *"native nested execution"* tanpa memutuskan **slot accounting**. D37 (bounded concurrency) memperparah. D96 (cycle safeguards) tidak menutup kasus ini karena bukan cycle — ini deadlock resource.

**Rekomendasi → D113.**
```
Pilih SALAH SATU:

(a) FLATTEN (rekomendasi) — subworkflow di-inline ke task graph parent.
    Tidak ada nested blocking. Satu scheduler, satu pool. Paling sederhana
    dan paling cocok dengan D3 (DAG + event-driven).

(b) PERSIST-AND-RELEASE — parent masuk state WAITING (pakai mekanisme D17),
    slot dilepas, child jalan sebagai execution terpisah, parent di-wake
    saat child selesai. Konsisten dengan D17/D97 tapi lebih kompleks.

(c) RESERVED SLOT BUDGET — parent hanya boleh pakai ≤ N-1 slot.
    Rapuh, tidak direkomendasikan.

+ WAJIB: subworkflow recursion depth limit (default 8) — lihat A-23.
```

---

### 🟠 A-10 — Crash reconciliation tidak mendefinisikan semantik side-effect (D40 ⊥ D60, D14, D100)

**Masalah.** D40 = *"automatic crash reconciliation"*. Tapi:

```
Node "POST /transfer" mulai → HTTP request terkirim → 💥 crash SEBELUM response dibaca
Recovery: apakah request tadi sudah terjadi di dunia nyata?
```

Engine **tidak mungkin tahu**. Ini masalah at-least-once vs exactly-once yang fundamental. D60 (idempotency keys) hanya melindungi eksekusi internal Anda, **bukan side effect di sistem pihak ketiga**.

D100 menaruh *correctness* di posisi pertama. Keputusan ini adalah correctness issue murni, dan belum diputuskan.

**Rekomendasi → D107.**
```
- Setiap node WAJIB mendeklarasikan:  side_effect: None | Idempotent | NonIdempotent
- Policy reconciliation:
    None           → aman di-retry otomatis
    Idempotent     → retry dengan idempotency key (D60)
    NonIdempotent  → TIDAK auto-retry. Masuk QUARANTINE (D69),
                     execution ditandai NEEDS_REVIEW, user yang memutuskan
- Node "in-doubt" (crash setelah send, sebelum ack) dicatat eksplisit
  di event log sebagai INDOUBT — jangan disembunyikan sebagai FAILED
```

---

### 🟠 A-11 — Cooperative cancellation tidak bisa enforce timeout (D20 ⊥ D58, D61)

**Masalah.** D20 = cooperative cancellation (`CancellationToken`). D58 = per-node timeout. Tapi cooperative cancellation **hanya berlaku di `.await` point**. Node yang:
- loop CPU-bound tanpa `.await`
- memanggil blocking syscall
- stuck di DNS/TCP connect tanpa timeout internal

→ **tidak akan pernah berhenti.** Di 2 core, satu node stuck = separuh kapasitas hilang selamanya.

Dan `JoinHandle::abort()` (solusi naif) **bukan** cooperative: ia drop future di await point berikutnya. Interaksinya dengan D61 (SQLite transaction boundaries) tidak didefinisikan — transaksi yang sedang terbuka bisa tertinggal, dan lease tidak dilepas.

**Rekomendasi.**
```
- Semua operasi blocking WAJIB lewat spawn_blocking dengan watchdog terpisah
- Semua I/O network wajib punya connect_timeout + read_timeout + total_timeout
  (bukan hanya mengandalkan node-level timeout)
- Timeout node = 3 lapis: cooperative cancel → grace period → abort + cleanup hook
- Definisikan eksplisit: saat abort, lease DILEPAS dan transaksi SQLite DI-ROLLBACK
  oleh writer actor (bukan oleh task yang di-abort)
- Node CPU-bound wajib yield periodik (cek token tiap N iterasi) — ini aturan Node SDK
```

---

### 🟠 A-12 — "Benchmark gate" tanpa angka bukan gate (D52, D4 ⊥ Phase 10)

**Masalah.** D52 = *"benchmark gate; setiap optimasi wajib terukur"*. Phase 10 = benchmark 10/100/1.000/10.000 node. **Tapi tidak ada satu pun angka target di seluruh register.**

Tanpa threshold, D52 tidak bisa dijalankan (gate terhadap apa?), dan D4 ("resource efficiency absolut") tidak bisa diverifikasi. Blueprint §22 bahkan menulis *"Tidak boleh mengatakan 'lebih cepat' tanpa benchmark"* — tapi tidak pernah mengatakan **berapa**.

**Rekomendasi → D112.** Usulan angka awal (harus dikalibrasi ulang setelah Phase 2):

| Metrik | 100 node | 1.000 node | 10.000 node |
|---|---|---|---|
| Peak RSS engine (idle workflow) | < 40 MB | < 60 MB | < 120 MB |
| Scheduler overhead per node | < 200 µs | < 200 µs | < 300 µs |
| Throughput (node trivial/detik, 2 core) | > 20.000 | > 20.000 | > 15.000 |
| Cold start → ready | < 300 ms | < 300 ms | < 500 ms |
| Crash recovery (1.000 node, 50% selesai) | — | < 2 s | < 10 s |
| Total RAM budget engine (hard ceiling) | — | — | **< 512 MB** |

> Catatan: angka ini **sengaja** untuk engine overhead saja, tidak termasuk payload node. Payload diatur terpisah oleh D111 (Disk Governor) dan D117 (streaming).

---

### 🟠 A-13 — Tidak ada MVP scope cut; "v1.0" yang didefinisikan = produk lengkap

**Keputusan terkait:** Blueprint §32/§34, D43/D44, D53–D55, D49, D65

**Masalah (a) — yang disebut "MVP" sebenarnya produk jadi.**
"MVP acceptance test" di blueprint §32 mensyaratkan: create → connect → execute → **stream** → **parallelize** → handle error → retry → persist → **crash → recover → continue**, PLUS UI penuh (create/edit/save/run/stop/inspect/debug/view execution).

Dan §34 "V1.0 Definition" menambah: n8n Import + Compatibility Tests + Performance Benchmarks + Security + Observability.

Itu bukan MVP. Itu **n8n**. Register D1–D100 mendeskripsikan produk lengkap, dan tidak ada satu pun keputusan yang menyatakan apa yang **dibuang** dari v1.0.

**Masalah (b) — UI adalah proyek terpisah yang disamaratakan.**
n8n punya ~400+ integrasi, dan editor-nya sendiri ratusan ribu baris kode. Canvas editor Vue dengan drag/connect + config panel + realtime execution view + per-node inspection + workflow versioning + **sistem rendering parameter node** (`INodeProperties` + `displayOptions` — lihat A-30) adalah upaya besar tersendiri. Menempatkannya di fase yang sama dengan engine berarti tidak ada yang pernah selesai.

**Masalah (c) — D53/D54/D55 premature.**
JWT + RBAC + workspace isolation untuk deployment **single-user di satu VPS**. Ini scope creep yang ditempatkan di Phase 7, bersaing dengan engine yang belum jadi. Bertentangan dengan D100 (correctness first).

**Rekomendasi → D116.**
```
MVP v0.1 — definisikan dengan APA YANG DIBUANG, bukan apa yang dimasukkan:

  ✅ MASUK (12 node):
     Manual Trigger · Schedule · Webhook · HTTP Request · Set
     IF · Switch · Merge · Code · NoOp · Wait · SQLite/Postgres

  ✅ MASUK (engine):
     Sequential + branch + merge · SQLite state · crash recovery
     Expression engine (QuickJS) subset: $json, $execution, $now
     CLI + REST API, bind 127.0.0.1, TANPA auth

  ❌ KELUAR dari v0.1 (bukan "nanti", tapi eksplisit dibuang):
     Vue editor · WebSocket · JWT/RBAC/workspace
     JS community nodes · n8n importer · OTel tracing · Prometheus
     Distributed · multi-user · adaptive rate limit (D59)

  Urutan: engine v0.1 stabil → baru REST+WS → baru UI sebagai proyek terpisah
```

---

### 🟠 A-14 — Tidak ada baseline versi n8n; `executionOrder` v0/v1 mengubah semantik (D1 ⊥ D31, D32, D49, D90)

**Masalah.** "Kompatibel dengan n8n" — **n8n versi berapa?** Tidak diputuskan di mana pun. Ini bukan formalitas:

n8n 1.0 **mengubah default execution order**. Dari dokumentasi resmi n8n:

> **v1 (recommended)** mengeksekusi tiap cabang secara berurutan, menyelesaikan satu cabang sebelum mulai cabang lain (urutan berdasarkan posisi di canvas, atas→bawah, lalu kiri→kanan).
> **v0 (legacy)** mengeksekusi node pertama tiap cabang, lalu node kedua tiap cabang, dst.

Dan: *"n8n removed this execution behavior in version 1.0"* — perilaku `If` + `Merge` di v0 berbeda nyata dengan v1.

Artinya DAG engine Anda harus **mendukung kedua mode**, karena workflow yang diimpor bisa berasal dari era mana pun. `settings.executionOrder` adalah bagian dari workflow JSON yang harus di-parse dan dihormati.

**Impact.** Tanpa ini, D32 (compatibility test suite) tidak punya definisi "benar". Anda bisa lulus test v1 dan gagal total di workflow v0, atau sebaliknya.

**Rekomendasi → D104.**
```
- Baseline target: n8n 1.x (pin versi spesifik, mis. 1.9x) untuk node & schema
- WAJIB dukung settings.executionOrder = 'v0' | 'v1', default 'v1'
- Scheduler harus bisa switch strategi:
    v1 = depth-first per branch (canvas order)
    v0 = breadth-first across branches
- Compatibility test corpus wajib berisi pasangan workflow yang sama
  dengan kedua executionOrder, dan expected output berbeda
```

---

## 3. Temuan MEDIUM

### 🟡 A-15 — Tidak ada Disk Governor (asimetri terhadap D5)

D5 membangun **3 lapisan perlindungan RAM** yang sangat detail. Disk dapat **nol** keputusan. Padahal di 50 GB:

```
SQLite (workflows + executions + events + checkpoints + audit)
+ blob store (D30)  + spill files (D5/D9)  + large logs (D86)
+ backups (D62)     + binary artifacts (D39)
= disk exhaustion adalah failure mode yang SANGAT realistis
```

Tidak ada watermark, tidak ada kebijakan saat disk penuh, tidak ada kuota per-kategori. **→ D111.**

### 🟡 A-16 — Event log payload tidak di-versioning (D36 ⊥ D6, D40, D99)

D36 = *typed Rust events* (internal, cepat). D6 = event log dipersist ke SQLite. D40 = recovery mereplay event. D99 = backward-compatible upgrade + rollback.

**Masalah:** kalau enum `ExecutionEvent` berubah antar versi, log lama **tidak bisa direplay** → D99 rollback jadi mustahil, dan upgrade bisa meng-orphan eksekusi yang sedang WAITING (D17/D97 — bisa berumur minggu).

D88 hanya meng-cover **DB schema**, bukan **payload format**. **→ D114.**

### 🟡 A-17 — OTel tracing terlalu berat (D65 ⊥ D26, D74)

`opentelemetry` + `tracing-opentelemetry` + OTLP exporter = dep tree besar. D26 bilang *"lightweight"*, D74 bilang *"minimal external dependencies"*. Di 2 GB, unsampled tracing juga memakan CPU & RAM nyata.

**Rekomendasi:** crate `tracing` + structured logs (D63) untuk MVP. OTLP export di belakang **feature flag** (`--features otel`), dengan **sampling decision** eksplisit (mis. 1% + always-sample-on-error). **→ D118.**

### 🟡 A-18 — D53/D54/D55 premature untuk MVP

Sudah dibahas di A-13(c). Ringkas: single-user VPS tidak butuh RBAC. **Tunda ke pasca-v1.0.** Untuk v0.1: bind `127.0.0.1`, akses via SSH tunnel. Lebih aman *dan* lebih murah daripada JWT yang diimplementasi buru-buru.

### 🟡 A-19 — "PostgreSQL adapter later" adalah jebakan (D80 ⊥ D38)

SQLx memang multi-DB, tapi begitu Anda menulis `INSERT OR REPLACE`, `last_insert_rowid()`, `json_extract()`, atau mengandalkan single-writer semantics, adapter Postgres jadi **rewrite**, bukan adapter.

**Rekomendasi:** tambahkan aturan eksplisit — *"Dilarang SQL dialek SQLite. Semua query wajib lolos `sqlx` compile-time check terhadap KEDUA backend di CI."* Kalau tidak sanggup, **hapus janji D80** dan komitmen ke SQLite saja. Janji yang tidak enforceable lebih buruk daripada tidak ada janji.

### 🟡 A-20 — RAM queue redundan & dual-write consistency tidak didefinisikan (D8 ⊥ D76)

Dengan D76 (one core process), "RAM fast queue" sebenarnya cuma `tokio::sync::mpsc` + `Semaphore` — bukan lapisan arsitektur. Menyebutnya "queue layer" menyiratkan kompleksitas yang belum perlu.

Yang lebih penting dan **tidak diputuskan**: task ditulis ke SQLite **dan** di-push ke RAM queue. Kalau keduanya tidak dalam satu transaction boundary (D61 tidak menyebut ini), Anda dapat drift — task ada di RAM tapi tidak di DB, atau sebaliknya.

**Rekomendasi:** SQLite = source of truth tunggal. RAM queue = **cache/index** yang selalu bisa di-rebuild dari DB (ini sudah disebut di D8, bagus). Tambahkan aturan: *"push ke RAM queue hanya SETELAH commit SQLite"*.

### 🟡 A-21 — GC bisa menghapus artifact milik eksekusi WAITING (D39 ⊥ D17, D85, D97)

Workflow bisa WAITING berhari-hari/mingguan (D17, D97 human approval). Selama itu ia mereferensikan spilled payload & blob. Kalau GC (D39) atau retention (D85) berjalan tanpa menyadari status WAITING → **data hilang, eksekusi tidak bisa resume**.

**Rekomendasi:** aturan eksplisit — *"GC dan retention WAJIB mengecualikan semua artifact yang direferensikan oleh eksekusi berstatus non-terminal (RUNNING/WAITING/PAUSED)."* Tambahkan juga max-wait duration (mis. 30 hari) setelah itu execution di-fail eksplisit.

### 🟡 A-22 — Idempotency tanpa spesifikasi (D60, D98)

Tidak diputuskan: di mana idempotency key disimpan, berapa lama TTL-nya, dan apa yang dikembalikan saat duplikat (cached response? 200 kosong? 409?). Juga: webhook delivery semantics (at-least-once diasumsikan?) belum dinyatakan.

### 🟡 A-23 — Tidak ada recursion depth limit (D96 ⊥ D19)

D96 meng-cover cycle & execution limits, tapi **nested subworkflow** (D19) bisa rekursi tanpa cycle di graph: A → B → A → B... setiap level sebagai execution terpisah. Butuh depth limit eksplisit. Masuk ke **D113**.

### 🟡 A-24 — "Dynamic registry" ambigu dan berpotensi berbahaya (D81)

Kalau yang dimaksud `libloading`/`.so`/`dlopen` → arbitrary code execution **tanpa sandbox**, build tidak reproducible, dan single-binary (D73) jadi bohong. Kalau yang dimaksud registry deklaratif → aman. **Harus dinyatakan eksplisit.** Lihat A-02 rekomendasi.

### 🟡 A-25 — Write amplification observability (D6, D63, D64, D65, D67)

Per satu node execution berpotensi menulis: event log (D6) + checkpoint (D6) + audit (D67) + structured log (D63) + metric (D64) + trace span (D65). **Enam append-only sink.** Di 2 core / 50 GB ini material. Perlu keputusan: mana yang sync, mana yang buffered, mana yang sampled. Sebagian dijawab oleh D108, sisanya perlu.

### 🟡 A-26 — D1 dan D100 saling menegang, tidak ada tie-breaker

- **D1:** "maximum practical n8n compatibility"
- **D100:** "correctness → measurable performance → **compatibility**" (compatibility di posisi terakhir)

Saat D11 (performance) bertabrakan dengan D1 (compatibility) — persis kasus A-03 — **D100 menyuruh pilih performance, D1 menyuruh pilih compatibility.** Register tidak punya mekanisme penyelesaian konflik.

**Rekomendasi:** tegaskan bahwa urutan D100 berlaku untuk **correctness vs optimization**, dan tambahkan klausul:

> *"Ketika kompatibilitas n8n bertentangan dengan optimasi internal, **kompatibilitas menang**, kecuali optimasi tersebut menyangkut correctness atau stabilitas sistem. Performa dicapai lewat implementasi, bukan lewat mengubah semantik yang terlihat pengguna."*

### 🟡 A-27 — Merek dagang & lisensi (D1, D55, judul blueprint)

**Fakta terverifikasi:**
- n8n didistribusikan **fair-code** di bawah **Sustainable Use License** + **n8n Enterprise License** (file `.ee.`). Bukan open source.
- SUL melarang: hosting n8n untuk dibayar orang lain, white-labeling untuk pelanggan berbayar, **mengumpulkan kredensial end-user untuk mengakses akun mereka**, dan mengubah/menghapus notice lisensi.
- **"n8n" adalah merek dagang.** Blueprint ini secara harfiah memberi judul produk **"n8n v1.0"**.

**Analisis:**
- Clean-room Rust reimplementation **kemungkinan besar bukan derivative work** → SUL tidak mengikat kode Anda. ✅ (keputusan clean-room di percakapan sudah tepat)
- **TAPI** menyalin *definisi node* (parameter schema, credential shape, deskripsi) dari `n8n-nodes-base` berisiko dianggap derivative — dan itu justru yang dibutuhkan untuk kompatibilitas.
- Nama produk **wajib diganti.** Jangan "n8n", jangan "n8n v1.0", jangan varian yang membingungkan.

**Rekomendasi:**
```
- Ganti nama produk sekarang, sebelum ada kode. Usulan: nama netral
  (mis. "Flowforge", "Rustflow", "Wireweaver" — cek ketersediaan).
- Dokumen kompatibilitas boleh menyebut "n8n-compatible" secara deskriptif
  (nominative fair use), tapi BUKAN sebagai nama produk.
- Untuk node definitions: tulis ulang dari DOKUMENTASI PUBLIK & perilaku
  teramati, bukan dari source n8n-nodes-base. Simpan bukti provenance per node.
- Tambahkan keputusan lisensi proyek Anda sendiri (Apache-2.0 / MIT / dual).
```

### 🟢 A-28 — Backup tanpa jadwal/tujuan (D62)
`VACUUM INTO` atau SQLite Online Backup API — teknisnya benar. Tapi tidak ada keputusan: seberapa sering, ke mana, retensi berapa, dan apakah muat di 50 GB.

### 🟢 A-29 — Webhook path routing belum diputuskan (D27)
D27 hanya meng-cover *ingestion*. Belum diputuskan: bagaimana path di-reserve, apa yang terjadi kalau dua workflow mengklaim path sama, dan bagaimana registrasi/deregistrasi saat workflow diaktifkan/dinonaktifkan.

### 🟢 A-30 — Node parameter UI schema belum ada keputusannya (D43, D44)
Sistem `INodeProperties` n8n (dengan `displayOptions` show/hide bersyarat, resource/operation cascade, typeOptions) adalah **mini-framework tersendiri** dan merupakan sebagian besar pekerjaan editor. Tidak ada satu pun keputusan yang menyentuhnya. Ini akan jadi kejutan besar di Phase 8/12.

---

## 4. Yang Sudah BENAR — jangan diubah

Audit bukan cuma mencari salah. Keputusan berikut **tepat dan sebaiknya dipertahankan apa adanya**:

| # | Keputusan | Kenapa benar |
|---|---|---|
| D2 | Headless/API-first | Keputusan arsitektur terbaik di seluruh register. Memisahkan UI dari engine adalah prasyarat semua hal lain |
| D3 | Hybrid DAG + Event-driven | Tepat. n8n memang butuh wait/resume/webhook-resume, yang tidak bisa dilakukan pure DAG |
| D6 | Event Log + Checkpoint | Pola durable execution yang benar (sejalan dengan Temporal/Restate) |
| D15 | Persistent state machine | Benar |
| D16 | Unified event-driven trigger | Benar, menyederhanakan banyak hal |
| D17 | Persisted wait + wake-up | Benar — dan ini solusi untuk A-09 opsi (b) |
| D23/D45 | REST + WebSocket | Tepat |
| D24 | SQLite WAL | Tepat untuk single-node VPS |
| D33 | Tokio | Standar de facto, matang |
| D34 | Axum | Pilihan tepat: ringan, tower-based, cocok single binary |
| D38 | SQLx + prepared queries | Tepat, dan multi-DB (mendukung D80 *jika* A-19 diikuti) |
| D41 | TOML + env override | Tepat, sederhana |
| D43 | Pertahankan UX n8n | Keputusan produk yang benar — familiaritas adalah nilai |
| D48 | Immutable execution + versioned workflow | Benar, penting untuk audit & debugging |
| D61 | SQLite transaction boundaries | Benar (perlu tambahan dari A-20) |
| D63/D64 | Structured logs + Prometheus | Tepat dan murah |
| D66 | `/health` + `/ready` | Standar, benar |
| D67 | Append-only audit | Benar (perlu batching dari A-06) |
| D70 | Async Tokio workers | Tepat untuk I/O-bound |
| D73/D76/D82 | Single binary, one process | **Sangat tepat** untuk VPS 2 GB — ini keputusan terbaik kedua setelah D2 |
| D75 | Docker opsional | Tepat |
| D79 | Tanpa Redis untuk MVP | **Tepat dan disiplin.** Banyak proyek gagal karena menambahkan Redis terlalu awal |
| D83/D84 | Optimistic versioning + conflict detection | Tepat, murah, cukup untuk skala ini |
| D87/D88/D89 | Versioned migration + backward-compatible schema + explicit node versions | Benar |
| D93 | Automatic secret redaction | Benar, dan sering dilupakan proyek lain |
| D99 | Backward-compatible upgrade + rollback | Benar (perlu D114 untuk bisa dijalankan) |
| D100 | Correctness → Performance → Compatibility | **Instingnya benar**, hanya perlu klarifikasi tie-breaker (A-26) |
| — | Clean-room implementation | **Keputusan paling penting yang sudah benar.** Tanpa ini seluruh proyek berisiko hukum |

---

## 5. Keputusan yang HILANG — usulan D101–D118

Delapan belas hal yang **tidak pernah diputuskan** tapi akan memblokir implementasi. Diurutkan berdasarkan urgensi.

### 🚫 Memblokir — wajib diputuskan sebelum satu baris kode pun

| # | Keputusan | Usulan |
|---|---|---|
| **D101** | **Static data per-workflow** | n8n punya `getWorkflowStaticData('global' \| 'node')` — node menyimpan state persisten antar-eksekusi (mis. cursor polling "last seen ID" untuk trigger SFTP/email/DB). **Tanpa ini, hampir semua polling trigger tidak bisa dibuat.** Simpan di SQLite, scoped per (workflow, node), serialize sebagai JSON. ⚠️ **Perilaku n8n yang wajib ditiru:** static data **hanya tersimpan pada PRODUCTION execution**, tidak pada manual/editor execution → terikat langsung ke **D106** |
| **D102** | **Timezone** | n8n punya `GENERIC_TIMEZONE` per-instance + per-workflow. Cron (D18), `$now`, `$today` semuanya bergantung ini. Wajib eksplisit, default dari config TOML (D41), simpan IANA tz name |
| **D103** | **Error Workflow** | Fitur n8n: workflow khusus yang ter-trigger saat workflow lain gagal. **Berbeda dari dead-letter (D14).** Perlu keputusan: didukung di v1.0 atau ditunda |
| **D104** | **Baseline versi n8n + executionOrder** | Lihat A-14 |
| **D105** | **Binary data container** | Semantik `binary` n8n: `mimeType`, `fileName`, `data` (base64) vs file path vs `moveBinaryData`. Harus konsisten dengan D10 (typed item) dan D30 (blob store) |
| **D106** | **Pin data & mode eksekusi** | n8n membedakan *manual/editor execution* vs *production execution*, plus `pinData` untuk mock output node saat development. **Ini bukan fitur kosmetik** — ia mengubah perilaku inti: static data (D101) hanya persist di production, dan `EXECUTIONS_DATA_SAVE_ON_*` mengatur apa yang disimpan. Fondasi UX editor (D43) — wajib diputuskan sebelum UI dibangun |
| **D109** | **Expression runtime = QuickJS embedded** | Lihat A-01. Ini resolusi untuk kontradiksi terbesar |
| **D115** | **Kernel freeze milestone** | Lihat A-05. Prasyarat sebelum crate apa pun ditulis |

### ⚠️ Penting — putuskan sebelum Phase 2 selesai

| # | Keputusan | Usulan |
|---|---|---|
| **D107** | **Side-effect classification & reconciliation policy** | Lihat A-10 |
| **D108** | **SQLite write batching / group commit** | Lihat A-06 |
| **D110** | **Community node runtime (out-of-process, opt-in, post-v1.0)** | Lihat A-02 |
| **D111** | **Disk Governor** | Watermark (mis. 80% warn / 90% critical / 95% refuse-write), kuota per kategori (blob/spill/logs/db/backup), perilaku saat disk penuh, retention paksa. Simetris dengan D5 |
| **D112** | **Performance SLO numerik** | Lihat A-12 |
| **D113** | **Subworkflow slot accounting + recursion depth limit** | Lihat A-09 |
| **D116** | **MVP scope cut eksplisit** | Lihat A-13 |
| **D117** | **Streaming JSON parse + HTTP body size gating** | Lihat A-07 |

### 📝 Perlu, tapi bisa menyusul

| # | Keputusan | Usulan |
|---|---|---|
| **D114** | **Event log payload versioning** | Field `schema_version` di setiap event record; aturan replay-compat; larangan mengubah varian event tanpa versi baru. Lihat A-16 |
| **D118** | **Tracing sampling + OTLP feature flag** | `tracing` selalu aktif; OTLP export di belakang feature flag; sampling 1% + always-on-error. Lihat A-17 |

---

## 6. Rencana Tindak Lanjut (urut, jangan dilewati)

```
LANGKAH 1 — Resolve 5 CRITICAL
  □ A-01 → putuskan D109 (QuickJS) + revisi D91
  □ A-03 → revisi D11 (item-list default, streaming opt-in)
  □ A-02 → revisi D22/D51/D81, tunda JS community node ke post-v1.0
  □ A-05 → jalankan D115 (Kernel Freeze) SEBELUM crate engine apa pun ditulis
  □ A-04 → konsolidasi register 100 → ~60, tulis ADR untuk yang critical-path

LANGKAH 2 — Putuskan 8 keputusan pemblokir (D101–D106, D109, D115)

LANGKAH 3 — Resolve HIGH
  □ A-06 → D108 (SQLite group commit)
  □ A-07 → D117 (streaming parse + size gating)
  □ A-08 → konsolidasi 6 control loop jadi 1 Resource Governor + mode deterministic
  □ A-09 → D113 (flatten subworkflow)
  □ A-10 → D107 (side-effect classification)
  □ A-11 → 3-layer timeout + watchdog
  □ A-12 → D112 (angka SLO)
  □ A-13 → D116 (MVP scope cut eksplisit)
  □ A-14 → D104 (baseline versi + executionOrder v0/v1)

LANGKAH 4 — Ganti nama produk (A-27). Sebelum ada kode.

LANGKAH 5 — Baru mulai Phase 0 (Project Constitution)
```

### Estimasi dampak kalau audit ini diabaikan

| Kalau tetap coding sekarang | Konsekuensi |
|---|---|
| A-01 tidak di-resolve | Expression engine ditulis ulang total setelah 3–6 bulan |
| A-03 tidak di-resolve | Compatibility test suite (D32) gagal massal; D1 tidak tercapai |
| A-05 tidak di-resolve | crate saling depend melingkar; `cargo build` gagal, lalu dipaksa jadi satu crate raksasa |
| A-06 tidak di-resolve | Engine "lambat tanpa alasan jelas" di 1.000+ node; susah dilacak balik |
| A-07 tidak di-resolve | OOM kill acak di production saat ada API response besar |
| A-09 tidak di-resolve | Deadlock permanen yang hanya muncul saat beban — bug tersulit untuk di-debug |

---

## 7. Penilaian Akhir

| Aspek | Nilai | Komentar |
|---|---|---|
| **Arah strategis** | 🟢 8/10 | D2 (headless), D73/D76 (single binary), D79 (no Redis), clean-room — semua keputusan besar yang benar |
| **Pemilihan tech stack** | 🟢 9/10 | Tokio + Axum + Serde + SQLx + SQLite adalah kombinasi yang nyaris optimal untuk constraint ini |
| **Konsistensi internal** | 🔴 4/10 | 13 konflik, termasuk 3 yang fundamental (A-01, A-02, A-03) |
| **Feasibilitas pada 2 GB** | 🟠 5/10 | Feasible **setelah** A-02, A-07, A-08, A-13 dibereskan. Tidak feasible seperti tertulis sekarang |
| **Kelengkapan** | 🟠 6/10 | 18 keputusan hilang, 2 di antaranya memblokir total (D101 static data, D102 timezone) |
| **Proses governance** | 🔴 3/10 | 88% keputusan lolos tanpa Why/Alternatives/Trade-offs — melanggar aturan sendiri |
| **Realisme scope** | 🔴 3/10 | Tidak ada MVP cut. "v1.0" yang didefinisikan = produk lengkap multi-tahun |

### Kesimpulan

> **Fondasi strategisnya bagus. Stack-nya bagus. Register-nya belum siap jadi blueprint implementasi.**
>
> D1–D12 adalah hasil penggalian yang sungguh-sungguh dan mayoritas tepat. D13–D100 adalah daftar keinginan yang dihasilkan cepat demi menghemat token, dan itu terlihat dari duplikasi serta kontradiksinya.
>
> **Rekomendasi tunggal paling penting:** jalankan **A-05 / D115 (Kernel Freeze)** dan resolve **A-01 + A-03** sebelum satu baris kode engine pun ditulis. Tiga hal ini menentukan apakah seluruh sisa register bisa berdiri.

---

*Dokumen audit. Semua klaim teknis tentang n8n (execution order v0/v1, Sustainable Use License, model data item-array, ekspresi `$('Node')`, `getWorkflowStaticData`) diverifikasi terhadap dokumentasi & paket publik n8n pada 2026-09-09.*
