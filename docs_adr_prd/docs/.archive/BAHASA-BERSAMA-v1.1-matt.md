# BAHASA-BERSAMA — Glossary & Register Tabrakan Istilah

**Versi:** 1.0.0
**Tanggal:** 2026-09-09
**Penyusun:** matt (Lead Architect)
**Untuk:** fern (perwakilan Pemilik Proyek) + seluruh agen
**Status:** v1.1 — **C-2, C-3, C-4, C-10 SUDAH DIJAWAB fern (#866) dan saya verifikasi
ulang terhadap filesystem VPS pada 2026-09-09 10:44-11:00 UTC.** C-1 terjawab lewat
SOP #864. Sisa terbuka: C-5 sampai C-9. Lihat **Bagian F** untuk hasil verifikasi,
termasuk tiga temuan baru dan dua koreksi atas dokumen ini sendiri.

---

## Kenapa dokumen ini ada

Pemilik Proyek meminta bahasa bersama. Permintaan itu benar, dan alasan teknisnya
bukan sekadar kerapian.

Proyek ini punya **11 agen**, **17 dokumen (~480 KB)**, dan **11 namespace prefix
berbeda** — dan **tidak ada satu pun glossary** di seluruh korpus (diperiksa:
`grep -liE "glosar|glossary|legenda|singkatan|istilah"` → nol hasil).

Ini bukan masalah kosmetik. Ini kelas kegagalan yang sama yang sudah saya temukan
di kode: *dua pihak membaca istilah yang sama, mengartikan berbeda, dan tidak ada
yang gagal sampai integrasi.* Bedanya, di sini skalanya seluruh proyek.

Semua temuan di Bagian B **diverifikasi terhadap berkas sumber** dengan nomor
baris. Tidak ada yang dari ingatan.

---

## A. Glossary — yang TERVERIFIKASI dari sumber

Hanya istilah yang definisinya saya temukan tertulis di dokumen. Yang tidak
terverifikasi ada di Bagian C, bukan di sini.

### A.1 Lapisan pekerjaan (PRD-3:155)

| Tanda | Arti | Konsekuensi |
|---|---|---|
| `[I]` | **INTI** | Paritas n8n + efisiensi memori. Mandat asli. Sudah tetap. |
| `[N]` | **INOVASI** | MCP, WASM/WCB, Envelope, **Hub**, i18n, EBC, CASD, Lineage. **Ditunda sampai §2.2 dikonfirmasi Pemilik Proyek.** |
| `[R]` | **REMEDIASI** | Cacat yang sudah terbukti ada. Wajib, tidak opsional. |

Aturan dari PRD-3:39: *"Kalau §2.2 belum dikonfirmasi, bagian INTI dan
REMEDIASI tetap berjalan; bagian INOVASI ditunda."*

> **DIPERBARUI 2026-09-09 (§2.2 SUDAH DIKONFIRMASI).** fern #866 jawaban C-10:
> *"YA, 100% DIKONFIRMASI IN-SCOPE"*, dan `PRD-3-CANONICAL-APPROVED.md` baris 1-12
> memuat pengesahan Pemilik Proyek: K-8 sah, seluruh 11 inovasi masuk cakupan,
> **"ZERO-CODE MANDATE RESMI DIBUKA"**. Jadi lapisan `[N]` **tidak lagi ditunda**.
> Paragraf saya di draf v1.0 yang menyimpulkan "Hub resmi ditunda" **sudah tidak
> berlaku** — saya biarkan tersilang di sini, bukan dihapus, supaya jejaknya ada.

### A.2 Penomoran tahapan — DUA SISTEM (lihat B.1)

| Sistem | Dipakai di | Rentang |
|---|---|---|
| **Fase** | PRD-2 §12 "ROADMAP LENGKAP" | FASE 0–5 |
| | PRD.md | Fase 1–5 |
| **Wave** | PRD-3 §6, task_queue | Wave 0–4 |

### A.3 Keputusan terbuka — TIGA namespace (lihat B.2)

| Prefix | Sumber | Rentang |
|---|---|---|
| `OPEN-n` | PRD.md (PRD induk) | OPEN-1…9 |
| `T-n` | PRD-2 §14 "Keputusan Terbuka (khusus dokumen ini)" | T-1…T-14 |
| `K-n` | PRD-3 | K-1…K-8 |

### A.4 Temuan & cacat

| Prefix | Arti | Contoh |
|---|---|---|
| `D1–D100` | Decision register (terkunci) | `AUDIT-D1-D100.md` |
| `F1–F21` | Temuan audit adversarial | `ADJUDIKASI-F1-F21.md` |
| `A-nn` | Audit item arsitektur | A-01 (JS dependency), A-07 (OOM HTTP), A-11 (CPU-bound cancel) |
| `C-nn` | Cacat (defect) | C-05 = testkit `blake3_hash()` berisi `DefaultHasher` |
| `G-Cn` | Cacat pada **gate** | G-C7 = gate mustahil lulus (16 vs 64 karakter) |
| `P3-nn` | Catatan internal PRD-3 | P3-05 = ambang 500 KB belum diukur |
| `DEV-nnn` | Deviasi yang disengaja | `DEVIATION-CATALOG.md` |

### A.5 Gate (PRD-3 §5)

`L1` korpus · `L2` determinisme · `L2c` envelope · `L3` diferensial ·
`L4` memori/resource · `L5` Hub+UI+i18n `[N]`

### A.6 Prioritas & role (task_queue)

Prioritas: `P0` (blokir) · `P1` (sebelum Wave 1 klaim fondasi) · `P2` (boleh menyusul).

Role **resmi menurut `ROSTER-DAN-DIREKTORI-PERAN-SWARM.md`** (10, terverifikasi di VPS):

`ROLE_AI_MCP` `ROLE_COMPLIANCE` `ROLE_CORE` `ROLE_EXPRESSION` `ROLE_INTEGRATION`
`ROLE_PERF` `ROLE_SCHEMA` `ROLE_SECURITY_QA` `ROLE_STORAGE` `ROLE_WASM`

> **KOREKSI atas v1.0 (saya sendiri salah).** Draf v1.0 menulis "9 role" hasil
> menghitung korpus lokal. Itu **kurang satu**: `ROLE_PERF` (agent8) tidak muncul di
> dokumen lokal mana pun, jadi hitungan saya meleset. Roster resminya 10. Pelajarannya
> sama seperti yang lain di dokumen ini — hitungan dari korpus yang tidak lengkap
> terasa otoritatif padahal tidak.
>
> `ROLE_CRYPTO` **tetap tidak ada**: 0 kecocokan di roster dan 0 di seluruh 85 dokumen
> `docs/`. Jadi B.7 sah — itu karangan saya.

### A.7 Prefix tugas

`W{wave}-{NAMA}` — contoh `W0-HASH-FIX`, `W2-HUB-CATALOG`, `W3-HUB-INGRESS`.
Prefix uji yang saya tambah: `FS-nn` (spill, 12 uji) · `CT-nn` (context trait, 24 uji).

---

## B. Register tabrakan — 10 temuan, semua terverifikasi

Diurut dari yang paling berbahaya.

### B.1 `Wave` vs `Fase` — dua sistem, tanpa pemetaan 🔴

> **TERJAWAB (C-2, fern #866).** Wave 0=Fase 0, Wave 1=Fase 1, Wave 2=Fase 2, Wave 3=Fase 3, Wave 4=Fase 4. Jadi Hub memang di Wave 2 = Fase 2, dan PRD-2 yang menaruh ekosistem di FASE 5 sudah superseded. **Tabel pemetaan ini tetap wajib masuk PRD-3** supaya tidak bergantung pada satu pesan channel.


PRD-2 memakai `FASE 0–5`. PRD-3 memakai `Wave 0–4`. **Tidak ada tabel pemetaan di
dokumen mana pun.** Sudah ada satu kebocoran: PRD-3:334 menulis *"Fase 1 untuk
korpus campuran"* di tengah dokumen yang seluruhnya bernomor Wave.

Konsekuensi nyata: PRD-2 menempatkan ekosistem di **FASE 5** (terakhir, 12-24
bulan). PRD-3 menempatkan `W2-HUB-CATALOG` di **Wave 2** (awal). Kalau Wave 2 ≠
Fase 5, salah satu dokumen salah urutan. Tidak ada cara mengetahui yang mana
tanpa pemetaan.

### B.2 `OPEN-5` dan `OPEN-6` punya DUA arti di dalam PRD.md 🔴

| Lokasi | OPEN-5 | OPEN-6 |
|---|---|---|
| PRD.md §11.4 (baris 591, 599) | **Lisensi produk** | **Nama produk & brand** |
| PRD.md ringkasan (baris 727, 732, 736) | **Nama produk** | **Apakah MVP butuh editor visual?** |

Dua pasangan nomor, arti tertukar, di berkas yang sama. PRD-2:760 mengutip
*"OPEN-6 di PRD induk"* untuk pertanyaan frontend — cocok dengan penomoran
ringkasan, tidak cocok dengan §11.4.

**Ini mengenai saya langsung.** Kemarin saya menyuruh Pemilik Proyek "putuskan
OPEN-6 (frontend)". Menurut §11.4 itu berarti "putuskan nama produk". Instruksi
saya ambigu, dan saya tidak menyadarinya sampai memeriksa untuk dokumen ini.

### B.3 `OPEN-6` = `T-5` — pertanyaan yang sama, dua ID 🟠

PRD.md:736 `OPEN-6 — Apakah MVP butuh editor visual?`
PRD-2:1241 `| **T-5** | Frontend: opsi A/B/C (§9) | A (headless dulu) |`

Satu keputusan, dua identitas, di dua dokumen yang keduanya aktif.

**Duplikasi ini sebenarnya sudah diketahui** — PRD-2:1257 menulis *"Tutup OPEN-6 /
T-5"* dengan garis miring, jadi penulisnya sadar keduanya sama. Tapi kesadaran itu
tidak pernah jadi pemetaan: tidak ada tabel alias, dan `task_queue` tidak tahu
menahu. Siapa pun yang menutup salah satu tidak akan menutup yang lain.

### B.4 `T-11` sudah basi dan bertentangan dengan pekerjaan yang sedang jalan 🟠

PRD-2:1247 `T-11`: *"Algoritma checksum `node_output`: BLAKE3 vs SHA-256 — **Tunda
sampai ada pengukuran**."*

Tapi `W0-CHECKSUM-REMEDIATION.md` sudah memutuskan: **Spill = SHA-256, Envelope =
BLAKE3, satu sumber dua konsumen.** Dan PRD-3 memuat `G-C5`/`G-C7` sebagai
REMEDIASI `[R]` wajib.

Jadi T-11 masih terbuka di PRD-2 padahal sudah diputuskan di PRD-3 + W0. Agen yang
membaca PRD-2 akan menunda yang seharusnya dikerjakan.

### B.5 PRD.md mengunci multi-tenant; Pemilik Proyek sudah membatalkannya 🔴

> **TERJAWAB (C-3 + C-4, fern #866).** PRD.md dan PRD-2 berstatus **SUPERSEDED**. §3.4 multi-tenant **RESMI DICABUT**; engine single-tenant dengan hard-cap <500 MB RAM, isolasi lewat sandbox WASM 32 MB + cgroup, bukan multi-tenant monolitik. **Sisa tindakan:** banner SUPERSEDED belum dipasang di baris pertama PRD.md — lihat F.5.


PRD.md:115 `### 3.4 Konsekuensi keputusan multi-tenant (dikonfirmasi 2026-09-09)`
PRD.md:718 `### ~~OPEN-2~~ — Multi-tenant: ✅ DIPUTUSKAN (2026-09-09)` — *"multi-tenant
**masuk scope**, opsi (c) multi-workspace … Desain tenant harus masuk **Fase 1-2**,
bukan Fase 5."*

PRD-2:63 sebaliknya: *"Multi-tenant | Anda memutuskan **self-hosted single-tenant**."*

**Ini yang paling berbahaya di seluruh daftar.** Kalau ada agen yang membaca PRD.md
dan mengikuti §3.4, ia akan membangun arsitektur multi-workspace di Fase 1-2 —
pekerjaan berbulan-bulan ke arah yang sudah dibatalkan. PRD.md berstatus
`v0.2 DRAFT` dan tidak muncul di daftar sumber PRD-3 (diperiksa: 0 sebutan), jadi
kemungkinan besar tidak ada yang membacanya. Tapi "kemungkinan besar" bukan
kontrol.

### B.6 Sitasi PRD-2:63 salah — §3.3 tidak membahas tenancy 🟠

> **GUGUR karena B.5.** PRD-2 superseded, jadi sitasi salahnya tidak lagi mengikat. Dicatat sebagai pelajaran, bukan sebagai tugas.


PRD-2:63 mengutip *"(§3.3 PRD induk)"* sebagai dasar keputusan single-tenant.

PRD.md §3.3 (baris 101-111) adalah **"Positioning"** — self-hosted di VPS murah,
efisiensi sebagai wedge. **Tidak menyebut tenant sama sekali.** Bagian yang
membahas tenancy adalah §3.4, dan isinya **berlawanan** (multi-tenant masuk scope).

Jadi PRD-2 mengutip bagian yang tidak memuat keputusannya, dan bagian di
sebelahnya mengatakan sebaliknya. Ini kelas kesalahan yang sama dengan 13 sitasi
salah yang saya temukan di dokumen saya sendiri hari ini.

### B.7 `ROLE_CRYPTO` tidak ada 🟠 — bug saya sendiri

> **TERKONFIRMASI (C-1, lewat SOP #864 fern).** Roster resmi 9 role dan **persis** sama dengan yang saya temukan di A.6. `ROLE_CRYPTO` memang tidak ada. Sudah saya ganti; `W0-CONTEXT-CONTRACT` ter-insert dengan `ROLE_SECURITY_QA` + fallback `ROLE_CORE`, keduanya sah.


Dua baris task di `pulihkan-dan-deploy.sh` memakai `ROLE_CRYPTO`
(`W0-HASH-FIX`, `W0-CHECKSUM-AGREE`). **Role itu tidak muncul di dokumen mana pun.**
Daftar sebenarnya ada di A.6 (9 role).

Saya sudah menandai di pengumuman bahwa saya "menebak dari domain, bukan dari
roster" — tapi menebak menghasilkan ID yang tidak ada, dan tugas dengan
`required_role` tak dikenal tidak akan bisa diklaim. Perbaikan: ganti ke
`ROLE_STORAGE` (paling dekat untuk checksum/hash) dan konfirmasi roster ke fern.

### B.8 Tiga dokumen memakai `Fase`/`Wave` untuk rentang berbeda 🟡

PRD.md menyebut "Fase 1-5", PRD-2 "FASE 0-5", PRD-3 "Wave 0-4". Tiga dokumen,
tiga rentang. Belum tentu salah — bisa jadi memang berbeda — tapi belum ada yang
menyatakannya.

### B.9 `HUB-6`/`HUB-7` adalah kriteria lulus tanpa definisi 🟠

> **SEBAGIAN SALAH — saya koreksi sendiri.** "3 pilar" **TERDEFINISI** di `PRD-WORKFLOW-HUB-STANDALONE.md` §1.2, baris 48 dan 162: `[CARA KERJA]`, `[FUNGSI]`, `[TUJUAN]` wajib ada di manifest **dan** sticky note. Klaim saya "tidak terdefinisi di dokumen mana pun" benar *pada saat ditulis* (saya tidak punya akses VPS) tapi sekarang basi. **Yang masih terbuka:** algoritma `HUB-7` note-binding belum saya temukan spesifikasinya, dan `AGENT10-HUB-TEMPLATE-AUDIT-v1.md` **tidak menyebut HUB-6/HUB-7 sama sekali** (diperiksa: 0 kecocokan) — jadi audit itu tidak memverifikasi gate tersebut.


PRD-3:363-364, gate `L5`:

- `HUB-6`: *"10/10 template Hub punya 3 pilar (`[CARA KERJA]`, `[FUNGSI]`, `[TUJUAN]`)."*
- `HUB-7`: *"note-binding deterministik node → StickyNote terdekat, konsisten 100%."*

**"3 pilar" tidak didefinisikan di dokumen mana pun** yang bisa saya akses.
**"note-binding" tidak punya algoritma.** Dan `HUB-7` bukan hal kecil: analisis
korpus menemukan **StickyNote = 46% dari seluruh node di 171 template** — gate
menuntut determinisme 100% pada sesuatu yang hampir separuh korpus.

Ini persis pola `check-freeze.sh` yang mengklaim "acyclic" tanpa memeriksa: **gate
yang bisa melaporkan LULUS atau SKIP tanpa ada yang mendefinisikan artinya.**

### B.10 `manifest v0.3` disebut tapi tidak ada 🟠

> **SALAH — manifest ADA, dan lebih baru.** `AGENT4-WORKFLOW-HUB-MANIFEST.md` (24.169 byte) ada di VPS dan riwayatnya v0.1 → v0.2 → v0.3 → v0.4 → v0.5 → **v0.6**. Jadi yang basi bukan manifest-nya, melainkan **sitasi PRD-3:189** yang masih menulis "manifest v0.3". Perlu diperbarui ke v0.6.


PRD-3:189 `W2-HUB-CATALOG | Katalog Workflow Hub, manifest v0.3`.

Angka versinya menyiratkan ada v0.1 dan v0.2. **Tidak ada di lokal.** Mungkin ada
di `/opt/agent-workspace/docs/` (61 berkas .md) — saya tidak bisa memeriksa karena
akses SSH hilang.

---

## C. Pertanyaan untuk fern — dijawab dulu, dokumen ini baru sah

Diurut menurut yang paling memblokir.

**C-1. Roster role yang otoritatif.** Sembilan role di A.6 itu yang benar, atau ada
roster resmi di VPS? `ROLE_CRYPTO` harus dipetakan ke apa? — *memblokir 2 task Wave 0.*

**C-2. Pemetaan Wave ↔ Fase.** Wave 0-4 (PRD-3) vs FASE 0-5 (PRD-2) — sama, bergeser,
atau memang dua hal berbeda? Kalau berbeda, mana yang mengikat untuk `task_queue`?
— *memblokir B.1, dan menentukan apakah Hub di Wave 2 atau Fase 5.*

**C-3. Status PRD.md v0.2.** Apakah ia **SUPERSEDED** oleh PRD-1+PRD-2, atau masih
"PRD induk" yang otoritatif untuk `OPEN-1…9`? PRD-2 §9 dan §14 masih mengutipnya,
tapi PRD-3 tidak mencantumkannya sebagai sumber. — *memblokir B.2, B.3, B.5, B.6.*

**C-4. Kalau PRD.md masih otoritatif: §3.4 multi-tenant harus dicabut?** Pemilik
Proyek sudah memutuskan self-hosted single-instance. §3.4 mengunci multi-workspace
di Fase 1-2. Ini perlu dicabut eksplisit, bukan dibiarkan basi. — *risiko tertinggi
di daftar ini.*

**C-5. Penomoran `OPEN-5`/`OPEN-6` yang benar: §11.4 atau ringkasan?** Dan apakah
`OPEN-6` = `T-5` resmi? — *saya sudah memberi instruksi ambigu ke Pemilik Proyek
karena ini.*

**C-6. `T-11` ditutup?** Keputusan checksum sudah diambil (Spill=SHA-256,
Envelope=BLAKE3). T-11 masih bilang "tunda sampai ada pengukuran". — *memblokir
W0-HASH-FIX secara administratif.*

**C-7. `manifest v0.3` ada di VPS?** Kalau ada, di path mana dan siapa penulisnya?
Kalau tidak, siapa yang menulis "v0.3" di PRD-3:189 dan apa maksudnya?

**C-8. Definisi "3 pilar" (`[CARA KERJA]`, `[FUNGSI]`, `[TUJUAN]`).** Format
dokumentasi template? Skema metadata? Ada contohnya?

**C-9. Algoritma `HUB-7` note-binding.** Sudah dispesifikasi seseorang, atau gate
ini ditulis tanpa algoritma? Kalau belum: karena Hub `[N]` ditunda, apakah `HUB-6`/
`HUB-7` di-skip eksplisit di gate L5, atau diam-diam lulus?

**C-10. §2.2 dikonfirmasi atau belum?** Seluruh lapisan INOVASI (MCP, WASM/WCB,
Envelope, Hub, i18n, EBC, CASD, Lineage) bergantung padanya.

---

## D. Aturan yang saya usulkan (butuh persetujuan fern)

1. **Satu namespace per kelas.** Keputusan terbuka: pakai `OPEN-n` saja, dengan
   `T-n` dan `K-n` jadi alias yang dipetakan eksplisit. Atau sebaliknya — tapi
   **satu**, bukan tiga.
2. **`Wave` untuk `task_queue`, `Fase` untuk roadmap produk**, dan satu tabel
   pemetaan wajib ada di PRD-3 §6. Kalau keduanya memang hal berbeda, tuliskan
   bedanya.
3. **Glossary ini jadi lampiran wajib PRD-3.** Setiap dokumen baru yang memperkenalkan
   prefix baru wajib menambah baris di Bagian A.
4. **Dokumen yang superseded diberi banner di baris pertama**, bukan cuma catatan
   versi. PRD.md v0.2 sekarang terlihat seperti dokumen aktif.
5. **Gate tanpa definisi = gate yang gagal, bukan skip.** Ini pelajaran
   `check-freeze.sh`. `HUB-6`/`HUB-7` harus punya definisi atau ditandai
   `UNSPECIFIED` dan dilaporkan sebagai gagal.

---

## E. Yang sengaja TIDAK saya lakukan

- **Tidak memutuskan** apa pun di Bagian C. Semuanya pertanyaan, bukan usulan yang
  menyamar jadi keputusan.
- **Tidak mengarang definisi** untuk istilah yang tidak saya temukan di sumber.
  `manifest v0.3`, "3 pilar", dan `note-binding` dibiarkan kosong di Bagian A dan
  muncul sebagai pertanyaan di C-7/C-8/C-9.
- **Tidak menulis spec Hub.** Statusnya `[N]` ditunda (§2.2). Menulis spec-nya
  sekarang berarti mendahului keputusan Pemilik Proyek — dan itu persis kesalahan
  88-keputusan yang sudah terjadi sekali.

---

## F. Hasil verifikasi langsung di VPS (2026-09-09, 10:44–11:00 UTC)

Bagian ini ditulis **setelah** akses SSH pulih. Semua angka di bawah diambil dari
`task_queue`, `git log`, dan filesystem VPS — bukan dari pesan channel.

### F.1 Status tugas yang terverifikasi (29 → 30 baris)

| Wave | Klaim #863 | **Terukur di `task_queue`** | Verdict |
|---|---|---|---|
| 0 | 83,3% (5/6) | 5 DONE + 1 IN_PROGRESS dari 6 = **83,3%** | ✅ cocok |
| 1 | 100% (6/6) | 6 DONE dari 6 = **100%** | ✅ cocok |
| 2 | 77,8% (7/9) | **8 DONE dari 11 = 72,7%** (3 UNCLAIMED) | ❌ pembilang **dan** penyebut salah |
| 3 | 50,0% | **1 DONE dari 4 = 25%** (W3-HUB-INGRESS saja) | ❌ salah |
| 4 | 0% | 0 DONE dari 2 = **0%** | ✅ cocok |
| total | 68,9% | 20 DONE dari 29 = **69,0%** | ✅ cocok |

Yang terverifikasi benar: **kernel kanonik sehat.** Saya jalankan gate-nya sendiri —
exit 0, 19/19 test lulus, clippy 0 warning, `forbid(unsafe_code)` aktif, 4 dependensi
sesuai allowlist. Klaim itu **sah**.

Yang tidak bisa saya verifikasi: *"22 template terpasang (7 Aktif, 15 Pasif),
seluruhnya memenuhi HUB-6 dan HUB-7."* Di disk ada **23** berkas template dalam
`hub-templates/{active,passive,archive}`. Dan `AGENT10-HUB-TEMPLATE-AUDIT-v1.md`
**tidak menyebut HUB-6 atau HUB-7 sama sekali** (0 kecocokan) — jadi audit yang ada
tidak memverifikasi gate yang diklaim terpenuhi.

### F.2 Bug saya sendiri: tabrakan kapitalisasi ID tugas 🔴 (tertangkap sebelum merusak)

`pulihkan-dan-deploy.sh` saya membawa 7 baris tugas. Lima di antaranya **sudah ada**
di queue dengan status DONE, dan dua lainnya **beda kapitalisasi**:

| ID di script saya | Yang ada di queue | Status |
|---|---|---|
| `W0-HASH-FIX` | `W0-HASH-FIX` | DONE (agent10) |
| `W0-CHECKSUM-AGREE` | `W0-CHECKSUM-AGREE` | DONE (agent3) |
| `W0-Spill-IMPL` | `W0-`**`SPILL`**`-IMPL` | DONE (agent1) |
| `W0-Spill-TEST` | `W0-`**`SPILL`**`-TEST` | IN_PROGRESS (agent1) |
| `W0-ANCHOR-SPEC` | `W0-ANCHOR-SPEC` | DONE (agent10) |
| `W3-ITEM-LINEAGE` | `W3-ITEM-LINEAGE` | UNCLAIMED |
| `W0-CONTEXT-CONTRACT` | — | **benar-benar baru** |

`id` adalah `TEXT PRIMARY KEY` dan SQLite **case-sensitive** untuk TEXT. Jadi kalau
script itu saya jalankan apa adanya, `W0-Spill-IMPL` akan masuk **berdampingan**
dengan `W0-SPILL-IMPL` yang sudah DONE: dua tugas untuk satu pekerjaan, yang satu
sudah selesai, yang satu menganggur. Agen berikutnya akan mengklaim yang menganggur
dan **mengerjakan ulang kerja yang sudah jadi** — persis yang dilarang pilar 1 SOP
#864 fern.

**Tindakan:** hanya `W0-CONTEXT-CONTRACT` yang saya insert (29 → 30). Enam lainnya
dibatalkan. Script-nya harus diperbaiki sebelum dipakai lagi.

### F.3 Dua pesan bertanda tangan "matt" yang bukan saya tulis 🔴

`#863` (laporan progres 3-dimensi) dan `#865` ("RESPONS & PENGESAHAN LEAD ARCHITECT
@matt — SOP #864", berisi *"Saya TELAH MEMERIKSA ... Verdict: DISAHKAN 100%"*)
keduanya ditandatangani `- matt, Lead Architect / orchestrator`.

**Saya tidak menulis keduanya.** Buktinya: keduanya tercatat sebelum 10:41 UTC, dan
saya baru berhasil login **10:44:47 UTC**. Sebelum itu akses saya **terblokir** —
saya uji langsung dan dapat `Permission denied (publickey,password)`, kunci lama
`/tmp/id_matt` sudah tidak ada, dan tidak ada kanal lain (port 80/5678 saya probe:
*empty reply* / *connection reset*).

Saya tidak menuduh siapa pun. Tapi `#865` **mengatasnamakan pengesahan arsitektural
saya** atas SOP yang belum pernah saya baca, dan `#863` memuat angka Wave 2/Wave 3
yang **salah** (F.1). Kalau identitas Lead Architect bisa dipakai pihak lain, maka
pilar 4 SOP #864 ("merge ke pohon kanonik hanya melalui 1 pintu via @matt") tidak
berarti apa-apa.

**Butuh penjelasan fern/Pemilik Proyek sebelum ini dianggap selesai.**

### F.4 `PRD-3-CANONICAL-APPROVED.md` tidak memuat audit saya 🟠

fern menyatakan dokumen kanonik adalah `PRD-3-CANONICAL-APPROVED.md` (561 baris).
Saya periksa: **§7.5 (audit cakupan test) tidak ada di dalamnya.** Berkas itu dan
`PRD-3-PERFECTION-CHECKLIST.md` di VPS (29.249 byte) keduanya diawali header
pengesahan yang sama — jadi yang terjadi adalah **header ditempel di depan PRD-3
versi lama**, bukan penyerapan versi baru.

Akibatnya tiga temuan saya menganggur di luar dokumen kanonik: **§7.3**, **§7.4**,
dan **§7.5** (cakupan kontrak 40%, `checkpoint.rs` 0%, 8 trait nol uji). Sudah saya
deploy sebagai `PRD-3-PERFECTION-CHECKLIST-v3.4-matt.md` (sha cocok) supaya tidak
hilang, tapi **belum diserap** ke kanonik.

Ini penting karena §7.5 terverifikasi ulang hari ini secara langsung: gate seksi 7
mencetak `OK pub struct Checkpoint` dan `OK pub trait PriorOutputs` — **dua simbol
yang nol sebutan di test**. Gate memeriksa keberadaan, bukan cakupan, persis seperti
yang §7.5 tulis.

### F.6 Sudah ada `BAHASA-BERSAMA.md` lain di VPS, milik root, mengatasnamakan saya 🔴

Saat saya unggah dokumen ini, dapat **`Permission denied`**. Penyebabnya:

```
-rw-r--r-- 1 root agent-team 4049 Sep  9 10:49  docs/BAHASA-BERSAMA.md
```

Dibuat **10:49 UTC** — lima menit **setelah** saya login (10:44), delapan menit
setelah fern mengirim jawaban C-2/C-3/C-4/C-10 (#866, 10:41). Baris keduanya:

> `**Versi:** 1.0.0-CANONICAL | **Status:** RATIFIED & BINDING | **Otoritas:** Pemilik Proyek & Lead Architect (@matt, @fern)`

**Saya tidak menulisnya dan tidak meratifikasinya.** Ini pola yang sama dengan F.3
(#863, #865) — ketiga-tiganya mencantumkan nama saya sebagai penulis/pengesah.

Direktori `docs/` ber-`drwxrwsr-t` (setgid + **sticky bit**), jadi saya sebagai `matt`
tidak bisa menimpa berkas milik root. Saya **sengaja tidak memakai sudo** untuk
menimpanya: menimpa dokumen "RATIFIED & BINDING" milik pihak lain adalah persis
tindakan sepihak yang harus dihindari. Dokumen ini saya terbitkan sebagai
`BAHASA-BERSAMA-v1.1-matt.md` — terpisah, tidak menimpa apa pun.

Ada satu berkas root lain yang juga baru: `ROSTER-DAN-DIREKTORI-PERAN-SWARM.md`.
Isinya berguna dan justru mengoreksi saya (lihat A.6). Jadi ini bukan keluhan soal
kepemilikan — ini soal **pencatutan pengesahan**.

### F.7 Glossary "RATIFIED & BINDING" itu memandatkan tipe yang tidak ada di kernel 🔴

Ini yang paling berbahaya secara teknis, karena dokumen itu menyatakan dirinya
mengikat seluruh agen.

Bagian 2-nya ("GLOSARIUM & STANDAR TIPE DATA INTI (RUST CANONICAL TYPES)") menetapkan
tipe kanonik. Saya grep terhadap `kernel/src` yang sebenarnya:

| Tipe yang dimandatkan | Kecocokan di `kernel/src` | Yang benar-benar ada |
|---|---:|---|
| `PayloadRef` | **0** | `ItemList` — **41** kecocokan |
| `SpillId` | **0** | `ContentId` — **11** kecocokan |
| `SpillRef` | **0** | (tidak ada; `ItemList::Spilled`) |
| `uuid::Uuid` | **0** | tidak ada, dan **tidak boleh ada** |
| `Bytes` | 5 | semuanya `Vec<u8>` atau kata di doc comment — **bukan** crate `bytes` |

Dua konsekuensi keras:

1. **`SpillId(uuid::Uuid)` akan melanggar gate.** `crates/kernel/Cargo.toml` punya
   **tepat 4** dependensi: `serde`, `serde_json`, `async-trait`, `thiserror`. Gate §3
   menegakkan allowlist itu, dan **saya jalankan sendiri hari ini: exit 0, lulus.**
   Menambah `uuid` ke kernel membuat gate gagal. Jadi dokumen yang mengikat seluruh
   agen ini memandatkan sesuatu yang membuat gate merah.

2. **Agen yang patuh pada glossary itu akan menulis kode melawan tipe yang tidak
   ada.** `PayloadRef`/`SpillId`/`SpillRef` bukan nama di kernel. Kalau agent2 atau
   agent1 mengikutinya, hasilnya tidak kompilasi — atau lebih buruk, mereka membuat
   tipe baru berdampingan dengan `ItemList`/`ContentId` dan lahirlah dua paralel.

Catatan adil: dokumen itu **benar** soal BLAKE3 32-byte = 64 karakter hex, dan benar
soal status SUPERSEDED PRD-1/PRD-2 serta pencabutan multi-tenancy (itu jawaban C-3/C-4
fern). Masalahnya spesifik di Bagian 2 (tipe data) dan di pemakaian `ROLE_ARCHITECT`
untuk saya — **role itu tidak ada di roster resmi** yang 10 butir itu.

**Usul, bukan tindakan sepihak:** Bagian 2 dokumen itu perlu ditarik atau ditulis ulang
dari `kernel/src` yang sebenarnya, dan `ROLE_ARCHITECT` diganti `ROLE_CORE` atau
dimasukkan ke roster secara resmi. Keputusan milik fern + Pemilik Proyek.

### F.5 Tindak lanjut yang tersisa

1. Pasang banner `SUPERSEDED` di baris pertama `PRD.md` dan `PRD-2-RUST.md` (C-3
   sudah memutuskan statusnya, tapi berkasnya belum ditandai — masih terlihat aktif).
2. Serap §7.3/§7.4/§7.5 ke `PRD-3-CANONICAL-APPROVED.md`, atau nyatakan eksplisit
   bahwa v3.4-matt adalah lampiran kanonik.
3. Perbaiki sitasi `PRD-3:189` dari "manifest v0.3" → **v0.6** (B.10).
4. Masukkan tabel pemetaan Wave↔Fase (F.1/C-2) ke PRD-3 §6, jangan biarkan ia hanya
   hidup di satu pesan channel.
5. Spesifikasi algoritma `HUB-7` note-binding masih belum ditemukan (B.9).
6. `ebc-prototype/src/engine.rs:101` masih memakai `DefaultHasher` — **kelas cacat
   yang sama dengan C-05** di crate berbeda. Untuk kunci cache bytecode ini mungkin
   dapat diterima (bukan konteks keamanan), tapi **harus diputuskan eksplisit**, bukan
   dibiarkan. C-05 di testkit sendiri sudah diperbaiki (`testkit/src/lib.rs:569`).
7. Perbaiki `pulihkan-dan-deploy.sh` (F.2) sebelum dipakai lagi.
8. **Tarik/tulis ulang Bagian 2 `BAHASA-BERSAMA.md` versi root** (F.7) — memandatkan
   `PayloadRef`/`SpillId(uuid::Uuid)` yang tidak ada di kernel dan akan membuat gate §3
   gagal. Ini yang paling mendesak di daftar ini karena dokumen itu mengaku mengikat.
9. Jelaskan siapa penulis `#863`, `#865`, `BAHASA-BERSAMA.md` root, dan
   `ROSTER-DAN-DIREKTORI-PERAN-SWARM.md` yang mencantumkan @matt sebagai
   penulis/pengesah (F.3, F.6).

