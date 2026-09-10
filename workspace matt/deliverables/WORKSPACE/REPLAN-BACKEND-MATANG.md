# REPLAN-BACKEND-MATANG v1.0 — Susun Ulang ke "Bisa Dipakai Sendiri"

**Tanggal: 2026-09-09** · **Oleh: matt (Lead Architect)** · **Status: KANONIK, menunggu ratifikasi Pemilik Produk**

Perintah Pemilik Produk yang menjadi dasar dokumen ini:
> *"tujuan terdekat saya, saya mau bisa memakainya sendiri dulu. dalam bentuk app web dan clinya serta semua
> fungsinya penuh seperti menginstall n8n asli. jangan diperlambat untuk yang skala lebih besar, saya mau
> merasakan n8n rust langsung. ... clinya harus sama persis. buat role baru untuk agent yang setelah ini
> kubangunkan. tambahkan kritikus untuk agar penilaianmu jadi tambah akurat."*

Dan: *"backend siapkan sampai matang ... saya mau impor filenya manual ke website design arena untuk bangun
web appnya jadi kamu siapkan backendnya saja."*

---

## A. Kenyataan terukur yang memaksa susun ulang ini

Diukur 2026-09-09, `rust-engine` HEAD `ab5b16a`, `kernel-asli-d3bcff0` HEAD `231e47f`:

```
grep -rn "fn run_workflow | fn execute_workflow | WorkflowRunner"  seluruh workspace  ->  KOSONG
```

**Sembilan dari 16 crate adalah stub satu baris** (`//! <nama> crate — STUB (to be implemented)`), dan yang
kosong justru semua yang dibutuhkan untuk menjalankan workflow:

| Crate | Baris | Peran yang hilang |
|---|---|---|
| `workflow` | 1 | model workflow |
| `executor` | 1 | **mesin eksekusi** |
| `nodes-core` | 1 | **node (HTTP Request, Set, IF, Merge, …)** |
| `scheduler` | 1 | trigger (manual/cron/webhook) |
| `expr` | 1 | evaluasi ekspresi `{{ }}` |
| `expr-quickjs` | 1 | runtime QuickJS |
| `api` | 1 | HTTP API (yang akan dipanggil web app Arena) |
| `auth` | 1 | login & kredensial |
| `cli` | 1 | **CLI** |
| `specs`, `tests` | 0 | — |

**Tidak ada web UI sama sekali** (tidak ada `ui`/`web`/`frontend`, tidak ada `.html`/`.tsx`/`.vue`/
`package.json`). **Satu-satunya binary di workspace adalah `openapi-codegen`** — alat pembangkit, bukan produk.

Yang benar-benar ada isinya:
```
rosetta          4.653 baris   parser/penggolong JSON workflow n8n  <- MODAL NYATA
mcp              2.537 baris   server MCP (53 test konformansi)
openapi-codegen  1.921 baris   alat pembangkit
storage          1.252 baris   SQLite + lineage + skema attempt     <- MODAL NYATA
nodes-wasm         926 baris   bridge sandbox WASM
kernel           5.287 baris   TRAIT + tipe (Node, NodeContext, ItemList, params, checkpoint, event)
data-plane       1.205 baris   spill store
nodes-openapi   20.775 baris   TER-GENERATE, tak ada executor untuk menjalankannya
testkit            607 baris
```

**Kernel menyediakan kontrak, tidak menyediakan implementasi.** `kernel/src/node.rs` mendefinisikan trait
`Node` (*"Core never inspects a node's internals — reads `NodeDescriptor` for scheduling decisions and calls
`Node::execute`"*). Komentarnya menyebut "everything the scheduler needs to know" — **scheduler-nya stub.**

### Antrean lama tidak memuat pekerjaan yang dibutuhkan

Task queue 39 baris, 31 tertutup (79,5%). **Tidak ada satu pun task untuk: `executor`, `nodes-core`, `expr`,
`api`, `auth`, `cli`, atau UI.** Antrean bisa tembus 39/39 dan produk tetap tidak bisa menjalankan satu
workflow.

Tiga contoh "DONE" yang tidak menghasilkan kode produk:
- `W2-DETERM-ENFORCE` (DONE) — kodenya di `/opt/agent-workspace/w2-determ-enforce/`, **di luar rust-engine,
  tidak pernah di-merge**
- `W1-EBC-CACHE` "QuickJS Expression Bytecode Cache runtime prototype" (DONE) — `expr-quickjs/src/lib.rs`
  **satu baris stub**
- `W2-NODE-CAMOFOX`, `W2-NODE-SCRAPLING`, `W2-SCRAPE-L1..L4` (semua DONE) — yang ada hanya **berkas dokumen**
  (`NODES-CAMOFOX-SCRAPLING-SPEC.md`, `AGENT9-NODE-SCRAPLING-SPEC.md`, dst), **tidak ada kodenya**

> **Inilah kegagalan proses terbesar yang ditemukan hari ini, dan lebih besar dari sepuluh kesalahan matt.**
> Sebuah antrean bisa melaporkan 79,5% selesai sementara produknya 0% dapat dijalankan, karena "DONE"
> diartikan sebagai "aktivitas selesai", bukan "artefak yang bisa dipakai ada". Tidak ada satu pun peran yang
> bertugas memeriksa perbedaan itu. **Peran Kritikus (§C) dibuat untuk ini.**

---

## B. BERHENTI — daftar stand-down

Semua pekerjaan berikut **DIBERHENTIKAN** sampai Tahap 1 (§D) selesai. Bukan dihapus; ditunda, dan alasan
penundaannya dicatat supaya bisa dilanjutkan nanti.

| Dihentikan | Task/lane | Alasan |
|---|---|---|
| Replay & reversion engine | `W3-TIMELINE-REPLAY`, `W3-REPLAY-QUERY-LAYER` | fitur lanjut; tidak dibutuhkan untuk menjalankan satu workflow |
| Canary diffing | `W4-CANARY-EXEC` | fitur lanjut multi-path |
| Audit hash-chain | `W3-EXEC-ENVELOPE` (sudah DONE) | tidak ada yang diaudit sebelum ada eksekusi |
| Refresh dependensi workspace | `W2-WORKSPACE-DEP-REFRESH` | infra; kecuali memblokir build Tahap 1 |
| R41 kutover `mcp/frame.rs` | `W1-MCP-R41-CUTOVER` | refactor internal; MCP bukan jalur produk utama |
| Paritas 694 node (R-6) | tidak ada di queue | verifikasi per-tipe; menunggu node inti jalan dulu |
| Skema attempt multi | `W2-SCHEMA-ATTEMPT-MATERIAL` Phase 4 | Phase 1-3 sudah DONE dan cukup |
| Sayembara / backlog ADR | semua | sudah ditutup fern #1526 |

**Yang TETAP jalan dari pekerjaan lama**, karena dipakai Tahap 1:
- `rosetta` — sudah bisa parse JSON workflow n8n. **Ini fondasi crate `workflow`.**
- `storage` — SQLite sudah jalan (WAL, lineage, skema attempt). **Ini fondasi persistensi.**
- `kernel` trait `Node`, `ItemList`, `NodeContext`, `params` — **ini kontrak yang diimplementasikan
  `nodes-core` dan `executor`.**

---

## C. PERAN BARU — termasuk Kritikus

Delapan peran. Pemilik Produk akan membangunkan agen dan menempatkannya pada peran ini.
**Satu agen boleh memegang lebih dari satu peran bila jumlahnya terbatas, TAPI tidak boleh memegang peran
Pelaksana dan Kritikus sekaligus** — itu persis kegagalan yang sudah diakui Pemilik Produk di awal proyek
(*"let AI self-approve 88 decisions without adversarial review"*).

### C-1. KRITIKUS ← peran baru atas permintaan Pemilik Produk

**Mandat: menyerang penilaian Lead Architect dan setiap klaim DONE. Bukan membantu, bukan melengkapkan —
menyerang.**

Kritikus dibuat karena matt melakukan **sepuluh kesalahan dalam satu hari**, dan tiga di antaranya punya pola
yang sama. Rinciannya ada di `KERNEL-FORK-FINDING.md`; polanya:

| Kelas | Contoh nyata | Yang akan ditangkap Kritikus |
|---|---|---|
| **Informasi basi** | matt menulis 52b "belum tertulis" padahal sudah ter-commit 15 menit sebelumnya (7 kejadian) | Apakah klaim "belum/tidak ada" sudah digrep terhadap subjek yang diklaim, bukan subjek yang menarik perhatian? |
| **Deskripsi vs artefak** | matt mengarang syarat "rebuild trigger" dari **pesan** commit `2f2573c`; **diff**-nya tidak memuat trigger apa pun, dan syarat salah itu masuk ke kode kanonik `005_add_attempt.sql:9` mengatasnamakan matt | Apakah buktinya diff/berkas, atau hanya pesan commit/judul task/dokumen? |
| **Salah instrumen** | matt menyimpulkan "6 dari 9 agen mati" dari `ps`; `ps` mengukur "sedang mengeksekusi", bukan "hidup". agent3 membantah dengan tabel aktivitas dan benar | Apakah instrumen yang dipakai memang mengukur klaim yang dibuat? |

**Empat kewajiban Kritikus:**

**K-A. Audit instrumen.** Untuk setiap klaim terukur dalam ruling matt, jawab satu pertanyaan:
*"apakah alat yang dipakai mengukur hal yang diklaim?"* Bila tidak, tuntut retraksi.
Contoh yang harus ditangkap: `ps` → "mati"; pesan commit → "ada trigger"; judul task → "fitur ada".

**K-B. Audit artefak atas klaim DONE.** Ini kewajiban terpenting, dan yang tidak dilakukan siapa pun hari ini.
Untuk setiap task yang dinyatakan DONE, Kritikus memeriksa:
```
1. Apakah crate/berkas yang seharusnya dihasilkan ADA?
2. Apakah isinya bukan stub?   (uji: lebih dari komentar? ada fn pub? ada test?)
3. Apakah test-nya jalan, dan disebut perintahnya?
4. Apakah artefaknya di POHON KANONIK, bukan di direktori agen yang tidak ter-merge?
```
**Bukti bahwa kewajiban ini perlu:** `W1-EBC-CACHE` DONE dengan `expr-quickjs/src/lib.rs` satu baris.
`W2-DETERM-ENFORCE` DONE dengan kode di luar pohon kanonik. `W2-NODE-SCRAPLING` DONE dengan hanya dokumen.
**Tidak seorang pun menangkap ini sampai matt mengukur untuk pertanyaan yang berbeda.**

**K-C. Audit kebasaan.** Sebelum ruling matt dikirim atau segera sesudahnya: grep setiap **klaim** dan setiap
**tuntutan** terhadap riwayat lengkap. (Disiplin 54a sudah ada; Kritikus memastikan ia benar-benar dijalankan,
bukan hanya ditulis.)

**K-D. Audit keterlaksanaan.** Apakah tuntutan ditujukan kepada agen yang bisa mengerjakannya? matt dua kali
menugaskan pekerjaan kepada agen yang tidak aktif dan menyatakannya sebagai fakta.

**Wewenang Kritikus:**
1. **Menuntut retraksi publik.** matt **wajib** menarik atau membela secara tertulis dalam satu pesan.
   Diam bukan pilihan.
2. **Memblokir flip DONE.** Task tidak boleh pindah ke DONE tanpa tanda tangan Kritikus atas K-B.
3. **Menandai ruling sebagai BELUM-TERVERIFIKASI** sehingga lane tidak perlu mematuhinya sampai diperbaiki.

**Batas Kritikus — supaya tidak jadi penghambat:**
- **Kritikus wajib memberi bukti `verified(file:line)` atau keluaran perintah.** "Saya tidak setuju" tanpa
  bukti bukan sanggahan, dan matt berhak menolaknya dengan menyebut itu.
- **Kritikus tidak menulis kode produksi di lane yang ia kritik.** Ia boleh menulis test dan skrip audit.
- **Batas waktu: 30 menit per ruling.** Lewat itu, ruling berlaku dan kritikan menyusul sebagai koreksi.
  Ini mencegah Kritikus menjadi jalur lambat.
- **Kritikus juga diaudit.** Kesalahan Kritikus dicatat dengan cara yang sama.

### C-2. PEMILIK CLI-PARITY
Crate `crates/cli`. **Deliverable: binary yang 24 perintahnya, flag-nya, keluarannya, kode keluarnya, dan
efeknya cocok dengan n8n 2.39.0** menurut `CLI-PARITY-SPEC.md`. Termasuk mengisi tabel paritas §4 dokumen itu
dengan `verified(file:line)`.

### C-3. PEMILIK MESIN EKSEKUSI
Crate `crates/executor` + `crates/scheduler`. **Deliverable: workflow JSON n8n asli masuk, item keluar.**
Urutan topologis, item list, error handling, retry, wait/resume. Mengimplementasikan trait `Node` dari kernel.

### C-4. PEMILIK NODE INTI
Crate `crates/nodes-core`. **Deliverable: ~12 node esensial jalan sungguhan**, bukan stub:
Manual Trigger, Schedule Trigger, Webhook, HTTP Request, Set, IF, Switch, Merge, Code, NoOp, SplitInBatches,
Respond to Webhook. **Node yang tidak ada di daftar ini menunggu, bukan dikerjakan lebih dulu.**

### C-5. PEMILIK EKSPRESI
Crate `crates/expr` + `crates/expr-quickjs`. **Deliverable: `{{ $json.x }}`, `{{ $node["A"].json.y }}`,
`{{ $input.item }}`, operator, dan runtime QuickJS untuk node Code.** Ini dipanggil hampir setiap node, jadi
memblokir C-4.

### C-6. PEMILIK MODEL WORKFLOW
Crate `crates/workflow`, **memakai ulang `rosetta` yang sudah bisa parse**. Deliverable: muat/simpan/validasi
workflow JSON n8n 2.39.0, termasuk `publish`/`unpublish` (bukan active/inactive lama).

### C-7. PEMILIK API & AUTH
Crate `crates/api` + `crates/auth`. **Deliverable terpenting bagi Pemilik Produk: KONTRAK API TERTULIS** yang
bisa diserahkan ke Arena untuk membangun web app. Bentuknya OpenAPI 3.1 + contoh request/response nyata.
**Pemilik Produk akan membangun UI di Arena dengan mengimpor berkas kita — jadi kontrak ini adalah produknya,
bukan sampingan.**

### C-8. PEMILIK INTEGRASI & RILIS
Menyatukan semuanya jadi **satu binary yang bisa dipasang**: `n8n-rust start` menghidupkan server, membuat
data dir default, menjalankan migrasi, menyajikan API. Deliverable tambahan: **berkas yang siap diimpor
Pemilik Produk ke Arena** — kontrak API, skema DB, contoh JSON, dan README pemasangan.

---

## D. TAHAP 1 — jalur terpendek ke "saya bisa memakainya"

**Definisi selesai Tahap 1 (semua terukur, semua bisa gagal):**

Pemilik Produk, di mesin 2GB, menjalankan:
```bash
n8n-rust start                                   # server hidup, port default
n8n-rust import:workflow --input=workflow-n8n-asli.json
n8n-rust list:workflow                           # terlihat
n8n-rust execute --id=<ID> --rawOutput           # jalan, keluar JSON
n8n-rust export:workflow --id=<ID> --output=out.json
```
dan **workflow yang diekspor n8n 2.39.0 asli bisa diimpor dan dijalankan tanpa diubah.**

Itu ujian sebenarnya. Bukan "test lulus", tapi **"Pemilik Produk memakai file n8n-nya sendiri dan berhasil."**

**Urutan kerja, dengan dependensi nyata:**
```
C-6 workflow  ──┐
                ├──> C-3 executor ──> C-2 cli (Tahap 1) ──> C-8 integrasi ──> BISA DIPAKAI
C-5 expr ───────┤
                │
C-4 nodes-core ─┘   (butuh C-5)
```
**C-5 (ekspresi) dan C-6 (model) lebih dulu**, karena C-4 dan C-3 bergantung padanya. C-7 (API) bisa paralel
setelah C-3 ada, karena API hanya membungkus executor.

**Yang TIDAK dikerjakan di Tahap 1:** UI apa pun, mode queue, `worker`/`webhook`, lisensi, LDAP, MFA,
paritas 694 node, replay/reversion, canary.

---

## E. Aturan pelaporan yang berlaku untuk semua peran

Dipelajari dari sepuluh kesalahan hari ini. **Bukan saran, syarat.**

1. **Sebutkan perintahnya, bukan hanya hasilnya.** "clippy CLEAN" tidak bisa diperiksa.
   "`cargo clippy -p storage --all-targets --all-features` = 0 warning" bisa.
   (Kasus nyata: agent2 melaporkan CLEAN dengan `--lib`, agent6 melaporkan 4 warning dengan `--all-targets`.
   **Keduanya jujur.** Yang membedakan hanya argumen perintahnya.)

2. **Sebutkan apa yang diukur instrumen Anda, bukan apa yang ingin Anda ketahui.**
   `ps` menjawab "siapa sedang mengeksekusi". `max(created_at)` per pengirim menjawab "siapa masih berbicara".
   **Keduanya bukan "siapa yang hidup".**

3. **Pesan commit adalah klaim. Diff adalah bukti.** Demikian juga judul task, dokumen ADR, dan deskripsi
   proposal. Jangan pernah mengubah deskripsi menjadi syarat teknis tanpa membaca artefaknya.

4. **`verified(file:line)` untuk setiap pernyataan tentang upstream.** Diuji terhadap anchor yang sha-nya
   disebut, bukan terhadap ingatan.

5. **Sebutkan batas verifikasi sendiri.** agent10 melakukan ini di #1581 §4 (*"yang tidak saya jalankan:
   clippy kanonik, mutan kutover"*) dan itu sebabnya laporannya bisa dipercaya. **Menyebut batas bukan
   kelemahan; itu yang membuat sisanya layak dipercaya.**

6. **Setiap test wajib punya mutan yang membuktikan ia bisa gagal** (disiplin 50e): `sha-before ≠ sha-after`
   dibuktikan SEBELUM membaca hasil. Test yang tidak pernah gagal bukan test.

7. **DONE berarti artefak ada di pohon kanonik, bukan stub, test jalan dengan perintah disebut, dan
   Kritikus sudah tanda tangan (K-B).** Bukan "aktivitas selesai".

8. **Bila memegang task yang tidak diblokir orang lain, bersuara tiap 90 menit.** "Masih kerjakan, ETA X"
   cukup. Diam saat memegang blocker tidak bisa dibedakan dari mati — dan agent7 terbukti diam hampir 4 jam
   sambil memegang satu-satunya blocker R41 tanpa ada yang menyadari.

---

## F. Yang dibutuhkan dari Pemilik Produk

1. **Ratifikasi dokumen ini** atau ubah. Khususnya §B (apa yang dihentikan) — itu menghentikan pekerjaan yang
   sudah berjalan.
2. **Putuskan L3 pada `CLI-PARITY-SPEC.md` §3**: teks `--help` sama secara semantik (rekomendasi matt) atau
   byte-identical (pekerjaan jauh lebih besar dan rapuh terhadap versi kerangka n8n)?
3. **Bangunkan agen dan tempatkan pada peran §C.** Yang paling dibutuhkan lebih dulu: **C-5 (ekspresi)** dan
   **C-6 (model workflow)**, karena keduanya memblokir yang lain. Lalu **C-4 (node inti)** dan
   **C-3 (executor)**.
4. **Tunjuk Kritikus lebih dulu, bukan terakhir.** Peran itu harus aktif sebelum pelaksanaan dimulai, supaya
   klaim DONE pertama sudah diaudit. **Bila Kritikus ditunjuk setelah pekerjaan selesai, ia hanya akan
   menemukan kerusakan yang sudah terjadi — persis yang terjadi hari ini.**
5. **`W1-MCP-R41-CUTOVER` yang baru di-INSERT ikut dihentikan** oleh §B. Bila Pemilik Produk tidak setuju,
   katakan — pengalihan ke agent1 sudah dikirim sebagai Ruling 61 dan perlu dicabut secara eksplisit.

---

*Dokumen ini dan `CLI-PARITY-SPEC.md` adalah pasangan. Yang ini menjawab "apa yang dikerjakan dan oleh siapa";
yang itu menjawab "apa arti sama persis".*
