# W0-Spill-TEST — Rencana Uji `FileSpillStore`, Diturunkan dari Sumber

**Versi:** 1.2.0
**Tanggal:** 2026-09-09
**Penyusun:** matt (Lead Architect)
**Untuk:** agent3 + agent5 (pemilik `W0-Spill-TEST`), koordinasi agent2 (`W0-Spill-IMPL`)
**Rujukan:** PRD-3 v3.4 §4 Wave 0, §7.3–7.5 · `SINTESIS-SPILLSTORE.md`
**Metode:** seluruh temuan di bawah diturunkan dari **membaca sumber kernel lokal**
(`rust-n8n-core` commit `db20e72`), bukan dari VPS dan bukan dari ingatan.
Baris disebut supaya tiap klaim bisa diperiksa ulang.

---

## 0. Kenapa dokumen ini ada

`FileSpillStore` adalah satu-satunya implementasi `SpillStore` yang menyentuh
disk, dan ia menghasilkan angka yang menjadi dasar klaim produk: **38,1 MB index /
50,4 MB peak RSS pada 5 juta item** (`BENCH-A03.md:104-105`).

Terverifikasi per-test: dari **19** `#[test]` di `crates/kernel/tests/contract.rs`,
**12** memakai `MemSpillStore`, **7** tidak menyentuh store sama sekali, dan
**0 — nol — menyentuh `FileSpillStore`**. Alasannya struktural: ia berada di
`crates/kernel/examples/spill_bench.rs`, dan `cargo test` tidak menjalankan
examples (0 `#[test]` di file itu; tidak ada deklarasi `[[example]]` di
`Cargo.toml`).

Jadi angka 50,4 MB itu berasal dari kode yang **tidak pernah diuji**. `W0-Spill-IMPL`
memindahkannya ke `crates/data-plane/src/` — dan begitu ia jadi target `cargo test`,
test yang ditulis melawan semantik `MemSpillStore` akan bertemu implementasi yang
**tidak sepenuhnya sama perilakunya**. Dokumen ini memetakan selisihnya lebih dulu,
supaya test-nya ditulis untuk menangkap perbedaan, bukan untuk lolos.

---

## 1. Yang sudah BENAR — jangan ditulis ulang

Tiga hal yang saya curigai cacat ternyata sudah benar. Dicatat supaya tidak ada
yang "memperbaiki" yang tidak rusak:

| Kontrak | Status | Bukti |
|---|---|---|
| `delete()` benar-benar menghapus file dari disk | **BENAR** | `spill_bench.rs:218` `std::fs::remove_file(p)` |
| Writer yang di-drop melepas file parsial (A-21) | **BENAR** | `impl Drop for FileSpillWriter` di `:313`, `remove_file` di `:316`, dijaga `if !self.finished` |
| Iterator kosong tidak membocorkan file | **BENAR** | `item.rs` `spill_from_iter`: `item.rs:580-583` — `if handle.len == 0 { store.delete(&handle).await?; return Ok(ItemList::empty()) }` |

Butir ketiga layak disebut karena jalurnya tidak kasatmata: `spill_from_iter`
memanggil `store.writer()` **lebih dulu dan tanpa syarat**, dan `writer()` membuat
file di disk lewat `open_rw()` (`:230`; definisi `open_rw` di `:110`). Untuk iterator kosong, `finish()` menandai
`finished = true` sehingga `Drop` **tidak** membersihkannya. Yang menyelamatkannya
adalah pemeriksaan `handle.len == 0` di `spill_from_iter` — **bukan** `Drop`.

**Konsekuensi untuk test:** kalau `W0-Spill-IMPL` memindahkan `FileSpillStore`
tanpa memindahkan logika `spill_from_iter` yang sama, atau kalau ada jalur lain
yang memanggil `writer()` lalu `finish()` tanpa pemeriksaan itu, kebocoran file
kosong muncul kembali. Uji `FS-07` di bawah menutup ini.

Juga terkonfirmasi identik: `disk_footprint()` mengembalikan `handle.total_bytes`
di **kedua** implementasi. Bukan sumber divergensi.

---

## 1.5 Matriks audit lengkap — 7 metode trait + 3 metode writer

Audit ini selesai, bukan sampel. Tiap sel dibandingkan dari sumber, bukan dari
nama fungsi. "Sama" berarti semantik sama, bukan hanya tanda tangan sama.

| Metode | `MemSpillStore` (tests) | `FileSpillStore` (examples) | Verdict |
|---|---|---|---|
| `write` | implementasi sendiri, `total += bytes.len()` (`:32-58`) | delegasi ke `writer()` (`:122-128`) | **sama** — keduanya mengecualikan prefiks 8-byte dari `total_bytes` |
| `read_at` | `Err(IndexOutOfBounds)` (`:67`) | `Err(IndexOutOfBounds)` | **sama** |
| `read_range` | delegasi per-indeks → **Err** saat OOB (`:77-88`) | clamp `min(n)` → **Ok terpotong** (`:173,176`) | **DIVERGEN** — §2 |
| `read_all` | `read_range(0, handle.len)` | `read_range(0, handle.len)` | sama *dalam praktik* — `handle.len` selalu tepat, jadi tidak pernah memicu §2 |
| `delete` | hapus entri peta | hapus entri peta **+ `remove_file`** (`:218`) | **sama, benar untuk mediumnya** |
| `disk_footprint` | `handle.total_bytes` | `handle.total_bytes` | **sama antar-impl**, tapi keduanya menyimpang dari trait doc — §3 |
| `writer` | `next_id` → `spill/{n}` (`:104`) | `alloc_name()` → `spill-{n}.bin` (`:72-76`) | **sama** — pencacah dibagi & monoton di kedua jalur; format path beda tapi konsisten internal |
| `push` | append ke `blobs` | tulis `len:u64 LE` + payload, `pos += 8+len`, `total += len` | **sama** secara semantik |
| `finish` | insert ke peta, `finished = true` | flush + `sync_all` + insert, `finished = true` | **sama** |
| `pushed` | `blobs.len()` | `offsets.len()` | **sama** |
| `Drop` | hapus entri peta bila `!finished` (`:161`) | `remove_file` bila `!finished` (`:313`) | **sama** — kontrak A-21 dipenuhi keduanya |

Dugaan yang **tidak terbukti**, dicatat supaya tidak ada yang mengejarnya lagi:

- *Tabrakan path antara `write()` dan `writer()`* — **TIDAK**. `MemSpillStore`
  memakai `next_id` yang sama di kedua jalur (`:33` dan `:104`); `FileSpillStore`
  melewatkan keduanya melalui `alloc_name()`. Pencacah monoton, tidak ada reuse.
- *`total_bytes` berbeda 8×len antar implementasi* — **TIDAK**. `push()`
  melakukan `self.total += len` dengan `len = bytes.len()`, prefiks tidak
  termasuk; `MemWriter.finish()` menjumlah `b.len()` yang juga tanpa prefiks.
  Keduanya konsisten. (Konsekuensi tak terduga dari ini jadi §3.)
- *Guard `finish() called twice` adalah divergensi* — **TIDAK**. `finish(self:
  Box<Self>)` mengkonsumsi writer, jadi `push` setelah `finish` tidak mungkin
  dipanggil; trait doc menyebutnya "impossible by construction". Guard di
  `FileSpillWriter` adalah kode mati, bukan selisih perilaku.
- *Iterator kosong membocorkan file* — **TIDAK**, lihat §1 butir 3.

Jadi dari 10 permukaan yang dibandingkan, **satu divergensi perilaku nyata**
(§2), **satu inkonsistensi internal** (§3), **satu penyimpangan dari trait doc**
(§3.5). Sisanya cocok.

---

## 2. Divergensi semantik `read_range` — nyata, tapi LATEN

Ini temuan utama. Dua implementasi trait yang sama berperilaku **berbeda** pada
masukan yang sama.

**`MemSpillStore`** (`tests/contract.rs:77-88`) mendelegasikan per-indeks:

```rust
for i in start..start + len {
    out.push(self.read_at(handle, i).await?);   // read_at -> Err(IndexOutOfBounds)
}
```
`read_at` memberi `Err(KernelError::IndexOutOfBounds { index, len })` di `:67`.
Jadi **`read_range` yang melampaui ujung = Err.**

**`FileSpillStore`** (`examples/spill_bench.rs:173-176`) meng-clamp:

```rust
if start >= n || len == 0 { return Ok(Vec::new()); }
let end = (start + len).min(n);
```
Jadi **`read_range` yang melampaui ujung = Ok, dipotong diam-diam.**

| Masukan pada list 300 item | `MemSpillStore` | `FileSpillStore` |
|---|---|---|
| `read_range(h, 30, 50)` | `Ok(items[30..80])` | `Ok(items[30..80])` — **sama** |
| `read_range(h, 290, 50)` | **`Err(IndexOutOfBounds{index:300,len:300})`** | **`Ok(10 item)`** |
| `read_range(h, 500, 10)` | **`Err(IndexOutOfBounds)`** | **`Ok(vec![])`** |
| `read_at(h, 300)` | `Err(IndexOutOfBounds)` | `Err(IndexOutOfBounds)` — **sama** |

### Kenapa tidak ada test yang menangkapnya

Satu-satunya test yang menyentuh `read_range` adalah
`writer_output_readable_through_store_api` (`:781`):

```rust
let r = block_on(store.read_range(&handle, 30, 50)).unwrap();   // 30+50 = 80 <= 300
assert_eq!(r, items[30..80]);
```

**Selalu dalam batas.** Test itu punya pemeriksaan out-of-bounds, tapi hanya untuk
`read_at` (`:788` `assert!(store.read_at(&handle, 300)).is_err()`) — tidak untuk
`read_range`. Jadi celahnya persis di tempat divergensinya berada.

### Tingkat keparahan: LATEN, bukan aktif

Saya hampir menuliskan ini sebagai bug aktif. Ia bukan. Satu-satunya pemanggil
internal, `next_chunk` (`item.rs:444-461`), meng-clamp sendiri **sebelum** memanggil:

```rust
let end = (self.pos + self.chunk_size).min(total);
...
    .read_range(s, self.pos, end - self.pos)
```

Jadi jalur cursor — yang dipakai benchmark dan yang akan dipakai node produksi —
tidak pernah mengirim argumen melampaui ujung. Divergensi hanya menggigit
**pemanggil langsung dari luar kernel**.

Itu tetap harus ditutup, karena dua alasan:

1. `W0-Spill-IMPL` menjadikan `FileSpillStore` target `cargo test`. Test yang
   ditulis dengan asumsi semantik `MemSpillStore` (Err) akan gagal — atau lebih
   buruk, seseorang "memperbaiki" test-nya agar cocok dengan clamp, dan
   menyemenkan perilaku yang berbeda tanpa memutuskan mana yang benar.
2. Ini **keputusan kontrak**, bukan preferensi. Trait doc di `item.rs:489` hanya
   menulis *"Read `len` items starting at `start`"* — tidak menyatakan perilaku
   out-of-bounds. Jadi kedua implementasi sama-sama "mematuhi" trait, dan itu
   persis masalahnya: kontraknya kurang spesifik.

**Rekomendasi:** pilih **Err**, lalu perbaiki `FileSpillStore`. Alasannya:
(a) `read_at` sudah Err, dan `read_range` yang Err konsisten dengannya;
(b) `MemSpillStore` adalah semantik yang sudah diuji dan yang diasumsikan 12 test;
(c) clamp diam-diam mengubah bug pemanggil menjadi data yang hilang tanpa suara —
di engine workflow, kehilangan item senyap adalah kelas cacat yang paling mahal.
`next_chunk` tetap aman karena ia sudah clamp sendiri sebelum memanggil.

Kalau tim memutuskan sebaliknya (clamp), itu sah — tapi **wajib** didaftar di
`DEVIATION-CATALOG.md` dan trait doc `item.rs:489` harus diperjelas. Yang tidak
boleh adalah membiarkan keduanya berbeda tanpa keputusan.

---

## 3. Inkonsistensi internal `FileSpillStore.read_range`

Terpisah dari divergensi di atas, dan ini **murni di dalam `FileSpillStore`**:
dua jalur truncation diperlakukan berbeda dalam satu loop yang sama
(`spill_bench.rs:192-205`):

```rust
for _ in start..end {
    if cur + 8 > raw.len() {
        break;                          // <- DIAM: keluar, kembalikan apa adanya
    }
    let l = u64::from_le_bytes(...);
    cur += 8;
    if cur + l > raw.len() {
        return Err(KernelError::Codec { // <- BERSUARA: error "truncated record"
            codec: "json", reason: "truncated record".into() });
    }
    ...
}
```

Header panjang 8-byte yang terpotong → `break` senyap dan mengembalikan `Ok`
dengan item lebih sedikit. Badan record yang terpotong → `Err(Codec)`.

Kedua kondisi itu berarti hal yang sama: **berkas rusak atau `span` salah hitung**.
Memperlakukan satunya sebagai sukses parsial adalah jalur kehilangan data senyap.
Catatan: `break` ini seharusnya **tidak pernah tercapai** kalau `span` dihitung
benar, jadi ia berfungsi sebagai penutup lubang — tapi penutup lubang yang
mengembalikan `Ok` akan menyembunyikan bug penghitungan `span` alih-alih
menyatakannya.

**Rekomendasi:** jadikan keduanya `Err(Codec { reason: "truncated header" })`.

---

## 3.5 `disk_footprint()` menyimpang dari dokumentasinya sendiri

Trait doc `item.rs:506-507`:

```rust
/// Bytes this handle occupies on disk.
fn disk_footprint(&self, handle: &SpilledList) -> u64;
```

Kedua implementasi mengembalikan `handle.total_bytes`, yang **mengecualikan**
prefiks panjang 8-byte per item. Berkas sebenarnya berukuran `total_bytes + 8×len`.

Pada 5 juta item:

```
prefiks total : 8 × 5.000.000 = 40.000.000 byte = 38,15 MiB
disk dilaporkan BENCH-A03    : 752,9 MB
under-report                 : 5,31%
```

**Severity: RENDAH, dan saya ingin presisi soal itu.** Tiga alasan:

1. `disk_footprint()` **tidak punya pemanggil produksi**. `grep -rn` di `src/`,
   `tests/`, `examples/` hanya menemukan deklarasi trait dan dua implementasi —
   tidak ada yang memanggilnya. Jadi tidak ada keputusan yang sedang dibuat
   berdasarkan angka ini hari ini.
2. Ini **bukan divergensi antar-implementasi**. Keduanya mengembalikan hal yang
   sama, jadi tidak ada test paritas yang akan gagal. Yang menyimpang adalah
   keduanya terhadap *dokumentasi*.
3. Untuk `MemSpillStore` pertanyaannya bahkan tidak bermakna — tidak ada disk.

Tetapi tetap harus dibereskan sebelum `W0-Spill-IMPL`, karena begitu
`gc_execution()` masuk (FS-09) dan Governor mulai membuat keputusan penghapusan
berdasarkan jejak disk, angka yang kurang 5,31% jadi input kebijakan. Pada VPS
dengan 2,3 GB bebas, 5% dari 753 MB adalah ~38 MB — kecil, tapi ini kelas angka
yang dipakai untuk memutuskan "masih muat atau tidak".

**Dua perbaikan yang sah, pilih satu:**

- (a) **Perbaiki implementasi**: `FileSpillStore::disk_footprint` mengembalikan
  `handle.total_bytes + 8 × handle.len`. `MemSpillStore` tetap apa adanya (tidak
  ada disk), dan itu didaftarkan sebagai deviasi yang disengaja.
- (b) **Perbaiki dokumentasi**: trait doc jadi *"Bytes of serialized item payload
  for this handle, excluding framing"* dan nama dipertimbangkan ulang.

Rekomendasi saya **(a)**, karena nama `disk_footprint` dan doc-nya sudah
menjanjikan okupansi disk, dan pemanggil masa depan (GC, Governor, kuota) akan
membacanya sesuai janji itu. Tapi (a) membuat `MemSpillStore` dan
`FileSpillStore` **sengaja berbeda**, jadi wajib masuk `DEVIATION-CATALOG.md`.

Perhatikan interaksi dengan §2: kalau tim memilih "samakan semua semantik"
sebagai prinsip, §3.5(a) melanggar prinsip itu secara sadar. Itu tidak apa-apa —
tapi harus diputuskan, bukan terjadi karena tidak diperhatikan.

---

## 4. Rencana uji

`[W]` = wajib sebelum `W0-Spill-IMPL` dinyatakan selesai.
Semua test ini harus dijalankan **terhadap `FileSpillStore`**, dengan direktori
sementara nyata. 12 test yang ada tetap terhadap `MemSpillStore` — jangan
dipindah, karena nilainya justru membandingkan keduanya.

| ID | Uji | Menangkap |
|---|---|---|
| **FS-01** `[W]` | Izin file spill = `0600` **dengan `umask 000`** | `SINTESIS-SPILLSTORE.md:170-179`: `OpenOptions` tanpa `.mode()` menghasilkan `0666 & ~umask` = **0644** pada umask default, **0666** pada umask 000. Tanpa `umask 000` test-nya menguji umask, bukan konstruktor |
| **FS-02** `[W]` | Korupsi 1 byte pada berkas → `Err`, bukan `Ok` dengan data salah | Integritas spill. Ini yang membedakan checksum nyata dari `wrapping_add` benchmark |
| **FS-03** `[W]` | `read_range` melampaui ujung → perilaku sesuai keputusan §2 | Divergensi §2. Test ini **gagal** sampai keputusannya diambil |
| **FS-04** `[W]` | Header 8-byte terpotong → `Err(Codec)`, bukan `Ok` pendek | Inkonsistensi §3 |
| **FS-05** `[W]` | 12 test `MemSpillStore` yang ada, dijalankan ulang terhadap `FileSpillStore` dengan hasil identik | Paritas semantik. `spill_from_iter_*`, `writer_*`, `binary_location_ram_accounting` |
| **FS-06** `[W]` | `delete()` menghapus **file di disk**, bukan hanya entri peta | Sudah benar di `:218`, tapi belum pernah diuji. Regresi di sini = kebocoran disk tanpa batas |
| **FS-07** | `spill_from_iter` dengan iterator kosong → tidak ada file tersisa di direktori | §1 butir 3: dijaga oleh `spill_from_iter`, bukan `Drop`. Rentan regresi saat dipindah |
| **FS-08** | Writer di-drop tanpa `finish()` → tidak ada file parsial | Kontrak A-21, `impl Drop` di `:313`. Sudah benar, belum pernah diuji |
| **FS-09** | `gc_execution()` — tidak ada handle non-terminal (RUNNING/WAITING/PAUSED) yang file-nya terhapus | `item.rs:502-503` menyebut audit A-21 eksplisit. `gc_execution` **belum ada** di kernel-asli; ia diserap dari data-plane |
| **FS-10** | SHA-256 di `SpilledList` cocok dengan isi berkas yang dibaca kembali | Belum ada di kernel-asli; diserap dari data-plane. Wajib konsisten dengan `W0-CHECKSUM-AGREE` |
| **FS-11** | `disk_footprint()` == ukuran berkas aktual di disk (`std::fs::metadata().len()`) | §3.5. Test ini **gagal** sampai keputusannya diambil — dan itulah gunanya: ia memaksa keputusan, bukan menyemenkan keadaan sekarang |
| **FS-12** | `write()` lalu `writer()` berurutan tidak menghasilkan path yang sama | Menutup dugaan tabrakan path yang **tidak terbukti** (§1.5). Murah, dan menjaganya tetap tidak terbukti setelah refactor `W0-Spill-IMPL` |

**FS-09 dan FS-10 tidak bisa ditulis lebih dulu** — keduanya menguji fitur yang
belum ada di kernel-asli dan baru masuk lewat `W0-Spill-IMPL`. Urutannya jadi:
FS-01…FS-08 bisa disiapkan sekarang, FS-09/FS-10 menyusul.

---

## 5. Jebakan pengukuran

**`umask` adalah keadaan proses, bukan parameter test.** FS-01 hanya berarti kalau
dijalankan dengan `umask 000`. Kalau dijalankan lewat `cargo test` biasa dengan
umask warisan (biasanya 022), berkas akan lahir 0644 dan test **lulus palsu** bila
yang diassert adalah "bukan 0666". Set umask di dalam test lewat
`std::os::unix::fs::PermissionsExt` + assert eksplisit `0o600`, atau jalankan
harness dengan `umask 000`. `SINTESIS-SPILLSTORE.md:178-179` sudah menyatakan ini;
saya ulangi karena inilah cara paling mungkin test-nya jadi hijau tanpa artinya.

**Direktori sementara harus unik per-test, bukan per-run.** Ini pola yang sama
dengan bug `check-freeze.sh` yang menulis log ke path hardcoded `/tmp/freeze-*.log`:
di mesin 12-akun, berkas itu dimiliki yang menjalankan lebih dulu dan user
berikutnya gagal menimpa. Gunakan `tempfile::TempDir` atau setara, jangan
`/tmp/spill-test` tetap.

**`CARGO_TARGET_DIR` per-akun.** `/mnt/extra-storage/cargo-target-$(id -un)`.
Target dir bersama dipakai 11 agen dan cargo menguncinya, jadi test paralel akan
saling blok dan hasilnya sulit direproduksi.

---

## 6. Definisi selesai

| Kriteria | Verifikasi |
|---|---|
| FS-01…FS-08, FS-11, FS-12 hijau terhadap `FileSpillStore` | `cargo test -p data-plane` |
| Keputusan §2 diambil dan diterapkan konsisten | `read_range` OOB berperilaku sama di kedua implementasi |
| Keputusan §3.5 diambil | `disk_footprint` sesuai doc, atau doc diperbaiki + deviasi terdaftar |
| §3 diperbaiki | tidak ada `break` senyap pada header terpotong |
| Tidak ada regresi kernel | 19 test tetap hijau, clippy 0 warning |
| Izin teruji pada `umask 000` | FS-01 dijalankan dengan umask 000, bukan warisan |
| Divergensi yang disengaja terdokumentasi | kalau §2 diputuskan clamp → masuk `DEVIATION-CATALOG.md` + trait doc `item.rs:489` diperjelas |

**Bukan bagian W0-Spill-TEST:** implementasi `gc_execution`/SHA-256 (itu
`W0-Spill-IMPL`, agent2), kontrak checksum testkit↔data-plane (itu
`W0-CHECKSUM-AGREE`), dan anchor eksternal (itu `W0-ANCHOR-SPEC`, agent10).
