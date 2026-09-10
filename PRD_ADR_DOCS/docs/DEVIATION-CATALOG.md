# DEVIATION-CATALOG.md
## Katalog Deviasi yang Disengaja — Engine Rust vs n8n

**Pemilik dokumen:** matt (Lead Architect) — per PRD-2 §10.3
**Kerangka + aturan pengisian disusun oleh:** agent5 (QA & Security Auditor), perintah fern #135
**Versi:** v0.1 (kerangka)
**Tanggal:** 2026-09-09
**Status korpus:** `DEV-000-BLOCKED` — belum satu pun deviasi dapat dicatat secara sah (§4)

---

## 1. Tujuan dokumen ini

PRD-2 §10.3 berbunyi: *"Deviasi yang disengaja dicatat di dokumen publik
DEVIATION-CATALOG.md."*

Ini bukan formalitas. Target L2 = 95% berarti **5% korpus akan gagal**, dan L3 = 90%
berarti 10% ekspresi akan berbeda. Angka itu hanya bisa dipercaya kalau kegagalan
yang tersisa **tercatat dan beralasan**. Tanpa katalog ini, "95% kompatibel" tidak
dapat dibedakan dari "95% dari hal-hal yang kebetulan mudah".

Katalog ini adalah satu-satunya tempat di mana tim boleh berkata:
*"di sini kami sengaja berbeda dari n8n, ini alasannya, ini dampaknya."*

---

## 2. Aturan pengisian (mengikat)

1. **Hanya deviasi deterministik yang boleh masuk.** Jalankan kasus pada n8n asli
   **3×**. Kalau hasilnya berbeda antar-run → klasifikasikan `FLAKY` di §5, **bukan**
   deviasi. Deviasi non-deterministik yang masuk ke dokumen publik akan merusak
   kredibilitas seluruh klaim kompatibilitas.
2. **Wajib ada cara reproduksi.** Setiap entri harus menyebut `case_id` korpus,
   **versi korpus** (sha256 manifest), versi n8n pembanding, dan hash build engine.
   Entri tanpa cara reproduksi **ditolak oleh QA**.
3. **Wajib ada bukti mentah.** Simpan output kedua sisi apa adanya (A = n8n asli,
   B = engine kita) di `qa/evidence/<DEV-ID>/`. Ringkasan lulus/gagal saja tidak
   cukup — bukti harus bisa diaudit ulang bertahun-tahun kemudian.
4. **Satu deviasi = satu perilaku.** Jangan menggabungkan "format tanggal berbeda"
   dan "urutan array berbeda" dalam satu entri.
5. **Alasan wajib salah satu dari daftar tertutup** (§3 kolom `alasan`). Alasan di
   luar daftar itu berarti deviasi belum disetujui arsitek.
6. **Normalisasi bukan deviasi.** Perbedaan yang hilang setelah normalisasi sah
   (`executionId`, timestamp, dsb.) dicatat di `NORMALIZATION-MANIFEST.md`, **bukan**
   di sini. Kalau sebuah "deviasi" bisa diselesaikan dengan menambah entri
   normalisasi, itu bukan deviasi — dan menambahkannya demi meluluskan gate adalah
   kecurangan yang harus ditolak reviewer.
7. **Perubahan status wajib bertanggal.** `DITERIMA` → `DITOLAK` harus terlihat
   jejaknya, bukan ditimpa diam-diam.

---

## 3. Skema entri

```yaml
- id: DEV-001                      # DEV-nnn, tidak pernah dipakai ulang
  status: DIAJUKAN | DITERIMA | DITOLAK | FLAKY | TERTUNDA
  kategori: SEMANTIK | FORMAT | ERROR | LIMIT | KINERJA | TIDAK-DIIMPLEMENTASI
  subkategori_limit: RESOURCE_LIMIT | DATA_LIMIT   # wajib bila kategori = LIMIT
  node_types: [node.n8n, node.n8n] # atau "expression" untuk L3
  tier: L1 | L2 | L3
  ringkasan: satu kalimat, perilaku apa yang berbeda
  perilaku_n8n: apa yang n8n asli lakukan (dengan contoh input → output)
  perilaku_engine: apa yang engine kita lakukan (input sama → output)
  reproduksi:
    case_id: <id di korpus>
    corpus_version: <sha256 manifest>
    n8n_version: <versi pin>
    engine_commit: <git sha>          # WAJIB; kalau belum ada git, sha256 binary
    engine_build: <timestamp build + profil dev/release>
    perintah: <satu baris yang bisa dijalankan siapa pun>
  alasan: LIHAT-DAFTAR-TERTUTUP
  dampak: siapa yang terkena dan seberapa parah
  risiko_migrasi: TINGGI | SEDANG | RENDAH
  bukti: qa/evidence/DEV-001/{a.json,b.json,run.log}
  diajukan_oleh: agentN
  diputuskan_oleh: matt
  tanggal: YYYY-MM-DD
```

### Daftar tertutup untuk kolom `alasan`

| Kode | Arti | Contoh yang sah |
|---|---|---|
| `N8N-BUG` | perilaku n8n adalah bug; kita sengaja tidak menirunya | wajib menyertai link issue/changelog n8n |
| `KEAMANAN` | meniru perilaku n8n akan melanggar batas keamanan | sandbox, isolasi, batas resource |
| `TIDAK-MUNGKIN` | tidak dapat direproduksi di Rust tanpa unsafe/UB | perilaku yang bergantung detail runtime V8 |
| `CAKUPAN` | di luar Tier yang diklaim didukung | node Tier 3 |
| `KINERJA` | kesetaraan perilaku menuntut biaya yang tidak wajar | harus menyertai angka perbandingan |
| `BELUM-SEMPAT` | deviasi sementara, wajib punya tiket penutup | **wajib** `tanggal_target`; kalau lewat → status jadi `TERTUNDA` dan dilaporkan |

`BELUM-SEMPAT` yang tidak pernah ditutup adalah cara paling umum katalog deviasi
berubah menjadi daftar utang yang diabaikan. Karena itu wajib bertanggal target.

---

## 4. Status saat ini: belum ada deviasi yang bisa dicatat

**[FAKTA]** Berikut kondisi yang saya verifikasi sendiri per 2026-09-09 05:00 UTC
(korpus = 198 file: 171 di root, 15 di `exec-diff-mvp/`, 12 di `_fixtures-repo-internal/`):

| Prasyarat pencatatan deviasi | Status |
|---|---|
| Sisi pembanding (n8n asli) bisa dijalankan | **TIDAK** — `docker` tidak ada di server; runtime alternatif belum diputuskan (Keputusan 1) |
| Versi n8n di-pin | **TIDAK** (Keputusan 2) |
| Toolchain Rust berfungsi | **TIDAK** — `cargo`/`rustc` gagal di semua akun (ERR-024); QA bahkan tidak punya toolchain sama sekali |
| Korpus kategori EKSPOR tersedia | **YA — 36 file** *(pada 04:20 masih 0)* |
| Korpus layak differential testing | **YA — subset `exec-diff-mvp/` 13 file**, sudah saya verifikasi byte-per-byte: 13/13 SHA-256 cocok manifest, 13/13 SrcSHA cocok sumber, `nodes`+`connections` identik, satu-satunya perubahan adalah field `name` yang ditambahkan dan **dinyatakan** di README-nya |
| Angka korpus yang dipakai sebagai dasar keputusan berasal dari proses yang byte-identik saat diulang | **SEBAGIAN** — angka keputusan `analisis_node.py` (63% / 153 / 98 / 106, SET recommended_26, full_coverage=23) **stabil 20-seed** (verifikasi agent5 #408, PYTHONHASHSEED 0-19). Sisa nondeterminisme = urutan tampilan entri ber-count sama di TOP-35 + urutan key FREQ.json (murni kosmetik, root cause: iterasi SET `for t in ns`). **CATATAN RISIKO LATEN:** ada tie tepat di batas cutoff (rank26/27 formTrigger=gmail=9; rank35/36 writeBinaryFile=aggregate=7) yang kebetulan tidak flip di 20 seed - jangan bekukan recommended_26/TOP-35 sebagai baseline PRD-3 sebelum tie-break `(-count, nama)` diterapkan. ERR-028 = LOW/OPEN-PENDING-FIX; ERR-029 = DITUTUP. |
| Korpus bebas cacat integritas | **YA** — 0 dari 198 file tersuntik field non-n8n *(pada 04:20 ada 40)*; 183 entri sha256 di manifest, 0 file tak tertelusur *(pada 04:20 ada 26)* |
| Harness differential + mock/replay ada | **TIDAK** |
| `NORMALIZATION-MANIFEST.md` ada | **TIDAK** — agent4 baru mengusulkan strukturnya di #179 |

**Perubahan penting sejak draf pertama:** sisi **korpus sudah siap** (13 file siap
pakai, terverifikasi). Yang masih menghalangi tinggal sisi **eksekusi**: tidak ada
runtime pembanding dan tidak ada toolchain untuk membangun engine.

**Kesimpulan:** setiap "deviasi" yang dicatat sekarang tidak akan punya bukti mentah
dari kedua sisi, karena tidak ada sisi yang bisa dijalankan. Karena itu katalog ini
sengaja **kosong**. Mengisinya dengan tebakan lebih merugikan daripada membiarkannya
kosong.

**Catatan koreksi atas pekerjaan saya sendiri:** pada ERR-022 saya melaporkan
`[REDACTED_INSTANCE_ID]` di `tpl-4722` sebagai bukti penyuntingan oleh tim. Itu
**salah** — saya tarik API n8n.io sendiri dan menemukan bahwa API-nya yang
mengembalikan literal tersebut (bersama `[REDACTED_TAG_ID]`, `[REDACTED_WEBHOOK_ID]`,
`[REDACTED_EMAIL]`). Temuan itu saya cabut, dan alat verifikasi saya perbaiki (v4)
karena ia punya mekanisme *false positive* yang bisa dipakai menuduh orang yang
tidak bersalah. Rincian di `AGENT5_QA_SECURITY_SPEC.md` §4.3 butir 4.

---

## 5. Entri

### 5.1 Deviasi tercatat

*(kosong — lihat §4)*

| ID | Status | Kategori | Tier | Ringkasan | Alasan |
|---|---|---|---|---|---|
| — | — | — | — | — | — |

### 5.2 Kasus FLAKY (bukan deviasi; dikecualikan dari denominator atau diperbaiki)

*(kosong)*

| ID | Kasus | Gejala | Tindakan |
|---|---|---|---|
| — | — | — | — |

### 5.3 Entri administratif

| ID | Status | Ringkasan |
|---|---|---|
| **DEV-000** | **TERTUNDA** | Katalog tidak dapat diisi: 7 prasyarat di §4 belum terpenuhi. Menunggu Keputusan 1–3 dari fern/matt (DM #186/#187, publik #189) dan perbaikan ERR-024. Diajukan agent5, 2026-09-09. |

---

## 6. Kandidat yang **perlu diselidiki** (belum diverifikasi — jangan dianggap deviasi)

Daftar ini sengaja dipisah dari §5 supaya tidak ada yang membacanya sebagai temuan.
Semuanya **[USUL/PERLU-UJI]**, berasal dari bacaan PRD-2 dan pengetahuan umum
tentang perbedaan runtime, dan **belum satu pun saya jalankan** karena blocker di §4.

| # | Perilaku yang perlu diuji | Kenapa bisa berbeda | Cara menguji |
|---|---|---|---|
| K1 | Konversi tipe pada `==` longgar di expression | V8 punya aturan coercion sendiri (`"" == 0`, `[] == false`); implementasi Rust harus meniru tabel itu persis | bandingkan hasil 100+ pasangan nilai |
| K2 | Presisi & format angka float | V8 memakai algoritma shortest-roundtrip (Grisu/Ryu); `serde_json` default bisa menghasilkan `1.0` vs `1` | bandingkan serialisasi angka batas |
| K3 | Urutan kunci object pada output JSON | V8 mengurutkan kunci integer-like lebih dulu; Rust `serde_json::Map` default mengikuti urutan insersi atau BTreeMap (alfabetis) | bandingkan byte output |
| K4 | Perilaku tanggal & zona waktu | `DateTime` n8n bergantung `luxon` + `process.env.GENERIC_TIMEZONE` | jalankan dengan TZ berbeda, bandingkan |
| K5 | Regex flavor | RegExp JS ≠ crate `regex` Rust (lookahead/lookbehind, flag `u`) | tabel kasus regex |
| K6 | Potongan string & indeks UTF-16 | JS `substring` berbasis **code unit UTF-16**, Rust `str` berbasis byte UTF-8 → emoji akan berbeda hasilnya | uji dengan string berisi emoji & CJK |
| K7 | Batas rekursi & ukuran struktur | V8 punya limit stack sendiri; QuickJS beda lagi | uji payload dalam |
| K8 | Pesan & kode error | teks error n8n sering dipakai workflow downstream (`$json.error.message`) | bandingkan pesan error node |
| K9 | Node `Merge`, `Split In Batches`, `Sort` | urutan output bergantung implementasi; rawan beda | kasus per mode |
| K10 | Perilaku `null` vs field hilang | JS membedakan `undefined`/`null`; Rust `Option` + `serde` bisa menyamakannya saat serialisasi | uji object dengan field null |

Kalau ada yang mengunggah hasil untuk K1–K10, **tetap wajib** melewati §2 (3× run,
bukti mentah, cara reproduksi) sebelum pindah ke §5.1.

---

## 6bis. Kandidat TERDAFTAR dari agent4 (#267, 2026-09-09) — dinilai menurut aturan §2

agent4 mengusulkan lima entri. Saya daftarkan di sini, bukan di §5.1, dan saya beri
penilaian per butir supaya jelas apa yang kurang. **Tidak satu pun saya tolak**; yang
ada adalah syarat yang belum terpenuhi.

| ID usulan | Isi | Penilaian QA | Yang kurang |
|---|---|---|---|
| **DEV-A1** | `stickyNote` diabaikan (dekoratif). agent4 menghitung 38/96 file; matt menghitung **79/171 (46%)** dan angka matt yang saya reproduksi | **BUKAN deviasi — ini aturan denominator.** Sudah masuk `AGENT5_QA_SECURITY_SPEC.md` §3.1 sebagai pengecualian wajib. matt juga menetapkan di PRD-2 v1.3 S7.3.1 poin 3 bahwa stickyNote **wajib bisa di-parse** dan `content`/`position`/`color`-nya disimpan — jadi ia dikeluarkan dari metrik eksekusi tapi tetap masuk L1. | tidak ada; sudah diserap |
| **DEV-A2** | `function`/`functionItem` dengan `require()` npm eksternal → impor **gagal eksplisit**, bukan eksekusi | **SIAP DIISI, menunggu satu hal.** Denominator sudah ada: agent1 menyapu 171 file dan menemukan **hanya 1** yang memuat `require()` — `tpl-1381-search-and-download-torrents-using-t*.json`, node `SearchTorrent`, `require('torrent-search-api')`, sha16 `0ba84f24722aeb98` (#310). Perilakunya juga sudah diputuskan matt di PRD-2 v1.3 S10.3: alias `function`→`code` **wajib gagal eksplisit** dengan pesan jelas, bukan diam-diam berjalan berbeda. Jadi kolom `alasan` sudah terisi dari daftar tertutup dan dampaknya terukur: **1/171 = 0,6% korpus**. | **pesan error persis** — belum ada parser untuk diuji. agent1 mengusulkan matt menetapkan *format pesan error impor gate-L1*; saya teruskan, karena PRD-2 §10.1 sudah menuntut "pesan eksplisit yang menyebut node mana". Tanpa format yang ditetapkan, differential testing atas perilaku error (kategori `ERROR`) tidak bisa dilakukan sama sekali. |
| **DEV-A3** | template API n8n.io tanpa field metadata ekspor (`versionId`/`meta`) → valid untuk L1 tapi tidak lolos uji `REAL_EXPORT` | **BUKAN deviasi — ini cacat pada kriteria validator agent2.** Saya sudah periksa: 48 file "gagal" hanya karena tidak punya `name`, semuanya FIXTURE tertelusur dan sah untuk L1. Lihat `AGENT5_QA_SECURITY_SPEC.md` §4.5. | tidak ada; yang harus diubah adalah validatornya, bukan katalognya |
| **DEV-A4** | `cron.triggerTimes` dengan detik acak dari `toCronExpression` → dibuang saat konversi (granularitas menit) | **Deviasi sungguhan, kategori `SEMANTIK`, tier L1/L2.** Ini mengubah perilaku: workflow n8n yang menjadwalkan pada detik tertentu akan jalan di waktu berbeda. | bukti mentah kedua sisi + uji determinisme 3× + alasan dari daftar tertutup (kemungkinan `N8N-BUG` atau `TIDAK-MUNGKIN`) + **dampak** (siapa yang terkena) |
| **DEV-A5** | tipe field `PARTIAL`/`DEFER` (`objectArray`, `resourceLocator` `initCode`, `filter`) → impor sukses + daftar field yang diabaikan dicatat | **Kandidat kuat, kategori `TIDAK-DIIMPLEMENTASI`, tier L1.** Syarat "daftar field diabaikan dicatat" justru yang membuat ini bisa diterima: perilakunya teramati dan terdokumentasi. | daftar field yang lengkap + contoh file korpus per tipe field + keputusan matt |

**Jawaban untuk pertanyaan agent4** ("mau saya format jadi draft seksi di file Anda, atau
cukup usulan di kanal ini?"): **cukup usulan di kanal**, dan sudah saya daftarkan di atas.
Alasannya praktis: §5.1 hanya untuk entri yang punya bukti mentah dan cara reproduksi, dan
menyerahkan penyusunan §5.1 ke banyak penulis sekaligus akan membuat skema YAML-nya
bercabang. Yang lebih berguna dari agent4: untuk **DEV-A2** dan **DEV-A5**, kirim daftar
file korpus + hash yang menunjukkan gejalanya. Untuk **DEV-A4**, kirim contoh pasangan
`triggerTimes` n8n → hasil konversi. Dengan itu saya bisa memindahkan keduanya ke §5.1
dalam satu langkah.

**Catatan untuk semua penulis:** DEV-A4 adalah contoh mengapa §2 butir 1 (uji 3×) ada.
"Detik acak" berarti perilakunya mungkin **tidak deterministik di n8n sendiri** — kalau
`toCronExpression` menghasilkan detik yang berbeda antar-run, ini masuk §5.2 `FLAKY`,
bukan §5.1. Itu harus diuji dulu, bukan diasumsikan.

---

## 7. Cara menambah entri

```bash
# 1. jalankan kasusnya 3x di kedua sisi, simpan bukti
/opt/agent-workspace/qa/run_case.py --case <id> --runs 3 \
    --out /opt/agent-workspace/qa/evidence/DEV-0NN/

# 2. isi blok YAML sesuai §3, tempel di §5.1
# 3. kabari QA untuk review bukti, lalu matt memutuskan status
msg send architecture "[DEV-0NN] diajukan: <ringkasan> — bukti di qa/evidence/DEV-0NN/"
```

> Skrip `run_case.py` **belum ada** — bagian dari harness yang terblokir (§4).
> Perintah di atas adalah bentuk yang saya usulkan supaya prosesnya seragam sejak awal.

---

*Katalog ini sengaja kosong pada v0.1. Katalog deviasi yang terisi tanpa bukti lebih
berbahaya daripada yang kosong, karena ia memberi kesan bahwa pengujian sudah
dilakukan. Isi begitu prasyarat §4 terpenuhi.*
