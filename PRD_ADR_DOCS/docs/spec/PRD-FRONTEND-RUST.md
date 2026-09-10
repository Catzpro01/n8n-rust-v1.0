# PRD-FRONTEND-RUST v1.0

**Disusun memakai skill `to-spec` dari `mattpocock/skills` v1.2.3** (MIT).
**Tanggal: 2026-09-10** · **Oleh: matt (Lead Architect)** · **Status: menunggu persetujuan Pemilik Produk**

Perintah Pemilik Produk: *"saya mau front endnya dari rust jadi buat prdnya."* Ini menggantikan keputusan
sebelumnya bahwa UI dibangun di Arena. **UI sekarang bagian dari produk, ditulis dalam Rust.**

---

## Diagnosa yang membentuk PRD ini

Skill `to-tickets` Pocock mendefinisikan **tracer bullet**: setiap tiket memotong **jalur sempit tapi
LENGKAP menembus setiap lapisan** (skema, API, UI, test) — **vertikal, BUKAN irisan horizontal satu lapisan.**
Selesai satu tiket = **bisa didemokan atau diverifikasi sendiri.**

**Antrean lama proyek ini adalah irisan horizontal seluruhnya.** Lihat namanya:
```
W2-STORAGE-L0    "Storage Layer 0 SQLite WAL dan Binary SpillStore"   <- satu lapisan
W1-MCP-IMPL      "Migrasi dan implementasi crates/mcp/"               <- satu lapisan
W1-WCB-BRIDGE    "WASM Community Node Bridge"                         <- satu lapisan
W1-ROSETTA-IMPL  "Compiler Rosetta paritas 694 nodes"                 <- satu lapisan
W3-EXEC-ENVELOPE "Execution Envelope dan Rolling Hash-Chain Audit"    <- satu lapisan
```
Tidak satu pun bisa didemokan sendiri. **Itu sebabnya 31/39 task DONE menghasilkan nol workflow yang bisa
dijalankan.** Setiap lapisan dikerjakan sampai "selesai", dan tidak ada lapisan yang pernah disambungkan ke
lapisan lain sampai tembus ke pengguna.

> **PRD ini menyusun ulang seluruh pekerjaan sebagai tracer bullet.** Bukan "bangun storage, lalu executor,
> lalu API, lalu UI" — itu irisan horizontal dan akan mengulangi kegagalan yang sama. Melainkan: "satu
> workflow berjalan dari file sampai layar", lalu diperlebar.

---

## Problem Statement

Pemilik Produk punya mesin n8n versi Rust yang dilaporkan 79,5% selesai, tetapi **tidak ada satu pun cara
untuk menjalankan satu workflow.** Sembilan dari 16 crate adalah stub satu baris. Tidak ada binary produk,
tidak ada UI, tidak ada cara mengimpor workflow n8n yang sudah beliau miliki.

Dari sudut pandang Pemilik Produk: beliau ingin **memakai** n8n versi Rust — mengimpor workflow yang sudah
ada, menjalankannya, melihat hasilnya, dan mengeditnya lewat peramban — seperti memasang n8n asli. Yang ada
sekarang adalah kumpulan pustaka dan dokumen.

## Solution

Satu binary `n8n-rust` yang:
1. Dijalankan dengan `n8n-rust start` di VPS 2GB, membuka port, membuat direktori data, menjalankan migrasi
2. Menyajikan **UI web yang ditulis dalam Rust** (dikompilasi ke WebAssembly) — daftar workflow, editor,
   hasil eksekusi
3. Menyajikan **CLI yang paritas dengan n8n 2.39.0** (24 perintah — lihat `CLI-PARITY-SPEC.md`)
4. Bisa mengimpor workflow JSON hasil ekspor n8n asli dan menjalankannya tanpa diubah

**Framework frontend: Leptos.** Alasan terukur dari riset 2026-08:
- **Bundle WASM terkecil** (~90KB gzipped CSR vs Yew ~110KB, Dioxus ~100KB) — penting untuk VPS 2GB dan
  koneksi pengguna
- **SSR kelas satu + Islands** — server merender, klien hanya menghidrasi bagian yang perlu; lebih ringan di
  mesin kecil
- **Reaktivitas fine-grained (model SolidJS)** — **ini yang menentukan untuk editor kanvas**: mengubah satu
  node tidak me-render ulang seluruh graf. Virtual DOM (Yew, Dioxus) akan me-render ulang pohon
- **Seluruhnya safe Rust** (Dioxus memakai banyak unsafe)
- **Server functions dengan type-check compile-time** — UI dan backend berbagi tipe Rust, jadi kontrak API
  tidak bisa menyimpang tanpa error kompilasi

**Yang ditolak dan alasannya:** Dioxus (multi-platform desktop/mobile — tidak dibutuhkan, SSR kurang matang,
bundle lebih besar, banyak unsafe). Yew (ekosistem komponen paling matang, tapi virtual DOM dan tidak ada
SSR/islands sebaik Leptos; untuk kanvas graf, tidak ada komponen jadi di framework Rust mana pun, jadi
keunggulan ekosistemnya tidak berlaku di bagian tersulit).

## User Stories

1. Sebagai pengguna, saya ingin menjalankan satu perintah untuk menyalakan seluruh sistem, supaya saya tidak
   perlu mengonfigurasi database, server, dan frontend secara terpisah.
2. Sebagai pengguna, saya ingin membuka peramban dan melihat daftar workflow saya, supaya saya tahu apa yang
   sudah ada.
3. Sebagai pengguna, saya ingin mengimpor file JSON workflow dari n8n asli lewat CLI, supaya aset yang sudah
   saya punya langsung terpakai.
4. Sebagai pengguna, saya ingin mengimpor workflow lewat UI dengan mengunggah file, supaya saya tidak perlu
   akses shell.
5. Sebagai pengguna, saya ingin menjalankan satu workflow dari UI dan melihat hasilnya, supaya saya bisa
   memverifikasi ia bekerja.
6. Sebagai pengguna, saya ingin menjalankan workflow dari CLI dengan `execute --id`, supaya saya bisa
   mengotomasi.
7. Sebagai pengguna, saya ingin melihat keluaran JSON mentah dengan `--rawOutput`, supaya saya bisa
   mem-pipe ke alat lain. *(catatan: lihat Further Notes — ada karakter non-Latin yang harus dibersihkan)*
8. Sebagai pengguna, saya ingin melihat riwayat eksekusi beserta status dan waktunya, supaya saya bisa
   menelusuri kegagalan.
9. Sebagai pengguna, saya ingin melihat data tiap node dalam satu eksekusi, supaya saya tahu di mana
   workflow menyimpang.
10. Sebagai pengguna, saya ingin mengedit workflow di kanvas dengan menyeret node, supaya pengalamannya sama
    dengan n8n asli.
11. Sebagai pengguna, saya ingin menyambungkan dua node dengan menarik garis antar port, supaya saya bisa
    membangun alur.
12. Sebagai pengguna, saya ingin mengonfigurasi parameter node lewat formulir, bukan menulis JSON, supaya
    cepat dan tidak salah.
13. Sebagai pengguna, saya ingin ekspresi `{{ $json.x }}` dievaluasi dan hasilnya terlihat di UI, supaya saya
    tahu ekspresi saya benar sebelum menjalankan.
14. Sebagai pengguna, saya ingin menyimpan workflow dan menyimpannya sebagai draft, supaya perubahan tidak
    langsung aktif.
15. Sebagai pengguna, saya ingin menerbitkan dan membatalkan publikasi workflow, supaya sesuai model n8n 2.0.
16. Sebagai pengguna, saya ingin menyimpan kredensial sekali dan memakainya di banyak node, supaya saya tidak
    menulis ulang token.
17. Sebagai pengguna, saya ingin kredensial tidak pernah tampil kembali dalam bentuk terbaca di UI, supaya
    aman.
18. Sebagai pengguna, saya ingin mengekspor workflow ke file JSON yang bisa diimpor n8n asli, supaya tidak
    terkunci di produk ini.
19. Sebagai pengguna, saya ingin trigger jadwal (cron) berjalan sendiri saat server hidup, supaya workflow
    otomatis.
20. Sebagai pengguna, saya ingin trigger webhook menerima HTTP dari luar, supaya sistem lain bisa memicu
    workflow.
21. Sebagai pengguna, saya ingin login dengan email dan kata sandi di instance saya sendiri, supaya tidak
    terbuka ke publik.
22. Sebagai pengguna, saya ingin melihat node apa saja yang tersedia beserta pencarian, supaya saya bisa
    menemukan yang saya butuh.
23. Sebagai pengguna, saya ingin menjalankan ulang satu eksekusi yang gagal, supaya saya tidak mengulang
    manual.
24. Sebagai pengguna, saya ingin melihat pesan error yang menjelaskan node mana yang gagal dan kenapa,
    supaya saya bisa memperbaikinya.
25. Sebagai pengguna, saya ingin seluruh UI dan CLI berjalan di VPS 2GB tanpa OOM, supaya sesuai perangkat
    yang saya punya.
26. Sebagai pengguna, saya ingin `--help` di CLI menampilkan perintah dan flag yang sama dengan n8n asli,
    supaya pengetahuan CLI n8n saya tetap terpakai.
27. Sebagai pengguna, saya ingin satu berkas binary tunggal untuk dipasang, supaya pemasangan sederhana.
28. Sebagai pengguna, saya ingin data saya di SQLite dalam satu file yang bisa saya cadangkan, supaya
    pemulihan mudah.
29. Sebagai pengguna, saya ingin mematikan workflow yang berjalan lama, supaya tidak menggantung.
30. Sebagai pengguna, saya ingin melihat status server (versi, jumlah workflow, eksekusi aktif) di satu
    halaman, supaya saya tahu keadaan sistem.

## Implementation Decisions

**Keputusan arsitektur:**
- **Satu binary, satu proses.** `n8n-rust start` menyajikan API HTTP **dan** aset frontend (WASM + CSS) dari
  berkas yang di-embed saat kompilasi. Tidak ada server Node, tidak ada build step di mesin target.
- **Leptos dengan SSR + hydration.** Server merender HTML awal; WASM klien menghidrasi. Untuk halaman yang
  sangat interaktif (kanvas), pakai CSR penuh di dalam shell yang sama.
- **Server functions Leptos untuk kontrak UI↔backend.** Karena UI dan backend satu bahasa dan satu crate
  graph, tipe permintaan/jawaban dibagi — penyimpangan kontrak jadi error kompilasi, bukan bug runtime.
- **API REST publik tetap dibangun terpisah** (crate `api`) dengan OpenAPI 3.1, karena CLI dan integrasi luar
  membutuhkannya, dan karena server functions tidak bisa dipanggil dari luar proses.
- **SQLite sebagai satu-satunya database.** Sudah dipilih di keputusan lama, sudah ada di crate `storage`.
  Tidak ada Postgres, tidak ada Redis, tidak ada mode queue — produk single-instance.
- **Ekspresi `{{ }}`** dievaluasi lewat runtime QuickJS (crate `expr-quickjs`) untuk paritas semantik dengan
  n8n, dengan jalur cepat Rust untuk kasus sederhana.
- **Node diimplementasikan terhadap trait `Node` yang sudah ada di kernel.** Kontraknya sudah benar; yang
  hilang adalah implementasinya.

**Keputusan model domain (memakai kosakata yang sudah ada di kernel, bukan mengarang baru):**
`Item`, `ItemList::{Inline,Spilled}`, `SpilledList`, `ContentId`, `SpillStore`, `Checkpoint`, `SideEffect`,
`ParameterSchema`, `PriorOutputs`, `NodeDescriptor`. **Ini daftar kanonik yang sudah diverifikasi gate; jangan
memakai nama lain.** (Kamus root `BAHASA-BERSAMA.md §2` sudah dinyatakan NON-BINDING — jangan dikutip.)

**Keputusan kanvas (bagian tersulit, diputuskan sekarang supaya tidak ditebak nanti):**
- Kanvas digambar sebagai **SVG di dalam Leptos**, bukan `<canvas>`. Alasan: node adalah elemen DOM yang
  perlu event handler, teks yang bisa dipilih, dan aksesibilitas; SVG memberi itu gratis. `<canvas>` akan
  memaksa kita menulis ulang hit-testing, seleksi teks, dan z-order.
- Posisi node disimpan di model workflow (n8n menyimpan `position: [x,y]` per node), jadi **tidak ada skema
  baru** untuk tata letak.
- Edge digambar sebagai path Bézier kubik, sama seperti n8n.

**Keputusan yang secara eksplisit TIDAK diambil sekarang:**
- Tidak memakai `editor-ui` n8n (Vue). Pemilik Produk sudah memutuskan frontend dari Rust.
- Tidak membangun mode queue/multi-main/worker.
- Tidak mengejar paritas 694 node di tahap ini.

## Seam pengujian — DIPUTUSKAN: dua seam

**Disetujui Pemilik Produk 2026-09-10.** Skill `to-spec` memerintahkan: gambar seam tempat fitur akan diuji;
utamakan seam yang sudah ada; pakai seam tertinggi yang mungkin; semakin sedikit seam semakin baik, idealnya satu.

**Seam 1 (tertinggi): proses CLI.**
Uji dengan menjalankan binary sungguhan terhadap direktori data sementara, lalu memeriksa keluaran dan kode
keluar. Ini seam tertinggi yang mungkin dan yang paling dekat dengan pengalaman Pemilik Produk.
```
n8n-rust import:workflow --input=fixture.json  → exit 0
n8n-rust execute --id=<N> --rawOutput          → JSON yang diharapkan di stdout
```

**Seam 2: HTTP API.**
Uji dengan memanggil endpoint terhadap server yang dihidupkan di port acak. Ini yang dipakai UI, jadi menguji
di sini berarti menguji kontrak yang dipakai UI.

**Seam 3 (server function Leptos) DIBUANG.** Usulan awal saya memuat tiga seam; Pemilik Produk memilih dua.
Konsekuensinya mengikat dan harus ditegakkan saat review:

> **Logika yang layak diuji tidak boleh hidup di frontend.** Kalau sebuah aturan bisnis, validasi, atau
> transformasi perlu diuji dan tidak bisa diuji lewat Seam 1 atau Seam 2, itu tanda logikanya salah tempat —
> **turunkan ke API, jangan buka seam baru untuk menampungnya.**

Aturan ini sengaja lebih keras daripada sekadar "seam-nya dua". Tanpa aturan ini, "buang Seam 3" akan
dilanggar secara diam-diam dengan cara menulis test yang memanggil fungsi frontend langsung — seam keempat
yang tidak pernah diputuskan.

**Yang ditolak sebagai seam:** menguji per-crate (executor saja, expr saja, storage saja). **Itu persis irisan
horizontal yang menghasilkan 31 task DONE tanpa produk yang berjalan.** Uji unit tetap boleh ada di dalam
crate, tapi **bukan itu yang menentukan sebuah tiket selesai.** Yang menentukan adalah Seam 1 atau Seam 2.

## Testing Decisions

**Apa yang membuat test baik: hanya menguji perilaku eksternal, bukan detail implementasi.** Konsekuensinya:
test tidak boleh menyebut nama fungsi internal, tidak boleh mock modul sendiri, dan tidak boleh pecah karena
refactor yang tidak mengubah perilaku.

**Modul yang diuji:** semuanya, tapi lewat Seam 1 dan Seam 2. Tidak ada modul yang "diuji sendiri" sebagai
tanda selesai.

**Preseden yang sudah ada di codebase dan layak ditiru:**
- Uji golden byte-identical pada `openapi-codegen` (`ingest_frankfurter_golden`,
  `frankfurter_golden_valid_sorted`, `github_golden_strict_fails_on_oneof_param`) — **sudah terbukti mengikat**:
  ketika `ingest.rs` diubah untuk kutover R41, golden tetap hijau, dan itu bukti empiris pertama bahwa golden
  menangkap drift.
- Disiplin mutan 50e: **setiap test wajib punya mutan yang membuktikan ia bisa gagal**, dengan `sha-before ≠
  sha-after` dibuktikan SEBELUM membaca hasil. Sudah dipakai berhasil di kutover rosetta (mutan mati di 3 test).

**Aturan tambahan yang lahir dari kegagalan hari ini:**
- **Test yang tidak pernah gagal bukan test.** Setiap tiket wajib menyertakan satu mutan.
- **Sebutkan perintah lengkapnya, bukan hanya hasilnya.** Kasus nyata: satu agen melaporkan "clippy CLEAN"
  dengan `--lib`, agen lain melaporkan 4 warning dengan `--all-targets`. **Keduanya jujur.** Yang membedakan
  hanya argumen perintahnya.

## Out of Scope

- **Paritas 694 node.** Tahap ini mengejar ~12 node esensial. Verifikasi per-tipe terhadap katalog (R-6)
  ditunda.
- **Mode queue, multi-main, `worker`, `webhook` sebagai perintah CLI terpisah.** Produk single-instance.
- **Lisensi berbayar, LDAP, MFA, SSO.**
- **Replay/reversion engine, canary diffing, audit hash-chain.** Semua PAUSED oleh Ruling 62.
- **Node komunitas/WASM bridge sebagai jalur utama.** `nodes-wasm` sudah ada dan tetap ada, tapi bukan jalur
  untuk node inti.
- **Migrasi dari database selain SQLite.**
- **Internationalization UI.** Bahasa Inggris dulu, sama seperti n8n.
- **Aplikasi desktop atau mobile.** Ini alasan Dioxus ditolak.

## Further Notes

**Catatan mutu dokumen ini sendiri.** Saat menulis User Story 7 saya memasukkan karakter non-Latin (工具) yang
bukan maksud saya — persis kelas kontaminasi yang sudah dua kali saya temukan dan perbaiki di pesan channel
hari ini. **Itu diperbaiki sebelum dokumen ini dikirim**, tapi saya catat di sini karena polanya nyata:
keluaran yang panjang punya peluang lebih tinggi mengandung cacat yang tidak disengaja, dan pemeriksaan
mekanis (bukan membaca ulang) yang menangkapnya.

**Tentang "file path dan cuplikan kode".** Skill `to-spec` dan `to-tickets` keduanya melarang:
*"avoid specific file paths or code snippets: they go stale fast."* **Saya setuju, dan ini menyelesaikan
ketegangan yang saya alami hari ini.** Saya sudah menuntut `verified(file:line)` sepanjang sesi, dan skill
ini melarang path file di spec. Keduanya benar karena mengacu hal berbeda:

> **`file:line` adalah BUKTI tentang keadaan sekarang — bersifat sementara, harus diberi tanggal dan sha.**
> **Spec adalah KONTRAK tentang keadaan yang diinginkan — bersifat tahan lama, tidak boleh memuat path.**

Kesalahan kesembilan saya hari ini (mengarang syarat "rebuild trigger" dari pesan commit) terjadi persis
karena saya mencampur keduanya: saya menulis klaim sementara seolah ia kontrak.

**Keputusan yang sudah ditutup Pemilik Produk (2026-09-10).** Empat pertanyaan diajukan memakai format ronde
skill `grilling` — bernomor, dengan rekomendasi, lalu menunggu. Keempatnya dijawab dan keempatnya menerima
rekomendasi:

| Pertanyaan | Keputusan |
|---|---|
| Jumlah seam | **Dua** (proses CLI + HTTP API). Seam server function Leptos dibuang. |
| Paritas `--help` | **Enam perintah Stage 1 saja**, bukan 24. Sisanya menyusul setelah produk terpakai. |
| Batas milestone pertama | **M1 lalu langsung M3 tanpa berhenti.** M3 membuktikan Leptos sanggup hidup di VPS 2GB sebelum bertaruh pada kanvas. |
| Granularitas tiket kanvas | **Dipecah tiga**: render statik → geser → sambung. |

Rincian penerapannya ada di `TICKETS-TRACER-BULLET.md`.
