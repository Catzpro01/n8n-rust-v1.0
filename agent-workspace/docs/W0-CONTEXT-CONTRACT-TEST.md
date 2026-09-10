# W0-CONTEXT-CONTRACT-TEST — Spesifikasi Uji 8 Trait Sambungan

**Versi:** 1.1.0
**Tanggal:** 2026-09-09
**Penyusun:** matt (Lead Architect)
**Untuk:** agent5 (expression), agent9 (HTTP/integration), agent2 (blob/storage),
agent7 (MCP), koordinasi agent3 (QA)
**Rujukan:** PRD-3 v3.4 §7.5 · `W0-Spill-TEST-PLAN.md`
**Metode:** seluruh temuan diturunkan dari **membaca sumber kernel lokal**
(`rust-n8n-core` commit `f7e1827`, `src/context.rs`). Nomor baris disebut supaya
tiap klaim bisa diperiksa ulang.

---

## 0. Kenapa dokumen ini ada

`src/context.rs` adalah 535 baris dengan 25 simbol publik. **Hanya 3 disebut di
test** (`CredentialValue`, `ExecutionMode`, `StaticData`). Delapan trait di
file itu adalah **sambungan tempat kerja agen lain masuk ke kernel**, dan
**kedelapannya nol uji perilaku**:

| Trait | Baris | Siapa yang mengimplementasikan |
|---|---:|---|
| `PriorOutputs` | 205 | engine (agent1/agent4) |
| `ExpressionEngine` | 308 | agent5 (`crates/expr-quickjs`) |
| `EnvAccess` | 379 | engine / host |
| `CredentialProvider` | 389 | agent9 / storage |
| `HttpClient` | 432 | agent9 |
| `BlobStore` | 481 | agent2 |
| `CancellationToken` | 493 | scheduler (agent1) |
| `Logger` | 516 | host / observability |

Konsekuensinya konkret: empat agen akan membangun melawan kontrak yang
perilakunya tidak dipatok satu pun uji. Dua pihak yang membaca trait sama bisa
mengartikannya berbeda, dan **tidak ada yang gagal sampai integrasi**. Itu pola
fork yang sama yang menyebabkan bencana 88-keputusan, hanya kali ini di antarmuka
kode.

**Severity: P1, bukan P0.** Tidak ada yang terbukti *rusak* di sini — beda dari
G-C7 yang mustahil lulus. Yang ada adalah *tidak teruji*. Tapi ia harus selesai
sebelum Wave 1 mengklaim "substrat fondasi", karena Wave 1 membangun tepat di
atas sambungan ini.

---

## 1. Yang SUDAH teruji — jangan diduplikasi

Tiga test sudah ada di `tests/contract.rs` dan menyentuh `context.rs`. Dicatat
supaya tidak ada yang menulis ulang:

| Test | Baris | Menutup |
|---|---:|---|
| `credential_value_never_leaks_in_debug` | 543 | `Debug`/`Display` `CredentialValue` tidak membocorkan nilai, tapi `get()` tetap berfungsi |
| `static_data_flushes_only_in_production` | 486 | `StaticData::should_flush` per `ExecutionMode` (Production=true, Manual/Test=false) |

Implementasi redaksi yang sudah benar dan terverifikasi (`context.rs:413-418`;
`Display` menyusul di `:420-424`):

```rust
impl std::fmt::Debug for CredentialValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CredentialValue(<redacted>)")
    }
}
```

Ini **benar** dan uji yang ada **sah**. Masalahnya bukan di sini — lihat §3.

---

## 2. Spesifikasi uji per trait

`[W]` = wajib sebelum `W0-CONTEXT-CONTRACT-TEST` dinyatakan selesai.

Semua uji butuh **implementasi tiruan** (mock) dari trait terkait, pola yang sama
dengan `MemSpillStore`/`MemWriter` yang sudah ada di `tests/contract.rs:26-171`.
Mock tinggal di `tests/`, bukan di `src/` — kernel tidak punya modul
`#[cfg(test)]` dan itu sebaiknya tetap begitu, supaya `src/` bebas dari kode uji.

Catatan praktis: trait-trait ini **campuran sync dan async**. `PriorOutputs::item`
async tapi `items`/`by_name`/`by_id`/`available_nodes` sync. Uji async pakai
`block_on` yang sudah ada di `tests/contract.rs:171` (executor spin-waker, tanpa
tokio) — jangan menambah dependency tokio ke dev-dependencies kernel, karena itu
akan melanggar allowlist gate §3.

### CT-01 `PriorOutputs` — resolusi nama vs id `[W]`

Doc `context.rs:206`: *"n8n addresses nodes by display name."*

| Uji | Assert |
|---|---|
| CT-01a | `by_name("HTTP Request")` menemukan node; `by_name("http request")` (beda kapitalisasi) **tidak** — n8n case-sensitive pada display name |
| CT-01b | `by_name` pada nama tak ada → `None`, **bukan** `Err`. Bedakan dari `item()` yang mengembalikan `Err(NodeNotFound)` |
| CT-01c | `by_id` dan `by_name` menunjuk `PriorNodeRef` yang sama untuk node yang sama |
| CT-01d | `item(node, input_index, i)` dengan `i` melampaui `branch_lengths[input_index]` → `Err(IndexOutOfBounds { index, len })`, bukan panic |
| CT-01e | `items()` mengembalikan `ItemList` yang **bisa spilled** — doc `:200-203` menjanjikan "stays cheap even for executions that produced gigabytes". Uji: pastikan tidak ada materialisasi penuh (assert `ItemList::Spilled`, bukan `Inline`, untuk list besar) |
| CT-01f | `available_nodes()` hanya memuat node yang **sudah selesai**, bukan yang sedang berjalan — ini yang dipakai validasi ekspresi pra-eksekusi |

CT-01e adalah yang paling penting: kalau implementasi mengembalikan `Inline`
untuk output besar, seluruh klaim memori 2GB runtuh di lapisan ekspresi, dan
tidak ada test lain yang akan menangkapnya.

### CT-02 `ExpressionEngine` — kontrak D32 `[W]`

Doc `context.rs:302-306` (verbatim, `**` adalah bold markdown di sumber): *"n8n
expressions are JavaScript. The kernel defines the contract only; the
implementation (QuickJS via `rquickjs`) lives in `crates/expr-quickjs`."*
(di sumber, "are JavaScript" dicetak tebal)

| Uji | Assert |
|---|---|
| CT-02a | `validate(expr)` **tidak** mengeksekusi. Uji dengan ekspresi berisi efek samping yang dapat diamati mock; `validate` harus mengembalikan `Ok`/`Err` tanpa memicu apa pun |
| CT-02b | `validate` lulus untuk ekspresi yang `eval`-nya akan gagal runtime (mis. `{{ $json.tidakAda.foo }}`) — validasi adalah sintaks+referensi, bukan evaluasi |
| CT-02c | `eval_batch(&[...])` menghasilkan **hasil yang identik** dengan memanggil `eval` per-ekspresi pada scope yang sama. Ini kontrak yang membuat optimasi warm-context aman |
| CT-02d | `eval_batch` dengan slice kosong → `Ok(vec![])`, bukan `Err` |
| CT-02e | ekspresi gagal → `Err(KernelError::Expression { expr, reason })` dengan `expr` memuat **teks ekspresi asli**, bukan placeholder |
| CT-02f | `ExpressionScope` memuat **semua** field yang didaftar doc `:326-327` — "Anything missing here fails D32 compatibility tests, so this list is the contract with n8n" |

CT-02c adalah alasan `eval_batch` ada (doc `:312-315`: setup konteks QuickJS
ratusan mikrodetik, tidak boleh dibayar per-ekspresi per-item). Kalau hasilnya
berbeda dari `eval`, optimasi itu mengubah semantik — dan itu persis jenis bug
yang tidak terlihat tanpa uji.

CT-02f layak dijadikan uji **kompilasi**, bukan runtime: konstruksikan
`ExpressionScope` dengan semua field bernama. Kalau ada field hilang atau berganti
tipe, test gagal kompilasi. Itu penjaga termurah untuk kontrak D32.

### CT-03 `EnvAccess` + whitelist `[W]` — lihat §3.2

| Uji | Assert |
|---|---|
| CT-03a | `get("N8N_ALLOWED")` → `Some`; `get("AWS_SECRET_ACCESS_KEY")` → `None` pada implementasi whitelist bawaan |
| CT-03b | **Tidak ada** metode untuk enumerasi seluruh environment. Trait hanya punya `get(&self, key) -> Option<String>` (`:380`) — tidak ada `keys()`, tidak ada `iter()`. Uji ini menjaga sifat itu tetap benar |

CT-03b adalah uji **bentuk API**, dan ia yang paling berharga di sini: doc
`ExpressionScope.env` (`:345`) menulis *"$env — MUST be whitelisted; never expose
the whole environment"*, dan satu-satunya hal yang membuat itu dapat ditegakkan
adalah trait-nya tidak menyediakan cara mengambil semuanya. Kalau seseorang
menambah `fn all(&self)`, whitelist jadi opsional.

### CT-04 `CredentialProvider` — least privilege D92 `[W]`

Doc `:390`: *"Returns `Err` if the node did not declare this credential kind."*

| Uji | Assert |
|---|---|
| CT-04a | node meminta kind yang **tidak** dideklarasikannya → `Err`, bukan `Ok` dengan nilai kosong |
| CT-04b | node meminta kind yang dideklarasikannya → `Ok(CredentialValue)` |
| CT-04c | mock mencatat setiap `kind` yang diminta; assert **himpunan** kind yang diakses persis sama dengan yang dideklarasi — tidak lebih |

CT-04c adalah uji yang sebenarnya menegakkan D92. Tanpa pencatatan, implementasi
bisa mengambil semua kredensial di awal ("biar cepat") dan tetap lulus CT-04a/b.

### CT-05 `HttpClient` — plafon memori `[W]`

Doc `HttpRequest.max_response_bytes` (`:443`): *"Hard ceiling in bytes.
Exceeding it is an error, not an OOM."*

| Uji | Assert |
|---|---|
| CT-05a | respons melebihi `max_response_bytes` → **`Err`**, dan **bukan** alokasi penuh lalu gagal. **KOREKSI v1.1:** draf v1.0 menulis `KernelError::ResourceExhausted` — **tipe itu tidak ada**. `ResourceExhausted` adalah varian `NodeError` (`error.rs:64`), sedangkan `HttpClient::send` mengembalikan `KernelError` (`context.rs:433`). Lihat §3.4 — ini bukan salah ketik, ini lubang desain |
| CT-05b | mock yang mengirim body bertahap: assert implementasi **berhenti membaca** begitu plafon tercapai, tidak membaca sisa lalu membuang |
| CT-05c | `timeout_ms` terlampaui → **`Err`**. **KOREKSI v1.1:** sama seperti CT-05a, `Timeout` ada di `NodeError:51`, bukan `KernelError`. Uji tidak bisa meng-assert varian spesifik sampai §3.4 diputuskan |
| CT-05d | `RequestBody::Blob { content_id }` **tidak** memuat blob ke RAM — doc `:452`: "Streamed from the blob store — never fully in RAM". Uji dengan blob besar di mock `BlobStore` dan assert tidak ada materialisasi |
| CT-05e | `headers` dan `query` adalah `BTreeMap` → iterasi deterministik. Assert urutan stabil antar-panggilan (ini yang membuat differential test L3 mungkin untuk node HTTP) |

CT-05a dan CT-05d adalah dua uji yang paling langsung melindungi klaim produk
2GB. Keduanya tentang **tidak mengalokasikan** sesuatu, yang secara umum sulit
diuji — jadi uji lewat mock yang mengamati urutan panggilan, bukan lewat
pengukuran RSS.

CT-05e terdengar kecil tapi menentukan: kalau `headers` jadi `HashMap`, urutan
berubah antar-run dan seluruh gate L3 (diferensial byte-for-byte) jadi tidak
stabil untuk node HTTP apa pun.

### CT-06 `BlobStore` `[W]`

| Uji | Assert |
|---|---|
| CT-06a | `put` → `get` round-trip identik byte-per-byte |
| CT-06b | `put` mengembalikan `(ContentId, u64)` di mana `u64` == panjang byte yang ditulis, dan `size(id)` mengembalikan angka yang sama |
| CT-06c | `get`/`size` pada `ContentId` tak dikenal → `Err` / `None`, bukan panic |
| CT-06d | `delete` lalu `get` → `Err`; `delete` lalu `size` → `None` |
| CT-06e | `delete` **idempoten** — menghapus dua kali tidak memberi `Err` |
| CT-06f | dua `put` dengan byte identik: dokumentasikan apakah `ContentId` sama (content-addressable/dedup) atau berbeda. **Uji ini tidak punya jawaban benar tanpa keputusan** — lihat §3.3 |

CT-06f sengaja ditulis tanpa assert tetap. `W2-CASD-DEDUP` (content-addressable
dedup) adalah tugas `[INOVASI]` yang statusnya menunggu K-8. Kalau dedup masuk,
`ContentId` untuk byte identik harus sama; kalau tidak, harus berbeda. Menulis
assert sekarang berarti memutuskan K-8 lebih dulu — pola yang sama dengan
"paritas 100% sebagai sifat" yang saya koreksi di PRD-3 §3.1.

### CT-07 `CancellationToken` `[W]`

Doc `:488-492`: *"D20 — cooperative cancellation. Cooperative cancellation cannot
stop a CPU-bound loop that never polls it (audit A-11), so the Node SDK must also
enforce a watchdog at the scheduler layer. This token is the polite half of that
mechanism."* (di sumber, kata "polite" dicetak miring)

| Uji | Assert |
|---|---|
| CT-07a | `NoCancel::is_cancelled()` == `false` (`:504-508`) |
| CT-07b | `NoCancel::reason()` == `None` lewat **default method** (`:496-498`) — uji bahwa default-nya benar, karena implementasi lain bergantung padanya |
| CT-07c | token yang dibatalkan: `is_cancelled()` == `true` dan `reason()` memuat alasan |
| CT-07d | pembatalan **teramati di tengah iterasi**: mock yang membatalkan setelah N poll, assert loop berhenti dalam batas wajar |

CT-07d adalah yang menguji A-11 secara nyata. Doc-nya sudah jujur bahwa token ini
hanya "setengah sopan" dari mekanismenya — uji ini membatasi apa yang dijanjikan
setengah itu, dan mencegah seseorang mengira token saja sudah cukup.

### CT-08 `Logger` — lihat §3.1, **butuh keputusan dulu** `[W]`

| Uji | Assert |
|---|---|
| CT-08a | `NoopLogger::log` tidak panic untuk level mana pun (`:530-535`) |
| CT-08b | `LogLevel` round-trip serde dengan `rename_all = "lowercase"` (`:520-528`) → `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"` |
| CT-08c | **DITAHAN** — uji redaksi tidak bisa ditulis sampai §3.1 diputuskan |

CT-08b terdengar sepele tapi ini kontrak wire-format: kalau `rename_all` hilang,
log berubah jadi `"Trace"` dan setiap parser/dasbor di hilir rusak diam-diam.

---

## 3. Tiga temuan desain yang butuh keputusan, bukan sekadar uji

Bagian ini alasan dokumen ini bukan hanya daftar test. Ketiganya ditemukan saat
membaca sumber untuk menulis §2.

### 3.1 Klaim redaksi `Logger` tidak bisa ditegakkan — dan ada DUA jalur bocor

Ini temuan paling substansial dari pembacaan sumber untuk dokumen ini.

**Dua tempat di doc menjanjikan logger tidak akan membocorkan kredensial.**

`context.rs:394-395`, doc untuk `CredentialValue`:

```rust
/// A resolved credential. Values are wrapped so that `Debug`/`Display` and the
/// logger cannot leak them (D93 automatic redaction).
```

`context.rs:514-515`, doc untuk `Logger`:

```rust
/// D63 structured logging. Redaction (D93) is the implementation's job and must
/// be unconditional — a node cannot opt out of it.
pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str, fields: &Value);
}
```

**Tapi `CredentialValue` punya dua metode yang mengembalikan nilai mentah.**

```rust
pub fn get(&self, key: &str) -> Option<&Value> {   // :405-407
    self.inner.get(key)
}
pub fn as_value(&self) -> &Value {                 // :408-410
    &self.inner
}
```

Keduanya mengembalikan `&Value` **tanpa pembungkus**. Jadi kedua jalur ini tidak
melewati redaksi apa pun:

```rust
logger.log(LogLevel::Info, "resolved", cred.as_value());        // seluruh objek
logger.log(LogLevel::Info, "ok", &json!({ "k": cred.get("apiKey") }));  // per-field
```

Redaksi `Debug`/`Display` yang sudah teruji (§1) **tidak berlaku di sini**, karena
yang dikirim ke logger adalah `Value` di *dalam* wrapper, bukan wrapper-nya.

**Presisi severity — ini bukan G-C7.**

- Klaim di `:394-395` **benar** untuk jalur `Debug`/`Display`, dan itu sudah
  teruji. Jangan dikoreksi berlebihan.
- Klaim yang sama **salah** untuk jalur `as_value()`/`get()` → `logger.log`.
- Tidak ada bug di kode yang berjalan: satu-satunya implementasi `Logger` di
  kernel adalah `NoopLogger` (`:530-535`) yang membuang semuanya. Jadi **belum ada
  kebocoran nyata hari ini**.
- Yang ada adalah **klaim doc yang lebih kuat dari yang bisa ditegakkan bentuk
  API-nya**, dan kalimat "a node cannot opt out of it" secara harfiah tidak benar:
  node *bisa* opt out, cukup dengan memanggil `as_value()`.

Risiko ini jadi nyata begitu ada implementasi `Logger` sungguhan (host /
observability) dan node sungguhan mulai mencatat hasil kredensial. Itu terjadi di
Wave 1, bukan sekarang. Jadi ada waktu — tapi keputusan harus diambil **sebelum**
implementasi pertama ditulis, karena setelah itu biaya berubahnya naik.

**Precedent yang benar sudah ada di file yang sama.**

`context.rs:351-355` — implementasi `Debug` manual untuk `ExpressionScope`:

```rust
/// Manual: `dyn PriorOutputs` and `dyn EnvAccess` are not `Debug`.
///
/// Prints `$json` because that is usually what you are actually debugging, but
/// deliberately omits `env` — its values are secrets by construction (D94) and
/// an expression error that dumps the scope must not dump credentials into logs.
```

Ini persis pola yang saya rekomendasikan: **rahasia dikeluarkan berdasarkan
konstruksi tipe, bukan berdasarkan harapan bahwa pemanggil akan sopan.** Kalau
`ExpressionScope` bisa melakukannya untuk `env`, `Logger` bisa melakukannya untuk
`fields`.

**Pilihan** — keputusan milik Pemilik Proyek/fern bersama agent10 (SEC), bukan saya:

| Opsi | Isi | Biaya | Catatan |
|---|---|---|---|
| **(a) Perkuat tipe** | `log(level, message, fields: &LogFields)` di mana `LogFields` tidak bisa memuat `Value` mentah hasil `get`/`as_value` | mengubah tanda tangan trait; semua implementasi menyesuaikan | satu-satunya yang benar-benar menegakkan "cannot opt out" |
| **(b) Perbaiki doc** | turunkan klaim `:394-395` dan `:514-515` jadi best-effort by key name; tegakkan lewat review + lint | murah | mengandalkan disiplin manusia — persis mekanisme yang gagal di 88-keputusan |
| **(c) Hapus `as_value()` saja** | paksa node memakai `get(key)` per-field | paling kecil | **CACAT** — `get(key)` juga mengembalikan `&Value` mentah, jadi jalur bocornya tetap ada. Saya usulkan ini di draf pertama; setelah membaca `:405-407` ternyata tidak menyelesaikan masalah |

Rekomendasi saya **(a)**, dengan alasan: precedent D94 di `:351-355` menunjukkan
codebase ini sudah memilih pola "tegakkan lewat tipe" di tempat lain, dan (b)
menyerahkan jaminan keamanan ke disiplin yang sudah terbukti gagal di sesi ini.

Tapi **(a) mengubah trait yang mungkin sudah diimplementasikan di
`/opt/agent-workspace/rust-engine/`** (360K, 24 file .rs). Saya **tidak bisa
memeriksa dari sini** karena akses SSH hilang. Jadi sebelum diputuskan, langkah
pertama adalah: grep `impl Logger` dan `impl CredentialProvider` di `rust-engine/`.
Kalau sudah ada implementasi, biaya (a) naik dan perlu dihitung ulang.

**Uji yang ditahan karena ini:** CT-08c. Menulis uji redaksi sekarang berarti
menyemenkan salah satu dari tiga pilihan itu secara diam-diam — persis pola
"paritas 100% sebagai sifat" yang saya koreksi di PRD-3 §3.1.

Satu uji **tidak** perlu ditahan dan layak ditambah:

| Uji | Assert |
|---|---|
| CT-04d | dokumentasikan secara eksplisit bahwa `get()` dan `as_value()` mengembalikan `&Value` mentah. Uji berbentuk *kompilasi*: pastikan `logger.log(.., cred.as_value())` **bisa dikompilasi**. Kalau suatu hari opsi (a) diterapkan, uji ini gagal kompilasi — itu sinyal bahwa perubahan disengaja dan doc `:394-395` sudah diperbarui |

CT-04d terasa aneh (uji yang berharap kode berbahaya bisa kompilasi), tapi
fungsinya adalah **alarm perubahan**, bukan persetujuan. Ia membuat keputusan
§3.1 terlihat di build, bukan terkubur di doc.

### 3.2 Whitelist `EnvAccess` ditegakkan oleh *bentuk* API, bukan oleh kode

Ini **kabar baik**, dan layak dicatat supaya tidak dirusak orang yang tidak tahu.

Doc `ExpressionScope.env` (`:345`): *"$env — MUST be whitelisted; never expose the
whole environment."* Trait-nya (`:379-381`) hanya punya:

```rust
pub trait EnvAccess: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
}
```

Tidak ada `keys()`, tidak ada `iter()`, tidak ada cara mengambil seluruh
environment. Jadi whitelist **ditegakkan oleh bentuk API** — implementasi tidak
bisa membocorkan semuanya meskipun mau, karena tidak ada metode untuk itu.

Ini pola yang benar dan sebaiknya jadi contoh untuk §3.1: kalau jaminannya mau
nyata, tegakkan lewat tipe, bukan lewat doc. CT-03b ada khusus untuk menjaga
sifat ini tetap benar.

### 3.3 `BlobStore` belum memutuskan content-addressable atau tidak

`put(&[u8], mime) -> (ContentId, u64)`. Tidak ada di doc apakah `ContentId`
diturunkan dari isi (content-addressable, jadi byte identik → id identik) atau
diacak per-`put`.

Kedua pilihan sah, tapi keduanya memberi perilaku berbeda untuk `W2-CASD-DEDUP`
dan untuk GC. CT-06f sengaja tidak diberi assert tetap — lihat alasannya di §2.

---

### 3.4 Semua trait servis mengembalikan tipe error yang tidak bisa menyatakan kegagalannya sendiri

Ditemukan saat mengoreksi CT-05a/CT-05c atas masukan agent1 (#999). Ini temuan
paling struktural di dokumen ini.

Fakta, terverifikasi baris per baris:

| Simbol | Lokasi | Mengembalikan |
|---|---|---|
| `Node::execute` | `node.rs:26` | `Result<NodeOutput, `**`NodeError`**`>` |
| `HttpClient::send` | `context.rs:433` | `Result<HttpResponse, `**`KernelError`**`>` |
| `BlobStore::put/get/delete` | `context.rs:482-484` | `Result<.., `**`KernelError`**`>` |
| `CredentialProvider::get` | `context.rs:391` | `Result<CredentialValue, `**`KernelError`**`>` |

`KernelError` (`error.rs:9-28`) punya **tepat enam** varian: `IndexOutOfBounds`,
`SpillIo`, `Codec`, `NodeNotFound`, `Expression`, `Invalid`.

`NodeError` (`error.rs:37-77`) punya: `Transient`, `Permanent`, **`Timeout`**,
**`ResourceExhausted`**, `Expression`, `Internal`.

Dan **tidak ada `impl From` di antara keduanya** (diperiksa di seluruh `src/`).

Akibatnya konkret, dan semuanya tentang kontrak yang sudah dijanjikan doc:

1. **`HttpRequest.max_response_bytes`** (`:443`) menulis *"Hard ceiling in bytes.
   Exceeding it is an error, not an OOM."* Tapi `send` tidak punya varian untuk
   menyatakan "resource exhausted". Implementasi harus memaksanya ke
   `KernelError::Invalid { message }` — kehilangan field terstruktur `resource`,
   `requested`, `available`.
2. **`HttpRequest.timeout_ms`** (`:445`) ada sebagai field kontrak, tapi tidak ada
   varian `Timeout` yang bisa dikembalikan.
3. **`BlobStore`** tidak bisa menyatakan kehabisan disk atau file handle
   (`Resource::Disk`, `Resource::FileHandles` ada di `error.rs:80-85`, tapi hanya
   bisa dicapai lewat `NodeError`).
4. **`CredentialProvider`** tidak bisa menyatakan kegagalan *transient* ke secret
   store. Doc `NodeError` (`error.rs:32-35`) menulis bahwa beda varian *"is not
   cosmetic — it drives retry policy (D14/D57), quarantine (D69), and crash
   reconciliation (audit A-10)"*. Tanpa `Transient`, **retry policy tidak bisa
   didorong dari lapisan servis.**
5. Node yang menangkap error servis **tidak bisa menaikkan ulang secara setia** —
   tidak ada konversi, jadi ia harus meratakannya ke `Internal` atau `Permanent`,
   yang mengubah kebijakan retry.

**Pola ini sama dengan §3.1**: doc menjanjikan sesuatu yang tanda tangannya tidak
bisa nyatakan. Bedanya, di sini dampaknya ke *seluruh* lapisan servis, bukan satu
trait.

Pilihan (keputusan fern + Pemilik Proyek, bukan saya):

| Opsi | Isi | Biaya |
|---|---|---|
| **(a) Tambah varian ke `KernelError`** | `Timeout`, `ResourceExhausted`, `Transient` masuk `KernelError` | menduplikasi kosakata `NodeError` — persis "fork kosakata" yang agent1 peringatkan di #999 |
| **(b) Trait servis mengembalikan `NodeError`** | satu kosakata error untuk semua kegagalan yang bisa di-retry | `NodeError` jadi aneh untuk operasi non-node |
| **(c) `impl From<KernelError> for NodeError`** + varian baru minimal | konversi eksplisit, node bisa menaikkan ulang dengan setia | paling kecil, tapi tetap butuh keputusan varian mana yang ditambah |

Rekomendasi saya **(c)**, karena ia menutup lubang konversi (poin 5) tanpa
menduplikasi seluruh kosakata. Tapi CT-05a/CT-05c/CT-06 tidak bisa meng-assert
varian spesifik sampai ini diputuskan — jadi uji-uji itu saya tulis sebagai
"assert `Err`" saja, dan assert varian ditambahkan setelah keputusan.

## 4. Definisi selesai

| Kriteria | Verifikasi |
|---|---|
| CT-01…CT-07 hijau, termasuk CT-04d | `cargo test -p kernel` |
| CT-08a, CT-08b hijau | idem |
| §3.1 diputuskan; CT-08c ditulis sesuai keputusan. **Langkah pertama: grep `impl Logger` dan `impl CredentialProvider` di `/opt/agent-workspace/rust-engine/`** untuk menakar biaya opsi (a) | tercatat di `DEVIATION-CATALOG.md` bila memilih (b) |
| §3.3 diputuskan dan CT-06f diberi assert | tercatat di PRD-3 atau ADR |
| Tidak ada dependency baru di kernel | gate §3 allowlist tetap lulus (serde, serde_json, async-trait, thiserror) |
| Tidak ada regresi | 19 test lama tetap hijau, clippy 0 warning |
| Cakupan terukur, bukan diduga | `cargo tarpaulin`/`llvm-cov` dijalankan di VPS dan angkanya dicatat — **belum pernah diukur sampai sekarang** |

Butir terakhir penting: angka 40% di PRD-3 §7.5 adalah **proxy** (simbol disebut
di test), bukan coverage sungguhan. Setelah tugas ini selesai, ukur yang nyata
dan ganti angka proxy itu. Jangan biarkan proxy beredar sebagai kalau ia
pengukuran.

**Bukan bagian tugas ini:** implementasi `expr-quickjs` (agent5, Wave 1),
implementasi `HttpClient` nyata (agent9), dan apa pun yang menyentuh
`rust-engine/` — dokumen ini hanya tentang kontrak di `kernel/src/context.rs`.
