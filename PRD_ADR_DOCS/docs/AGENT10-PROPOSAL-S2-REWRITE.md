# PROPOSAL — Penulisan Ulang BAHASA-BERSAMA.md §2 (Tabel Tipe Kanonik)

| | |
|---|---|
| **Pengusul** | agent10 (ROLE_COMPLIANCE) |
| **Versi** | v1.2.1 (Ruling 20 #1116 @matt: (1) gate §6 tambah `store_ref` + `Arc<dyn SpillStore>`; (2) nitpick item.rs:212-219 → 212-219. v1.2: koreksi sitasi `id.rs` per @agent1 `#1123`; v1.1 menyerap N1/N2/N3 `#1043` + temuan pasca-eksekusi §8) |
| **Dasar** | Ruling 3(b) @matt `#1000`: *"§2 ditulis ulang seluruhnya dari kernel/src yang nyata, bukan ditambal per sel."* |
| **Status konsensus** | **2/2 `[CONSENSUS-ACK]`**: peer-1 @agent1 `#1043` (APPROVE + 3 non-blocking, verifikasi sitasi mandiri 8/8), peer-2 @agent7 `#1047` (APPROVE + 2 non-blocking, sudut konsumen eksternal MCP). Menunggu ratifikasi @matt / @fern |
| **Status RFC §2 sel Envelope** | `[CONSENSUS-REACHED]` (Ruling 2 `#1000`, peer-1 @agent1 `#957`, peer-2 @matt `#1000`) — **sudah dieksekusi di disk**, lihat §8 |
| **Pohon sumber bukti** | `/opt/agent-workspace/kernel-asli-d3bcff0` @ HEAD `f4adc8f` |
| **Metode** | Setiap baris mengutip `berkas:baris`. Tidak ada satu pun tipe di bawah ini yang ditulis dari ingatan. |
| **Eksekusi berkas** | @fern / Pemilik Proyek — berkas §2 saat ini milik `root` dan @matt tidak bisa (dan sengaja tidak mau) menimpanya. |

---

## 1. Masalah yang ditutup dokumen ini

§2 versi root (10:49 UTC) memandatkan tipe yang **tidak ada** di kernel. Terverifikasi grep di
`kernel-asli-d3bcff0/crates/*/src` (mengecualikan `examples/`):

| Tipe yang dipandatkan §2 root | Kecocokan di kernel/src | Keterangan |
|---|---|---|
| `PayloadRef` | **0** | Tidak ada. Yang ada `ItemList` |
| `SpillId` | **0** | Tidak ada. Yang ada `SpillPath` |
| `SpillRef` | **0** | Tidak ada |
| `uuid::Uuid` | **0** | Tidak ada, dan `uuid` **melanggar** gate allowlist 4-dependensi |
| `Bytes` (sebagai tipe) | **0** sebagai tipe | Token `Bytes` hanya ada sebagai **nama varian** `RequestBody::Bytes(Vec<u8>)` (context.rs:451) dan `ResponseBody::Bytes(Vec<u8>)` (context.rs:470) — bukan tipe `bytes::Bytes` |

Bukti bahwa ini bukan risiko teoretis: kurang dari satu jam setelah dokumen itu terbit, satu crate
sudah mengutipnya sebagai otoritas untuk tipe fiktif (temuan F-1, `#956`; dikonfirmasi @matt `#1000`
Ruling 6). Sudah diperbaiki — lihat §5.

## 2. Kosakata resmi: re-ekspor publik kernel

Sumber tunggal yang paling dapat diandalkan adalah **apa yang diekspor kernel**, karena itu kontrak
publik yang dikompilasi, bukan prosa. `crates/kernel/src/lib.rs:80-95`:

```
error : ErrorCode, KernelError, NodeError, Resource
event : execution_status, EventRecord, ExecutionEvent, ExecutionStatus, GovernorLevel,
        WaitReason, SCHEMA_VERSION
id    : CheckpointId, ContentId, ExecutionId, NodeId, NodeKind, SpillPath, TaskId,
        WorkerId, WorkflowId
item  : spill_from_iter, BinaryContainer, BinaryData, BinaryLocation, Item, ItemCursor,
        ItemList, ItemView, PairedItem, SpillCodec, SpillPolicy, SpillStore, SpillWriter,
        SpilledList
node  : BufferingMode, Node, NodeDescriptor, NodeGroup, NodeOutput, ResourceHint,
        SideEffect, Weight
```

Catatan penting — **DIKOREKSI di v1.2, dan koreksi ini penting karena saya sendiri yang salah**:
tipe `id::*` **tidak semuanya** numerik. Hanya enam yang dibuat makro `numeric_id!` (`id.rs:9`,
dipanggil di `id.rs:34-38,41`):

| Newtype **angka (u64)** via `numeric_id!` | Newtype **String** (opaque, didefinisikan manual) |
|---|---|
| `WorkflowId` (id.rs:34) | `NodeId(pub String)` (id.rs:49) |
| `ExecutionId` (id.rs:35) | `NodeKind(pub String)` (id.rs:71) |
| `TaskId` (id.rs:36) | `SpillPath(pub String)` (id.rs:97) |
| `CheckpointId` (id.rs:37) | |
| `WorkerId` (id.rs:38) | |
| `ContentId` (id.rs:41) | |

Kesalahan v1.0/v1.1 dokumen ini: §2 menyatakan *"seluruh tipe `id::*` dibuat oleh makro
`numeric_id!`, jadi semuanya newtype angka (u64)"*, dan §3.1 menyitir `SpillPath` sebagai
"via `numeric_id!`". **Keduanya salah**, dikoreksi oleh @agent1 di `#1123` dan saya verifikasi
sendiri ke `id.rs`. Konsekuensi praktisnya nyata: `SpillPath` adalah String opaque, jadi ia
**tidak** bisa diperlakukan sebagai angka, dan perbandingan/pengurutan mengikuti semantik String.

Konsekuensi kepatuhan yang **tetap berlaku** setelah koreksi ini (ia hanya bergantung pada
`ContentId`, yang memang `numeric_id!`): id sekuensial **bukan** jangkar identitas isi, jadi
`ContentId` tidak boleh menjadi kunci lineage/erasure selama C-06 terbuka (`#920`/`#939`).

## 3. USULAN §2 PENGGANTI (drop-in)

### 3.1 Payload & spill

| Konsep | Tipe kanonik | Bukti | Catatan kepatuhan |
|---|---|---|---|
| Daftar item antar-node | `ItemList::Inline(Vec<Item>)` \| `ItemList::Spilled(SpilledList)` | `item.rs:203-209` | Satu-satunya kosakata payload. **Jangan** menulis `PayloadRef` |
| Metadata spill | `SpilledList { path: SpillPath, len: u32, total_bytes: u64, codec: SpillCodec }` | `item.rs:212-219` | `len` disimpan agar `len()` tidak menyentuh store |
| Satu item | `Item { json: serde_json::Value, binary: Option<…> }` | `item.rs:52-62` (keputusan D105) | `json` sengaja bukan generic param |
| Identitas node | `NodeId(pub String)` | `id.rs:49` | **String opaque**, bukan numerik — jangan di-join sebagai angka |
| Jenis node | `NodeKind(pub String)` | `id.rs:71` | **String opaque**; dipakai Rosetta/codegen |
| Rujukan berkas spill | `SpillPath(pub String)` — **opaque String, BUKAN numerik** | `id.rs:97`, re-ekspor `lib.rs:87` | Bukan `SpillId`, bukan `SpillRef`. **Koreksi v1.2**: sitasi "via `numeric_id!`" di v1.0/v1.1 salah (@agent1 `#1123`) |
| Identitas isi blob | `ContentId` | `id.rs:41` (`numeric_id!`) | u64 sekuensial → **tidak boleh** jadi kunci lineage/erasure selama C-06 terbuka |
| Trait penyimpanan spill | `SpillStore` (+ `SpillWriter`, `SpillPolicy`, `SpillCodec`) | re-ekspor `lib.rs:89-91` | Implementasi `FileSpillStore` saat ini **hanya** di `crates/kernel/examples/spill_bench.rs` — belum di `src/` (temuan `#960` butir 5) |

### 3.2 Body HTTP (batas masuk/keluar)

| Konsep | Tipe kanonik | Bukti |
|---|---|---|
| Body permintaan | `RequestBody::{Json(Value), Bytes(Vec<u8>), Blob { content_id: ContentId }, Form(BTreeMap<String,String>) }` | `context.rs:449-455` |
| Body respons | `ResponseBody::{Empty, Json(Value), Bytes(Vec<u8>), Blob { content_id: ContentId, size_bytes: u64 } }` | `context.rs:468-475` |
| Respons | `HttpResponse { status: u16, headers: BTreeMap<String,String>, body: ResponseBody, duration_ms }` | `context.rs:460-466` |

`BTreeMap` (bukan `HashMap`) untuk header: terurut → serialisasi deterministik. Ini patut ditulis
eksplisit di kamus karena ia alasan struktural, bukan kebetulan.

### 3.3 Digest & checksum (tetap, sudah benar)

| Konsep | Aturan | Status |
|---|---|---|
| Digest integritas | BLAKE3, `[u8; 32]` (bukan `String`), hex 64-char lowercase | **PERTAHANKAN** — selaras Piagam `#922` §A dan praktik teruji (`envelope`, `determ`) |
| Checksum berkas fisik | SHA-256 | **PERTAHANKAN** (`#922` §A, G-C7 `#751`) |
| Kanonikalisasi digest | **per kontrak tiap artefak**, bukan satu aturan global | lihat 3.4 |

### 3.4 Kanonikalisasi: satu baris per artefak, bukan satu aturan untuk semua

Ini pelajaran dari RFC `#945`. Kesalahan §2 root adalah menetapkan **satu** mekanisme kanonikalisasi
("JSON terurut alfabetis") untuk seluruh artefak. Yang benar:

| Artefak | Kanonikalisasi | Bukti / alasan |
|---|---|---|
| **Execution Envelope** | Biner: 10 field, prefix panjang **u32 big-endian**, `CTX="n8nrust/envelope/v1"` | `AGENT10-EXEC-ENVELOPE-SPEC.md` v1.2 §2; kutipan persis pasal C-02: *"Framing kanonik: semua field panjang-variabel memakai prefix u32 big-endian; TIDAK ADA JALUR ENCODING KEDUA (C-02)."* Implementasi: `entry.rs:80` `to_be_bytes()`, `:84` `(b.len() as u32).to_be_bytes()`; gate pergeseran batas `entry.rs:185,219`. **40 gate lulus** (21 unit + 30 demo + 10 E2E, TSA nyata) |
| **RecordSet** (determ) | Biner BE, prefix panjang u32/u64, `SCHEMA_VERSION=1` | `w2-determ-enforce/src/lib.rs`; 9 gate lulus |
| **TimelineEvent** | JSON dengan **field struct tetap** (bukan map dinamis) + serializer ter-pin | spec `#910` **v0.7 (sha `e92f7083`)** — sitasi dimutakhirkan per N1 @agent1 `#1043`; bila butuh map dinamis → pakai framing biner ala envelope |
| **Manifest/template hub** | JSON (dokumen, bukan digest rantai) | tidak masuk rantai hash |

**Alasan mengikat** (ditambahkan @matt di Ruling 2, saya dukung penuh): kanonikalisasi JSON membuat
digest bergantung pada keluaran persis serializer (format float, escaping unicode, urutan kunci
nested). Digest berubah saat versi crate berubah **tanpa satu byte data pun berubah**. Untuk rantai
hash yang di-anchor ke TSA eksternal, cacat itu baru muncul bertahun-tahun kemudian dan **tidak bisa
diperbaiki retroaktif** — anchor lama tidak akan pernah cocok lagi. Framing biner menutupnya secara
struktural.

## 4. Daftar tipe TERLARANG (jangan pernah muncul di kamus atau kode)

`PayloadRef` · `SpillId` · `SpillRef` · `uuid::Uuid` · `Bytes` sebagai tipe · `HashMap` untuk field
yang ikut di-hash.

Pengecualian yang harus dinyatakan eksplisit: sebuah crate **boleh** punya tipe lokal bernama sama
hanya bila (i) tidak diklaim sebagai tipe kernel, (ii) dipetakan ke tipe kanonik di dokumen
integrasi, **dan (iii) — per N2 @agent1 `#1043` — dokumen pemetaan itu BERPEMILIK NAMA dan BERKAS
BERNAMA, terbit SEBELUM merge Batch 1.** Tanpa (iii) pengecualian ini jadi lubang: siapa pun bisa
menyebut "tipe lokal" untuk menghindari kanon.

Kasus nyata yang menunggu (iii): `pub enum PayloadRef` di `w2-determ-enforce/src/lib.rs:78`
(dipakai `src/verify.rs:97-101`, `tests/gates.rs:103,108`). Usul penunjukan: pemilik = agent10
(implementor crate), berkas = `docs/MAPPING-PAYLOADREF-determ-ke-ItemList.md`, isi = tabel
`PayloadRef::Inline(Vec<u8>) → ItemList::Inline(Vec<Item>)` dan `PayloadRef::Spilled(SpillId) →
ItemList::Spilled(SpilledList)`, plus keputusan apakah rename atau wrapper. Lihat §5.3.

## 5. Temuan ikutan dari grep seluruh workspace (F-1(c), Ruling 6(c))

### 5.1 SUDAH DIPERBAIKI — `w2-determ-enforce`
- F-1(a) tipe nyata di komentar: dikerjakan agent10 sesi B (`lib.rs:66-74`).
- F-1(b) hapus kutipan `(BAHASA-BERSAMA §2)` sebagai otoritas: dikerjakan agent10 sesi A di
  `src/lib.rs:65-66`, `:7`, `:208`. Bukti tidak ada regresi: **9/9 test PASS, clippy 0 warning**.
  sha256 `src/lib.rs` sesudah = `08a02715bdfd1b92cfcb4cd73d7010f033cc4901c62673694ec45477d6c8c519`.
- Sisa rujukan `BAHASA-BERSAMA` di `src/` = 1 baris, dan bunyinya justru peringatan bahwa §2 root
  NON-BINDING. Itu dipertahankan dengan sengaja.

### 5.2 BELUM — duplikat sumber basi di root crate
`/opt/agent-workspace/w2-determ-enforce/lib.rs` (13.029 byte, 11:23) adalah **salinan lama** dari
`src/lib.rs` (13.135 byte, 11:30), beda 3 baris, dan masih memuat teks pra-perbaikan. `Cargo.toml`
tidak punya bagian `[lib]`, jadi yang dikompilasi adalah `src/lib.rs` — duplikat ini **mati tapi
menyesatkan**: grep dan reviewer akan menemukannya lebih dulu. Rekomendasi: hapus, atau sinkronkan.
Saya **tidak** menghapusnya tanpa izin karena itu berkas agen lain (sesi B).

### 5.3 BELUM — fork kosakata `PayloadRef` di crate determ
`pub enum PayloadRef { Inline(…), Spilled(SpillId) }` **nyata ada** di `w2-determ-enforce/src/lib.rs:78`
dan dipakai di `src/verify.rs:97-101` + `tests/gates.rs:103,108`. Jadi §2 root bukan hanya salah —
ia sudah **berhasil menumbuhkan tipe paralel** di crate yang akan masuk Batch 1. Ini prasyarat merge:
samakan ke `ItemList::{Inline,Spilled}` atau tulis pemetaannya secara eksplisit di dokumen integrasi.
Saya tidak mengganti nama tipe publik itu sekarang karena ia menyentuh kode (butuh konsensus), dan
crate itu implementasinya milik sesi B.

### 5.4 PENTING untuk keputusan `#962`/`#996`/`#999` — `uuid` NYATA dipakai di `rust-engine`
`rust-engine/Cargo.toml:32` → `uuid = { version = "=1.7.0", features = ["v4","serde"] }`, dipakai
`crates/data-plane/Cargo.toml:13` dan `crates/storage/Cargo.toml:12`. Di kode:
`data-plane/src/spill.rs:30`, `storage/src/spill.rs:71,209,245` — semuanya `uuid::Uuid::new_v4()`.
Dua konsekuensi yang harus diputuskan SEBELUM port:
1. **Allowlist**: memindahkan implementasi itu apa adanya akan memasukkan `uuid` ke pohon kanonik dan
   **melanggar gate allowlist 4-dependensi** (gate yang hari ini exit 0).
2. **Determinisme & audit**: nama berkas spill dari `Uuid::new_v4()` **tidak bisa direproduksi**.
   Untuk jejak audit yang harus bisa diverifikasi ulang bertahun-tahun kemudian, nama berkas spill
   sebaiknya diturunkan dari `ContentId`/digest isi, bukan dari RNG. Ini juga memengaruhi dedup CASD:
   dua isi identik menghasilkan dua nama berkas berbeda.

## 6. Kriteria selesai (falsifiable, tanpa LLM)

```bash
K=/opt/agent-workspace/kernel-asli-d3bcff0/crates
B=docs/BAHASA-BERSAMA.md
# 1. Tipe fiktif harus 0 di kernel/src
for T in PayloadRef SpillId SpillRef 'uuid::Uuid' store_ref 'Arc<dyn SpillStore>'; do
  test "$(grep -rn "$T" "$K" --include=*.rs | grep -v /examples/ | wc -l)" -eq 0 || echo "GAGAL: $T"
done
# 2. Kamus baru harus menyebut tipe nyata
grep -c 'ItemList' "$B"      # >= 1
# 2b. (per N3 @agent1 #1043) Machine-checkable TANPA pengecualian ambigu:
#     tipe terlarang hanya boleh muncul di baris yang MEMUAT penanda larangan.
#     Penanda kanon: kata "DILARANG" pada baris yang sama. Jadi:
grep -nE 'PayloadRef|SpillId|SpillRef|uuid::Uuid|store_ref|Arc<dyn SpillStore>' "$B" | grep -vE 'DILARANG|Pengganti resmi|TIDAK punya field'
#     -> keluaran HARUS KOSONG. Penanda kanon utk sebutan-larangan/negasi: kata 'DILARANG'
#        pada baris yang sama, ATAU 'Pengganti resmi <tipe>' (banner pengganti sah), ATAU
#        'TIDAK punya field <tipe>' (negasi eksplisit). Di luar 3 penanda itu, sebutan
#        tipe terlarang = temuan.
# 3. Setiap baris tabel §2 baru punya rujukan berkas:baris yang bisa diverifikasi
grep -nE '\| .*(item|context|id|lib)\.rs:[0-9]+' "$B" | wc -l   # >= jumlah baris tabel tipe
```

## 7. Yang TIDAK diusulkan dokumen ini

- Tidak mengubah kode apa pun.
- Tidak menyatakan `rust-engine` non-otoritatif — itu Ruling @matt/@fern (`#991` butir 2, opsi (i)/(ii)).
- Tidak memutuskan `data-plane` vs `storage` sebagai rumah `FileSpillStore` (`#962`) — dokumen ini
  hanya menyajikan bukti dependensi (§5.4) agar keputusan itu diambil dengan fakta.
- Tidak menyentuh BAHASA-BERSAMA-v1.1-matt.md: sesuai Ruling 3(d) ia register temuan, bukan kamus.

---

## 8. VERIFIKASI PASCA-EKSEKUSI (11:31 UTC) — dan SATU TEMUAN BARU

@fern sudah mengeksekusi sebagian besar proposal ini. Saya verifikasi ke disk, bukan ke ringkasan:

| Obyek | Fakta terukur | Status |
|---|---|---|
| `docs/BAHASA-BERSAMA.md` | pemilik `root`, 6.470 byte, 78 baris, mtime 11:31 UTC, sha256 `2cf2c62d347a03dcaeff742c…` | diperbarui |
| baris 17 | *"Tipe fiktif PayloadRef, SpillId(uuid), SpillRef, Bytes resmi DILARANG"* + rujukan Ruling 3 `#1000` & Directive `#994` | **sesuai §4 proposal ini** |
| baris 22 | `enum ItemList { Inline(Vec<Item>), Spilled(SpilledList) }` — "41 kecocokan di kernel. Pengganti resmi PayloadRef" | **sesuai §3.1** |
| baris 24 | `struct ContentId(pub u64)` — "BUKAN UUID!" | **sesuai §2/§3.1** |
| baris 32 | Envelope = "Framing biner kanonik u32-BE prefix per field (C-02); BUKAN JSON terurut" | **RFC `#945` TEREKSEKUSI** |
| baris 40-48 | definisi `DONE-CODE` / `DONE-DOC`; DONE-CODE mewajibkan *"suite `#[test]` yang lulus 100% (**termasuk uji negatif/mutan**)"* + tinjauan independen | **field (d) usulan `#991` §4 terserap** |
| kriteria §6 | `PayloadRef`/`SpillId`/`SpillRef`/`uuid::Uuid` di kernel/src = **0, 0, 0, 0**; `ItemList` di kamus = 1 | **LULUS** |

### 8.1 TEMUAN BARU (P1): baris 23 menyatakan field yang TIDAK ADA

Baris 23 kamus yang sudah dikoreksi itu berbunyi:

> | **SpilledList** | Penanda data item yang tersimpan di disk storage saat melewati ambang RAM. | `struct SpilledList` | Mengandung `store_ref: Arc<dyn SpillStore>` dan `content_id: ContentId`. |

Kenyataan di `kernel-asli-d3bcff0/crates/kernel/src/item.rs:212-219`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpilledList {
    pub path: SpillPath,
    /// Number of items. Kept here so `len()` never touches the store.
    pub len: u32,
    /// Total serialized size, for Governor accounting (D72) and metrics (D64).
    pub total_bytes: u64,
    pub codec: SpillCodec,
}
```

Bukti tambahan (semua grep, seluruh `crates/`):
- `store_ref` → **0 kecocokan**. Field itu tidak ada di kernel mana pun.
- `content_id` di dalam `SpilledList` → **0 kecocokan**.
- `dyn SpillStore` → ada, tetapi **hanya sebagai parameter fungsi** (`item.rs:303, 319, 337, 561`),
  bukan sebagai field struct. `Arc<dyn SpillStore>` sebagai field: tidak ada.

**Ini penting justru karena ia lahir dari koreksi yang tujuannya menghapus tipe fiktif.** Kamus yang
baru saja diperbaiki kini memuat satu klaim fiktif baru, di baris yang persis menyebut tipe nyata.
Dampaknya bukan kosmetis: `SpilledList` dikonsumsi DDL @agent2, codegen @agent9, manifest @agent6,
dan spec Timeline @agent1. Siapa pun yang mengimplementasikan `SpilledList { store_ref, content_id }`
akan gagal kompilasi — atau lebih buruk, "memperbaiki" kernel agar cocok dengan kamus, yang akan
merusak 19 test + gate yang hari ini hijau.

**Baris pengganti siap-tempel** (mohon @fern eksekusi; berkas milik `root`, saya tidak menimpanya):

```
| **SpilledList** | Metadata daftar item yang di-spill ke disk saat melewati ambang RAM. | `struct SpilledList { path: SpillPath, len: u32, total_bytes: u64, codec: SpillCodec }` | item.rs:212-219. TIDAK punya field `store_ref`/`content_id`; store diakses sebagai parameter `&dyn SpillStore` (item.rs:303,319). |
```

### 8.2 Catatan kecil, non-blocking

- Baris 21 (`Item`): "Membungkus `serde_json::Value`" benar tetapi tidak lengkap — `Item` juga punya
  field `binary` (n8n `binary`, keputusan D105, `item.rs:61-62`). Usul tambahan: *"Field: `json:
  serde_json::Value` + `binary: Option<…>` (D105)"*.
- Baris 25 (`SpillStore`): "Konstruktor wajib umask 0600" adalah **kebijakan implementasi**, bukan
  sifat trait. Trait-nya sendiri (`item.rs`, re-ekspor `lib.rs:89`) tidak menetapkan umask. Usul:
  pindahkan klausul umask ke baris implementasi (`FileSpillStore`) agar kamus tidak menyatakan
  kewajiban yang tidak bisa ditegakkan oleh tipe.
