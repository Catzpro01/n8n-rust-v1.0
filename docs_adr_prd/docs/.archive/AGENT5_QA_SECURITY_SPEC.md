# AGENT5_QA_SECURITY_SPEC.md
## Spesifikasi QA & Batas Keamanan — n8n Upgraded Rust Engine

**Penulis:** agent5 (QA & Security Auditor)
**Versi:** v0.1 (draf untuk review matt & fern)
**Tanggal:** 2026-09-09
**Acuan:** PRD-1-N8N-ANALYSIS.md, PRD-2-RUST.md §10 (Strategi Kompatibilitas) & §11 (Keamanan)
**Status mandat:** ditulis atas perintah fern #100, #103, #135

---

## 0. Cara membaca dokumen ini

Setiap butir ditandai salah satu dari tiga status ini. Jangan campur ketiganya.

| Tanda | Arti |
|---|---|
| **[FAKTA]** | Sudah saya verifikasi sendiri di server atau terhadap sumber eksternal. Ada cara reproduksinya. |
| **[ASUMSI]** | Menggantung pada keputusan yang belum diambil. Saya nyatakan nilai sementaranya supaya dokumen bisa dikerjakan; **wajib diganti** begitu keputusan turun. |
| **[USUL]** | Rekomendasi saya sebagai auditor. Belum disetujui siapa pun. |

Permintaan keputusan yang menghalangi beberapa butir sudah dikirim ke fern & matt
(DM #186, #187; ditempel publik di #189).

---

## 1. Ruang lingkup

Dokumen ini menetapkan:
1. Kriteria lulus yang **terukur** untuk gate kompatibilitas L1/L2/L3 (PRD-2 §10.1).
2. Syarat korpus yang layak pakai untuk differential testing (§10.2).
3. Proses dan skema katalog deviasi (§10.3).
4. Batas keamanan yang **bisa diuji** untuk setiap baris di tabel PRD-2 §11.

Di luar cakupan: keputusan arsitektur crate, pilihan library, dan roadmap fase.
Itu wewenang matt sebagai Lead Architect.

**Prinsip yang mendasari seluruh dokumen ini:**

> Kontrol yang tidak dinyatakan sebagai pengujian yang bisa gagal
> bukanlah kontrol, melainkan niat.

dan

> **Yang menguji tidak boleh pihak yang mengimplementasi.** Harness differential
> dan fuzz target ditulis oleh QA; engine ditulis oleh yang lain. Kalau satu orang
> menulis keduanya, gate akan lulus apa pun yang terjadi.

---

## 2. Kondisi server yang membatasi (semua [FAKTA])

| Komponen | Status | Dampak |
|---|---|---|
| `docker` | **TIDAK ADA** | PRD-2 §10.2 mensyaratkan n8n asli via Docker → sisi pembanding belum bisa dijalankan |
| `node` | v18.19.1 ✅ | jalur alternatif `npx n8n` terbuka |
| `npm` | 9.2.0 ✅ | bisa pasang n8n tanpa container |
| RAM / CPU | **5925 MB** total, 5489 MB tersedia + swap 4 GB / 2 vCPU *(naik dari 1.9 GB pada 04:20)* **[FAKTA, `free -m`]** | `npx n8n` sekarang realistis; Docker pun mungkin |
| `cargo`/`rustc` | **berfungsi, tapi hanya lewat jalur tertentu** — lihat §2.1 | QA sudah bisa memverifikasi klaim build |
| `target/` | kepemilikan campur agent2/agent3; **file sumber pun campur** (ERR-025) | build lintas-agen bertabrakan; QA memakai `CARGO_TARGET_DIR` di `/tmp` sendiri |

**Konsekuensi:** sisi *build* sudah bisa diverifikasi (§2.1). Gate L2 dan L3 tetap
**belum dapat dieksekusi** karena sisi pembanding (n8n asli) belum bisa dijalankan.
Saya tidak akan melaporkan angka kelulusan untuk gate yang tidak bisa saya jalankan.

### 2.1 Status toolchain Rust — dikoreksi dua kali dalam sejam **[FAKTA]**

Riwayatnya perlu dicatat karena tiga pernyataan berbeda beredar dalam satu jam:

| Waktu | Pernyataan | Hasil verifikasi saya |
|---|---|---|
| 04:4x | agent5 (ERR-024): toolchain rusak untuk semua akun | benar pada saat itu |
| 05:0x | fern #205: "ERR-024 TUNTAS: rustc & cargo 1.98.1 resmi aktif di `/usr/bin` level sistem untuk seluruh akun" | **sebagian benar, sebagian salah** — lihat bawah |
| 05:12 | agent5: verifikasi ulang | toolchain **berfungsi untuk agent5**, tapi bukan karena alasan yang disebut |

Yang saya ukur sendiri:

1. `/usr/bin/cargo` dan `/usr/bin/rustc` **bukan** rustc/cargo asli. Keduanya
   **shim rustup**: ukuran identik 21 113 232 byte dan md5 identik satu sama lain
   (`bdf5b89af30385f0e3729e1baa51adf9`) — dua binary berbeda nama dengan isi sama
   adalah tanda shim, bukan compiler. Menjalankannya tanpa toolchain default
   menghasilkan `error: rustup could not choose a version of rustc to run`.
2. **Regresi:** paket apt `rustc 1.75.0` dan `cargo 1.75.0` **masih tercatat terpasang**
   (`dpkg -l`), dan `/usr/lib/rustlib/x86_64-unknown-linux-gnu/lib/` masih berisi
   `libstd-*.rlib` — tetapi binary `/usr/bin/rustc` miliknya **sudah ditimpa shim**.
   Jadi toolchain yang tadinya berfungsi lewat apt sekarang tidak bisa dipakai sama
   sekali. Ini persis pola yang membuat ERR-024 muncul pertama kali.
3. Yang **benar-benar** membuat toolchain berfungsi: rustup mengunduh
   `stable-x86_64-unknown-linux-gnu` **1.98.1** ke `~/.rustup/toolchains/` milik akun
   masing-masing (unduhan saya: 1.5 GB). Ini **per-akun**, bukan "level sistem".
   Akun yang belum mengunduh tetap tidak bisa build.
4. Cara memanggil yang berfungsi untuk agent5 **tanpa mengubah konfigurasi server**:
   ```
   export RUSTUP_TOOLCHAIN=stable
   export CARGO_TARGET_DIR=/tmp/agent5-cargo-target
   rustc --version   # rustc 1.98.1 (48a229cea 2026-09-01)
   ```
   Uji nyata: kompilasi + jalankan hello world → **berhasil**.

**[USUL] perbaikan yang benar untuk ERR-024:** (a) tentukan **satu** jalur toolchain
untuk seluruh akun — rustup sistem (`RUSTUP_HOME=/opt/rustup` bersama, satu unduhan,
bukan 1.5 GB × 6 akun), atau apt; (b) kalau rustup, set default **sekali** di level
sistem sehingga tidak ada akun yang perlu tahu tentang `RUSTUP_TOOLCHAIN`;
(c) kalau apt dipakai, jangan timpa `/usr/bin/rustc` dengan shim; (d) nyatakan MSRV
di satu tempat — `kernel` matt menulis `rust-version = "1.80"` sementara apt
menyediakan 1.75, jadi **jalur apt tidak akan pernah bisa membangun kernel matt**.
Selisih ini belum diperhatikan siapa pun.

**[KONFIRMASI matt #434]:** matt memverifikasi secara independen temuan S2.1(d) ini saat
menguji kernelnya sendiri: *"JALUR APT TIDAK AKAN PERNAH bisa membangun kernel ini... Toolchain
harus rustup, dan per-akun."* Jadi selisih MSRV yang saya tandai "belum diperhatikan siapa pun"
kini dikonfirmasi oleh pemilik kernel sendiri. Status: ditutup sebagai FAKTA, bukan lagi usul.

### 2.1.1 Tiga jebakan lingkungan yang MENYAMAR sebagai kegagalan kode (verifikasi matt #434) **[FAKTA]**

matt mengalami sendiri tiga jebakan saat menguji kernelnya, dan mengirimkannya ke saya karena
ketiganya membuat pengujian GAGAL dengan gejala yang TERLIHAT seperti bug kode padahal murni
soal lingkungan. Ini kelas yang sama dengan ERR-024/025/027. QA WAJIB memeriksa ketiganya
SEBELUM menyimpulkan sebuah build/test gagal karena kode:

| # | Jebakan | Gejala (menyerupai kegagalan kode) | Pemeriksaan QA |
|---|---|---|---|
| E1 | Direktori kerja tidak dimiliki akun yang menjalankan cargo | `PermissionError` saat cargo/python menulis target atau output; uji batal dan terlihat seperti kegagalan | `stat -c %U:%G <cwd>` harus = akun yang menjalankan. JANGAN buat `/tmp/...` sebagai root lalu jalankan sebagai agen. |
| E2 | Toolchain rustup per-akun tidak bisa dipinjam lintas akun | `permission denied` saat akun lain memanggil `~/.rustup` akun pertama; akun tanpa rustup tidak bisa build sama sekali | tiap akun CI/build butuh rustup sendiri, ATAU `RUSTUP_HOME` bersama yang readable. Kendala CI nyata — lihat §2.1 [USUL] (a). |
| E3 | Skew versi rustdoc: apt 1.75 di PATH default vs cargo/rustc rustup 1.98 | `error: the -Z unstable-options flag must also be passed to enable the flag check-cfg` + `doctest failed` — TERLIHAT seperti kegagalan doctest/kode | `rustdoc --version` harus 1.98.x. Fix: `PATH=~/.rustup/toolchains/stable-*/bin:$PATH` agar rustdoc rustup yang terpakai, bukan apt. |

**Aturan QA yang lahir dari sini:** sebelum melaporkan "build/test gagal", jalankan lebih dulu
`rustc --version && cargo --version && rustdoc --version && stat -c %U:%G .` dan sertakan
keempatnya di laporan. Kalau salah satu tidak konsisten (rustdoc dari apt, cwd milik root,
toolchain akun lain), itu kegagalan LINGKUNGAN, bukan kegagalan kode — jangan dicatat sebagai
ERR kode. matt sempat menyimpulkan salah karena E3, dan tiga ujinya batal karena E1; keduanya
bukan bug kernel. Verifikasi lingkungan dulu, baru nilai kode.

### 2.1.2 Gate BENCH-REPRO — reproduksi benchmark wajib menyertakan batas memori eksplisit (matt #471) **[ATURAN]**

matt menemukan BENCH-A03 (38,1 MB index / 50,4 MB peak; "inline OOM-killed di 5 juta item") diukur
di mesin 2 GB RAM / swap 0 / 2 core, sedangkan VPS dev sekarang 5,8 GB RAM / swap 4 GB. Akibatnya
hasil headline "inline mati OOM" TIDAK akan terulang di VPS dev — inline mencapai 1,68 GB anon-rss
sebelum di-kill (exit 137), dan 1,68 GB muat longgar di 5,8 GB + swap 4 GB. Siapa pun yang mengukur
ulang tanpa menyamakan batas akan menyimpulkan benchmark salah/dibuat-buat, padahal kondisi mesinnya
yang berbeda. matt mengakui kekurangan dokumennya sendiri: tidak memberi perintah reproduksi ber-batas-memori.

**GATE BENCH-REPRO (WAJIB untuk setiap klaim benchmark di proyek ini):**
1. Setiap angka benchmark HARUS menyebut lingkungan pengukurannya: RAM fisik, swap, `MemoryMax`,
   `MemorySwapMax`, jumlah core, dan versi toolchain. Tanpa ini, angka tidak dapat dibandingkan lintas mesin.
2. Reproduksi yang sah WAJIB memaksa batas memori eksplisit, bukan mengandalkan RAM fisik:
   `systemd-run --scope -p MemoryMax=2G -p MemorySwapMax=0 ./target/release/<bench>`
   atau cgroup v2 langsung (`memory.max` + `memory.swap.max=0`). Tanpa `MemorySwapMax=0`, swap
   menyerap alokasi dan yang terukur adalah thrashing, bukan OOM.
3. Pisahkan dua jenis klaim (matt #471): (a) jejak kode kita sendiri (peak RSS spilled, ukuran index)
   = TRANSFER antar mesin, relevan untuk klaim "jalan di VPS 2 GB"; (b) klaim "reduksi = tak terhingga"
   yang bergantung pada inline DI-KILL = SPESIFIK HOST, tidak boleh diklaim ulang tanpa batas memori.
4. QA TIDAK menerima argumen "benchmark saya beda jadi benchmark asli salah" kecuali lingkungan
   disamakan lebih dulu lewat batas memori eksplisit (poin 2).

Gate ini melengkapi §2.1.1 (E1–E3) sebagai prasyarat lingkungan sebelum menilai hasil build/benchmark:
E1–E3 = "apakah toolchain/cwd benar", BENCH-REPRO = "apakah batas memori disamakan". Keduanya harus
dipenuhi sebelum sebuah angka dipakai untuk mendukung atau membantah klaim.

### 2.2 Klaim build yang sudah saya reproduksi sendiri **[FAKTA]**

Sebelum 05:00 saya tidak bisa memverifikasi klaim build siapa pun. Sekarang bisa,
dan hasilnya:

| Klaim | Sumber | Hasil verifikasi agent5 |
|---|---|---|
| `cargo check` workspace 15 crate PASSED | agent2 #174 | **TERBUKTI.** `cargo check --workspace` selesai 45.27 s, `Finished dev profile`, 0 error. 15/15 crate punya sumber, semua anggota `members` ada di disk. |
| 16 test hijau | agent2 #174 | **TERBUKTI.** `cargo test --workspace` → 16 passed, 0 failed. |
| kernel asli 19/19 test hijau | matt #236/#237 | **TERBUKTI, tapi hanya lewat workaround.** `tests/contract.rs` → **19 passed, 0 failed**. Nama test-nya substantif, bukan hiasan: `credential_value_never_leaks_in_debug`, `spilled_list_holds_no_payload_in_ram`, `streaming_ram_is_independent_of_n`, `spilled_and_inline_are_observationally_identical`. |
| kernel asli bisa dibangun seperti diunggah | matt #236 | **TIDAK TERBUKTI — workspace-nya gagal load.** Lihat ERR-027. |

Workaround yang saya pakai untuk kernel matt, saya nyatakan terbuka supaya bisa
dinilai: `Cargo.toml` workspace-nya menyebut 4 anggota tetapi hanya 2 yang ada di
disk, dan salah satunya (`data-plane`) tidak punya file `.rs` sama sekali, sehingga
`cargo` menolak memuat manifest — bahkan `cargo test -p kernel` gagal. Saya salin
crate `kernel` ke `/tmp` dan membuat manifest workspace minimal yang hanya menyebut
`crates/kernel`, lalu mengujinya di sana. **Saya tidak mengubah apa pun di direktori
matt.** Artinya: 19/19 itu nyata untuk crate `kernel`, tetapi klaim "workspace
terunggah" belum benar.

Catatan kepemilikan yang memperkuat ERR-025: `rust-engine/crates/testkit/src/lib.rs`
dimiliki **agent2** tetapi ditulis oleh **agent3** (isi dan #200/#202). 15 dari 16
test yang saya jalankan di workspace agent2 sebenarnya test milik agent3. Bukan
masalah benar/salah, tapi berarti "16 test agent2" dan "15 test agent3" adalah
**test yang sama** — dua klaim terpisah untuk satu pekerjaan, dan itu membuat
jumlah test terlihat lebih banyak daripada kenyataannya.

---

## 3. Gate kompatibilitas — kriteria lulus terukur

### 3.1 Satuan hitung **[ASUMSI — menunggu Keputusan 3]**

PRD-2 §10.1 menulis "L2 ≥95% dari korpus" dan "L3 ≥90% dari korpus" tanpa
mendefinisikan satuannya. Saya pakai asumsi berikut sampai ada keputusan:

```
denominator(L2) = jumlah node-execution pada korpus yang type node-nya
                  termasuk Tier yang diklaim didukung
numerator(L2)   = jumlah node-execution yang SELURUH item output-nya
                  cocok setelah normalisasi

denominator(L3) = jumlah evaluasi expression unik pada korpus
numerator(L3)   = jumlah evaluasi yang hasilnya identik setelah normalisasi
```

**Pengecualian yang wajib dinyatakan eksplisit:** `stickyNote` adalah node dekoratif
UI, bukan node eksekusi — ia **tidak masuk** numerator maupun denominator, di tingkat
mana pun. agent4 menemukan node ini di 38 file korpus **[FAKTA]**. Menghitungnya akan
menekan tingkat kelulusan secara artifisial; mengabaikannya diam-diam akan membuat
angka tidak bisa direproduksi. Karena itu ia harus disebut di spesifikasi.

**Aturan pelaporan yang mengikat:** setiap angka gate WAJIB dilaporkan sebagai
`numerator/denominator = persentase (versi korpus, versi n8n, hash build)`.
Persentase tanpa denominator **ditolak** oleh QA, karena korpus yang didominasi
workflow sepele bisa meluluskan gate tanpa menguji apa pun.

### 3.2 Definisi "cocok"

Dua output node dianggap cocok bila, **setelah normalisasi** (§3.3), berlaku:

1. Jumlah item sama.
2. Untuk tiap item: himpunan kunci sama; untuk tiap kunci, nilai setara menurut tipe:
   - string/boolean/null → setara persis
   - integer → setara persis
   - float → `|a-b| <= max(1e-9, 1e-9 * max(|a|,|b|))` **[USUL]**
   - array → urutan signifikan **kecuali** node tercatat di manifest sebagai
     order-insensitive (mis. hasil `Merge` mode tertentu) — pengecualian harus
     eksplisit, tidak boleh default
   - object → rekursif dengan aturan yang sama
3. `binary` dibandingkan atas **nama field, mimeType, dan sha256 isi**, bukan atas
   `fileName` atau `data` mentah.

### 3.3 Normalisasi — wajib ada manifest, wajib direview pihak lain

Output n8n pasti memuat data non-deterministik. Tanpa normalisasi, harness akan
menghasilkan ribuan *false positive* dan tim akan berhenti memercayainya dalam
seminggu. **[FAKTA dari pengalaman, bukan tebakan]**

`NORMALIZATION-MANIFEST.md` harus menyatakan, **per node-type**:

| Kelas | Perlakuan | Contoh |
|---|---|---|
| `MASK` | diganti penanda tetap | `executionId`, `id`, `webhookUrl`, durasi |
| `HASH` | diganti sha256-nya | isi binary, body respons yang besar |
| `ROUND` | dibulatkan | timestamp ke detik, float ke presisi tetap |
| `DROP` | dibuang dari perbandingan | header `Date`, `X-Powered-By` |
| `CANON` | kanonisasi struktur | urutkan kunci JSON, urutkan elemen himpunan |

**Aturan anti-kecurangan [USUL, saya anggap penting]:** manifest normalisasi adalah
tempat paling mudah menyembunyikan kegagalan — longgarkan satu entri `MASK` dan
gate lulus tanpa produk benar-benar kompatibel. Karena itu:
- setiap perubahan manifest wajib lewat review agent yang **bukan** penulis node-nya;
- setiap entri `DROP`/`MASK` wajib punya alasan tertulis;
- harness harus bisa dijalankan dalam mode `--strict-no-normalization` untuk audit.

### 3.4 Sisi pembanding (n8n asli)

**[ASUMSI — menunggu Keputusan 1 & 2]**

- Runtime: `npx n8n@<VERSI_PIN>` **atau** image Docker, tergantung Keputusan 1.
- Versi: **satu** versi di-pin; disimpan di server (tarball/image) supaya tidak
  bergantung registry eksternal saat reproduksi.
- Mode: `EXECUTIONS_MODE=queue` dimatikan; `N8N_DIAGNOSTICS_ENABLED=false`;
  telemetri dimatikan; zona waktu dan locale **ditetapkan eksplisit** (mis. `UTC`,
  `en-US`) karena hasil format tanggal bergantung keduanya.
- **Mock wajib.** Korpus saat ini memuat node `gmail`, `slack`, `openAi`,
  `googleSheets`, `telegram`, `pinecone`, `youTube`, `googleCalendar` **[FAKTA,
  hasil verify_corpus.py]**. Menjalankannya apa adanya berarti panggilan API
  sungguhan: non-deterministik, kena rate limit, butuh kredensial, dan bisa
  **benar-benar mengirim email**. Karena itu sisi pembanding wajib berjalan
  terhadap backend rekaman:
  - mode `record` → proxy merekam pasangan request/response nyata
  - mode `replay` → server lokal deterministik memutar rekaman
  - manfaat ganda: rekaman itu sekaligus **bukti** bahwa korpus pernah dijalankan
    di n8n asli, bukan dikarang.

### 3.5 Penanganan flakiness

Untuk setiap kasus: jalankan n8n asli **N=3** kali **[USUL]**.
- hasil identik semua → `deterministic` → boleh masuk katalog deviasi
- hasil berbeda → `flaky` → **tidak boleh** masuk katalog deviasi, dicatat terpisah

Tanpa aturan ini, deviasi non-deterministik akan menyusup ke dokumen publik §10.3
dan merusak kredibilitasnya.

### 3.6 Versi korpus

Setiap laporan gate wajib menyebut **versi korpus** = sha256 manifest atas seluruh
file. `verify_corpus.py --json` sudah menghasilkan `manifest_sha256` untuk ini
**[FAKTA, terpasang di `/opt/agent-workspace/qa/`]**. Tanpa versi, angka gate tidak
bisa dibandingkan antar waktu dan kemajuan semu mudah tercipta.

---

## 4. Syarat korpus

### 4.1 Kondisi saat ini **[FAKTA — diverifikasi 2026-09-09 05:00 UTC]**

Korpus tumbuh sangat cepat saat dokumen ini ditulis. Tiga pengukuran dalam 40 menit
memberi tiga angka yang berbeda jauh — ini sendiri bukti bahwa **angka korpus tidak
berarti tanpa timestamp dan tanpa cakupan pemindaian**.

| Metrik | 04:20 (78 file) | 04:52 (121 file) | **05:00 (198 file)** |
|---|---|---|---|
| Kategori EKSPOR (punya metadata instance) | 0 | 27 | **36** |
| FIXTURE (tertelerusur ke manifest) | 52 | 69 | **162** |
| MENCURIGAKAN (tidak tertelusur) | 26 | 0 | **0** |
| Isi terubah dari bentuk asli | 40 | 1 | **0** |
| Entri sha256 di manifest | 52 | 108 | **183** |
| Nama file tak disebut di manifest mana pun | 26 | 0 | **0** |
| Layak untuk differential testing | 12 | 120 | **198** |

Yang berubah di antara pengukuran itu: agent1 mengunduh ulang file yang tadinya
memuat field suntikan lalu menambah wave3, agent4 mengunggah batch2 (56 file) beserta
manifestnya, dan agent4 menambahkan subset `exec-diff-mvp/`.

**Kondisi sekarang: seluruh 198 file tertelusur dan tidak ada satu pun yang isinya
terubah dari bentuk aslinya.** ERR-022 dan ERR-023 saya tutup setelah verifikasi
ulang, bukan setelah klaim.

#### Subset `exec-diff-mvp/` — diverifikasi byte-per-byte **[FAKTA]**

13 file yang tercatat di `README-EXEC-DIFF-MVP.md`:

| Pemeriksaan | Hasil |
|---|---|
| SHA-256 cocok dengan manifest | **13/13** |
| `SrcSHA-256` cocok dengan file sumber di root | **13/13** |
| `nodes` + `connections` identik dengan sumber | **13/13** |
| Perbedaan lain | hanya field `name` yang ditambahkan — persis seperti yang README nyatakan |

Ini contoh provenance yang benar: **dua hash** (hasil dan sumber), transformasi
**dinyatakan**, dan bisa diverifikasi pihak lain tanpa memercayai pemiliknya.
13 file ini adalah korpus differential pertama kita yang sah.

*Catatan kecil:* ada **2 file tambahan** di direktori itu
(`tpl-2221-…`, `tpl-871-…`) yang **tidak** tercatat di README-nya. Keduanya saya
periksa: `nodes`+`connections` identik dengan file root, `name` ada. Jadi isinya
sah, hanya **manifest-nya yang belum diperbarui**. Bukan integritas, tapi
kelengkapan catatan — perlu dilengkapi supaya aturan "dua hash" tetap berlaku
untuk semua file di direktori itu.

#### Temuan agent4 yang mengubah cara gate harus dihitung **[FAKTA]**

`NODE-COVERAGE-REPORT-agent4.md` melaporkan, atas 96 template publik:

- hanya **6 file (6%)** yang seluruh node-nya masuk MVP-26;
- hanya **5 file (5%)** yang seluruh node-nya Tier-1;
- `stickyNote` muncul di **38 file** — node dekoratif UI, bukan node eksekusi;
- template publik saat ini didominasi node AI/LLM di luar target MVP;
- `splitInBatches` (Loop Over Items) dipakai 8 file tapi **tidak ada** di MVP.

Ini menguatkan Keputusan 3 secara empiris. Kalau denominator L2 = seluruh
node-execution di seluruh korpus, gate **tidak akan pernah** lulus dan angka itu
tidak memberi informasi apa pun. Kalau denominator = node pada Tier yang diklaim
didukung, gate bisa lulus sambil tetap jujur — **asal denominatornya selalu
dilaporkan**. Dan `stickyNote` **wajib dikeluarkan** dari perhitungan node L1,
kalau tidak tingkat kelulusan L1 ditekan secara artifisial oleh node yang bahkan
bukan node eksekusi.

#### Analisis korpus matt (v1.3) — direproduksi QA, sebagian angkanya tidak stabil

matt menerbitkan `ANALISIS-KORPUS-NODE.md` + `docs/tools/analisis_node.py` (#289) dan
menaikkan PRD-2 ke v1.3. Saya jalankan skripnya sendiri empat kali pada korpus yang
sama. Hasilnya dua bagian **[FAKTA]**:

**Stabil dan cocok dengan klaimnya** (jadi temuan strategisnya kokoh):
`171 template · 1.810 instance node (non-sticky) · 185 tipe unik · stickyNote 79 file
(46%) · AI/LC 39 file (23%) · 112 tipe di ≤2 file (61%) · 89 tipe di tepat 1 file
(48%) · instance tercakup 576/1810 = 32% · workflow jalan penuh 23/171 = 13% ·
+1 node = 54 wf`.

**Tidak stabil antar run pada input yang sama:**

| Metrik | Nilai yang saya dapat | Klaim #289 |
|---|---|---|
| node untuk 90% coverage | 172, 175, 182, 186 | **187** |
| TEMUAN 5 — AI | 91, 92 | 95 |
| TEMUAN 5 — non-AI | 105, 110 | 107 |
| urutan TOP 35 pada nilai seri | `wait`/`telegram` bertukar | — |

Akar masalahnya sudah saya buktikan sebab-akibat, bukan dugaan: `PYTHONHASHSEED=0`
dijalankan dua kali menghasilkan output **identik byte-per-byte**; `PYTHONHASHSEED=1`
vs `2` menghasilkan output **berbeda**. Greedy set-cover di baris 93–98 menelusuri
`set` (`for t in pool`) dengan tie-break `if g > bg`, sehingga pada gain seri yang
menang adalah node yang kebetulan lebih dulu di urutan set. Rincian + perbaikan 2
baris ada di **ERR-028**.

**Kenapa ini relevan untuk spesifikasi QA:** metrik *counting* stabil, metrik
*greedy/search* tidak. Aturan umumnya — **setiap angka yang diterbitkan sebagai dasar
keputusan harus berasal dari proses yang menghasilkan byte identik saat diulang.**
Itu sebabnya §3.6 mewajibkan versi korpus berupa hash, dan §3.5 mewajibkan uji 3×
sebelum sesuatu disebut deterministik. Skrip ini lolos uji substansi tapi gagal uji
yang kedua, dan justru uji kedua itu yang membuatnya bisa dipercaya orang lain.

**Catatan dokumen:** PRD-2 sekarang v1.3, sha256
`6e66438f5e5e5700d01c236052934bfa631a82c45aeb1dee76f30360992bb030` (59.912 byte).
Hash beku yang diumumkan fern (#156) adalah `0de2dc45d9f841…` (51.647 byte). Selama
keduanya beredar, tidak jelas dokumen mana yang jadi acuan — sudah saya minta
diperbarui. Audit saya atas §10/§11 di dokumen ini mengacu pada **v1.3**; §10.1 v1.3
**masih** menulis "L2 ≥95% dari korpus" tanpa satuan, jadi Keputusan 3 tetap terbuka.

**Koreksi atas angka stickyNote saya sendiri:** di §4.1 dan pesan #272 saya mengutip
"38 file" dari laporan agent4. Angka matt — **79 file dari 171 (46%)** — yang benar
dan sudah saya reproduksi. Selisihnya karena agent4 menghitung atas 96 file, matt atas
171. Saya sebut ini karena saya sudah memakai angka 38 itu di dua tempat, dan angka
yang salah walau kecil tetap harus diluruskan.

### 4.2 Tiga kategori keaslian

| Kategori | Ciri | Untuk apa |
|---|---|---|
| `EKSPOR` | punya `id`, `meta.instanceId`/`templateId`, `versionId`, `active`, `createdAt`, `updatedAt` | **gate L2/L3** — mewakili workflow produksi yang berantakan |
| `FIXTURE` | tanpa metadata instance, tapi sha256/nama file tertelusur di manifest ke sumber upstream | **gate L1** (parser) + katalog deviasi |
| `MENCURIGAKAN` | tanpa metadata **dan** tanpa penelusuran manifest | ditolak — inilah yang dimaksud PRD sebagai "ditulis tangan untuk lulus" |

### 4.3 Aturan integritas isi

1. **Dilarang menyuntik field apa pun** ke dalam JSON workflow. Pada 04:20 ada
   **40 file** memuat `_corpus_meta` yang bukan dihasilkan n8n (ERR-022). Pada
   05:00 jumlah itu sudah **0** — agent1 mengunduh ulang. Aturan ini tetap berlaku:
   provenance **hanya** di manifest, tidak pernah di dalam file.
2. **Simpan byte apa adanya.** Sisi pembanding harus mengimpor ekspor n8n murni.
   Kalau sebuah transformasi memang diperlukan (mis. meratakan struktur agar bisa
   diimpor), transformasinya wajib **dinyatakan** dan **dua hash** dicatat — hash
   hasil dan hash sumber. `exec-diff-mvp/` melakukannya dengan benar.
3. **Setiap batch wajib punya manifest** dengan: URL sumber, commit/template_id,
   dan sha256 per file. Manifest agent4 adalah contoh yang baik — sha256-nya saya
   verifikasi cocok byte-per-byte **[FAKTA]**.
4. **Redaksi: ini koreksi atas temuan saya sendiri.** Saya melaporkan di ERR-022
   bahwa `meta.instanceId = "[REDACTED_INSTANCE_ID]"` pada `tpl-4722` adalah bukti
   penyuntingan oleh tim. **Itu salah.** Saya tarik sendiri n8n.io API dan
   menemukan bahwa API-nya **sendiri** mengembalikan literal itu, bersama
   `[REDACTED_TAG_ID]`, `[REDACTED_WEBHOOK_ID]`, `[REDACTED_EMAIL]`,
   `[REDACTED_MESSAGE_ID]`. Jadi placeholder berpola `[REDACTED_*]` adalah
   **redaksi hulu yang sah**, bukan cacat integritas.

   Akibatnya dua: (a) bagian itu dari ERR-022 saya cabut; (b) alat verifikasi saya
   sendiri punya *false-positive mechanism* — ia menandai redaksi hulu sebagai
   penyuntingan, dan karena itu bisa dipakai untuk menuduh orang yang tidak
   bersalah. Sudah diperbaiki di v4: hanya redaksi **tidak berpola** (mis.
   `"REDACTED"` polos, `"<redacted>"`) yang dicurigai.

   Pelajaran umumnya: **redaksi yang dilakukan hulu tidak bisa dibedakan dari
   penyuntingan tanpa membandingkan ke sumbernya.** Karena itu aturan "simpan byte
   apa adanya" tetap benar, tapi penegakannya harus lewat perbandingan hash ke
   sumber, bukan lewat pencocokan pola string.

### 4.4 Komposisi yang saya usulkan

`fixture` upstream bagus untuk L1, tetapi cenderung rapi dan terkurasi sehingga
tidak menguji kasus berantakan produksi. Untuk L2/L3 perlu minimal **20 file
kategori EKSPOR** — syarat ini **sudah terpenuhi sepuluh kali lipat** (36 file pada 05:00) **[FAKTA]**.
Yang masih kurang bukan jumlah, tapi **cakupan node**: 13 file `exec-diff-mvp/`
dipilih karena ≥70% node-nya masuk MVP-26, jadi korpus differential pertama ini
sengaja sempit. Perlu diperluas bertahap seiring node yang diimplementasikan.
**[USUL]**

### 4.5 Dua validator yang berbeda hasil — sudah didamaikan

agent2 membuat `validate_corpus.py` sendiri (#175) dan melaporkan `96 file,
1 REAL_EXPORT, 56 LIKELY_EXPORT, 37 INVALID (missing name field), 2 SYNTHETIC`.
Saya sudah memeriksa kode alatnya langsung **[FAKTA]**. Selisihnya **selisih
definisi, bukan selisih fakta**, dan saya sudah menemukan akarnya:

| Kriteria agent2 | Masalahnya | Verifikasi saya |
|---|---|---|
| `REQUIRED_WORKFLOW_FIELDS = ["name","nodes","connections"]` → tanpa `name` = **INVALID** | Terlalu keras. Ekspor/template n8n yang sah bisa tidak memuat `name` di tingkat atas. | **48 file** gagal hanya karena tidak punya `name`. Saya periksa: semuanya punya `nodes` + `connections` lengkap, dan **semuanya FIXTURE tertelusur** di manifest. Jadi 37–48 file itu **bukan sampah** — sah untuk L1. |
| `REAL_EXPORT` = ≥5 dari 8 field metadata; `LIKELY_EXPORT` = 3–4 | Ambang angka tanpa dasar; menghasilkan 56 "mungkin" yang tidak bisa ditindaklanjuti. | Klasifikasi saya pakai penelusuran manifest sebagai pembeda, sehingga hasilnya biner: tertelusur atau tidak. |
| tidak memeriksa integritas isi | File bisa "LIKELY_EXPORT" secara struktur dan tetap tidak layak karena isinya disunting. | v4 memeriksa field suntikan + redaksi non-hulu. |

**Kesimpulan:** tidak ada yang berbohong; dua alat mengukur hal berbeda. Usulan saya
tetap: **satu alat, satu definisi** — kriteria agent2 dimasukkan ke
`verify_corpus.py` sebagai pemeriksaan terpisah (`missing_name` sebagai *peringatan*,
bukan penentu kategori), lalu dijalankan bersama. Selama ada dua validator dengan dua
angka, tidak ada satu pun angka korpus yang bisa dipakai menutup gate.


---

## 5. Katalog deviasi (PRD-2 §10.3)

Kerangka ada di `DEVIATION-CATALOG.md` (dokumen terpisah). Aturan pengisiannya:

1. Hanya deviasi **deterministik** (§3.5) yang boleh masuk.
2. Setiap entri wajib memuat: gejala di n8n, gejala di engine kita, **cara
   reproduksi** (id kasus korpus + versi korpus), alasan, dan dampak.
3. Entri tanpa cara reproduksi **ditolak** oleh QA.
4. Katalog ini dokumen **publik** — karena itu ia satu-satunya yang membuat klaim
   "kompatibel" bisa dipercaya. Tanpa itu, klaim kompatibilitas kosong.

---

## 6. Batas keamanan yang bisa diuji (PRD-2 §11)

Tabel §11 di PRD berisi niat, bukan spesifikasi. Hampir setiap barisnya melanggar
kriteria *ZERO ASSUMPTION* yang fern tetapkan di #93 karena tidak memuat angka,
parameter, atau mekanisme. Berikut versi yang bisa diuji.

### SEC-01 — Kredensial at-rest

| | |
|---|---|
| **[FAKTA] masalah** | PRD mengizinkan key dari `env`. Env proses terbaca lewat `/proc/PID/environ` oleh pemilik proses dan root. Di VPS ini **semua akun punya sudo NOPASSWD** (ERR-012, masih terbuka), jadi key di env praktis terbuka untuk seluruh tim. |
| **[USUL] syarat** | Produksi **wajib** keyfile mode `0400` milik user service. Env hanya untuk pengembangan dan harus ditolak saat `--production`. |
| **[USUL] skema** | AES-256-GCM, nonce 96-bit acak per record (tidak pernah dipakai ulang untuk key yang sama), tag 128-bit. AAD = `credential_id`. |
| **[USUL] key hierarchy** | Master Key (dari keyfile) → membungkus Data Encryption Key per record. Rotasi hanya perlu meng-enkripsi ulang DEK, bukan seluruh payload. |
| **kriteria lulus** | (a) test: mode production dengan key dari env → **gagal start**; (b) test: `/proc/self/environ` tidak memuat key saat mode keyfile; (c) test: nonce tidak berulang pada 10⁶ enkripsi dengan key sama; (d) test: mengubah `credential_id` membatalkan dekripsi (AAD bekerja). |

### SEC-02 — Rotasi key

| | |
|---|---|
| **[FAKTA] masalah** | PRD-1 §11.4 sendiri menyebut rotasi key sebagai area paling menyakitkan di n8n. Tapi skema PRD-2 §5.2 dan tabel §11 **tidak punya kolom versi key**. Tanpa itu, rotasi = dekripsi + enkripsi ulang seluruh data dalam satu operasi; gagal di tengah berarti data rusak dan tidak bisa dibatalkan sebagian. |
| **[FAKTA] sudah ada di desain** | `AGENT2_STORAGE_SPEC.md` sudah memuat tabel `encryption_key` dengan kolom `rotated_from INTEGER REFERENCES encryption_key(id)` dan kolom `enc_key_version INTEGER NOT NULL DEFAULT 1` pada tabel credential, plus prosedur rotasi 3 langkah dan penanganan baris legacy (`enc_key_version = 0`). Saya verifikasi barisnya langsung, bukan dari ringkasan. Jadi butir ini **sudah terjawab di tingkat desain**; yang belum ada adalah implementasi + test. |
| **[USUL] syarat tambahan** | Prosedur rotasi di spec agent2 berbunyi "UPDATE credential SET ... WHERE enc_key_version = old_version" per baris. Itu benar, tapi wajib dinyatakan **transaksional per batch** dan **dapat dilanjutkan setelah gagal** — kalau proses mati di baris ke-500 dari 5 000, sistem harus bisa melanjutkan, bukan mengulang dari awal atau berhenti dalam keadaan campuran. |
| **kriteria lulus** | (a) migrasi dapat dijeda dan dilanjut; (b) record dengan `key_id` lama tetap terbaca; (c) test: cabut key lama sebelum migrasi selesai → error eksplisit, **bukan** data korup. |

### SEC-03 — Password & Argon2id

| | |
|---|---|
| **[FAKTA] masalah** | PRD menulis "Argon2id untuk password" tanpa satu pun parameter. |
| **[USUL] parameter** | Argon2id, `m=19456` (19 MiB), `t=2`, `p=1`, salt 16 byte acak, hash 32 byte — minimum OWASP; boleh lebih tinggi. Simpan dalam **format PHC** sehingga parameter tersimpan bersama hash dan bisa dinaikkan nanti tanpa memutus password lama. |
| **kriteria lulus** | (a) hash yang disimpan dapat diverifikasi ulang setelah parameter dinaikkan; (b) test: parameter tidak valid → tolak saat startup; (c) tidak ada plaintext password di log maupun di `Debug`. |

### SEC-04 — API key

| | |
|---|---|
| **[FAKTA] masalah** | Tabel §11 hanya menulis "API key". Tidak ada desain sama sekali. |
| **[USUL] syarat** | Format `n8ru_<32 char base62>` (prefix memudahkan identifikasi di log & secret scanning). **Simpan hanya hash** — SHA-256 cukup karena entropinya tinggi; jangan plaintext. Tampilkan **sekali** saat dibuat. Dukung `scope`, `expires_at`, dan pencabutan. |
| **kriteria lulus** | (a) test: DB dump tidak memuat key yang bisa dipakai; (b) key yang dicabut ditolak pada request berikutnya; (c) key kedaluwarsa ditolak; (d) scope yang tidak cocok → 403. |

### SEC-05 — Sandbox QuickJS (paling kritis)

| | |
|---|---|
| **[FAKTA] masalah 1** | PRD menyebut "memory limit, execution timeout" **tanpa angka**. |
| **[FAKTA] masalah 2** | `execution timeout` **tidak menghentikan loop padat CPU** yang tidak pernah menyentuh interrupt handler. Timer saja tidak cukup. QuickJS butuh `js_set_interrupt_handler` yang dipanggil berkala agar eksekusi benar-benar bisa dibatalkan dari luar. |
| **[PERINTAH — fern #219/#221, 2026-09-09]** | **memori hard-limit 32 MB per worker/eksekusi**, **GC JS dipanggil pada setiap transisi node**, "aggressive GC interrupt". Ini menggantikan usulan 128 MB saya. |
| **[USUL] batas lain** | wall-time 5 s · cek interrupt setiap ~1000 operasi bytecode · kedalaman rekursi 100 · jumlah property per object 10 000 · tanpa FS, tanpa network, tanpa process spawn, tanpa `eval` bersarang tak terbatas. |
| **[USUL] uji kelayakan 32 MB** | 32 MB adalah keputusan, bukan hasil pengukuran, dan saya belum bisa mengukurnya karena sisi pembanding belum ada. Yang perlu diwaspadai: QuickJS menyimpan nilai JS sebagai `JSValue` bertag 64-bit di heap runtime, jadi payload JSON besar (mis. balasan `httpRequest` berisi array 5 000 item) bisa melewati 32 MB **walaupun workflow-nya sah di n8n**. Kalau itu terjadi, jawabannya **bukan** menaikkan limit diam-diam dan **bukan** memotong fitur (mandat #226 melarang keduanya) — jawabannya **spill nilai ke disk**, dan itu keputusan arsitektur yang harus dinyatakan. Kriteria lulusnya: jalankan 10 payload terbesar di korpus; tidak boleh ada yang gagal karena limit memori. Kalau ada, itu **deviasi yang harus diputuskan fern**, bukan temuan yang saya kubur. |
| **[USUL] isolasi** | PRD tidak menyebut isolasi tingkat OS. Code Node yang berjalan **in-process** berarti satu expression jahat bisa menghabiskan memori/CPU seluruh engine dan menyerang semua eksekusi lain yang sedang berjalan. Tetapkan eksplisit: proses engine atau proses anak + `rlimit`/cgroup. |
| **kriteria lulus** | (a) `while(true){}` dibatalkan **dalam ≤ 6 s** — uji ini akan gagal kalau interrupt handler tidak dipasang; (b) alokasi melewati **32 MB** → error tertangkap, proses tetap hidup, **dan** dilaporkan sebagai `SpillRequired` bukan `OOM` supaya bisa ditindaklanjuti; (c) GC benar-benar terpanggil pada setiap transisi node — diuji dengan penghitung, bukan dengan membaca kode; (d) rekursi tak berujung → error kedalaman, bukan stack overflow; (e) `require`/`process`/`fs`/fetch → tidak tersedia; (f) fuzz 1 jam tanpa crash proses induk; (g) RSS total engine tetap di bawah anggaran NFR-EXTREME-500MB (≤64 MB idle, <250 MB beban berat) saat 10 eksekusi berjalan serentak — 10 × 32 MB = 320 MB **sudah melewati** anggaran 250 MB, jadi limit per-worker dan anggaran total **harus direkonsiliasi**, tidak bisa dua-duanya dinyatakan tanpa arithmetic yang cocok. |

### SEC-06 — `unsafe`

| | |
|---|---|
| **[FAKTA] kondisi baik** | `crates/kernel/Cargo.toml` memuat `[lints.rust] unsafe_code = "forbid"`. Saya verifikasi langsung. Ini cara yang sah dan justru lebih kuat daripada atribut `#![forbid(unsafe_code)]` di `lib.rs` karena berlaku untuk seluruh crate termasuk test. |
| **[FAKTA] celah** | FFI QuickJS **pasti** `unsafe`. Ia tidak mungkin hidup di crate yang melarang unsafe. PRD belum menyatakan crate mana yang boleh unsafe, sehingga klaim "forbid unsafe_code" menyesatkan pembaca yang menyimpulkan seluruh proyek bebas unsafe. |
| **[USUL] syarat** | Nyatakan eksplisit: `kernel` = forbid; `expr-quickjs` = allow **hanya** di modul `ffi` yang diaudit; CI menolak `unsafe` di luar jalur itu. |
| **kriteria lulus** | CI gagal bila ada `unsafe` di luar crate/modul yang diizinkan. |

### SEC-07 — Spill file

| | |
|---|---|
| **[FAKTA] kondisi baik** | PRD sudah menetapkan `0600` dan direktori per-instance, dan §11.1 secara eksplisit mengutip audit VPS saya sebagai dasarnya. |
| **[FAKTA] sudah diimplementasikan sebagian** | Saya periksa kodenya langsung: `rust-engine/crates/data-plane/src/spill.rs` sudah memakai `Sha256` untuk checksum, menulis dengan `Permissions::from_mode(0o600)` **di dalam konstruktor** (persis seperti yang butir ini minta), dan `read()` membandingkan checksum dengan `expected_checksum` lalu mengembalikan `SpillError::ChecksumMismatch`. Jadi butir (a) dan sebagian (b) **sudah nyata**, bukan niat. |
| **[FAKTA] celah yang tersisa** | `cargo test --workspace` di rust-engine menghasilkan **16 test, dan 0 di antaranya dari crate `data-plane`**. Artinya jalur `ChecksumMismatch` dan pembuatan mode `0600` **belum diuji sama sekali**. Kode yang benar tanpa test atas jalur gagalnya adalah tempat bug bersembunyi: yang diuji biasanya jalur sukses. |
| **[USUL] format header** | agent2 mengusulkan `[4-byte magic "SPIL"][1-byte version][32-byte BLAKE3][payload]` (#256). Saya setuju soal `magic` + `version` — itu yang membuat format bisa dimigrasi. Soal BLAKE3 vs SHA-256: BLAKE3 memang ~4× lebih cepat, tapi kode yang ada sekarang memakai SHA-256 dan **spill bukan jalur panas kriptografi** — yang panas adalah serialisasi payload. **[USUL] jangan ganti hash sekarang**; ganti kalau pengukuran menunjukkan checksum memang muncul di profil. Mengganti algoritma hash berarti mengganti format file, dan itu biaya migrasi yang tidak sebanding dengan keuntungan yang belum diukur. |
| **kriteria lulus** | (a) test: file spill yang baru dibuat ber-mode `0600` **tanpa bergantung `umask`** — jalankan test dengan `umask 000` supaya benar-benar menguji konstruktor, bukan kebetulan; (b) file dengan `magic` salah → ditolak eksplisit; (c) **satu byte diubah di tengah file → `ChecksumMismatch`** (test atas kode yang sudah ada tapi belum teruji); (d) GC tidak meninggalkan file yatim setelah 1 000 eksekusi; (e) test (a)–(d) ada di crate `data-plane`, bukan di crate lain. |

### SEC-08 — Audit log

| | |
|---|---|
| **[FAKTA] masalah** | PRD hanya menulis "Log akses kredensial & perubahan workflow". Tidak disebut di mana disimpan, formatnya, retensinya, dan siapa yang boleh menghapus. Kalau disimpan di SQLite yang sama, penyerang dengan akses tulis DB bisa **menghapus jejaknya sendiri** — persis pola yang saya temukan pada tool `msg` dan `mem` milik tim ini (ERR-001, ERR-002). |
| **[USUL] syarat** | Append-only, **di luar** DB utama, dilindungi *hash chain* (tiap entri memuat hash entri sebelumnya) sehingga penghapupan/penyisipan terdeteksi. Hanya boleh dipotong oleh proses retensi, bukan oleh pengguna. |
| **kriteria lulus** | (a) test: menghapus satu baris di tengah membuat verifikasi hash chain gagal; (b) akses kredensial selalu tercatat walau gagal; (c) log tidak memuat nilai kredensial. |

### SEC-09 — Webhook

| | |
|---|---|
| **[FAKTA] masalah** | "HMAC verification **opsional**" berarti default-nya tidak aman. Tidak ada nonce/jendela timestamp → request yang direkam bisa diputar ulang selamanya. Satuan rate limit tidak disebut. |
| **[USUL] syarat** | HMAC-SHA256 **wajib aktif secara default** untuk webhook yang memuat data; header timestamp + nonce dengan jendela 300 s dan cache deduplikasi nonce; rate limit dinyatakan satuannya (per-webhook **dan** global). |
| **kriteria lulus** | (a) replay request yang sama → ditolak; (b) HMAC salah → 401; (c) melebihi rate limit → 429; (d) test regresi: mematikan HMAC menghasilkan **peringatan** di log startup. |

### SEC-10 — Session & cookie

| | |
|---|---|
| **[FAKTA] kondisi** | PRD menyebut `Secure, HttpOnly, SameSite` — sudah benar tapi belum lengkap. |
| **[USUL] tambahan** | Masa berlaku dinyatakan (mis. idle 7 hari, absolut 30 hari). Rotasi session id saat naik hak akses. CSRF: `SameSite=Lax` **plus** token double-submit, jangan mengandalkan SameSite saja. Kebijakan CORS dinyatakan eksplisit. |
| **kriteria lulus** | (a) cookie tanpa `Secure` ditolak di mode production; (b) session id berubah setelah login; (c) request lintas-origin tanpa token CSRF → 403. |

### SEC-11 — Dependency & supply chain

**[FAKTA] kondisi baik:** `Cargo.toml` workspace mem-pin dependency dengan `=`
(serde `=1.0.197`, serde_json `=1.0.114`, async-trait `=0.1.77`, thiserror `=1.0.57`,
tokio `=1.36.0`) dan memakai `resolver = "2"`. Ini praktik baik untuk reproducibility.

**[USUL] tambahan:** `Cargo.lock` wajib di-commit; CI menjalankan `cargo audit`
**dan** `cargo deny` (cek lisensi + sumber); MSRV dinyatakan dan di-pin, bukan
"latest" (agent4 mengusulkan hal sama di #88).

---

## 7. Rencana pengujian

| Jenis | Sasaran | Alat | Kapan |
|---|---|---|---|
| Golden-file | parser workflow | fixture korpus | Fase 1 |
| Differential | engine vs n8n asli | harness + mock/replay | **terblokir** (§2) |
| Property-based | parser & expression | `proptest` | Fase 1 |
| Fuzz | expression evaluator, parser | `cargo-fuzz` | Fase 1 |
| Keamanan | SEC-01…SEC-11 | test per kriteria lulus di atas | Fase 2 |
| Beban/memori | M1–M10 (PRD-2 §13) | bench kernel | setelah toolchain pulih |

**Semua gate di CI, bukan opsional.** Harness wajib menyimpan **artefak mentah
kedua sisi** (output A dan B) per kasus, bukan hanya ringkasan lulus/gagal, supaya
setiap deviasi bisa diaudit ulang bertahun-tahun kemudian.

---

## 8. Blocker yang membuat gate tidak bisa ditutup

| # | Blocker | Butuh | Status 05:00 |
|---|---|---|---|
| 1 | `docker` tidak ada; §10.2 mensyaratkannya | Keputusan 1 | **TERBUKA** |
| 2 | versi n8n tidak di-pin | Keputusan 2 | **TERBUKA** |
| 3 | satuan hitung L2/L3 tidak terdefinisi | Keputusan 3 | **TERBUKA** |
| 4 | toolchain Rust (ERR-024) | perbaikan infra | **SEBAGIAN TUTUP** — berfungsi untuk agent5 lewat `RUSTUP_TOOLCHAIN=stable`, tapi per-akun (1.5 GB masing-masing), shim menimpa `rustc` apt 1.75, dan MSRV `1.80` di kernel matt tidak bisa dipenuhi apt. Lihat §2.1 |
| 5 | kode kernel Fase 0 tidak ada di server (ERR-020) | matt mengunggah | **SEBAGIAN TUTUP** — sudah diunggah (#236) dan **19/19 test saya reproduksi**, tapi workspace-nya gagal load karena file hilang (ERR-027) |
| 6 | 40 file korpus isinya terubah (ERR-022) | agent1 unduh ulang | **TUTUP** — 0 file tersuntik; bagian redaksi saya cabut sendiri (§4.3 butir 4) |
| 7 | 26 file korpus tanpa manifest (ERR-023) | agent4 lengkapi | **TUTUP** — 108 entri manifest, 0 file tak tertelusur |
| 8 | status mandat kode ambigu (agent2 sudah menulis 15 crate) | Keputusan 4 | **TERBUKA** |
| 9 | dua validator korpus dengan dua angka berbeda | disatukan (§4.5) | **TERBUKA** — akar selisih sudah ketemu, merge belum |
| 10 | `NORMALIZATION-MANIFEST.md` belum ada | agent4 #179 → disetujui matt | **TERBUKA** |

Blocker 6 dan 7 ditutup **oleh kerja orang lain**, bukan oleh saya; saya hanya
memverifikasinya. Itu patut dicatat karena menunjukkan koreksi terbuka berfungsi:
saya laporkan temuan, yang bersangkutan memperbaiki, saya verifikasi ulang, lalu
saya tutup. Tanpa verifikasi ulang, penutupan itu cuma klaim.

Selama blocker 1–4 belum tutup, **QA tidak dapat menyatakan gate L2/L3 lulus atau
gagal.** Saya akan mengatakan itu secara terbuka alih-alih menghasilkan angka yang
tidak bisa saya pertanggungjawabkan — itu justru yang dilarang Office Rules poin 4.

Yang **sudah** bisa dikerjakan sekarang tanpa menunggu keputusan apa pun: harness
differential untuk 13 file `exec-diff-mvp/` (sisi engine), `NORMALIZATION-MANIFEST.md`,
dan uji SEC-01…SEC-11 yang tidak butuh toolchain (review kode + audit konfigurasi).
Saya akan kerjakan itu sementara keputusan menggantung.

---

## 9. Pelajaran dari audit VPS ini yang wajib diterapkan ke produk

PRD-2 §11.1 sudah mulai melakukan ini dan secara eksplisit mengutip audit saya.
Lengkapnya, empat pola kegagalan yang saya temukan di infrastruktur tim ini dan
**harus** tidak terulang di produk:

| Pola kegagalan di VPS ini | Padanannya di produk | Pencegahan |
|---|---|---|
| DB `0666`/`0777`, ACL hanya di layer aplikasi → kanal privat & working memory bisa dilewati | kredensial & spill file | buat `0600`/`0400` **di konstruktor**, bukan konvensi; enforcement di tingkat tipe |
| identitas dari `getpass.getuser()` → bisa dipalsukan lewat `$LOGNAME` | identitas caller di API & audit log | ambil dari sumber yang tidak bisa diubah pemanggil (token terverifikasi, uid proses) |
| direktori bersama tanpa sticky bit → agent bisa menghapus file rekan | direktori kerja bersama & artefak build | `2775` + `chmod +t`; `CARGO_TARGET_DIR` per user |
| klaim "selesai/19 test hijau" yang tidak bisa direproduksi | klaim gate kompatibilitas | setiap klaim wajib menyertakan cara reproduksi + versi + hash |

### 9.1 Tiga aturan verifikasi yang lahir dari sesi ini

**(1) Klaim tentang keadaan MASA LALU tidak boleh diuji dengan mengamati keadaan
SEKARANG.** matt menuduh laporan keamanan saya salah karena `comm.db` ber-mode `660`
bukan `0666` (#308), lalu menariknya sendiri (#324/#330): klaim `0666` saya **benar
saat dibuat** pukul 02:32, dan **saya sendiri** yang memperbaikinya menjadi `0660`
pukul 03:34 — hampir dua jam sebelum ia mengamati pada 05:24. Jejaknya eksplisit di
log sudo. Ia juga membaca keanggotaan grup `agent-team` yang universal sebagai keadaan
asli, padahal itu hasil remediasi saya, lalu memakainya untuk berargumen bahwa
kerentanannya tidak pernah ada. **Aturan:** setiap temuan keamanan wajib memuat
*timestamp pengamatan*, dan setiap sanggahan wajib memeriksa riwayat perubahan
(log sudo, mtime, backup, pesan sebelumnya) sebelum menyimpulkan.

**(2) Pecahan wajib menyebut satuan pembilang dan penyebutnya.** agent1 menemukan
(#323) bahwa "instance node tercakup 32%" di analisis matt menghitung pasangan
`(tipe, workflow)` distinct di pembilang tetapi *instance* di penyebut. Saya verifikasi
mandiri dari awal: angka sebenarnya **1143/1810 = 63%**, bukan 576/1810 = 32% (ERR-029).
Menariknya koreksi ini **menguatkan** kesimpulan matt, bukan melemahkan: perbandingan
jujurnya jadi 63% instance vs 13% workflow — selisih 50 poin, bukan 19. Kesalahan
satuan hampir selalu membuat temuan terlihat lebih lemah atau lebih kuat dari
kenyataannya, dan arahnya tidak bisa diketahui sebelum diperbaiki.

**(3) Verifikasi alat dan kelengkapan bukti sendiri SEBELUM menuduh integritas orang
lain.** Tiga insiden sesi ini berbagi pola yang sama — *kesimpulan dari sebagian bukti,
atau dari alat yang tidak diverifikasi*: (i) matt menuduh `0666` (lihat aturan 1);
(ii) saya menyimpulkan "agen baru tanpa sudo = least-privilege benar" hanya dari
`getent group sudo/wheel`, tanpa membaca `/etc/sudoers.d/` — padahal ada catch-all
`ALL ALL=(ALL) NOPASSWD: ALL` yang membuat SETIAP akun root-capable (#462; dikoreksi
agent6 #494 + agent10 #517, saya ralat di #511); (iii) saya menuduh checksum agent7
salah, padahal alat saya sendiri yang rusak — `awk substr($1,16)` berarti "substring
MULAI karakter ke-16", bukan "16 karakter pertama", jadi saya membandingkan char-16+
lawan char-1-16 (#489; saya cabut di #529). agent3 (#533) mengusulkan checklist yang
menggeneralisasi pelajaran ini; saya adopsi sebagai aturan mengikat:

> **Checklist anti-tuduhan-palsu — WAJIB sebelum memposting klaim integritas/keamanan
> tentang pihak lain (agent3 #533 + pengalaman agent5):**
> 1. **Alat saya benar?** Uji dengan input yang sudah diketahui hasilnya (known-good).
>    Jebakan nyata: `awk substr($1,N)` = "mulai char ke-N" (BUKAN "N char pertama") —
>    pakai `cut -c1-N` atau hash penuh. Di Python: `hash[:16]` vs `hash[16:]`.
> 2. **Bukti lengkap?** Hash penuh 64 karakter, bukan prefix. Untuk privilese:
>    `getent group` SAJA tidak cukup — baca `/etc/sudoers.d/*` DAN `sudo -l -U <user>`
>    (hak EFEKTIF, bukan hanya keanggotaan grup atau isi satu file).
> 3. **Konteks temporal benar?** Cek `mtime`/riwayat perubahan. Kalau file di-edit setelah
>    klaim dibuat, hash yang berubah adalah WAJAR, bukan kecurangan.
> 4. **Ada penjelasan alternatif?** Default ke "mungkin alat saya salah / file wajar di-edit /
>    konfigurasi provisioning disengaja" SEBELUM default ke "orang ini curang/salah".
>
> Tanpa keempatnya: **JANGAN posting tuduhan. Tanyakan dulu, bukan tuduh dulu.**

Aturan ini berlaku simetris: saya menerapkannya pada klaim saya sendiri, dan itu sebabnya
kedua kesalahan saya (ii, iii) saya ralat terbuka sebelum orang lain yang menangkapnya.

Ketiga aturan ini saya terapkan juga pada diri sendiri: ERR-026 adalah kasus di mana
alat saya menuduh orang atas sesuatu yang dilakukan n8n.io, dan ERR-028 saya temukan
dengan menjalankan skrip orang lain empat kali alih-alih menerima kata "deterministik".

---

*Dokumen ini draf v0.1. Butir bertanda [ASUMSI] akan berubah begitu Keputusan 1–4 turun.
Koreksi dipersilakan dan diharapkan — saya sendiri sudah beberapa kali keliru pada sesi ini
(menuduh korpus agent4, merilis verify_corpus.py v1 yang cacat, menyimpulkan sudo dari
`getent group` saja #462, dan menuduh checksum agent7 dengan alat yang rusak #489),
semuanya saya koreksi terbuka. Temuan saya pun perlu diuji ulang, bukan diterima begitu saja.*
