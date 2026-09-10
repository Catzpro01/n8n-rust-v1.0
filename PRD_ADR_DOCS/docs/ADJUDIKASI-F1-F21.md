# ADJUDIKASI AUDIT F1–F21 — Respons atas Review Adversarial agent1

**Tanggal:** 2026-09-09
**Diaudisi oleh:** agent1 (pesan #109–#111+ di `#n8n-upgraded-rust`)
**Diadjudikasi oleh:** matt (agent), diverifikasi ulang terhadap berkas sumber
**Objek:** `PRD-2-RUST.md` sha256 `4e280c281372c688d8b4b496c56114a415d32c9e308afb5f50cb15e80c0b3bec`

> **Metode:** setiap temuan diuji terhadap berkas nyata, bukan dinilai dari
> kewenangan pengaudisi. Perintah verifikasi disertakan. Ini sengaja dilakukan
> karena kegagalan asli proyek ini adalah 88 keputusan yang disetujui sendiri
> oleh pihak yang merekomendasikannya. Obatnya bukan membalik kewenangan
> (auditor selalu benar), tapi **memverifikasi kedua arah**.

---

## Ringkasan putusan

| Putusan | Jumlah | Temuan |
|---|---|---|
| **DITERIMA penuh** | 15 | F2, F3, F5, F7, F9, F10, F11, F12, F13, F14, F15, F17, F19, F20, F21 |
| **DITERIMA sebagian** (substansi benar, angka/framing salah) | 3 | F4, F6, F18 |
| **DITOLAK** (fakta tidak mendukung) | 3 | F1, F8, F16 |

Tingkat akurasi auditor: **18/21 substansial (86%)**, 3 keliru. Itu audit yang
bagus — dan tiga yang keliru itu justru penting untuk dikoreksi, karena dua
di antaranya (F8, F16) akan memicu pekerjaan yang tidak perlu kalau diterima.

---

## TEMUAN TERKUAT (harus dibaca dulu)

### F20 — DITERIMA. Skema credential mengulang kesalahan n8n yang sudah saya dokumentasikan sendiri

**Klaim agent1:** skema `credential` tidak punya `key_version`, padahal PRD-1 §11.4 sudah memperingatkan bahwa n8n sakit di rotasi key.

**Verifikasi:**
```
$ grep -n 'credential(' PRD-2-RUST.md
223:credential(id, name, type, data_encrypted, created_at, updated_at)

$ grep -n 'rotasi' PRD-1-N8N-ANALYSIS.md
713:- ... indikasi bahwa **rotasi/migrasi key adalah area yang sudah pernah menyakitkan**
      (commit 44d9d90 "encryption usage guardrails", 4b36c51 "Repair legacy-format
       data-encryption keys during bootstrap")
```

**Putusan: DITERIMA, dan ini temuan paling memalukan sekaligus paling berharga.**
Saya mendokumentasikan bahwa n8n terluka karena tidak punya versioning key —
lalu merancang skema dengan kelemahan persis sama. Tanpa `enc_key_id`, rotasi
key berarti **semua credential yang tersimpan tidak bisa didekripsi lagi**, dan
tidak ada cara mendekripsi bertahap. Perbaikan masuk §5.2.

---

### F10 — DITERIMA. Kernel dibekukan sebelum punya satu pun konsumen

**Klaim:** membekukan kernel saat belum ada satu pun crate yang memakainya adalah *premature freezing*.

**Verifikasi:** kernel punya 19 test, semuanya internal. `grep` untuk import
kernel di crate lain: tidak ada — crate lain belum ada (workspace bahkan tidak
build, lihat T-1).

**Putusan: DITERIMA.** API yang dibekukan tanpa konsumen hampir pasti salah di
titik yang tidak terduga, karena test internal hanya membuktikan kernel
konsisten dengan dirinya sendiri, bukan bahwa ia enak dipakai. Ini **temuan
strategis terpenting** di seluruh audit karena membatalkan asumsi proses saya.

Perbaikan: kernel pindah dari **BEKU** ke **PROVISIONAL-FREEZE** — boleh berubah
selama Fase 1, setiap perubahan butuh ADR singkat, dan dibekukan sungguhan
hanya setelah `executor` + minimal 3 node nyata memakainya. `check-freeze.sh`
tetap jalan (mencegah cycle & unsafe), tapi tidak lagi melarang perubahan API.

---

### F19 — DITERIMA. Crash antara tulis-spill dan catat-DB menyebabkan leak permanen

**Klaim:** kalau proses mati setelah file spill ditulis tapi sebelum baris DB tercatat, file itu yatim dan mark-sweep tidak akan pernah menemukannya.

**Verifikasi:** `§5.2` punya `spill_gc(spill_path, execution_id, ref_count, created_at)`.
Mark-sweep yang berjalan dari baris DB **tidak bisa** menemukan file yang tidak
punya baris. Jadi `spill_gc` justru tidak menutup kasus ini.

**Putusan: DITERIMA.** Urutan harus dibalik: **catat intent di DB dulu, baru
tulis file**, plus *orphan sweeper* yang memindai direktori spill dan
membandingkannya dengan DB. Gate Fase 1 "nol leak setelah 1.000 eksekusi" tidak
cukup — harus ada uji **kill -9 di tengah tulis spill**.

---

## DITERIMA PENUH (ringkas)

| ID | Temuan | Perbaikan |
|---|---|---|
| **F2** | 50,4 MB diukur di **kernel**, disajikan sebagai bukti klaim **produk**; 150 MB adalah proyeksi | §1 diberi kolom tingkat-bukti: "TERBUKTI (tingkat kernel, komponen terisolasi)" — bukan "TERBUKTI" tanpa kualifikasi |
| **F3** | Tabel bukti tanpa tanggal/commit → tidak bisa difalsifikasi | §1 ditambah kolom reproduksi (berkas + sha256 + tanggal) |
| **F5** | Prinsip "kompatibel di semantik bukan di bug" tanpa definisi operasional | Ditambah kriteria: deviasi = perilaku yang **terdokumentasi** berbeda; bug = perilaku yang **bertentangan dengan dokumentasi n8n sendiri** atau menghasilkan data rusak. Pemilik katalog deviasi ditunjuk eksplisit |
| **F7** | tokio multi-thread tanpa satu pun angka; thread × chunk × item bisa menjebol budget governor | §6 ditambah konfigurasi runtime eksplisit + uji governor pada thread maksimum |
| **F9** | crate `api-types` (direkomendasikan PRD-1 §2.3) hilang dari §3.2 → risiko contract drift FE/BE | Ditambah ke §3.2 dan Fase 2 |
| **F11** | edition 2021 tanpa MSRV; edition 2024 sudah stable | §4: edition 2024, MSRV dinyatakan eksplisit (toolchain terpasang 1.98.1) |
| **F12** | `sqlx` compile-time check butuh `DATABASE_URL` saat build; strategi CI tidak disebut | §4.1: putuskan `.sqlx` offline cache yang di-commit |
| **F13** | `rquickjs` (kode C) × target musl-static di Fase 4 = risiko build laten | **Spike build musl + rquickjs dipindah ke Fase 1.** Ditemukan di Fase 4 berarti 12 bulan terlambat |
| **F14** | cron + DST (02:30 ambigu) tanpa perilaku rujukan n8n | Masuk kandidat katalog deviasi + test DST wajib |
| **F15** | Klaim "postcard ~3x lebih kecil, ~5x lebih cepat" tanpa sumber — di dokumen yang prinsip #6-nya menuntut setiap klaim punya cara ukur | Ditandai **⚠️ BELUM TERVERIFIKASI**. Ini inkonsistensi diri sendiri, persis kelas kesalahan yang saya kritik di PRD-1 §18 |
| **F17** | checksum untuk `node_output`; BLAKE3 lebih cepat dari SHA-256 dan tetap kriptografis | Diterima sebagai pertimbangan; keputusan algoritma ditunda sampai ada pengukuran |
| **F21** | Retensi menghapus spill saat execution terminal, tapi differential testing butuh output setelah eksekusi → **kontradiksi langsung** | §5.3 ditambah mode `retain-outputs` untuk korpus uji |

F21 layak disebut: tanpa mode itu, **harness differential testing di Fase 1
secara desain tidak bisa bekerja** — outputnya sudah terhapus sebelum
dibandingkan. Dua bagian dokumen saya saling membatalkan.

---

## DITERIMA SEBAGIAN — substansi benar, angka atau framing salah

### F4 — index spill O(N). Substansi DITERIMA, framing DITOLAK

**Klaim agent1:** prinsip "no-O(N)" dilanggar desain sendiri; index 8 B/item → 38 MB pada 5 jt item, "proyeksi 800 MB pada 100 jt".

**Verifikasi independen (saya hitung ulang dari BENCH-A03):**
```
N=  200.000  index= 1,5 MB  peak=14,0 MB -> 7,86 B/item  index=11% dari peak
N=1.000.000  index= 7,6 MB  peak=20,0 MB -> 7,97 B/item  index=38% dari peak
N=5.000.000  index=38,1 MB  peak=50,4 MB -> 7,99 B/item  index=76% dari peak

proyeksi linear 8 B/item:
N= 10.000.000 ->  76,3 MB
N= 50.000.000 -> 381,5 MB
N=100.000.000 -> 762,9 MB
```

**Yang benar:** index memang tumbuh linear, dan pada 5 juta item ia **76% dari
seluruh peak RSS** — angka yang tidak saya sorot di PRD-2. Pada ~100 juta item
index saja memakan 763 MB, yang **menghancurkan tesis "jalan di VPS 2 GB"**
jauh sebelum payload jadi masalah. Ini nyata dan penting.

**Yang salah:** framing "prinsip dilanggar". Prinsip #2 berbunyi *"tidak ada
struktur data yang tumbuh O(N) **tanpa disadari**"*. Pertumbuhan ini disadari
dan dinyatakan eksplisit di §6.5 ("8 byte/item"). Jadi bukan pelanggaran
prinsip — tapi **memang batas yang belum saya beri pagar**.

**Perbaikan:** §6.5 ditambah (a) pernyataan bahwa index adalah komponen memori
dominan pada N besar, (b) **N-maks yang didukung eksplisit** per budget RAM,
(c) opsi index berjenjang di disk sebagai jalan keluar kalau N-maks perlu
dinaikkan. Contoh: budget 1 GB → index maksimum ~125 juta item; di atas itu
wajib index berjenjang.

### F6 — lease. Observasi DITERIMA, solusi DITOLAK

**Klaim:** `lease_holder`/`lease_expires_at` hanya perlu untuk multi-worker, padahal topologi §3.1 single-process. Rekomendasi: hapus dari Fase 1 (YAGNI).

**Verifikasi:** kernel **sudah** punya `lease: Option<Lease>` (`task.rs:44`)
dengan komentar bahwa lease kedaluwarsa memungkinkan crash recovery. Kernel
sedang (provisional-)freeze.

**Putusan:** Benar bahwa di single-process `lease_holder` redundan — saat
restart, **semua** task `Running` pasti milik proses yang mati, jadi tidak perlu
identifikasi pemegang. Tapi **menghapusnya dari kernel salah**: field itu
`Option`, jadi biayanya nol saat tidak dipakai, dan menghapusnya berarti
memecah kontrak yang sudah teruji demi penghematan yang tidak terukur.

**Perbaikan:** Fase 1 **tidak mengisi dan tidak bergantung** pada lease;
recovery memakai aturan "restart ⇒ semua `Running` = curiga". Field tetap ada
untuk Fase 5. Ini juga menjawab F10: jangan ubah API kernel berdasarkan
spekulasi.

### F18 — event_log. Kekhawatiran DITERIMA, magnitudo SALAH ~6 orde

**Klaim:** "event_log 1 tabel utk **puluhan juta baris/execution**, replay & VACUUM tersedak."

**Verifikasi:** event per execution = transisi task + node-run. Untuk workflow
26 node itu orde **puluhan sampai ratusan baris per execution**, bukan puluhan
juta. Klaim agent1 salah satuan sekitar 5–6 orde magnitudo.

**Tapi kekhawatiran dasarnya sah:** satu tabel append-only yang tidak pernah
dirotasi tetap tumbuh tanpa batas seiring jumlah execution, dan `VACUUM` di
tabel besar memang mahal.

**Perbaikan:** §5.2 ditambah catatan rotasi/partisi per rentang waktu dan uji
pada 10 juta baris **kumulatif** (bukan per execution). Angka agent1 tidak
dipakai.

---

## DITOLAK — fakta tidak mendukung

### F1 — DITOLAK. §1 sudah mengkualifikasi dan sudah merujuk §10

**Klaim:** "S1 klaim kompatibel tanpa kualifikasi; L1/L2/L3 baru di S10. REK: tesis merujuk S10."

**Verifikasi:**
```
$ sed -n '/^| Klaim | Status | Bukti |/,/^Klaim pertama/p' PRD-2-RUST.md
| Klaim | Status | Bukti |
| Lebih hemat memori secara struktural | ✅ TERBUKTI | BENCH-A03: ... |
| Kompatibel dengan workflow n8n       | ❌ BELUM    | Butuh korpus uji + differential testing (§10) |
| Setara jumlah integrasi              | ❌ TIDAK... | Strategi berbeda: subset + codegen (§8.4) |
```

Baris kompatibilitas **sudah** berstatus `❌ BELUM` dan **sudah** merujuk `(§10)`.
Rekomendasi agent1 persis keadaan yang sudah ada. **Ditolak.**

Catatan: yang valid dari F2 (bukan F1) adalah bahwa baris *pertama* — klaim
memori — yang kurang kualifikasi. agent1 menunjuk baris yang benar untuk isu
yang salah.

### F8 — DITOLAK. Tidak ada cycle yang "hampir pasti"; `ParameterSchema` sudah ada di kernel

**Klaim:** "Validasi parameter butuh schema milik `nodes-core`, arah `workflow`→`nodes` tak didefinisikan; **cycle hampir pasti** lalu check-freeze menolak di tengah Fase 1."

**Verifikasi:**
```
$ grep -rn 'pub struct ParameterSchema' rust-n8n-core/crates/kernel/src/
params.rs:22:pub struct ParameterSchema {
$ grep -n 'fn validate' rust-n8n-core/crates/kernel/src/params.rs
44:    pub fn validate(&self, params: &Value) -> Vec<ValidationError>
$ grep -n 'params' rust-n8n-core/crates/kernel/src/node.rs
47:    pub params: ParameterSchema,
```

Tipe skema **dan** metode validasinya sudah di kernel, bukan di `nodes-core`.
Crate `workflow` memvalidasi terhadap `ParameterSchema` yang dikirim lewat
registry — ia tidak perlu bergantung pada tipe `nodes-core`. **Tidak ada cycle
yang dipaksa.** Klaim ditolak.

**Sisa yang sah (diterima sebagai gap dokumentasi, bukan sebagai temuan cycle):**
§3.3 memang tidak menyatakan **di mana registry node hidup** dan bagaimana
`workflow` memperoleh descriptor saat runtime. Itu perlu ditulis — tapi itu
satu kalimat klarifikasi, bukan redesain dependensi. Kalau F8 diterima apa
adanya, tim akan merombak graf crate untuk masalah yang tidak ada.

### F16 — DITOLAK. D56 ada, di empat berkas

**Klaim:** "`task.priority` rujuk (D56) yg **TAK ADA** di dokumen."

**Verifikasi:**
```
$ grep -rn 'D56' *.md
AUDIT-D1-D100.md:323: A-08 — ... (D29, D37, D56, D59, D71, D72)
AUDIT-D1-D100.md:331: | D56 | Adaptive priority | Urutan |
KERNEL-SPEC.md:463:   pub priority: i8,   // D56
rangkuman-rust-workflow-os.md:495: | D56 | Queue priority | C — Adaptive priority |
```

D56 terdefinisi di tiga dokumen dan **sudah diimplementasikan** di
`KERNEL-SPEC.md:463`. Klaim "tak ada di dokumen" salah.

**Sisa yang sah:** PRD-2 mengutip "D56" tanpa memberi tahu pembaca di mana
definisi itu berada. Itu masalah keterbacaan satu baris, bukan keputusan yang
hilang. Diperbaiki dengan rujukan silang, bukan dengan "definisikan atau hapus"
seperti rekomendasi agent1 — menghapus akan membuang keputusan yang sudah ada
dan sudah terimplementasi.

---

## Pola yang terlihat dari audit ini

Tiga temuan yang ditolak punya bentuk yang sama: **"X tidak ada di dokumen"**,
padahal X ada — hanya di berkas lain dalam set yang sama. Auditor membaca
`PRD-2-RUST.md` secara terisolasi. Itu masuk akal, dan perbaikan yang benar
adalah **rujuk silang eksplisit** di PRD-2, bukan menganggap keputusannya belum
diambil.

Pelajaran untuk proses: dokumen yang akan diaudit pihak lain harus
**self-contained dalam rujukan**. Kalau sebuah keputusan hidup di berkas lain,
PRD harus menyebut berkas dan barisnya.

Dua temuan (F18, F4-sebagian) punya pola kedua: **substansi benar, magnitudo
salah**. F18 meleset ~6 orde. Ini alasan setiap angka dalam audit harus
diverifikasi ulang, bukan diteruskan — persis alasan §18 ada di PRD-1.

---

## Perubahan yang diterapkan ke PRD-2

| § | Perubahan | Temuan |
|---|---|---|
| 1 | Kolom tingkat-bukti + reproduksi (tanggal, sha256) | F2, F3 |
| 2 | Kriteria operasional deviasi-vs-bug + pemilik katalog | F5 |
| 3.2 | crate `api-types` ditambahkan | F9 |
| 3.3 | Lokasi registry node + cara `workflow` memperoleh descriptor | F8 (sisa sah) |
| 4 | edition 2024 + MSRV eksplisit | F11 |
| 4 | postcard ditandai ⚠️ BELUM TERVERIFIKASI | F15 |
| 4.1 | strategi `.sqlx` offline cache untuk CI | F12 |
| 5.2 | `credential` + `enc_key_id`, `enc_algo`; write-ahead intent spill; catatan rotasi event_log | F20, F19, F18 |
| 5.3 | mode `retain-outputs` | F21 |
| 6 | konfigurasi runtime tokio eksplisit + uji governor di thread maks | F7 |
| 6.2 | lease tidak diisi di Fase 1; aturan restart | F6 |
| 6.5 | index sebagai komponen dominan + N-maks per budget RAM | F4 |
| 12 | kernel: BEKU → PROVISIONAL-FREEZE + kriteria unfreeze | F10 |
| 12 F1 | spike build musl+rquickjs; uji kill-9 tengah-spill; test DST | F13, F19, F14 |
| 14 | rujukan silang D56 | F16 |

**Tidak ada kode yang ditulis.** Semua perubahan bersifat dokumentasi, sesuai
instruksi pemilik proyek.
