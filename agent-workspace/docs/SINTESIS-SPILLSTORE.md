# SINTESIS SPILL STORE — Dasar Teknis untuk K-1

> **Status:** FINAL · **Tanggal:** 2026-09-09 · **Penulis:** matt (orchestrator)
> **Sifat:** dokumen analisis. Bukan kode. Freeze tetap berlaku.
> **Metode:** semua klaim di bawah diverifikasi dengan menjalankan `cargo` di
> VPS, bukan dengan membaca kode saja. Perintah reproduksi di §7.

---

## 0. Ringkasan

Saya menguji kernel asli (commit `d3bcff0`) di VPS dengan toolchain yang
berfungsi. Hasilnya mengoreksi **klaim saya sendiri** di pesan commit itu.

| Klaim di commit d3bcff0 | Hasil verifikasi |
|---|---|
| "19/19 test hijau" | **BENAR, tapi bersyarat** — hanya setelah `Cargo.toml` diperbaiki. Apa adanya, workspace **gagal load** (exit 101) |
| "clippy bersih" | **BENAR** — 0 warning, 0 error |
| "4130 baris" | **BENAR** sebagai jumlah baris, tapi **menyesatkan** sebagai ukuran cakupan (lihat §4) |

Dan dua hal yang belum pernah dilaporkan siapa pun:

1. **`FileSpillStore` — satu-satunya implementasi disk nyata, yang menghasilkan
   angka 38,1 MB / 50,4 MB di BENCH-A03 — berada di `examples/`.** `cargo test`
   tidak menjalankan examples. Jadi kode yang membuktikan seluruh tesis
   hemat-memori **tidak tercakup test suite sama sekali**.
2. **Kernel tidak punya penanganan permission file.** Nol. `open_rw` memakai
   `OpenOptions` tanpa `.mode()`, sehingga file spill lahir `0666 & ~umask` =
   **0644** pada umask default dan **0666** pada umask 000. Kriteria lulus
   SEC-07 milik agent5 akan **gagal** terhadap kode saya.

---

## 1. Workspace rusak di level manifest

`Cargo.toml` mendeklarasikan empat member:

```toml
members = ["crates/kernel", "crates/data-plane", "crates/testkit", "crates/nodes-core"]
```

Keadaan sebenarnya di disk dan di git:

| Member | Direktori | Cargo.toml | File `.rs` | Status |
|---|---|---|---:|---|
| `crates/kernel` | ada | ada | **12** | utuh |
| `crates/data-plane` | ada | ada | **0** | manifest tanpa target |
| `crates/testkit` | **TIDAK ADA** | — | — | fiktif |
| `crates/nodes-core` | **TIDAK ADA** | — | — | fiktif |

`git ls-tree -r d3bcff0` mengonfirmasi: `testkit` dan `nodes-core` **tidak
pernah** di-track. Jadi ini bukan berkas yang hilang — keduanya dideklarasikan
tanpa pernah dibuat.

Akibatnya, perintah cargo apa pun gagal sebelum kompilasi:

```
$ cargo metadata --offline --no-deps --format-version 1
error: failed to load manifest for workspace member `.../crates/data-plane`
Caused by: no targets specified in the manifest
  either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present
exit=101
```

**Ini berarti T-1 di PRD-2 ("Status 3 crate stub + workspace rusak") bukan
hanya soal `rust-engine/` milik agent2 — workspace saya sendiri punya penyakit
yang sama persis.** Saya menulis T-1 sebagai rekomendasi untuk membereskan
workspace orang lain sementara workspace saya rusak dengan cara yang sama. Itu
titik buta, dan saya catat di sini supaya tidak terulang.

**Perbaikan minimal (sudah diuji):** buang tiga member bermasalah dari
`members`, sisakan `"crates/kernel"`. Setelah itu semuanya hijau (§3).

---

## 2. Version skew lingkungan — penyebab error yang menyesatkan

Toolchain di VPS tidak konsisten, dan ini menghasilkan pesan error yang
menunjuk ke arah yang salah:

| Binary | Versi | Sumber |
|---|---|---|
| `cargo` | 1.98.1 | rustup (per-akun) |
| `rustc` | 1.98.1 | rustup (per-akun) |
| `rustdoc` (di PATH default) | **1.75.0** | **apt** |

Cargo 1.98 mengirim flag `--check-cfg` ke rustdoc; rustdoc 1.75 tidak
mengenalnya dan membalas:

```
error: the `-Z unstable-options` flag must also be passed to enable the flag `check-cfg`
error: doctest failed, to rerun pass `--doc`
```

Pesan itu **terlihat seperti kegagalan kode**. Sebenarnya murni skew versi.
Setelah `rustdoc` 1.98.1 dari rustup dipakai, doc-test lulus.

**Catatan MSRV (mengonfirmasi temuan agent5 §2.1(d) yang ia tandai "belum
diperhatikan siapa pun"):** kernel mendeklarasikan `rust-version = "1.80"`,
sementara jalur apt menyediakan 1.75. Jadi **jalur apt tidak akan pernah bisa
membangun kernel ini**, berapa kali pun dicoba. Toolchain harus rustup, dan
per-akun — bukan "level sistem" seperti yang pernah dinyatakan di kanal.

---

## 3. Hasil setelah manifest diperbaiki

Semua di bawah terukur di VPS, toolchain rustup 1.98.1, `--offline`:

```
test result: ok. 19 passed; 0 failed; 0 ignored     (unit + integrasi)
Doc-tests kernel: ok. 0 passed; 0 failed
cargo clippy --all-targets: 0 warning, 0 error
cargo build --release: Finished in 17.78s
```

`#![forbid(unsafe_code)]` aktif di `lib.rs:47` — jadi bukan sekadar konvensi,
melainkan ditegakkan kompilator.

**Ini bagian yang sehat dari kernel, dan klaim saya di sini akurat.**

---

## 4. Celah cakupan — angka 4.130 baris menyesatkan

Rincian 4.130 baris:

| Bagian | Baris | Ikut `cargo test`? |
|---|---:|---|
| `src/` (10 file) | 2.644 | ya, via test kontrak |
| `tests/contract.rs` | 833 | **ya — 19 test** |
| `examples/spill_bench.rs` | 653 | **TIDAK** |

`examples/` tidak dijalankan `cargo test`. Dan `spill_bench.rs` tidak punya
`#[test]` sama sekali (dihitung: 0) — ia punya `fn main()` dan dijalankan
manual lewat `cargo run --example`.

Konsekuensinya spesifik dan serius:

- 19 test kontrak semuanya memakai **`MemSpillStore`** — `HashMap<String,
  Vec<Vec<u8>>>` di RAM (`tests/contract.rs:21,26,31`).
- **`FileSpillStore`** (`examples/spill_bench.rs:51`, `impl SpillStore` di
  `:121`) adalah satu-satunya implementasi yang benar-benar menulis ke disk
  dengan offset index nyata (`offsets: Vec<u64>` di `:45`).
- **`FileSpillStore` itulah yang menghasilkan 38,1 MB index / 50,4 MB peak pada
  5 juta item** — angka yang jadi dasar seluruh tesis 2 GB VPS (D3) dan
  keunggulan terukur kita atas n8n.

Jadi: **klaim performa paling penting di proyek ini dihasilkan oleh kode yang
tidak pernah disentuh test suite.** Ia sudah dijalankan sebagai benchmark
(itu nyata dan berharga), tapi tidak ada test yang menjaganya tetap benar
saat kode berubah.

---

## 5. Celah keamanan — kernel tidak punya penanganan permission

Dihitung di seluruh crate kernel (`src/`, `tests/`, `examples/`):

```
grep -rn 'PermissionsExt|from_mode|set_permissions|\.mode(' → NOL hasil
```

`open_rw` di `examples/spill_bench.rs:110`:

```rust
OpenOptions::new().create(true).write(true).read(true).truncate(true).open(path)
```

Tanpa `.mode()`. Di Unix, `create(true)` menghasilkan `0666 & ~umask`:

| umask | Mode file spill |
|---|---|
| 022 (default) | **0644** — bisa dibaca semua user |
| 000 | **0666** — bisa dibaca dan ditulis semua user |

Kriteria lulus SEC-07 agent5 (`AGENT5_QA_SECURITY_SPEC.md:514`) mensyaratkan:
*"file spill yang baru dibuat ber-mode 0600 tanpa bergantung umask — jalankan
test dengan `umask 000` supaya benar-benar menguji konstruktor, bukan
kebetulan."* **Kode saya gagal kriteria itu.**

Sebaliknya, `data-plane/spill.rs` milik agent2 **lulus**:

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o600);
    tokio::fs::set_permissions(&path, perms).await?;
}
```

Ini persis pola "enforcement di konstruktor, bukan konvensi" yang saya tulis
di PRD-2 §11.1 sebagai prinsip — dan kode saya sendiri tidak menerapkannya.

**Catatan checksum:** `spill_bench.rs` memuat variabel bernama `checksum`
(`:437,447,484,494,516`), tapi itu akumulator `u64` untuk memverifikasi bahwa
benchmark membaca kembali data yang sama — **bukan** mekanisme integritas
persisten. Jadi pernyataan "kernel tidak punya checksum at-rest" tetap benar.

---

## 6. Sintesis untuk K-1 — keduanya saling melengkapi, dan kini presisi

Setelah membaca seluruh `data-plane/spill.rs` (91 baris) dan seluruh
`FileSpillStore` (bagian dari 653 baris), pemetaannya ternyata bersih:

| Kemampuan | `FileSpillStore` (kernel/examples) | `data-plane` (agent2) |
|---|:--:|:--:|
| Implement trait `SpillStore` | **ya** (7/7 method) | tidak (struct sendiri, tanpa `use kernel`) |
| Offset index → `read_at` O(1) | **ya** (`offsets: Vec<u64>`) | **tidak ada** |
| `SpillWriter` streaming + `Drop` cleanup | **ya** | tidak ada |
| Checksum integritas at-rest | tidak ada | **SHA-256 + `ChecksumMismatch`** |
| Permission 0600 di konstruktor | tidak ada | **ya** |
| GC per-execution | tidak ada | **`gc_execution()`** |
| Tercakup test suite | **tidak** (di `examples/`) | **tidak** (0 test di data-plane, temuan agent5 §512) |

**Tidak ada yang bisa dibuang.** `FileSpillStore` punya akses terindeks yang
jadi satu-satunya keunggulan terukur kita; `data-plane` punya tiga hal yang
membuatnya aman dan bisa dirawat.

### 6.1 Urutan kerja yang saya usulkan (butuh K-1 disetujui dulu)

1. **Perbaiki manifest** — buang `testkit`/`nodes-core` dari `members` sampai
   keduanya benar-benar ada; beri `data-plane` satu `src/lib.rs`. Ini membuat
   workspace bisa load. Sudah terbukti berhasil.
2. **Pindahkan `FileSpillStore` dari `examples/` ke `crates/data-plane/src/`**
   sebagai implementasi produksi. Benchmark tetap ada, tapi memanggil
   implementasi yang sama — bukan salinan.
3. **Serap tiga hal dari `spill.rs` agent2** ke implementasi itu: checksum
   SHA-256 di `SpilledList`, `from_mode(0o600)` di konstruktor, `gc_execution`.
4. **Tambah test yang hilang** — inilah yang menutup §4 dan §5 sekaligus:
   test `FileSpillStore` nyata (bukan `MemSpillStore`), dijalankan dengan
   `umask 000`, plus uji korupsi satu byte → `ChecksumMismatch`.
5. **T-11 ditutup dengan bukti:** SHA-256 sudah terimplementasi dan bekerja.
   Jangan ganti ke BLAKE3 sekarang — menambah `magic` 4-byte + `version`
   1-byte (usulan agent2 #256, disetujui agent5) memungkinkan migrasi nanti
   tanpa format ganda.

### 6.2 Konsekuensi untuk testkit agent3

Testkit agent3 (785 baris, 15/15 hijau) menguji `InMemorySpillStore` terhadap
**stub** 112 baris yang tidak punya trait `SpillStore` sama sekali. Setelah
langkah 1–4 di atas, testkit harus ditulis ulang melawan trait nyata — dan
kali ini ia akan menguji implementasi disk yang sebenarnya, bukan tiruan
in-memory. agent3 sudah menawarkan ini di #293 dan menunda dirinya di #389;
urutannya memang harus setelah K-1.

---

## 7. Reproduksi

Semua dijalankan sebagai `matt` di VPS, memakai toolchain rustup milik agent5
(read-only, artefak build di `/tmp`, dibersihkan sesudahnya):

```bash
T=/tmp/uji; sudo -u agent5 mkdir -p "$T"          # ownership benar dari awal
sudo -u agent5 cp -r /opt/agent-workspace/kernel-asli-d3bcff0/. "$T"/

run(){ sudo -u agent5 env HOME=/home/agent5 \
  RUSTUP_HOME=/home/agent5/.rustup CARGO_HOME=/home/agent5/.cargo \
  RUSTUP_TOOLCHAIN=stable CARGO_TARGET_DIR="$T/target" \
  PATH=/home/agent5/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin \
  bash -c "cd '$T' && $1" 2>&1; }

run 'cargo metadata --offline --no-deps >/dev/null; echo $?'   # -> 101 (rusak)
# buang 3 member dari Cargo.toml, lalu:
run 'cargo test --offline'                                     # -> 19 passed
run 'cargo clippy --offline --all-targets'                     # -> 0 warning
run 'cargo build --offline --release'                          # -> Finished 17.78s
```

Dua jebakan yang saya alami sendiri, supaya tidak diulang:

- **`PATH` harus menaruh bin rustup di depan**, atau `rustdoc` apt 1.75 yang
  terpakai dan doc-test gagal dengan error yang menyesatkan (§2).
- **Direktori kerja harus dimiliki akun yang menjalankan cargo.** Saya membuat
  `/tmp/...` sebagai root lalu memanggil python sebagai agent5 → `PermissionError`,
  dan tiga uji saya batal karena itu, bukan karena kodenya.

---

## 8. Yang berubah untuk keputusan Anda

**K-1 tetap: kernel asli kanonik.** Verifikasi ini memperkuatnya — trait-nya
utuh, 19 test lulus, clippy bersih, `forbid(unsafe_code)` aktif, release build
sukses, dan hanya ia yang punya akses terindeks.

Tapi kualifikasinya sekarang tegas dan harus Anda ketahui:

- Klaim "19/19 hijau" **tidak berlaku pada keadaan ter-commit**. Perlu
  perbaikan manifest dulu (satu baris, sudah diuji).
- Angka benchmark 38,1 MB / 50,4 MB **valid sebagai pengukuran** (ia benar
  dijalankan), tapi berasal dari kode di `examples/` yang **tidak dijaga test**.
  Ia harus dipindah ke `data-plane` dan diberi test sebelum dijadikan dasar
  klaim publik.
- Kernel **gagal** kriteria keamanan SEC-07 yang sudah disetujui tim. Ini
  bukan alasan menolak kernel — ini alasan langkah 3 di §6.1 wajib, bukan
  opsional.
