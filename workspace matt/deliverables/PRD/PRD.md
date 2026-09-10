# PRD — Workflow Automation Engine (Rust)

**Versi:** 0.2 — DRAFT UNTUK REVIEW
**Tanggal:** 2026-09-09
**Penulis:** Arena.ai Agent Mode (drafting) — **belum disetujui**
**Pemilik keputusan:** Anda

**Riwayat perubahan:**

| Versi | Perubahan |
|---|---|
| 0.1 | Draft awal: 16 section, 8 keputusan terbuka, 10 risiko |
| 0.2 | Keputusan "produk untuk publik + multi-tenant" dikunci. Ditambahkan §3.4 (konsekuensi arsitektur multi-tenant), FR-MT-03…07, OPEN-9 (skenario A vs B), R10 (permukaan keamanan multi-tenant). Fase 2 naik L→XL; estimasi total naik ~1-2 bulan full-time. |

> ⚠️ **Status dokumen ini: DRAFT. Bukan persetujuan.**
>
> Dokumen ini sengaja mengandung bagian **§14 KEPUTUSAN TERBUKA** yang berisi
> hal-hal yang *tidak boleh* saya putuskan sendiri. Kegagalan proses sebelumnya
> adalah 88 keputusan lolos tanpa ada yang menantang. Dokumen ini tidak akan
> mengulang itu: setiap asumsi saya tandai, setiap risiko saya tulis apa adanya,
> dan setiap keputusan yang membutuhkan Anda saya biarkan terbuka.
>
> **Jangan mulai implementasi sebelum §14 ditutup.**

---

## 1. Ringkasan Eksekutif

Membangun produk otomasi workflow **self-hosted untuk publik**, ditulis dari nol
dalam Rust, yang secara fungsional setara dengan n8n untuk subset inti yang
didefinisikan, dan secara signifikan lebih hemat memori.

**Diferensiasi utama:** efisiensi resource yang terukur, bukan diklaim. Sudah
ada bukti empiris (lihat §6.2): payload 753 MB diproses dalam peak RSS 50,4 MB,
sementara pendekatan inline (yang dipakai n8n) di-OOM-kill pada 1,68 GB.

**Realitas scope yang harus diakui di depan:**

| Komponen | Perkiraan porsi effort |
|---|---|
| Engine (kernel ✅, scheduler, executor) | ~20% |
| Expression engine (QuickJS + kompatibilitas n8n) | ~10% |
| Node library (20-30 node MVP) | ~10% |
| Storage + API + auth + multi-tenancy | ~15% |
| **Editor UI (canvas, palette, forms, execution viewer)** | **~30%** |
| Docs, onboarding, installer, support tooling | ~15% |

**Editor UI adalah komponen terbesar dan paling sering diremehkan.** Kernel yang
sudah jadi (2.644 baris) adalah lapisan kontrak — bukan 20% dari produk,
melainkan fondasi yang memungkinkan 100% sisanya tidak saling bertabrakan.

---

## 2. Konteks & Dokumen Terkait

PRD ini berdiri di atas:

| Dokumen | Isi | Status |
|---|---|---|
| `rangkuman-rust-workflow-os.md` | D1–D100, blueprint 34 section | Terkunci (tapi lihat audit) |
| `AUDIT-D1-D100.md` | 30 temuan, 5 CRITICAL / 9 HIGH | Selesai |
| `KERNEL-SPEC.md` | D115 Kernel Freeze + §13 delta implementasi | Selesai, kode ada |
| `BENCH-A03.md` | Bukti empiris spill-to-disk | Selesai, terukur |
| `rust-n8n-core/crates/kernel/` | Kode Rust: 19/19 test hijau, clippy bersih | **Selesai** |

**Skor audit yang relevan untuk PRD ini** (dari AUDIT-D1-D100.md):

| Dimensi | Skor | Implikasi untuk PRD |
|---|---|---|
| Arah strategis | 8/10 | Kuat — tidak perlu diubah |
| Tech stack | 9/10 | Kuat — Rust tepat untuk tujuan ini |
| Konsistensi internal | 4/10 | **Lemah** — PRD ini harus jadi sumber kebenaran |
| Kelayakan | 5/10 | **Sedang** — butuh pemfasaan agresif |
| Governance | 3/10 | **Lemah** — §14 (keputusan terbuka) dan §16 (catatan governance) menjawab ini |
| Realisme scope | 3/10 | **Lemah** — §7 dan §13 menjawab ini |

PRD ini secara khusus harus menaikkan tiga skor terbawah.

---

## 3. Definisi Produk

### 3.1 Apa produk ini

Sistem otomasi workflow self-hosted yang:
- Dijalankan pengguna di infrastrukturnya sendiri (VPS, container, on-prem)
- Mengeksekusi workflow berbasis node yang didefinisikan secara visual
- Menyediakan editor web, REST API, dan scheduler
- Mengimpor workflow JSON berformat n8n (lihat §10 untuk batasan persisnya)

### 3.2 Apa produk ini BUKAN

Batasan ini sama pentingnya dengan definisi:

- **Bukan** layanan hosted/managed (Anda menjual software, bukan menjalankan workflow pengguna)
- **Bukan** fork n8n — ditulis dari nol, tidak berbagi kode (lihat §11)
- **Bukan** platform iPaaS enterprise (tidak menargetkan persaingan dengan MuleSoft/Boomi)
- **Bukan** framework developer — produk akhir dengan UI, bukan library
- **Bukan** pengganti n8n untuk *semua* kasus pada rilis pertama

### 3.3 Positioning

> **[ASUMSI — PERLU KONFIRMASI ANDA]**
>
> "Otomasi workflow self-hosted yang muat di VPS murah. Menjalankan workflow
> berat yang membuat n8n kehabisan memori, tanpa perlu upgrade ke VPS 8GB."

Positioning ini memilih **efisiensi resource sebagai wedge**, bukan "n8n tapi
Rust". Alasan: "lebih hemat RAM" adalah klaim yang bisa diukur dan dibuktikan
(sudah ada benchmark-nya), sedangkan "setara n8n" adalah klaim yang akan selalu
kalah karena n8n punya 400+ node dan 10 tahun pengembangan.

---

### 3.4 Konsekuensi keputusan multi-tenant (dikonfirmasi 2026-09-09)

Anda memilih produk publik dengan multi-tenant sebagai fokus. Ini keputusan yang
sah, tapi punya konsekuensi arsitektur yang **tidak bisa ditunda ke fase akhir**.
Bagian ini menulis konsekuensi itu secara eksplisit supaya tidak ditemukan
belakangan.

#### 3.4.1 Multi-tenant bukan fitur, melainkan properti skema

Multi-tenant yang ditambahkan belakangan hampir selalu berakhir sebagai salah
satu dari dua hal: kolom `tenant_id` yang lupa difilter di beberapa query
(= kebocoran data antar pelanggan), atau rewrite besar. Keduanya mahal.

Yang harus dirancang sejak Fase 1:

| Area | Konsekuensi multi-tenant |
|---|---|
| **Storage** | Setiap tabel punya `tenant_id`; setiap query wajib ter-scope. Enforcement di tingkat tipe/query builder, bukan disiplin programmer. |
| **Auth** | User ↔ tenant ↔ role. Session harus membawa tenant aktif. |
| **Kredensial** | Encryption key per-tenant, bukan satu key global. Kalau satu key global, kompromi satu tenant = kompromi semua. |
| **Spill files** | Direktori per-tenant + kuota disk. Tanpa kuota, satu tenant memenuhi disk dan menjatuhkan semua tenant lain. |
| **Resource governor** | Batas RAM/CPU per-tenant. Tanpa ini, satu workflow berat mematikan seluruh instance. |
| **Execution queue** | Fair scheduling antar tenant. Tanpa ini, tenant sibuk memonopoli worker. |
| **Observability** | Log & metric ter-tag tenant, tapi tidak boleh bocor lintas tenant. |

#### 3.4.2 Ketegangan dengan positioning §3.3

Positioning kita adalah **"muat di VPS murah"**. Multi-tenant menarik ke arah
sebaliknya: banyak tenant dalam satu instance berarti resource dibagi, dan
isolasi yang benar (key per-tenant, kuota, fair scheduling) memakan RAM dan CPU.

Kedua hal ini bisa didamaikan, tapi harus diputuskan sadar:

- **Skenario A — multi-tenant untuk agency (P2).** Satu instance melayani beberapa klien agency. VPS tetap kecil, tenant sedikit (5–50). Isolasi penting, tapi beban ringan. **Ini paling cocok dengan positioning.**
- **Skenario B — multi-tenant sebagai SaaS (P3).** Ratusan tenant asing dalam satu instance. Ini **bukan lagi self-hosted product** — ini layanan hosted, dan bertentangan dengan §3.2.

> ⚠️ **OPEN-9: Skenario A atau B?** Jawaban Anda ("self-hosted product atau
> lisensi") mengarah ke **A**, dan saya menulis PRD ini dengan asumsi A. Tapi
> ini perlu konfirmasi eksplisit, karena B mengubah §3.2, model bisnis, dan
> seluruh arsitektur isolasi.

#### 3.4.3 Yang berubah di §7 (fase)

| Fase | Sebelum | Sesudah keputusan multi-tenant |
|---|---|---|
| 1 | Engine headless single-user | Engine headless, **skema storage sudah tenant-aware** |
| 2 | API + auth | API + auth **+ tenant, role, key per-tenant, kuota** |
| 5 | "keputusan multi-tenant" | **sudah diputuskan** — diganti: hardening isolasi |

**Estimasi effort Fase 2 naik dari L ke XL.** Total waktu ke produk yang bisa
dijual (§7.3) naik sekitar 1–3 bulan full-time, atau 3–7 bulan part-time.

#### 3.4.4 Syarat yang tidak bisa ditawar

Karena ini produk publik dengan data pelanggan nyata:

- **Kebocoran lintas tenant adalah bug kelas tertinggi** — setara kehilangan data. Harus ada test otomatis yang mencoba membaca data tenant lain dari setiap endpoint.
- **Audit log** untuk akses kredensial dan perubahan workflow.
- **Backup/restore per-tenant**, bukan hanya seluruh instance.

Ketiganya saya tambahkan ke §8.7 sebagai FR-MT-04 sampai FR-MT-06.

---

## 4. Target Pengguna

Karena produk publik, ada beberapa persona. **Persona primer menentukan
prioritas; yang lain dilayani tapi tidak mengorbankan yang primer.**

> ⚠️ **OPEN-1: Siapa persona PRIMER?** Pilihan Anda "produk untuk publik"
> belum menjawab ini. Lihat §14.

### P1 — Self-hosted hobbyist / indie developer *(kandidat primer)*
- **Konteks:** VPS 2-4 GB, $5-20/bulan. Menjalankan otomasi pribadi atau untuk bisnis kecil.
- **Pain:** n8n memakan RAM; workflow berat OOM; upgrade VPS mahal.
- **Kemampuan teknis:** Tinggi. Nyaman dengan Docker, CLI, log.
- **Kesabaran onboarding:** Sedang. Mau baca docs jika jelas.
- **Kemampuan bayar:** Rendah-menengah ($0-20/bulan).
- **Volume:** Besar, tapi ARPU rendah.

### P2 — Agency / konsultan otomasi
- **Konteks:** Deploy untuk banyak klien. 5-50 instance.
- **Pain:** Biaya VPS per klien menumpuk; butuh isolasi kredensial; butuh deploy ulang cepat.
- **Kemampuan teknis:** Tinggi.
- **Kemampuan bayar:** Menengah-tinggi ($50-500/bulan).
- **Kebutuhan kritis:** multi-tenancy, backup/restore, audit log, white-label *(lihat §11 — white-label punya implikasi lisensi)*.

### P3 — SMB non-teknis
- **Konteks:** Tidak punya tim IT. Ingin otomasi tanpa coding.
- **Pain:** n8n terlalu teknis; Zapier terlalu mahal.
- **Kemampuan teknis:** **Rendah.** Tidak bisa deploy sendiri.
- **Implikasi:** **hampir tidak bisa dilayani oleh produk self-hosted murni.** Melayani P3 praktis berarti menyediakan hosted version — yang bertentangan dengan §3.2.

### P4 — Developer yang ingin extend
- **Konteks:** Menulis node kustom.
- **Kebutuhan:** SDK node, dokumentasi API internal, contoh.
- **Catatan:** ini adalah "ekosistem sendiri dari nol" yang Anda sebut. Bernilai, tapi **baru relevan setelah ada basis pengguna** — bukan kebutuhan MVP.

**Konsekuensi yang harus diakui:** P1 dan P2 bisa dilayani self-hosted. P3
tidak bisa. Memilih P3 sebagai primer berarti mengubah model produk menjadi
hosted SaaS, yang mengubah seluruh PRD ini.

---

## 5. Masalah yang Dipecahkan

### 5.1 Masalah teknis (terbukti)

n8n adalah aplikasi Node.js yang **memory-bound, bukan CPU-bound**. Data dari
riset sebelumnya:

| Beban | Kebutuhan n8n |
|---|---|
| Minimum | 1 vCPU / 1 GB RAM / SQLite — 50-100 workflow ringan |
| Direkomendasikan | 2 vCPU / 2-4 GB RAM |
| Berat | 4+ vCPU / 8-16 GB + PostgreSQL + Redis queue mode |

Idle ~100-500 MB. ~50 MB per workflow aktif. Penyebab struktural: **seluruh
array item ditahan di heap JS** selama eksekusi.

Pada VPS 2 GB tanpa swap, workflow yang memproses dataset besar akan OOM. Tidak
ada konfigurasi n8n yang memperbaiki ini — itu konsekuensi arsitektur.

### 5.2 Bukti bahwa pendekatan kita memperbaiki ini

Dari `BENCH-A03.md` — **terukur, bukan estimasi**, pada mesin 2 GB / 2 CPU /
tanpa swap (identik dengan VPS target):

| Item | Payload | Peak RSS (spill) | Peak RSS (inline / cara n8n) |
|---|---|---|---|
| 200.000 | 29,4 MB | 14,0 MB | 200,1 MB |
| 1.000.000 | 148,0 MB | 20,0 MB | 987,5 MB |
| 5.000.000 | 752,9 MB | **50,4 MB** | **OOM-killed pada 1,68 GB** |

Rasionya **tumbuh** dengan N (14x → 49x → ∞) karena inline skala O(N) sedangkan
spill skala O(chunk + index). Ini keunggulan struktural, bukan optimasi yang
bisa disusul dengan tuning.

**Biaya yang dibayar** (harus diakui di PRD, bukan disembunyikan):
- Scan penuh ~19% lebih lambat
- Butuh disk ~1x ukuran payload + GC yang benar
- Codec masih JSON; codec biner belum diimplementasi

### 5.3 Masalah non-teknis (belum terbukti)

> ⚠️ **Ini bagian yang paling lemah dalam seluruh rencana dan harus diakui.**

Belum ada validasi bahwa **ada orang yang mau membayar** untuk ini. Yang ada:
- Bukti teknis: kuat ✅
- Bukti kebutuhan teknis Anda sendiri: kuat ✅ (Anda menjalankan n8n di VPS 2GB)
- Bukti pasar: **tidak ada** ❌

n8n gratis untuk self-host (Sustainable Use License). Pesaing kita gratis, punya
400+ integrasi, komunitas besar, dan dokumentasi matang. Kita meminta orang
membayar untuk produk yang lebih hemat RAM tapi punya 5-10% jumlah node.

**Ini bukan alasan untuk berhenti** — keputusan membangun sudah final dan saya
tidak akan membahasnya ulang. Tapi ini **alasan mengapa §12 (metrik sukses)
harus memuat validasi pasar sebagai gate fase**, dan mengapa §13 (risiko)
menempatkan ini sebagai risiko #1.

---

## 6. Goals & Non-Goals

### 6.1 Goals (terukur)

| ID | Goal | Ukuran keberhasilan |
|---|---|---|
| G1 | Efisiensi memori unggul terukur | Peak RSS ≤ 25% dari n8n pada workload identik, terverifikasi benchmark |
| G2 | Paritas fungsional subset inti | 100% workflow uji dari korpus kompatibel lolos tanpa modifikasi |
| G3 | Kompatibilitas impor n8n | Impor workflow JSON n8n v1.x untuk node yang didukung |
| G4 | Koneksi luas via codegen | ≥100 API tersedia dari OpenAPI spec tanpa menulis kode node manual |
| G5 | Layak dijual sebagai produk | Onboarding < 30 menit dari download ke workflow pertama jalan |
| G6 | Validasi pasar | Lihat §12 — gate eksplisit sebelum investasi fase lanjut |

### 6.2 Non-Goals (eksplisit, untuk rilis 1.0)

Menulis apa yang **tidak** dikerjakan adalah cara paling efektif mencegah scope
creep — masalah yang membuat skor realisme scope audit hanya 3/10.

| Bukan goal | Alasan |
|---|---|
| Paritas 400+ node n8n | Tidak realistis solo. Lihat §7. |
| Hosted/managed service | §3.2. Mengubah model bisnis dan arsitektur. |
| ~~Multi-tenant dalam satu instance~~ | **DIPINDAHKAN KE SCOPE** — Anda konfirmasi multi-tenant sebagai fokus (§3.4) |
| Mobile app | Tidak ada permintaan teridentifikasi |
| AI agent orchestration | Berbeda produk; godaan scope creep terbesar |
| Kompatibilitas node komunitas n8n (npm) | Node npm = kode JS. Mendukungnya berarti menarik seluruh runtime JS ke produk — menghancurkan keunggulan resource. |
| Editor visual sekelas n8n pada MVP | 30% effort. Lihat §7 fase. |

> ⚠️ **OPEN-2: Multi-tenant.** Anda menyebut multi-tenant sebagai fokus. Tapi
> multi-tenant + self-hosted + persona P1 (hobbyist solo) hampir tidak pernah
> dibutuhkan bersamaan. Multi-tenant terutama bernilai untuk P2 (agency).
> Ini keputusan arsitektur besar yang mengubah storage, auth, dan isolasi
> resource. **Belum saya putuskan.** Lihat §14.

---

## 7. Ruang Lingkup & Pemfasaan

### 7.1 Prinsip pemfasaan

Setiap fase harus menghasilkan sesuatu yang **bisa dijalankan orang lain**,
bukan komponen internal. Fase tanpa artefak yang bisa dipakai adalah fase yang
tidak akan pernah selesai.

### 7.2 Fase

#### FASE 0 — Fondasi *(sebagian selesai)*
**Keluaran:** kernel crate + bukti empiris.
**Status: ✅ SELESAI** — 2.644 baris, 19/19 test, clippy bersih, benchmark terukur.

⚠️ **Utang teknis yang harus dibereskan sebelum fase lain:** workspace Cargo.toml
sekarang mereferensikan 4 crate tapi hanya 2 manifest yang ada → `cargo build`
di level workspace **gagal**. Ini harus diperbaiki duluan.

**Gate keluar:** `scripts/check-freeze.sh` PASSED setelah workspace dirapikan.

#### FASE 1 — Engine headless yang benar-benar menjalankan workflow
**Keluaran:** binary CLI. `engine run workflow.json` benar-benar mengeksekusi.
**Isi:** scheduler, executor, storage SQLite, expression engine (QuickJS), ~10 node inti, HTTP/webhook trigger.
**Tidak ada UI.** Input: file JSON. Output: log + hasil.
**Wajib tenant-aware sejak sini (§3.4.1):** skema storage sudah membawa `tenant_id` dan semua query ter-scope, walau Fase 1 hanya punya satu tenant. Menunda ini ke Fase 2 berarti migrasi skema.
**Ukuran:** XL — ini fase tersulit secara teknis setelah kernel.
**Gate keluar:** 10 workflow n8n nyata (diimpor, bukan ditulis tangan) berjalan benar end-to-end.

> **Mengapa headless dulu:** editor UI adalah 30% effort dan tidak bisa diuji
> sampai engine-nya benar. Membangun UI duluan berarti membangun UI untuk
> engine yang berubah.

#### FASE 2 — API + auth + tenant + operasi
**Keluaran:** REST API lengkap, auth, tenant & role, kredensial terenkripsi per-tenant, execution history, kuota.
**Ukuran:** **XL** (naik dari L karena keputusan multi-tenant §3.4.3).
**Isi tambahan akibat multi-tenant:** model user↔tenant↔role, key per-tenant (FR-MT-05), kuota disk + fair scheduling (FR-MT-07), audit log (FR-MT-06).
**Gate keluar:** semua operasi Fase 1 bisa dilakukan lewat API; kredensial terenkripsi at-rest; **dan test anti-kebocoran lintas tenant (FR-MT-04) hijau untuk setiap endpoint.** Gate terakhir ini tidak bisa ditawar — kebocoran lintas tenant pada produk publik adalah kehilangan data pelanggan.

#### FASE 3 — Editor UI minimum
**Keluaran:** web editor — canvas, node palette, form parameter (dari `ParameterSchema`), execution viewer.
**Ukuran:** XL — kemungkinan lebih lama dari Fase 1.
**Gate keluar:** pengguna non-author bisa membuat workflow sederhana tanpa menyentuh JSON.

#### FASE 4 — Produk yang bisa dijual
**Keluaran:** installer/Docker image, dokumentasi, onboarding, lisensi & aktivasi, telemetry opsional, error reporting.
**Ukuran:** L.
**Gate keluar:** G5 terpenuhi — onboarding < 30 menit, diukur pada ≥5 orang yang bukan Anda.

#### FASE 5 — Validasi pasar & scale
**Isi:** rilis publik, dukungan, codegen OpenAPI massal, hardening isolasi tenant.
*(Keputusan multi-tenant sudah ditutup di §3.4 — tidak lagi ditunda ke sini.)*
**Gate masuk:** G6 — lihat §12. **Jangan masuk fase ini tanpa gate itu.**

### 7.3 Estimasi effort

> **Peringatan kejujuran:** estimasi ini asumsi solo developer, part-time hingga
> full-time, tanpa pengalaman sebelumnya membangun editor visual. Angka ini
> **kasar** dan bisa meleset 2-3x. Saya tidak punya data historis Anda untuk
> mengkalibrasi. Perlakukan sebagai urutan magnitudo, bukan komitmen.

| Fase | Solo full-time | Solo part-time (15 jam/minggu) |
|---|---|---|
| 0 | ✅ selesai | ✅ selesai |
| 1 | 3-6 bulan | 8-16 bulan |
| 2 | 2-4 bulan *(naik: multi-tenant)* | 6-11 bulan |
| 3 | 4-8 bulan | 12-24 bulan |
| 4 | 1-2 bulan | 3-5 bulan |
| **Total ke produk yang bisa dijual** | **10-20 bulan** | **29-56 bulan** |

*(Naik ~1-2 bulan full-time / ~3-6 bulan part-time dari estimasi sebelum
keputusan multi-tenant. Lihat §3.4.3.)*

**Implikasi:** jika ini part-time, "produk untuk publik" adalah komitmen
multi-tahun. Itu mungkin saja benar — tapi harus diputuskan dengan mata terbuka,
bukan ditemukan 18 bulan kemudian.

---

## 8. Functional Requirements

Prioritas: **M** = Must (1.0), **S** = Should, **C** = Could, **W** = Won't (rilis ini).

### 8.1 Workflow

| ID | Requirement | P |
|---|---|---|
| FR-WF-01 | Impor workflow JSON format n8n v1.x | M |
| FR-WF-02 | Simpan workflow dengan versioning (D89) | M |
| FR-WF-03 | Ekspor kembali ke format yang bisa dibaca n8n | S |
| FR-WF-04 | Validasi workflow sebelum eksekusi (node ada, koneksi valid, parameter valid) | M |
| FR-WF-05 | Sub-workflow (Execute Workflow node) | S |
| FR-WF-06 | Aktif/nonaktifkan workflow | M |

### 8.2 Eksekusi

| ID | Requirement | P |
|---|---|---|
| FR-EX-01 | Eksekusi manual (dari UI/CLI) | M |
| FR-EX-02 | Eksekusi terjadwal (cron) | M |
| FR-EX-03 | Eksekusi via webhook | M |
| FR-EX-04 | Retry dengan backoff pada error transient (D14/D57) | M |
| FR-EX-05 | Error workflow (n8n "Error Workflow") | S |
| FR-EX-06 | Cancel eksekusi berjalan (D20, kooperatif) | M |
| FR-EX-07 | Resume setelah crash dari checkpoint (D6/D40) | M |
| FR-EX-08 | Wait node — persist, tidak menahan worker (D17) | M |
| FR-EX-09 | Execution history dengan filter & pencarian | M |
| FR-EX-10 | Tandai `InDoubt` → `NEEDS_REVIEW` untuk side-effect non-idempotent yang crash (A-10) | M |
| FR-EX-11 | Execution order v1 dan v0 legacy (D104) | M / S |
| FR-EX-12 | Partial re-run dari node tertentu | C |

### 8.3 Semantik data

| ID | Requirement | P |
|---|---|---|
| FR-DT-01 | Semantik array item identik dengan n8n (A-03) — **sudah terbukti di kernel** | M |
| FR-DT-02 | Spill-to-disk transparan untuk node | M |
| FR-DT-03 | Binary data (file) dengan storage terpisah (D105) | M |
| FR-DT-04 | `pairedItem` tracking untuk lineage | S |
| FR-DT-05 | Static data per-workflow/per-node (D101), flush hanya production (D106) | M |

### 8.4 Expression

| ID | Requirement | P |
|---|---|---|
| FR-EXP-01 | Evaluasi `{{ ... }}` kompatibel n8n | M |
| FR-EXP-02 | `$json`, `$input`, `$binary`, `$now`, `$today`, `$vars`, `$execution`, `$workflow` | M |
| FR-EXP-03 | `$('Node')`, `$items()`, `$node["X"]` — akses acak output node lain (A-01) | M |
| FR-EXP-04 | `$env` dengan allowlist (D94) | M |
| FR-EXP-05 | Semua helper bawaan n8n (`$jmespath`, dll.) | S |
| FR-EXP-06 | Validasi expression saat user mengetik di editor | S |

> ⚠️ **FR-EXP adalah tempat "paritas" paling sering mati.** Expression n8n
> adalah JavaScript dengan banyak helper dan perilaku edge yang tidak
> terdokumentasi. Mengklaim kompatibel tanpa korpus uji besar adalah klaim
> kosong. Lihat §10.3 dan risiko R3.

### 8.5 Node

| ID | Requirement | P |
|---|---|---|
| FR-ND-01 | Node inti MVP — daftar persis di **OPEN-3** | M |
| FR-ND-02 | HTTP Request node (paling kritis) | M |
| FR-ND-03 | Code node (JS via QuickJS, sandboxed) | M |
| FR-ND-04 | Node dari OpenAPI spec via codegen (D108/A-02) | S — *tapi ini diferensiator utama, lihat §13* |
| FR-ND-05 | SDK untuk node kustom pihak ketiga (P4) | C |

### 8.6 Kredensial & keamanan

| ID | Requirement | P |
|---|---|---|
| FR-CR-01 | Kredensial terenkripsi at-rest | M |
| FR-CR-02 | Least privilege — node hanya baca kredensial yang dideklarasikan (D92) | M |
| FR-CR-03 | Redaksi kredensial di log & error (D93) — **sudah di tipe `CredentialValue`** | M |
| FR-CR-04 | Sharing kredensial antar user | S |
| FR-CR-05 | Rotasi encryption key | C |

### 8.7 Produk & operasi

| ID | Requirement | P |
|---|---|---|
| FR-API-01 | REST API untuk semua operasi | M |
| FR-API-02 | API key auth + session auth | M |
| FR-MT-01 | Multi-user dengan role (owner/admin/member) | M |
| FR-MT-02 | Multi-tenant — isolasi workspace/tenant (§3.4) | **M** (dikonfirmasi) |
| FR-MT-03 | Skema storage tenant-aware; setiap query ter-scope, di-enforce di tingkat tipe/query-builder bukan disiplin programmer | **M** |
| FR-MT-04 | **Test otomatis anti-kebocoran lintas tenant** — setiap endpoint dicoba membaca data tenant lain dan harus gagal. Bug kelas tertinggi (§3.4.4) | **M** |
| FR-MT-05 | Encryption key per-tenant untuk kredensial. Satu key global = kompromi satu tenant membocorkan semua | **M** |
| FR-MT-06 | Audit log akses kredensial & perubahan workflow; backup/restore **per-tenant** | **M** |
| FR-MT-07 | Kuota disk spill per-tenant + fair scheduling antar tenant di execution queue | **M** |
| FR-OB-01 | Structured logging | M |
| FR-OB-02 | Metrics (Prometheus-compatible) | S |
| FR-OB-03 | Health check endpoint | M |
| FR-ON-01 | Installer / Docker image | M |
| FR-ON-02 | Dokumentasi pengguna | M |
| FR-ON-03 | Wizard onboarding pertama kali | S |
| FR-LIC-01 | Aktivasi lisensi / enforcement | M — *tapi model bisnisnya OPEN-4* |
| FR-UP-01 | Upgrade backward-compatible + rollback (D99) | M |

---

## 9. Non-Functional Requirements

Setiap NFR punya **target terukur** dan **cara mengukurnya**. NFR tanpa cara
ukur adalah harapan, bukan requirement.

| ID | Kategori | Target | Cara ukur |
|---|---|---|---|
| NFR-MEM-01 | Memori idle | ≤ 60 MB RSS | Binary jalan tanpa workflow aktif |
| NFR-MEM-02 | Memori beban berat | Peak RSS ≤ 25% dari n8n pada workload identik | Benchmark head-to-head, script otomatis |
| NFR-MEM-03 | Batas absolut | Workflow 5 juta item jalan di mesin 2 GB | **Sudah terbukti di tingkat kernel** — harus diulang di engine penuh |
| NFR-PERF-01 | Throughput | ≥ n8n untuk workflow I/O-bound | Benchmark |
| NFR-PERF-02 | Latency spill | Overhead ≤ 25% vs inline | **Terukur 19%** di kernel — pertahankan |
| NFR-PERF-03 | Cold start | ≤ 500 ms ke siap-terima-request | Timer |
| NFR-REL-01 | Crash recovery | Tidak ada eksekusi hilang; tidak ada side-effect terduplikasi diam-diam | Fault injection test |
| NFR-REL-02 | Durability | Setelah SIGKILL, restart melanjutkan dari checkpoint | Test otomatis |
| NFR-SEC-01 | Kredensial | Tidak pernah muncul di log, error, atau response API | Test + review |
| NFR-SEC-02 | Sandboxing | Code node tidak bisa baca FS/host di luar yang diizinkan | Uji penetrasi |
| NFR-SEC-03 | Dependency | Audit CVE otomatis di CI | CI gate |
| NFR-COMPAT-01 | Impor | ≥95% workflow dari korpus uji impor tanpa error | Korpus — lihat §10 |
| NFR-OPS-01 | Disk | Spill file ter-GC; tidak ada leak | Long-running test + assert |
| NFR-OPS-02 | Upgrade | Rollback ke versi sebelumnya aman | Test |

> ⚠️ **NFR-MEM-02 dan NFR-COMPAT-01 butuh artefak yang belum ada:**
> (a) harness benchmark head-to-head melawan n8n nyata, (b) korpus workflow uji.
> Keduanya harus dibangun di Fase 1, bukan ditunda. Tanpa keduanya, semua klaim
> di dokumen pemasaran tidak bisa diverifikasi — dan itu persis kesalahan yang
> ditemukan audit A-03.

---

## 10. Kompatibilitas n8n — definisi persis

"Setara dengan n8n" tanpa definisi adalah klaim yang tidak bisa diuji. Ini
rinciannya.

### 10.1 Yang kompatibel

- **Format workflow JSON** (struktur node, connections, settings) — untuk node yang kita dukung
- **Semantik item array** — teruji di kernel
- **Expression syntax** `{{ }}` dan variabel `$` utama
- **Perilaku execution order** v1 dan v0

### 10.2 Yang TIDAK kompatibel

- **Node komunitas npm** — kode JS, tidak akan didukung (menghancurkan keunggulan memori)
- **Node di luar subset yang didukung** — workflow yang memakainya gagal impor dengan pesan eksplisit, bukan diam-diam salah
- **Behavior bug n8n** — jika n8n punya perilaku yang jelas bug, kita tidak mereplikasi. Harus didokumentasikan per kasus.

### 10.3 Cara membuktikan kompatibilitas

**Wajib dibangun di Fase 1:**

1. **Korpus workflow uji** — minimal 50 workflow n8n nyata (dari template publik n8n, workflow Anda sendiri, dan kontribusi). Bukan ditulis tangan untuk lulus.
2. **Differential testing** — jalankan workflow yang sama di n8n dan di engine kita, bandingkan output item-per-item.
3. **Katalog deviasi** — setiap perbedaan yang disengaja dicatat dan dipublikasikan.

Tanpa ketiganya, FR-EXP dan NFR-COMPAT-01 tidak bisa diklaim.

---

## 11. Legal & Lisensi

> 🔴 **Bagian ini adalah risiko tertinggi untuk produk publik dan saya bukan
> lawyer.** Yang berikut adalah analisis teknis, **bukan nasihat hukum**. Sebelum
> rilis publik, dapatkan review dari pengacara yang paham lisensi software —
> idealnya di yurisdiksi target pasar Anda, bukan hanya Indonesia.

### 11.1 Fakta yang sudah diverifikasi

- Lisensi n8n: **Sustainable Use License** (fair-code) + n8n Enterprise License untuk sebagian kode
- SUL mengizinkan: penggunaan internal, self-hosting
- SUL **melarang**: menyediakan n8n ke pihak ketiga sebagai layanan hosted, white-labeling, menghapus/mengaburkan fitur berlisensi
- **"n8n" adalah merek dagang n8n GmbH**

### 11.2 Posisi kita

Kita menulis **implementasi bersih dari nol dalam Rust, tanpa berbagi kode**.
Secara umum:
- Hak cipta melindungi **ekspresi** (kode), bukan **fungsi** atau **ide**
- Reimplementasi API/format umumnya lebih aman daripada fork — ada preseden (mis. Google v. Oracle di AS soal reimplementasi API), tapi **ini bukan yurisdiksi Anda dan bukan area yang bisa saya pastikan**
- Format JSON workflow adalah data fungsional, bukan karya kreatif

**Ini posisi yang secara teknis jauh lebih kuat daripada fork n8n.** Tapi "lebih
kuat" bukan "pasti aman".

### 11.3 Yang HARUS dihindari

| Jangan | Kenapa |
|---|---|
| Menyalin kode n8n (bahkan "sedikit", bahkan dengan modifikasi) | Mengubah clean-room menjadi derivative work |
| Menggunakan nama "n8n" di nama produk, domain, atau logo | Merek dagang |
| Melihat kode sumber n8n saat menulis implementasi | Mengkontaminasi clean-room. **Idealnya ada pemisahan: yang baca spec ≠ yang menulis kode.** |
| Klaim "n8n compatible" tanpa dasar | Nominative fair use mungkin berlaku, tapi butuh kehati-hatian dan dasar faktual (§10.3) |
| Mengklaim node n8n spesifik dengan nama yang identik | Area abu-abu; gunakan nama generik ("HTTP Request" aman, nama brand node n8n tertentu mungkin tidak) |

### 11.4 Yang harus diputuskan

> ⚠️ **OPEN-5: Lisensi produk kita sendiri.** Pilihan:
> - **Proprietary/commercial** — paling jelas untuk dijual, tapi menghambat adopsi P1
> - **Open-core** (engine open source, fitur enterprise berbayar) — strategi n8n sendiri, terbukti bekerja
> - **Fair-code** (mirip SUL) — membatasi kompetitor hosted
> - **Full open source** (MIT/Apache) — adopsi maksimal, monetisasi paling sulit
>
> Ini keputusan bisnis, bukan teknis. **Tidak saya putuskan.**

> ⚠️ **OPEN-6: Nama produk & brand.** Butuh cek merek dagang sebelum dipakai.

---

## 12. Metrik Sukses

### 12.1 Metrik teknis (bisa diukur sekarang)

| Metrik | Target | Status |
|---|---|---|
| Peak RSS vs n8n, workload identik | ≤ 25% | Harness belum ada |
| Workflow 5 juta item di mesin 2 GB | Lolos | ✅ Terbukti di tingkat kernel |
| Overhead spill vs inline | ≤ 25% | ✅ Terukur 19% |
| Korpus workflow impor sukses | ≥ 95% | Korpus belum ada |
| Test coverage kernel | ≥ 80% | 19 test; coverage belum diukur |
| Onboarding time | < 30 menit | Fase 4 |

### 12.2 Gate validasi pasar (G6) — **gate terpenting**

> **Jangan masuk FASE 5 sebelum gate ini lolos.**

Alasan: §5.3 — bukti teknis kuat, bukti pasar nol. Membangun Fase 3-4 (6-10
bulan) sebelum memvalidasi bahwa ada yang mau bayar adalah cara paling mahal
untuk menemukan jawaban itu.

**Gate G6 lolos jika minimal 3 dari 5 berikut terpenuhi:**

| # | Sinyal | Ambang |
|---|---|---|
| 1 | Orang di luar Anda yang deploy dan menjalankan Fase 1 (headless) | ≥ 5 orang |
| 2 | Orang yang melaporkan masalah nyata dan kembali untuk perbaikan | ≥ 3 orang |
| 3 | Orang yang menyatakan bersedia membayar, dengan angka | ≥ 10 orang, ≥$5/bulan |
| 4 | Pre-order / letter of intent | ≥ 3 (untuk P2/agency) |
| 5 | Trafik organik ke landing page yang menjelaskan positioning §3.3 | ≥ 1.000 kunjungan, ≥3% konversi ke waitlist |

**Cara termurah menguji gate ini: kerjakan Fase 1, lalu rilis headless ke publik
sebelum membangun UI.** Engine headless yang bisa menjalankan workflow JSON
sudah cukup untuk menguji apakah orang peduli pada efisiensi memori. Jika tidak
ada yang peduli pada versi headless, UI tidak akan mengubah itu.

---

## 13. Risiko & Adversarial Review

Bagian ini ditulis untuk **menantang** PRD, bukan mendukungnya. Setiap risiko
punya mitigasi yang bisa dikerjakan. Jika mitigasi tidak dikerjakan, risiko
itu nyata.

### R1 — 🔴 Scope: "produk untuk publik" solo adalah komitmen multi-tahun
**Tantangan:** §7.3 memperkirakan 9-18 bulan full-time, atau 26-50 bulan part-time, hanya untuk mencapai produk yang bisa dijual. Audit memberi skor realisme scope 3/10. Pola historis Anda: 88 keputusan dibuat tanpa tekanan penolakan — indikasi optimisme sistematis.
**Mitigasi:** Fasilitasi pemfasaan §7.2 yang memaksa artefak bisa-pakai di tiap fase. **Jangan mulai Fase 3 (UI) sebelum Gate G6 lolos.** Jika part-time, pertimbangkan menurunkan target dari "produk publik" ke "alat pribadi yang dipublikasikan".
**Pemilik:** Anda.

### R2 — 🔴 Pasar: tidak ada bukti orang mau membayar
**Tantangan:** Pesaing gratis, 400+ node, komunitas besar, docs matang. Kita menawarkan 5-10% jumlah node dengan harga > 0. Keunggulan memori nyata tapi hanya relevan bagi yang sudah kena OOM — populasi yang lebih kecil dari yang diasumsikan.
**Mitigasi:** Gate G6 (§12.2) sebagai syarat masuk Fase 5. Rilis headless lebih awal untuk menguji dengan biaya minimum.
**Pemilik:** Anda.

### R3 — 🟠 Teknis: kompatibilitas expression adalah 80% kerja tersembunyi
**Tantangan:** Expression n8n adalah JavaScript dengan banyak helper dan perilaku edge yang tidak terdokumentasi. Tanpa korpus uji besar (§10.3), setiap klaim kompatibilitas kosong. Ini biasanya ditemukan terlambat — setelah UI jadi dan pengguna mencoba workflow nyata.
**Mitigasi:** Bangun differential testing di **Fase 1**, bukan Fase 3. Korpus minimal 50 workflow nyata.
**Pemilik:** Teknis.

### R4 — 🟠 Editor UI diremehkan secara sistematis
**Tantangan:** ~30% effort, kemungkinan komponen terlama. Canvas editor dengan drag-drop, koneksi bezier, undo/redo, form generation, execution overlay — ini produk frontend tersendiri. Tidak ada indikasi pengalaman Anda membangun ini.
**Mitigasi:** Tunda sampai Fase 3. Pertimbangkan memakai library canvas yang ada daripada menulis dari nol. **Pertimbangkan serius apakah MVP benar-benar butuh editor visual** — banyak pengguna P1 (teknis) bisa hidup dengan JSON + API.
**Pemilik:** Anda — terkait OPEN-7.

### R5 — 🟠 Legal: merek dagang & klaim kompatibilitas
**Tantangan:** §11. Produk publik memperbesar eksposur secara drastis dibanding alat pribadi. n8n GmbH adalah entitas aktif yang menjaga mereknya.
**Mitigasi:** Review pengacara sebelum rilis publik. Clean-room discipline (§11.3). Cek merek dagang untuk nama produk.
**Pemilik:** Anda.

### R6 — 🟡 Bus factor = 1
**Tantangan:** Satu orang. Sakit, kehilangan motivasi, atau perubahan situasi hidup menghentikan produk. Untuk produk publik dengan pengguna yang bergantung, ini bukan cuma risiko Anda tapi risiko mereka.
**Mitigasi:** Dokumentasi arsitektur yang cukup untuk orang lain melanjutkan (KERNEL-SPEC adalah awal yang baik). Open-core atau lisensi yang memungkinkan fork komunitas mengurangi risiko bagi pengguna.
**Pemilik:** Anda.

### R7 — 🟡 Beban dukungan untuk produk publik
**Tantangan:** Produk publik = tiket, issue, pertanyaan, CVE, rilis patch. Untuk solo developer ini bisa mengonsumsi seluruh waktu dan menghentikan pengembangan fitur.
**Mitigasi:** Batasi kanal dukungan di awal. Dokumentasi self-serve. Pertimbangkan dukungan berbayar saja. **Jangan janjikan SLA.**
**Pemilik:** Anda.

### R8 — 🟡 Codegen OpenAPI lebih sulit dari kelihatannya
**Tantangan:** G4/FR-ND-04 adalah diferensiator utama (ribuan API tanpa menulis node manual). Tapi spec OpenAPI publik berkualitas sangat bervariasi — banyak yang tidak lengkap, ambigu, atau salah. Codegen yang menghasilkan node rusak lebih buruk daripada tidak ada node.
**Mitigasi:** Kurasi. Mulai dari 10-20 spec berkualitas tinggi, bukan 1.000 spec acak. Uji setiap node hasil generate terhadap API nyata atau mock.
**Pemilik:** Teknis. **Catatan:** ini dijadwalkan di Fase 5 tapi merupakan diferensiator — pertimbangkan memajukan prototipe ke Fase 2.

### R9 — 🟡 Utang teknis yang sudah ada sekarang
**Tantangan:** Workspace Rust saat ini **rusak** — `Cargo.toml` mereferensikan 4 crate, hanya 2 manifest ada, `cargo build` gagal. Ada 3 direktori kosong (`data-plane` tanpa `src/`, `testkit`, `nodes-core`).
**Mitigasi:** Perbaiki sebelum pekerjaan lain apa pun. Dua pilihan: (a) lengkapi 3 crate itu, (b) hapus dan kembalikan ke 1 crate. **Jangan biarkan menggantung.**
**Pemilik:** Teknis — butuh keputusan Anda, lihat OPEN-8.

### R10 — 🟠 Multi-tenant memperbesar permukaan kegagalan keamanan
**Tantangan:** keputusan §3.4 menambah tenant isolation, key per-tenant, kuota, dan fair scheduling. Setiap satu adalah tempat kebocoran data bisa terjadi. Untuk solo developer tanpa tim keamanan, ini adalah kenaikan risiko yang nyata — dan pada produk publik, satu kebocoran lintas tenant bisa mengakhiri produk.
**Mitigasi:** FR-MT-04 (test anti-kebocoran otomatis untuk setiap endpoint) sebagai **gate Fase 2 yang tidak bisa ditawar**. Enforcement scoping di tingkat tipe/query-builder, bukan disiplin programmer — manusia lupa, type system tidak. Pertimbangkan audit keamanan eksternal sebelum rilis publik.
**Catatan jujur:** jika OPEN-9 ternyata (B) SaaS, risiko ini naik dari 🟠 ke 🔴 dan praktis membutuhkan lebih dari satu orang.
**Pemilik:** Anda + teknis.

### R11 — 🟢 Yang sudah terbukti (bukan risiko, tapi aset)
Kernel compile bersih, clippy bersih, 19/19 test hijau, dan keunggulan memori
**terukur pada perangkat keras target yang sebenarnya**. Ini nyata dan tidak
perlu diulang. Tapi ini adalah **lapisan kontrak** — nilainya adalah mencegah
tabrakan antar komponen nanti, bukan fitur yang bisa dijual.

---

## 14. KEPUTUSAN TERBUKA

> 🔴 **Semua item di bawah ini membutuhkan keputusan Anda. Saya sengaja tidak
> memutuskannya.** Setiap satu ditandai dengan apa yang berubah tergantung
> jawabannya.

### OPEN-1 — Persona primer
**Pilihan:** P1 (hobbyist) / P2 (agency) / P3 (SMB non-teknis) / P4 (developer)
**Mengubah:** prioritas fitur, harga, tingkat polish UI, kebutuhan dukungan.
**Catatan:** P3 praktis tidak bisa dilayani produk self-hosted murni (§4). Memilih P3 berarti mengubah model menjadi hosted — dan seluruh PRD ini harus ditulis ulang.
**Rekomendasi teknis (bukan keputusan):** P1 sebagai primer untuk MVP, P2 sebagai target ekspansi. P1 paling cocok dengan keunggulan terukur kita (VPS kecil) dan paling murah untuk dilayani.

### ~~OPEN-2~~ — Multi-tenant: ✅ DIPUTUSKAN (2026-09-09)
**Keputusan:** multi-tenant **masuk scope**, opsi (c) multi-workspace.
**Konsekuensi yang harus diterima:** lihat §3.4. Desain tenant harus masuk **Fase 1-2**, bukan Fase 5. Menunda berarti migrasi skema storage yang menyakitkan.

### OPEN-3 — Daftar node MVP
**Butuh:** daftar eksplisit 10-30 node untuk FR-ND-01.
**Mengubah:** ukuran Fase 1, dan apakah korpus uji (§10.3) bisa lulus.
**Catatan:** daftar ini harus diturunkan dari workflow nyata yang ingin Anda dukung, bukan dari daftar node n8n. **Saran proses:** kumpulkan 10-20 workflow n8n yang benar-benar ingin Anda jalankan, hitung node apa saja yang muncul, itu daftar MVP-nya.

### OPEN-4 — Model bisnis & lisensi
**Pilihan:** proprietary / open-core / fair-code / open source (lihat §11.4)
**Mengubah:** FR-LIC-01, strategi adopsi, eksposur legal, kelayakan Gate G6 sinyal #3-4.
**Catatan:** ini keputusan bisnis murni. Tidak ada jawaban teknis yang benar.

### OPEN-5 — Nama produk
**Butuh:** nama + cek merek dagang.
**Catatan:** jangan pakai apa pun yang mirip "n8n".

### OPEN-6 — Apakah MVP butuh editor visual?
**Pilihan:** (a) ya, Fase 3 wajib sebelum rilis, (b) tidak — rilis headless + API dulu, UI menyusul
**Mengubah:** waktu ke rilis publik berkurang 4-8 bulan jika (b).
**Catatan:** ini kemungkinan keputusan dengan leverage terbesar di seluruh PRD. Untuk P1 (teknis), JSON + API + dokumentasi baik mungkin cukup untuk memvalidasi pasar. Gate G6 bisa diuji **tanpa UI sama sekali**.
**Rekomendasi teknis (bukan keputusan):** (b). Uji pasar dengan headless. Bangun UI hanya setelah ada bukti permintaan.

### OPEN-7 — Status 3 crate yang belum selesai
**Pilihan:** (a) lengkapi `data-plane` + `testkit` + `nodes-core`, (b) hapus, kembalikan workspace ke kernel saja
**Mengubah:** apakah workspace bisa di-build sekarang (R9).
**Catatan:** (a) berguna — `testkit` akan menjadi bukti bahwa kontrak integrasi benar-benar bisa diimplementasikan, yang menutup celah A-05 secara nyata. Tapi itu pekerjaan kode, dan Anda meminta PRD dulu. Jadi: **tunda, tapi putuskan.**

### OPEN-8 — Full-time atau part-time
**Mengubah:** seluruh timeline (§7.3), dan apakah "produk untuk publik" realistis dalam horizon yang Anda bayangkan.
**Catatan:** jika part-time, angka 26-50 bulan perlu dihadapi sekarang, bukan nanti.

---

### OPEN-9 — Multi-tenant: skenario A (agency) atau B (SaaS)?
**Diperkenalkan oleh:** keputusan multi-tenant Anda, dirinci di §3.4.2.
**Pilihan:**
- **(A) Agency** — satu instance melayani 5-50 tenant yang Anda/klien kenal. Tetap self-hosted. **Konsisten dengan §3.2 dan positioning §3.3.**
- **(B) SaaS** — ratusan tenant asing per instance. Ini **bukan** self-hosted product lagi; bertentangan dengan §3.2 dan mengubah model bisnis jadi hosted.
**Mengubah:** jika (B), seluruh §3.2, §11 (legal — SUL n8n secara eksplisit melarang menyediakan layanan hosted berbasis n8n; posisi kita lebih aman karena clean-room, tapi tetap butuh review), pricing, dan arsitektur isolasi harus ditulis ulang.
**Asumsi saya saat menulis PRD ini:** (A), karena Anda menulis "self-hosted product atau lisensi". **Perlu konfirmasi eksplisit.**

## 15. Langkah Berikutnya (setelah PRD disetujui)

**Jangan mulai di sini sebelum §14 ditutup.**

1. ~~Tutup OPEN-2~~ ✅ sudah diputuskan (multi-tenant masuk scope, §3.4).
   **Tutup OPEN-9, OPEN-1, OPEN-6** — ketiganya mengubah bentuk PRD secara material. Yang lain bisa menyusul.
2. Perbaiki R9 (workspace rusak) — keputusan OPEN-7.
3. Bangun artefak verifikasi yang belum ada: harness benchmark head-to-head + korpus workflow uji (§9, §10.3). Tanpa keduanya, tidak ada klaim yang bisa dibuktikan.
4. Revisi PRD ini ke v1.0 berdasarkan keputusan §14.
5. Baru kemudian: Fase 1.

---

## 16. Catatan Governance

Proses yang menghasilkan D1–D100 gagal karena **yang merekomendasikan juga yang
menyetujui**. Dokumen ini mencoba memperbaiki itu dengan tiga cara:

1. **§14 memisahkan rekomendasi dari keputusan.** Saya boleh menulis "rekomendasi teknis", tapi labelnya eksplisit dan keputusannya tetap milik Anda.
2. **§13 ditulis untuk menantang dokumen ini sendiri**, bukan mendukungnya.
3. **§12.2 memasang gate yang bisa menggagalkan rencana.** Gate yang tidak pernah bisa gagal bukan gate.

Yang **belum** diperbaiki, dan harus Anda sadari:
- Saya masih satu-satunya penulis dokumen ini. Adversarial review dari AI yang sama dengan yang menulis rencananya jauh lebih lemah daripada review dari pihak yang benar-benar independen.
- Tidak ada bukti pasar. §5.3 dan R2 nyata, dan tidak ada dokumen yang bisa memperbaikinya — hanya kontak dengan pengguna nyata yang bisa.
- Estimasi §7.3 tidak terkalibrasi oleh data historis Anda.

**Cara terkuat memperbaiki ketiganya: tunjukkan dokumen ini ke orang lain — idealnya seseorang yang pernah menjual produk developer, dan seseorang yang tidak setuju dengan Anda.** Itu satu-satunya adversarial review yang benar-benar independen.

---

*Dokumen ini adalah DRAFT v0.1. Bukan persetujuan. §14 harus ditutup oleh Anda sebelum implementasi dimulai.*
