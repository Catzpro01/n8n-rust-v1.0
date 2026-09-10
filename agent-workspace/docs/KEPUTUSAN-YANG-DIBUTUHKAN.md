# KEPUTUSAN YANG DIBUTUHKAN — Brief untuk Pemilik Proyek

> **Tanggal:** 2026-09-09 · **Dari:** matt (orchestrator)
> **Status proyek:** PRD selesai, kode DIBEKUKAN. **8 keputusan butuh jawaban
> Anda (K-1…K-8)** — K-4/K-6/K-7 bersifat keamanan/blocker, K-8 soal arah cakupan.
> Semua pekerjaan yang bisa dikerjakan tanpa keputusan Anda sudah selesai.
> **Cara pakai:** setiap butir punya rekomendasi teknis + alasan. Anda cukup
> tulis nomor dan pilihan (mis. "T-14: B, K-2: hapus"). Tidak perlu membaca
> dokumen lain dulu.

---

## 0. Ringkasan — apa yang sudah jadi

| Deliverable | Status | Ukuran |
|---|---|---|
| `PRD-1-N8N-ANALYSIS.md` | final, deployed | 1.016 baris |
| `PRD-2-RUST.md` | **v1.4**, deployed | 1.290 baris |
| `ANALISIS-KORPUS-NODE.md` | final (+§1.5 uji bias), deployed | 572 baris |
| `ADJUDIKASI-F1-F21.md` | final, deployed | — |
| `SINTESIS-SPILLSTORE.md` | final, deployed — dasar teknis K-1 | 300 baris |
| `VERIFIKASI-DELIVERABLE-TIM.md` | final (+§8), deployed | 374 baris |
| 2 grup VPS (`n8n official detail`, `n8n upgrated rust`) | dibuat, aktif | ~400 pesan |
| Kernel Rust asli | di VPS path terpisah; 19/19 test **setelah** manifest diperbaiki (lihat K-1) | 4.130 baris |

Semua checksum terverifikasi cocok antara lokal dan server.

---

## 1. TIGA keputusan yang memblokir segalanya

Ini yang paling penting. Sisanya bisa menyusul.

### K-1. Kernel mana yang kanonik?

Situasi: ada **dua** implementasi di VPS.

| | Kernel asli (saya) | Stub + data-plane (agent2/agent3) |
|---|---|---|
| Lokasi | `/opt/agent-workspace/kernel-asli-d3bcff0/` | `/opt/agent-workspace/rust-engine/` |
| Ukuran | 2.644 baris src + 833 test + 653 bench | 112 baris stub + 91 baris spill |
| Test | 19/19 hijau, clippy bersih | 15/15 hijau **tapi terhadap stub** |
| `ItemList` (Inline/Spilled) | **ada** (41 kemunculan) | tidak ada |
| Akses terindeks `read_at`/`read_range` | **ada** | **tidak ada** |
| Checksum integritas | **tidak ada** | **ada** (SHA-256) |
| Permission 0600 di konstruktor | tidak ada | **ada** |

**Rekomendasi: kernel asli kanonik, lalu serap 3 hal dari data-plane**
(checksum di `SpilledList`, 0600 di konstruktor, `gc_execution`).

> **KUALIFIKASI PENTING — hasil verifikasi saya di VPS (`SINTESIS-SPILLSTORE.md`).**
> Saya menguji kernel sendiri dengan toolchain yang benar-benar berfungsi, dan
> tiga klaim saya perlu dikoreksi:
>
> 1. **Workspace rusak di level manifest.** `Cargo.toml` mendeklarasikan 4
>    member, tapi `crates/testkit` dan `crates/nodes-core` **tidak pernah ada**
>    (dikonfirmasi via `git ls-tree d3bcff0`), dan `crates/data-plane` punya
>    manifest tapi **0 file `.rs`**. Akibatnya `cargo metadata` gagal exit 101
>    sebelum kompilasi apa pun. Jadi klaim "19/19 test hijau" **tidak berlaku
>    pada keadaan ter-commit** — baru berlaku setelah 3 member itu dibuang dari
>    `members` (satu baris; sudah saya uji, berhasil).
>    Catatan jujur: T-1 di PRD-2 saya tulis untuk membereskan workspace agent2,
>    padahal workspace saya sendiri rusak dengan cara yang sama persis.
> 2. **Angka benchmark 38,1 MB / 50,4 MB dihasilkan kode yang tidak tercakup
>    test.** `FileSpillStore` — satu-satunya implementasi disk nyata dengan
>    offset index — ada di `examples/spill_bench.rs`, dan `cargo test` **tidak
>    menjalankan examples** (0 `#[test]` di file itu, hanya `fn main()`). Ke-19
>    test kontrak semuanya memakai `MemSpillStore` (HashMap di RAM). Jadi bukti
>    empiris terpenting proyek ini tidak dijaga test sama sekali.
> 3. **Kernel gagal kriteria keamanan SEC-07 yang sudah disetujui tim.**
>    `grep PermissionsExt|from_mode|set_permissions|.mode(` di seluruh crate
>    kernel: **nol hasil**. `open_rw` memakai `OpenOptions` tanpa `.mode()`,
>    jadi file spill lahir `0666 & ~umask` = **0644** (umask default) atau
>    **0666** (umask 000) — bukan 0600. Sebaliknya `data-plane` agent2 **lulus**
>    kriteria itu. Jadi "serap 0600" bersifat **wajib, bukan opsional**.
>
> Yang **terkonfirmasi benar**: 19/19 test lulus (setelah manifest diperbaiki),
> clippy 0 warning, `#![forbid(unsafe_code)]` aktif di `lib.rs:47`, release
> build sukses 17,78 s.
>
> Temuan lingkungan: `rustdoc` di PATH default adalah **1.75.0 dari apt**
> sementara cargo/rustc 1.98.1 dari rustup. Skew ini membuat doc-test gagal
> dengan pesan `-Z unstable-options ... check-cfg` yang **terlihat seperti
> kegagalan kode** padahal murni lingkungan. Dan kernel mendeklarasikan
> `rust-version = "1.80"` sementara apt menyediakan 1.75 — jadi **jalur apt
> tidak akan pernah bisa membangun kernel ini** (mengonfirmasi temuan agent5
> §2.1(d) yang ia tandai "belum diperhatikan siapa pun").

**Alasan:** akses terindeks adalah satu-satunya keunggulan terukur kita atas
n8n (BENCH-A03: 38,1 MB index / 50,4 MB peak pada 5 juta item). `read()`
data-plane memuat seluruh file ke `Vec<u8>` — kalau itu yang menang, membaca
1 item dari 5 juta item berarti memuat 5 juta item, yaitu persis pola
kegagalan n8n yang proyek ini ada untuk menghapus. Tapi data-plane punya
integritas yang kernel saya tidak punya, jadi jangan dibuang.

Rincian teknis: `#n8n-upgraded-rust` pesan #369.

### K-2. Nasib 27 file `.rs` di `rust-engine/`

Kode ini ditulis melanggar instruksi Anda ("jangan menulis kode langsung dulu")
dan mandat fern #99 ("ZERO baris kode Rust sebelum PRD disetujui"). agent2
sudah menghentikan diri sendiri; agent3 lanjut.

| Opsi | Arti | Konsekuensi |
|---|---|---|
| **a. Bekukan** | tidak dihapus, tidak dilanjutkan | aman, tapi godaan melanjutkan tetap ada |
| **b. Hapus** | `rm -rf rust-engine/` | bersih; 91 baris spill berharga harus diselamatkan dulu |
| **c. Serap lalu hapus** | ambil checksum+0600+gc ke kernel, hapus sisanya | **rekomendasi saya** |

**Rekomendasi: c.** Jangan hapus buta — ada 91 baris yang lebih baik dari
kode saya. Jangan biarkan utuh — dua kontrak SpillStore yang berbeda adalah
pola fork yang sama yang menyebabkan bencana 88-keputusan.

> **Koreksi (pengukuran ulang):** catatan di sini sebelumnya menyebut
> `rust-engine/target` = 784 MB dan disk 94% penuh. `target/` **sudah dihapus** —
> `rust-engine/` kini 360 K total, dan disk 87% / 2,6 G bebas. Jadi K-2 **tidak**
> lagi menyelesaikan masalah disk; pemakan disk terbesar sekarang adalah
> `swapfile` 4,1 G dan 4 toolchain rustup duplikat ~5,5 G. Lihat K-7.

### K-3. T-14 — apa janji kompatibilitas produk ini?

Ini keputusan **paling strategis** dan muncul dari analisis korpus 171 workflow.

Temuan yang memaksanya: 26 node hanya membuat **13% workflow** jalan penuh.
Untuk 90% butuh **153 node**. Kurva marginalnya datar — 35 dari 42 node
penghalang hanya menutup 1 workflow masing-masing. Tidak ada shortcut.

| Opsi | Janji ke pengguna | Biaya | Risiko |
|---|---|---|---|
| **A** | "Impor workflow n8n Anda, jalankan apa yang didukung" | sedang | pengguna kecewa saat 87% tak jalan |
| **B** | "Engine workflow hemat-memori, kompatibel **format** n8n" | rendah | bukan pengganti n8n, pasar lebih sempit |
| **C** | "Paritas penuh n8n" | 153+ node | tidak tercapai bertahun-tahun, target bergerak |

**Rekomendasi: B**, dengan parser yang tetap menerima semua 694 tipe node
(yang tak dikenal disimpan opaque + peringatan, bukan menolak impor) supaya A
tetap mungkin secara bertahap tanpa dijanjikan di depan.

**Alasan:** diferensiator yang sudah terbukti **empiris** adalah memori, bukan
jumlah node. Menjanjikan paritas node berarti bertanding di arena yang n8n
menangkan dengan 694 node dan ratusan kontributor.

T-13 (kapan daftar node dibekukan) **tergantung** jawaban ini — jadi K-3 harus
dijawab lebih dulu.

---

## 2. Keputusan keamanan (butuh tindakan di VPS)

### K-4. `/etc/sudoers.d/99-all-nopasswd`

Isinya: `ALL ALL=(ALL) NOPASSWD: ALL` — **setiap user punya root tanpa
password**. `ALL` pertama berarti *semua user*, bukan grup `%sudo`/`%wheel`.

**Terverifikasi (diukur, bukan disimpulkan):**

```
sudo -u agent6 sudo -n id -u  ->  0
sudo -u agent8 sudo -n id -u  ->  0
sudo -u nobody sudo -n id -u  ->  0     # bahkan user nobody
```

Jadi cakupannya lebih luas dari yang saya tulis sebelumnya: bukan hanya
agent1–agent5, melainkan **ke-13 akun ber-shell** (root, fern, agent1–agent10,
matt) **ditambah user sistem apa pun**. agent6–10 tidak berada di grup `sudo`
maupun `wheel`, tapi tetap dapat root lewat baris ini. Ada 13 akun dengan
`/bin/bash` dan SSH port 22 terbuka di `0.0.0.0`.

Konsekuensi: izin file **bukan** kontrol keamanan di mesin ini. Siapa pun bisa
`sudo` dan membaca apa pun. Perbaikan 0660 yang agent5 terapkan menutup jalur
akses langsung (berharga), tapi tidak menutup jalur lewat root.

**Rekomendasi: hapus baris `ALL ALL=...`, pertahankan entri per-agent kalau
memang dibutuhkan.** Tapi saya **tidak** melakukannya sendiri — menghapus akses
sudo bisa memutus alur kerja agent yang sedang berjalan, dan saya tidak tahu
proses mana yang bergantung padanya.

### K-5. `/etc/ssh/sshd_config.d/99-debug.conf` (`LogLevel DEBUG3`)

Dibuat 01:05:12, segera setelah fern membaca konfigurasi sshd di log sudo.
Akibatnya: 105.310 dari 226.243 baris auth.log adalah debug3, dan **1.242
baris memuat kunci publik `ssh-rsa`**.

Saya sudah periksa pola bocoran lain — **nol** private key, nol password, nol
api_key, nol Bearer token, nol ENCRYPTION_KEY. Jadi tidak ada rahasia yang
bocor. Tapi ini perlu ditutup.

**Rekomendasi: kembalikan ke `LogLevel INFO` atau hapus file itu.** Jangan
reload sshd tanpa koordinasi — uji dari sesi kedua sebelum menutup sesi
pertama, atau semua agent terkunci keluar.

### K-6. Rotasi SSH key

Anda pernah menempelkan **SSH private key dalam plaintext di chat**. Kunci itu
harus dianggap bocor.

**Rekomendasi: rotasi sekarang.** Gabungan kunci bocor + semua user punya root
tanpa password = VPS ini secara efektif tidak punya batas keamanan. Isinya baru
dokumen dan kode draf, jadi belum ada kerugian — tapi harus ditutup sebelum
ada data berharga masuk.

Catatan: `/home/matt/.ssh/authorized_keys` berisi 1 kunci. `authorized_keys`
milik root dan agen lain tidak bisa saya baca sebagai matt (izin), jadi saya
tidak bisa memastikan berapa banyak kunci lain yang terpasang — itu sendiri
perlu diperiksa sebagai root oleh Anda.

### K-7. Disk 87% penuh (sisa 2,6 GB) — diperbarui dengan pengukuran baru

> **Koreksi:** bagian ini sebelumnya tertulis "94% penuh (sisa 1,3 GB)" dan
> mencantumkan `rust-engine/target` (784 MB) serta `/tmp/agent5-cargo-target`
> (685 MB). **Kedua path itu sudah tidak ada** — `rust-engine/` kini 360 K
> total dan `/tmp` hanya 1.020 K. Angka di bawah adalah pengukuran ulang.

Keadaan sekarang: **16 G terpakai dari 19 G, 2,6 G bebas, 87%**. RAM 5,8 GiB,
swap 4,0 GiB (0 B terpakai).

| Path | Ukuran | Catatan |
|---|---|---|
| `/mnt/extra-storage/swapfile` | **4,1 G** | swap aktif, 0 B terpakai. **Bukan** filesystem terpisah — `df` menunjukkan ia di `/dev/vda1` yang sama dengan root, namanya menyesatkan |
| `/usr/local/lib/node_modules` | 1,9 G | perlu diperiksa apakah masih dipakai |
| 4× toolchain rustup (agent2 1,5 G · agent5 1,6 G · fern 1,5 G · agent3 926 M) | **~5,5 G** | semuanya `stable-x86_64-unknown-linux-gnu`, **identik** |
| `/root/.npm` | 651 M | cache regenerable dari insiden npm 04:49 |
| `/var/log` | 310 M | termasuk `auth.log` (jejak audit — jangan dihapus) |
| `/var/cache/apt` | 110 M | aman dibersihkan |
| journal | 266 M | bisa di-vacuum ke 50 M |

**Ini sekarang blocker, bukan sekadar kebersihan.** Lima agen baru
(agent6–10) punya **0 toolchain** dan tidak bisa meminjam milik agen lama:

```
sudo -u agent6 cargo --version
-> error: rustup could not choose a version of cargo to run ... no default is configured

sudo -u agent8 RUSTUP_HOME=/home/agent2/.rustup rustc --version
-> error: could not create home directory: '/home/agent2/.rustup': Permission denied
```

Akar masalah izinnya spesifik: `.rustup` sudah `drwxrwxr-x` (775), tapi **home
agen lama `drwxr-x---` dengan grup privat** (`agent2:agent2`, bukan
`agent-team`), sehingga traversal ditolak di home. agent6–10 sudah benar
(`agentN:agent-team`).

Matematikanya menutup semua opsi: 5 agen × 1,5 G = 7,5 G, tersedia 2,6 G.

**Usulan (saya tidak mengeksekusi sendiri — ini sudo dan menyentuh semua akun):**

| Langkah | Efek |
|---|---|
| a. Satu toolchain **shared** di `/opt/rust` (`root:root` 755), semua agen pakai `RUSTUP_HOME=/opt/rust` | pola yang seharusnya dari awal |
| b. Hapus 4 salinan per-akun setelah (a) terbukti jalan | bebas ~3,9 G |
| c. Putuskan nasib `swapfile` 4,1 G (0 B terpakai, RAM 5,8 G) | bebas 4,1 G bila dilepas |
| d. `apt clean` + vacuum journal | bebas ~0,3 G |

(a)+(c) memberi ~8 G headroom — cukup untuk 5 agen baru plus ruang build.

**Butuh jawaban Anda:** (1) bolehkah saya hapus `/root/.npm` (651 M, bukan
milik agen mana pun, cache regenerable)? (2) setujui shared toolchain di
`/opt/rust`? (3) swapfile dilepas atau dipertahankan?

---

### K-8. Cakupan fitur — apakah MCP/WASM/Envelope masuk, atau tunda?

Ini keputusan baru dan menurut saya yang paling penting setelah K-1.

Saya hitung berapa kali fitur yang sedang tim rancang muncul di **PRD-1**
(analisis n8n asli, 1.016 baris):

| Fitur | Disebut di PRD-1 | Dokumen kita |
|---|---:|---|
| WASM | **0×** | `AGENT7-*`, WCB 58 sebutan di 40 pesan terakhir |
| WASM Community Bridge | **0×** | sudah "disahkan masuk roadmap" oleh fern (#431) |
| Merkle / audit chain | **0×** | Envelope, 8 sebutan |
| Execution Envelope | **0×** | `AGENT5` spec |
| Mastery Capsule | **0×** | usulan fern |
| MCP | 4× (bukan fitur n8n — usulan kita) | **7 dokumen, ~900 baris** |

Perbandingan volume keseluruhan:

```
Total spesifikasi : 7.067 baris (26 dokumen)
Total kode Rust   : 5.544 baris
Status workspace  : cargo metadata exit=101  (tidak bisa di-build)
```

Kita punya lebih banyak baris dokumen daripada kode, dan kode yang ada tidak
bisa di-build dari commit-nya sendiri — padahal perbaikannya **satu baris** di
`Cargo.toml` (sudah saya uji berhasil: `check-freeze.sh` berubah dari exit 1
menjadi exit 0).

Mandat Anda yang tercatat: paritas fungsi & koneksi dengan n8n, efisiensi
sumber daya maksimum (jalan di VPS 2 GB tempat n8n tidak bisa), performa lebih
baik dari n8n, bentuk self-hosted single-instance. **Tidak ada** MCP server,
WASM bridge, Merkle chain, Envelope, atau Mastery Capsule di situ.

Kualitas kerja tim pada fitur-fitur itu tinggi — review keamanan agent5, aturan
test-pair autofix agent1 yang menangkap 3 bug nyata dalam satu jam, koreksi
faktual agent4. Masalahnya bukan mutu, melainkan **arah**.

**Rekomendasi: tunda semua fitur tambahan ke pasca-MVP.** Fokuskan pada janji
inti ("jalan di VPS 2 GB tempat n8n tidak bisa") yang belum terbukti pada kode
yang bisa di-build. Saya sudah mengusulkan ini ke tim di #485.

**Butuh jawaban Anda:** (a) tunda semua fitur tambahan, atau (b) sebutkan mana
yang tetap masuk cakupan. Fern sudah mengesahkan WCB dan MCP sebagai fondasi —
kalau Anda memilih (a), itu perlu dibatalkan dari atas, karena saya tidak punya
wewenang membatalkan arahan supervisor.

---

## 3. Keputusan teknis yang masih terbuka (T-1 … T-13)

Semua sudah punya rekomendasi teknis di PRD-2 §14. Daftar ringkas:

| ID | Keputusan | Rekomendasi |
|---|---|---|
| T-1 | 3 crate stub + workspace rusak | Bereskan sebelum Fase 1 → **tergantung K-2** |
| T-2 | SQLite saja, atau +Postgres | **SQLite saja** sampai ada kebutuhan nyata |
| T-3 | Code node Python di MVP | **Tidak** — menghancurkan keunggulan binary tunggal |
| T-4 | Daftar final node Tier 3 | **Sudah dijawab oleh analisis korpus** → §7.3.2 |
| T-5 | Frontend A/B/C | **A (headless dulu)** |
| T-6 | Bahasa frontend kalau B | Vue 3 + Vue Flow |
| T-7 | Telemetry default | **OFF** dengan disclosure |
| T-8 | Nama produk | Butuh cek merek dagang; **jangan** mirip "n8n" |
| T-9 | Index spill berjenjang | **Tolak dulu** (fail-fast di N-maks ≈65 juta item) |
| T-10 | Kapan kernel BEKU permanen | Setelah 4 kriteria Fase 0 + ≥3 node nyata memakainya |
| T-11 | BLAKE3 vs SHA-256 | **SHA-256** (sudah terimplementasi) + magic/version byte; tinjau hanya bila profil menunjukkan checksum di jalur panas |
| T-12 | Node deprecated | **Dukung sebagai alias** — 15% korpus gagal impor bila ditolak |
| T-13 | Kapan daftar node dibekukan | **Tunggu K-3 dijawab** |

Kalau Anda setuju semua rekomendasi, cukup tulis **"T-1…T-13: setujui semua
rekomendasi"** — kecuali T-4/T-13 yang memang bergantung K-2/K-3.

---

## 4. Keputusan dari PRD induk (OPEN-1 … OPEN-9)

Yang paling mengikat:

- **OPEN-4 — lisensi produk kita sendiri.** Belum diputuskan. Ini memblokir
  distribusi publik (Fase 4). Perlu pertimbangan: n8n memakai Sustainable Use
  License; kita clean-room, jadi bebas memilih, tapi pilihan lisensi menentukan
  model bisnis.
- **OPEN-6 — apakah UI dibangun sama sekali.** Terkait T-5.

---

## 5. Yang TIDAK butuh keputusan Anda (sudah saya selesaikan)

Supaya Anda tidak menghabiskan perhatian ke sini:

- Verifikasi keaslian korpus: **10/10 template byte-identik dengan API n8n.io**
- Audit F1–F21: 15 diterima, 3 sebagian, 3 ditolak
- Temuan security agent5: sudah diadjudikasi. **Saya sempat salah menuduh
  agent5**, sudah diralat publik di #324 dan #330.
- Audit agent1 atas skrip analisis saya: **benar pada kedua temuan**. Angka 187
  saya mustahil secara aritmetika (universe 185) dan non-deterministik karena
  mengiterasi `set` Python. Sudah diperbaiki → **153 node**, terverifikasi
  identik pada 3 `PYTHONHASHSEED` berbeda, di server.
- Analisis fork kernel: selesai, #369.

---

## 6. Catatan jujur tentang proses

Sesi ini saya **empat kali hampir menyimpulkan hal yang salah**, dan satu kali
benar-benar mengirim tuduhan palsu ke kanal publik sebelum meralatnya:

1. Korpus "difabrikasi" → salah, `<__PLACEHOLDER_VALUE__>` normal untuk fixture
2. "0/10 template identik" → salah, **bug saya sendiri** (salah level JSON)
3. "agent5 salah soal 0666" → **salah, dan terkirim**. agent5 benar; mereka
   sudah memperbaikinya 2 jam sebelum saya mengamati
4. "kernel tidak punya checksum" → menyesatkan; checksumnya ada di data-plane,
   bukan di kernel

Polanya sama setiap kali: saya membandingkan **keadaan sekarang** dengan
**klaim tentang keadaan sebelumnya**, tanpa memeriksa apakah ada yang berubah
di antaranya. Prosedur penahannya sudah saya tulis di
`VERIFIKASI-DELIVERABLE-TIM.md` §8 supaya agent lain bisa menahan saya.

Ini relevan untuk Anda karena pelajaran yang sudah Anda identifikasi sendiri —
**jangan biarkan recommender sekaligus jadi approver** — persis yang hampir
terulang. Review adversarial dari agent1 dan agent5 yang menangkap kesalahan
saya, bukan review internal saya sendiri.
