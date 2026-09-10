# W0-CHECKSUM-AGREE — Spesifikasi Remediasi Kontrak Checksum

**Versi:** 1.0.0
**Tanggal:** 2026-09-09
**Penyusun:** matt (Lead Architect)
**Untuk:** agent2 (data-plane) + agent3 (testkit), koordinasi bersama
**Gate yang ditutup:** G-C5, G-C7 · **Cacat:** C-05 · **Rujukan:** PRD-3 v3.2 §5 L2c, §7.2
**Status:** siap dikerjakan — tidak butuh keputusan Pemilik Proyek

---

## 1. Masalahnya, dalam angka yang sudah dieksekusi

Bukan hasil pembacaan kode. Kedua implementasi saya jalankan pada blob yang sama:

```
testkit    fn blake3_hash() : 8c702834b42e9a48                                        (16 char)
data-plane SHA-256 (:44)    : 2bde382e929946a83cdaf5d764574cb21264c8b3f66a609044aa9e8a27ccc3e2  (64 char)
identik?                    : FALSE
```

Selisihnya **struktural**, bukan kebetulan satu input:

| | Sumber | Lebar keluaran |
|---|---|---|
| testkit | `format!("{:016x}", hasher.finish())` atas `u64` | 8 byte = **16 hex, selamanya** |
| data-plane | SHA-256 | 32 byte = **64 hex, selamanya** |

Tidak ada input apa pun yang membuat keduanya sama. Jadi **G-C7 mustahil lulus** dengan kode sekarang — bukan "susah", melainkan tidak ada jalur ke hijau.

Kode yang menyebabkan (`testkit/src/lib.rs`, terkonfirmasi di sumber):

```rust
:70   let checksum = blake3_hash(data);            // dipanggil di jalur write()
:567  fn blake3_hash(data: &[u8]) -> String {
:568    // Simple hash for testing. In production, use blake3 crate.
:570    use std::collections::hash_map::DefaultHasher;
:574    format!("{:016x}", hasher.finish())
```

`Cargo.toml` testkit **tidak punya** dependency `blake3`. Komentarnya jujur; **namanya tidak** — dan nama itulah yang dibaca gate. G-C5 (`hash_name_matches_impl`) ada persis untuk menangkap ini, dan hasilnya **GAGAL**.

Catatan keamanan yang membuat ini bukan sekadar kosmetik: `DefaultHasher` adalah SipHash-1-3 dengan **kunci tetap** — bukan rahasia, bukan acak per-proses. Collision bisa dicari. Dipasang di jalur bernama "checksum integritas", artinya integritas itu **tidak ada**.

---

## 2. Keputusan yang harus diambil: dua bidang, dua algoritma

Ini bagian yang paling sering salah dibaca, dan saya sendiri hampir salah menuduhnya sebagai kontradiksi di PRD-3. **BLAKE3 vs SHA-256 bukan pertentangan** — keduanya benar, di bidang berbeda:

| Bidang | Pemilik | Algoritma | Kenapa |
|---|---|---|---|
| **Envelope / rantai audit** | agent10 | **BLAKE3** | butuh kecepatan + tree hashing untuk Merkle; dihitung sekali per eksekusi |
| **Checksum spill / blob** | agent2 | **SHA-256** | dihitung per-chunk, sangat sering; jejak memori 17× lebih kecil |

**G-C7 hanya menguji bidang kedua.** Testkit dan data-plane harus sama di bidang spill. Domain envelope punya gate sendiri di L2c dan tidak tersentuh remediasi ini.

### Kenapa SHA-256 untuk spill, bukan BLAKE3

Alasannya anggaran transien, dan ini sudah terukur oleh agent10 (#646):

| Hasher | `size_of` terukur | % dari budget 4 KB |
|---|---:|---:|
| `blake3::Hasher` | **1.920 B** | **47%** |
| `sha2::Sha256` | **112 B** | **2,7%** |

Aturan agent5 R1/R2 mengikat: **satu hasher hidup per jalur**, sequential. Dengan itu:

```
1 hasher hidup  =  47% budget   (BLAKE3)   -> aman, tipis
2 hasher hidup  =  94% budget              -> praktis melewati 100% setelah buffer entri
3 hasher hidup  = 141% budget              -> MELANGGAR
```

Buffer entri kanonik (~256 B + `len(node_id)`) **belum termasuk** di angka itu. Jadi memilih BLAKE3 untuk jalur spill yang dipanggil per-chunk berarti menghabiskan hampir setengah anggaran pada struktur yang hidupnya paling sering. SHA-256 di 112 B menyisakan ruang untuk hasher envelope hidup bersamaan tanpa melanggar.

Kecepatan BLAKE3 nyata, tapi tidak relevan di sini: spill dibatasi I/O disk, bukan throughput hash. Mengorbankan 17× memori untuk kecepatan yang tidak menjadi bottleneck adalah trade yang salah.

---

## 3. Patch yang harus diterapkan

### 3.1 agent3 — testkit (menutup C-05 + G-C5)

Tiga perubahan, semuanya di `crates/testkit/`:

**(a) Ganti implementasi, dan ganti namanya sekaligus.** Nama `blake3_hash` untuk fungsi SHA-256 akan mengulang penyakit yang sama dalam bentuk baru — G-C5 akan gagal lagi, kali ini dengan arah terbalik.

```rust
// HAPUS: fn blake3_hash() berbasis DefaultHasher (baris 567-575)

/// Checksum kanonik untuk bidang SPILL. Wajib identik dengan data-plane.
/// Bidang envelope/audit memakai BLAKE3 - lihat blake3_digest() di bawah.
pub fn spill_checksum(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    format!("{:064x}", h.finalize())   // 32 byte -> 64 hex, eksplisit
}

/// Checksum kanonik untuk bidang ENVELOPE. Terpisah, jangan ditukar.
pub fn blake3_digest(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}
```

`{:064x}` ditulis eksplisit, bukan `{}` — supaya lebar keluaran menjadi bagian dari kontrak yang terlihat, bukan sifat kebetulan dari tipe. Lebar adalah hal yang membuat G-C7 mustahil; jangan biarkan ia implisit lagi.

**(b) Tambahkan dependency sungguhan.** Ini yang membuat G-C5 berarti:

```toml
[dependencies]
sha2   = "0.10"
blake3 = "1"
```

Tanpa `blake3` di `Cargo.toml`, fungsi bernama BLAKE3 tidak bisa ada — dan itu justru penjaga yang kita mau.

**(c) Perbarui pemanggil di baris 70** dari `blake3_hash(data)` ke `spill_checksum(data)`.

**Verifikasi yang wajib dijalankan agent3 sendiri, sebelum lapor DONE:**

```
cargo tree -p <testkit> | grep -E 'blake3|sha2'     # keduanya HARUS muncul
grep -rn "DefaultHasher" crates/testkit/src/        # HARUS kosong
```

`grep DefaultHasher` kosong adalah uji yang membedakan "sudah diganti" dari "sudah diganti namanya saja". Penyakit aslinya adalah nama yang berbohong; jangan sampai perbaikannya jadi nama baru yang berbohong lagi.

### 3.2 agent2 — data-plane (menutup G-C7)

Data-plane sudah SHA-256, jadi **tidak perlu ganti algoritma**. Yang perlu:

**(a) Ekspor fungsinya sebagai kontrak publik**, bukan detail internal. Testkit harus memanggil sumber yang sama, bukan menyalin implementasi:

```rust
// crates/data-plane/src/checksum.rs
pub const CHECKSUM_ALGO: &str = "sha2-256";
pub const CHECKSUM_HEX_LEN: usize = 64;

pub fn spill_checksum(data: &[u8]) -> String { /* implementasi yang ada sekarang */ }
```

Kalau testkit punya salinan sendiri, keduanya bisa menyimpang lagi tanpa terdeteksi — persis cara C-05 terjadi. **Satu sumber, dua pemakai.**

**(b) Tambahkan uji yang mengunci lebarnya:**

```rust
#[test]
fn checksum_width_is_contractual() {
    let c = spill_checksum(b"x");
    assert_eq!(c.len(), CHECKSUM_HEX_LEN, "lebar checksum adalah kontrak, bukan kebetulan");
    assert!(c.chars().all(|ch| ch.is_ascii_hexdigit()));
}
```

### 3.3 Uji gabungan — ini gate G-C7 yang sebenarnya

```rust
#[test]
fn g_c7_testkit_matches_dataplane() {
    let blob = b"node_output:payload:uji";
    assert_eq!(
        testkit::spill_checksum(blob),
        data_plane::checksum::spill_checksum(blob),
        "G-C7: kontrak checksum testkit dan data-plane harus identik"
    );
}
```

Uji ini **tidak bisa ditulis sebelum 3.1 dan 3.2 selesai**, dan itu sebabnya urutan agent2 di #653 benar: `CHECKSUM-AGREE` lebih dulu, lalu `Spill-IMPL`, lalu `Spill-TEST`, baru `W2-STORAGE-L0`.

---

## 4. Jebakan yang harus dihindari

**Jangan samakan bidang envelope ke SHA-256 demi "konsistensi".** Godaannya nyata: satu algoritma untuk semua terdengar rapi. Tapi agent10 sudah menetapkan BLAKE3 untuk Merkle/rantai audit, dan mengganti itu berarti menulis ulang `AGENT10-EXEC-ENVELOPE-SPEC.md` serta membatalkan vektor KAT yang sudah lolos (G-C6 LULOS). Dua bidang, dua algoritma, **dan tiap fungsi dinamai menurut bidangnya** — `spill_checksum` vs `blake3_digest`, bukan dua-duanya `hash`.

**Jangan pakai `DefaultHasher` di mana pun sebagai checksum.** Ia sah untuk HashMap internal. Ia tidak sah untuk integritas.

**T-11 masih terbuka dan bukan urusan remediasi ini.** T-11 bertanya algoritma apa untuk checksum `node_output` pada umumnya, dan butuh pengukuran (throughput vs memori pada beban nyata), bukan keputusan di dokumen. W0 menetapkan kontrak **antara testkit dan data-plane** supaya keduanya berhenti berbeda. Kalau nanti T-11 memutuskan algo lain, keduanya berubah bersama — dan itu justru poin dari 3.2(a): satu sumber.

---

## 5. Definisi selesai

| Kriteria | Cara verifikasi |
|---|---|
| C-05 tertutup | `grep -rn DefaultHasher crates/testkit/src/` → kosong |
| G-C5 LULUS | `blake3` dan `sha2` keduanya ada di `cargo tree`; nama fungsi sesuai implementasi |
| G-C7 LULOS | uji 3.3 hijau, dan `spill_checksum("x").len() == 64` di **kedua** crate |
| Tidak ada regresi | 19 test kernel tetap hijau; clippy 0 warning |
| Anggaran transien | tidak ada jalur dengan >1 hasher hidup (R1/R2 agent5) |

**Yang bukan bagian W0:** `FileSpillStore` (itu `W0-Spill-IMPL`/`W0-Spill-TEST`, milik agent2+agent3+agent5), anchor eksternal (`W0-ANCHOR-SPEC`, agent10), dan T-11.
