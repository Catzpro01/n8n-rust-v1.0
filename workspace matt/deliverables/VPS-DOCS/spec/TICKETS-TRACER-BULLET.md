# TICKETS-TRACER-BULLET v1.0

**Disusun memakai skill `to-tickets` dari `mattpocock/skills` v1.2.3.**
**Tanggal: 2026-09-10** · **Status: DISETUJUI — granularitas dan tepi blokir sudah dikonfirmasi Pemilik Produk**
**Sumber: `PRD-FRONTEND-RUST.md`, `CLI-PARITY-SPEC.md`, `REPLAN-BACKEND-MATANG.md`**

> **Kata pemandu: _tracer bullet_.** Setiap tiket menembus semua lapisan, dari penyimpanan sampai layar.
> Selesai satu tiket = Pemilik Produk bisa **melihatnya bekerja**. Bukan "lapisannya sudah ada".
>
> **Yang dilarang: irisan horizontal.** Tiket bernama "bangun lapisan X" tidak boleh ada lagi. Itu yang
> menghasilkan 31/39 DONE tanpa satu workflow pun yang bisa dijalankan.

---

## Milestone

| Milestone | Isi | Definisi selesai |
|---|---|---|
| **M0 — Berjalan** | TB-01 | Satu workflow hardcoded tereksekusi, JSON keluar di terminal |
| **M1 — Bisa dipakai via CLI** | TB-02, TB-03 | Workflow n8n asli diimpor, dijalankan, diekspor. Ekspresi `{{ }}` bekerja |
| **M2 — Ada server & API** | TB-04 | `n8n-rust start` melayani REST + `--help` paritas untuk 6 perintah |
| **M3 — Ada UI baca** | TB-05 | Daftar workflow & hasil eksekusi terlihat di peramban |
| **M4 — UI bisa mengedit** | TB-06, TB-07a, TB-07b, TB-07c | Workflow bisa dibuat & disunting: formulir node, lalu kanvas |
| **M5 — Produksi sendiri** | TB-08, TB-09, TB-10 | Kredensial, auth, trigger jadwal & webhook, publish |

**Keputusan Pemilik Produk 2026-09-10: M1 lalu langsung M3, tanpa berhenti di antaranya.**
Alasan yang beliau setujui: M3 membuktikan Leptos sanggup hidup di VPS 2GB **sebelum** kita bertaruh pada
kanvas di M4. Berhenti di M1 berarti menunda satu-satunya uji nyata bahwa stack frontend-nya viable.

---

## Keputusan yang sudah ditutup (langkah 4 skill `to-tickets` — SELESAI)

Empat pertanyaan diajukan memakai format ronde skill `grilling`. Keempatnya dijawab, keempatnya menerima
rekomendasi.

| # | Pertanyaan | Keputusan | Yang berubah di dokumen ini |
|---|---|---|---|
| 1 | Jumlah seam pengujian | **Dua**: proses CLI + HTTP API | TB-05 tidak lagi memakai Seam 3. Logika yang layak uji wajib turun ke API. |
| 2 | Paritas `--help` | **Enam perintah Stage 1 saja** | Kriteria TB-04 dipersempit dari 24 ke 6 |
| 3 | Batas milestone pertama | **M1 lalu langsung M3** | Lihat catatan milestone di atas |
| 4 | Granularitas tiket kanvas | **Dipecah tiga** | TB-07 → TB-07a, TB-07b, TB-07c |

**Enam perintah yang wajib paritas `--help`:** `--help` root, `start`, `import:workflow`, `list:workflow`,
`execute`, `export:workflow`. Sisanya (18 perintah) tetap ada di `CLI-PARITY-SPEC.md` sebagai utang yang
sadar, bukan yang terlupa.

---

## TB-01 · Satu workflow menembus semua lapisan

**What to build:** Pengguna menjalankan satu perintah, satu workflow dua-node (Manual Trigger → Set)
tereksekusi, dan keluaran JSON tercetak di terminal. Belum ada CLI yang lengkap, belum ada HTTP, belum ada
UI — tapi **jalur lengkap dari model workflow → eksekusi → node → keluaran sudah ditembus.**

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Ada binary yang bisa dijalankan: `cargo run --bin <apa pun> -- --demo`
- [ ] Workflow dua-node (Manual Trigger → Set) dieksekusi dan menghasilkan satu item dengan field yang
      ditetapkan node Set
- [ ] Keluaran tercetak sebagai JSON valid ke stdout
- [ ] Node `Set` diimplementasikan terhadap trait `Node` yang ada di kernel — **bukan** jalur pintas hardcoded
- [ ] Ada SATU mutan yang membuktikan test bisa gagal, dengan `sha-before ≠ sha-after` dibuktikan sebelum
      membaca hasil
- [ ] Perintah verifikasi dilaporkan **lengkap dengan argumennya** (`cargo test --all-targets`, bukan `--lib`)

---

## TB-02 · Impor, daftar, jalankan, ekspor workflow n8n asli

**What to build:** Pengguna mengambil file JSON hasil ekspor n8n 2.39.0 asli, mengimpornya, melihatnya
terdaftar, menjalankannya, dan mengekspornya kembali — lewat CLI dengan nama perintah dan flag yang sama
persis dengan n8n.

**Blocked by:** TB-01

**Status:** ready-for-agent

- [ ] `n8n-rust import:workflow --input=<file.json>` menyimpan workflow ke SQLite dan mencetak ID-nya
- [ ] `n8n-rust list:workflow` menampilkan ID dan nama
- [ ] `n8n-rust execute --id=<ID> --rawOutput` menjalankan dan mencetak JSON
- [ ] `n8n-rust export:workflow --id=<ID> --output=out.json` menulis file yang **bisa diimpor n8n asli
      kembali** (uji round-trip)
- [ ] Fixture: minimal 2 workflow nyata hasil ekspor n8n 2.39.0, disimpan di repo. **Fixture harus berasal
      dari ekspor n8n NYATA, bukan ditulis agar cocok dengan implementasi** — lihat catatan di bawah
- [ ] `--help` tiap perintah di atas cocok dengan n8n asli (lihat `CLI-PARITY-SPEC.md`)
- [ ] **Node `Set` menerima KEDUA bentuk parameter**: `fields.values[]` (legacy) DAN `assignments.assignments[]`
      (kanonik V2). Bentuk kanonik dikonfirmasi di upstream `Set/v2/manual.mode.ts` baris 170
- [ ] **Parameter `include` ditangani**, minimal `include=none` dan `include=all`
- [ ] **Parameter `duplicateItem` ditangani atau ditolak secara eksplisit** — diabaikan tanpa suara DILARANG
- [ ] Fixture `workflow-a-manual-to-set.json` jalan **tanpa diubah**
- [ ] **Kode keluar tidak nol saat sebuah node gagal.** _Kriteria ini dipertahankan, tapi alasan aslinya
      DICABUT karena salah faktual — lihat koreksi di bawah._ Binary TB-01 **sudah** mengembalikan `exit=1`
      saat node gagal dan `exit=0` saat sukses. Yang belum diuji adalah binary `n8n-rust execute` yang
      sesungguhnya di tiket ini

> **KOREKSI 2026-09-10 — klaim awal di butir ini salah.** Ditulis semula sebagai "binary TB-01 mengembalikan
> `exit=0` walaupun node gagal". Diuji langsung: `tb01` terhadap fixture yang membuat node gagal →
> **EXIT=1**, pesan ke stderr; terhadap fixture sukses → **EXIT=0**. Sumber di `tb01.rs` memang memanggil
> `process::exit(1)`. Klaim palsu itu berasal dari laporan agen yang matt teruskan ke dokumen mengikat tanpa
> menguji jalur gagalnya. **Butirnya tetap ada karena keputusannya masih terbuka** — lihat pertanyaan paritas
> di bawah — tapi alasannya bukan "ini sudah rusak".

> **PERTANYAAN PARITAS YANG TERBUKA (kategori C, keputusan Pemilik Produk).** agent9 menemukan dan agent1
> memverifikasi independen: **CLI n8n asli TIDAK mengubah kode keluar saat node gagal.** Di `execute.ts`
> satu-satunya `process.exit(1)` adalah untuk workflow-id-tidak-ditemukan (baris 67–70); jalur node-gagal
> (baris 124–137) melempar error yang ditangkap `catch` sendiri (baris 144–151) yang hanya mencatat ke log
> tanpa rethrow dan tanpa exit — jadi proses berakhir normal. Satu-satunya pembeda adalah teks keluaran.
>
> Artinya binary kita **sudah menyimpang** dari n8n asli.
>
> **DIPUTUSKAN 2026-09-10 oleh Pemilik Produk: divergensi disengaja.** Kode keluar tidak-nol saat node gagal.
> Alasannya: pemakai menjalankan ini lewat skrip dan cron, dan `exit 0` pada kegagalan membuat kegagalan tidak
> terdeteksi oleh apa pun di luar teks. Terdaftar sebagai **D-1 di `CLI-PARITY-SPEC.md` §DIVERGENSI SADAR** —
> bukan dibiarkan sebagai drift. Langkah 3 TB-02 tidak lagi terblokir.
- [ ] `workflow-b-set-chain-expressions.json` **tidak** diwajibkan hijau di tiket ini. Bagian ekspresinya milik
      "TB-03 Ekspresi dan node HTTP Request". Lihat catatan batas di bawah
- [ ] Satu mutan, bukti sha

> **Kriteria tiga butir di atas ditambahkan 2026-09-10 setelah verifikasi ke upstream**, bukan ada sejak awal.
> TB-01 sudah hijau dengan `SetNode` yang hanya membaca `fields.values[]`, dan `grep include|duplicateItem`
> di `nodes-core` menghasilkan **nol kemunculan** — keduanya diabaikan tanpa suara. TB-02 akan gagal di hari
> pertama kalau ini tidak ditangkap.
>
> **Prinsip yang berlaku untuk SEMUA node ke depan:** n8n nyata punya beberapa bentuk parameter per
> `typeVersion`. Node yang hanya menerima satu bentuk akan lolos test-nya sendiri lalu gagal di workflow
> pengguna. **Mengabaikan parameter yang tidak dikenal tanpa suara adalah cacat**, bukan kemudahan — kalau
> belum didukung, kembalikan error yang menyebut namanya.

---

## TB-03 · Ekspresi `{{ }}` dan node HTTP Request

**What to build:** Pengguna menulis `{{ $json.nama }}` di parameter node, dan nilainya benar-benar terisi saat
eksekusi. Pengguna bisa memanggil URL eksternal dan memakai jawabannya di node berikutnya.

**Blocked by:** TB-02

**Status:** ready-for-agent

- [ ] Node `HTTP Request` berfungsi (GET dan POST) lewat Seam 1 — diuji terhadap server lokal sementara,
      **bukan** mock internal
- [ ] Ekspresi `{{ $json.x }}` terevaluasi; hasil salah terekspresi sebagai error yang menyebut nama node
- [ ] Node `If` dan `Merge` berfungsi (percabangan minimum yang membuat workflow nyata berguna)
- [ ] Semantik `executionOrder` sesuai n8n 2.x (per-branch), **bukan** breadth-first v0
- [ ] Set golden dari n8n asli: ≥5 ekspresi dengan keluaran terkunci byte-per-byte
- [ ] Satu mutan, bukti sha

---

## TB-04 · Server HTTP dan API publik

**What to build:** Pengguna menjalankan `n8n-rust start`, server hidup di port yang ditentukan, membuat
direktori data, menjalankan migrasi, dan melayani REST API yang bisa dipakai CLI maupun UI.

**Blocked by:** TB-02 (paralel dengan TB-03 diperbolehkan)

**Status:** ready-for-agent

- [ ] `n8n-rust start` hidup di VPS 2GB tanpa OOM; mencetak URL yang bisa diklik
- [ ] REST: daftar workflow, ambil satu, buat, jalankan, ambil hasil eksekusi
- [ ] OpenAPI 3.1 dihasilkan dari kode (crate `openapi-codegen` sudah ada dan golden-nya terbukti mengikat)
- [ ] **`--help` dicetak persis seperti n8n asli untuk ENAM perintah**: `--help` root, `start`,
      `import:workflow`, `list:workflow`, `execute`, `export:workflow` — **diputuskan Pemilik Produk
      2026-09-10; sebelumnya 24, dipersempit**
- [ ] Delapan belas perintah sisanya tetap ada dan tetap berfungsi, tapi teks `--help`-nya **tidak** diwajibkan
      paritas pada milestone ini
- [ ] Setiap endpoint punya test lewat Seam 2 (server nyata di port acak)
- [ ] Satu mutan, bukti sha

---

## TB-05 · UI baca: daftar workflow dan hasil eksekusi

**What to build:** Pengguna membuka peramban, melihat daftar workflow, mengeklik satu, melihat riwayat
eksekusinya, dan melihat data tiap node dalam satu eksekusi. **Belum bisa mengedit apa pun** — itu TB-06.

**Blocked by:** TB-04

**Status:** ready-for-agent

- [ ] Leptos SSR: halaman dimuat dengan HTML dari server (bukan layar kosong lalu WASM mengisi)
- [ ] Daftar workflow tampil dengan nama, status, dan waktu eksekusi terakhir
- [ ] Detail eksekusi menampilkan status, durasi, dan **data per-node**
- [ ] Bundle WASM terukur dan dilaporkan (target < 200KB gzipped; baseline Leptos ~90KB)
- [ ] **Tidak ada logika yang layak uji hidup di frontend.** Setiap aturan, validasi, atau transformasi diuji
      lewat Seam 1 atau Seam 2. Kalau ada yang tidak bisa, **turunkan ke API** — Seam 3 sudah dibuang oleh
      keputusan Pemilik Produk
- [ ] Satu mutan, bukti sha

---

## TB-06 · UI edit: palet node dan formulir parameter

**What to build:** Pengguna membuat workflow baru di peramban, menambah node dari palet, mengisi parameternya
lewat formulir yang dihasilkan dari `ParameterSchema`, dan menyimpan. **Belum ada kanvas** — daftar node
berurutan sudah cukup untuk membuktikan jalur tulis tembus.

**Blocked by:** TB-05

**Status:** ready-for-agent

- [ ] Palet node dengan pencarian, dibangun dari deskripsi node yang sudah ada di backend
- [ ] Formulir parameter **dihasilkan dari `ParameterSchema`**, bukan ditulis per-node
- [ ] Simpan workflow baru dan perbarui yang ada lewat API yang sama dengan CLI
- [ ] Workflow yang disimpan lewat UI **bisa dijalankan lewat CLI** — bukti kedua jalur memakai model yang sama
- [ ] Satu mutan, bukti sha

---

## TB-07a · Kanvas: graf terlihat

**What to build:** Pengguna membuka satu workflow dan **melihat** grafnya: node sebagai kartu, koneksi sebagai
garis lengkung, di posisi yang tersimpan. Belum bisa disentuh — hanya dilihat. Ini membuktikan Leptos sanggup
menggambar graf sebelum kita membangun interaksinya.

**Blocked by:** TB-06

**Status:** ready-for-agent

- [ ] Node dirender sebagai **SVG** (bukan `<canvas>`) — lihat PRD §keputusan kanvas
- [ ] Posisi node dibaca dari model workflow (`position: [x,y]`, skema n8n) — **tidak ada skema tata letak baru**
- [ ] Koneksi digambar sebagai path Bézier kubik, sama seperti n8n
- [ ] Workflow yang punya ≥10 node tetap terbaca dan tidak berkedip saat dimuat
- [ ] **Ukuran graf nyata diukur**: berapa elemen SVG untuk workflow 10 node, dan berapa milidetik untuk
      merendernya. Angka dilaporkan, bukan diperkirakan
- [ ] Satu mutan, bukti sha

---

## TB-07b · Kanvas: node bisa digeser

**What to build:** Pengguna menyeret node ke tempat lain, dan posisinya **tersimpan** — buka lagi workflow-nya,
node tetap di tempat baru.

**Blocked by:** TB-07a

**Status:** ready-for-agent

- [ ] Seret node memperbarui posisinya di layar tanpa jeda yang terlihat
- [ ] Posisi tersimpan lewat API yang sama dengan CLI — **bukan** disimpan hanya di memori peramban
- [ ] **Mengubah posisi satu node tidak me-render ulang seluruh graf** — diukur dan dilaporkan, bukan
      diasumsikan. Ini alasan Leptos dipilih; kalau ternyata gagal di sini, keputusannya perlu ditinjau ulang
- [ ] Seret tidak ikut menggerakkan koneksi ke node lain secara salah
- [ ] Satu mutan, bukti sha

---

## TB-07c · Kanvas: node bisa disambung dan dihapus

**What to build:** Pengguna menarik garis dari port keluar satu node ke port masuk node lain dan alurnya
terbentuk. Pengguna juga bisa menghapus node dan menghapus koneksi. **Di titik ini workflow bisa dibangun
sepenuhnya di peramban** — setara pengalaman n8n asli.

**Blocked by:** TB-07b

**Status:** ready-for-agent

- [ ] Menarik dari port keluar ke port masuk membuat koneksi yang tersimpan
- [ ] Melepas tarikan di tempat kosong membatalkan, tanpa koneksi menggantung
- [ ] Menghapus node ikut menghapus koneksi yang menempel padanya — tidak ada koneksi yatim
- [ ] Menghapus satu koneksi tidak menghapus node di ujungnya
- [ ] Workflow yang dibangun di kanvas **bisa dijalankan lewat CLI** — bukti bahwa kanvas menulis ke model
      yang sama, bukan ke salinan terpisah
- [ ] Satu mutan, bukti sha

---

## TB-08 · Kredensial dan autentikasi

**What to build:** Pengguna menyimpan kredensial sekali, memakainya di banyak node, dan harus login untuk
mengakses instance-nya sendiri.

**Blocked by:** TB-06 (paralel dengan TB-07a/07b/07c diperbolehkan)

**Status:** ready-for-agent

- [ ] Simpan, daftar, hapus kredensial
- [ ] Nilai kredensial **tidak pernah** kembali ke klien dalam bentuk terbaca setelah disimpan
- [ ] Node bisa mengambil kredensial saat eksekusi
- [ ] Login email + kata sandi; sesi; akses ditolak tanpa sesi
- [ ] Uji lewat Seam 2 — termasuk test yang mencoba mengambil kembali nilai kredensial dan **harus gagal**
- [ ] Satu mutan, bukti sha

---

## TB-09 · Trigger: jadwal dan webhook

**What to build:** Workflow berjalan sendiri tanpa disentuh: pada jadwal cron saat server hidup, dan saat ada
HTTP masuk dari sistem luar.

**Blocked by:** TB-04, TB-06

**Status:** ready-for-agent

- [ ] Trigger cron menjadwalkan dan mengeksekusi workflow saat server hidup
- [ ] Trigger webhook membuka endpoint publik yang bisa dipanggil dari luar
- [ ] Eksekusi dari trigger tercatat di riwayat sama seperti eksekusi manual
- [ ] Diuji lewat Seam 1/2 dengan jam yang bisa dikendalikan, **bukan** `sleep` di test
- [ ] Satu mutan, bukti sha

---

## TB-10 · Publish, unpublish, dan status server

**What to build:** Pengguna menerbitkan workflow agar aktif, membatalkannya, dan melihat keadaan sistem di
satu halaman. Menutup paritas CLI n8n 2.0.

**Blocked by:** TB-09

**Status:** ready-for-agent

- [ ] `publish:workflow` / `unpublish:workflow` di CLI **dan** di UI (n8n 2.0 menggantikan active/inactive)
- [ ] Hanya workflow ter-publish yang dijalankan oleh trigger
- [ ] Halaman status: versi, jumlah workflow, eksekusi aktif, ukuran database
- [ ] Menjalankan ulang satu eksekusi yang gagal
- [ ] Satu mutan, bukti sha

---

## Frontier saat ini

Tiket yang blokirnya sudah selesai dan **bisa langsung dimulai**:

```
TB-01   (tidak ada blokir)
```

Setelah TB-01 hijau: **TB-02**. Setelah TB-02: **TB-03 dan TB-04 paralel**.

> Skill `to-tickets`: *"Work the frontier: any ticket whose blockers are all done."*
> **Frontier sekarang berisi satu tiket.** Artinya sebagian besar agen menganggur sampai TB-01 dan TB-02
> tembus — dan itu **benar**, bukan pemborosan. Mempercepat dengan memberi semua agen lapisan terpisah
> adalah persis yang sudah gagal.

### Rantai lengkap setelah pemecahan kanvas

```
TB-01 -> TB-02 -> TB-03
                \
                 -> TB-04 -> TB-05 -> TB-06 -> TB-07a -> TB-07b -> TB-07c -> TB-09 -> TB-10
                                        \
                                         -> TB-08 (paralel dengan 07a/07b/07c)
```

Jumlah tiket: **12** (sebelumnya 10, kanvas dipecah tiga). Satu-satunya jalur kritis menuju M5 melewati
kanvas — itu alasan pemecahannya dilakukan sekarang, bukan saat sudah macet di tengah.

---

## Prefactor yang harus dilakukan lebih dulu

Skill `to-tickets`: *"Look for opportunities to prefactor the code to make the implementation easier. Make
the change easy, then make the easy change."*

Prefactor yang saya identifikasi — **semuanya harus selesai sebelum atau di dalam TB-01/TB-02:**

1. **Isi sembilan stub crate atau hapus dari workspace.** Stub satu baris membuat `cargo build --workspace`
   hijau sambil menyembunyikan bahwa tidak ada kode. Ini yang membuat status "selesai" bisa dipercaya salah.
2. **Kunci kosakata domain.** Daftar kanonik di PRD. Setiap crate yang memakai nama lain harus disamakan
   sekarang, bukan nanti saat 10 agen menulis kode baru.
3. **Satu jalur penyimpanan.** `SpillStore`/`SpilledList`/`ContentId` sudah ada di kernel — pastikan
   penyimpanan SQLite TB-02 memakai kontrak itu, bukan membuat yang kedua.


---

## Catatan batas TB-02 ↔ TB-03 (ditambahkan 2026-09-10)

Fixture agent9 `workflow-b-set-chain-expressions.json` memuat ekspresi `{{ }}`, dan README fixture itu
menyebut keluarannya sebagai "kontrak end-to-end TB-02". **Itu melampaui batas tiket yang sudah disetujui.**
Tiket yang memerintah, bukan README fixture.

Ekspresi `{{ }}` adalah lingkup **TB-03**. Maka di TB-02:
- `workflow-a-manual-to-set.json` (murni Set kanonik, tanpa ekspresi) **wajib hijau**
- `workflow-b-set-chain-expressions.json` **boleh tetap merah** pada bagian ekspresinya

Alasannya bukan mempermudah agent1. Kalau ekspresi minimal diselundupkan ke TB-02 supaya satu fixture hijau,
maka TB-03 kehilangan test merahnya — dan kita mengulangi penyakit yang sama: memindahkan pekerjaan ke tiket
yang sudah dinyatakan selesai.

Fixture agent9 tetap bernilai penuh; ia hanya **terbukti lebih awal dari tiket yang menampungnya**, yang
justru bagus karena itu berarti TB-03 lahir dengan test merah yang sudah siap.
