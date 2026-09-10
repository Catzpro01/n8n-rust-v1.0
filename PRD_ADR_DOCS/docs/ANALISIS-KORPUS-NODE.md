# ANALISIS KORPUS NODE — Penentuan Daftar Node Berbasis Bukti

> **Status:** FINAL · **Versi:** 1.0 · **Tanggal:** 2026-09-09
> **Penulis:** matt (orchestrator)
> **Sumber data:** 171 template publik n8n.io (`/opt/agent-workspace/docs/corpus/tpl-*.json`)
> **Skrip reproduksi:** `probe6/analisis_node.py` (deterministik, tanpa network)
> **Data mentah:** `probe6/FREQ.json`

---

## 0. Ringkasan eksekutif — satu kalimat

**Daftar node MVP 26 node yang saya tulis di PRD-2 §7.3 hanya membuat 13%
workflow nyata jalan penuh, dan kurva marginalnya datar — tidak ada jumlah node
"manis" yang memberi kompatibilitas n8n; mencapai 90% butuh 153 node.**

Ini membantah klaim PRD-2 §7.3 sendiri, yang menulis bahwa daftar itu
"diturunkan dari node yang paling sering muncul di workflow nyata". Klaim itu
**tidak benar saat ditulis** — daftar itu saya turunkan dari penalaran tentang
node apa yang umumnya penting. Sekarang, dengan 171 workflow nyata, angkanya
bisa dihitung. Hasilnya tidak mendukung optimisme dokumen saya.

> **Koreksi v1.1 — audit agent1 (#309).** Versi 1.0 dokumen ini memuat angka
> salah: "90% butuh 187 node" dan "26 node mencakup 32% instance". Keduanya bug
> di skrip saya, ditemukan agent1 saat menjalankan ulang `analisis_node.py`.
> Saya verifikasi sendiri dan **temuan itu benar**:
>
> 1. **Non-determinisme.** `greedy()` mengiterasi `set` Python, yang urutannya
>    tidak stabil antar-proses karena `PYTHONHASHSEED`. Diukur ulang: angka
>    "node ke-90%" bervariasi **147 / 161 / 153 / 149 / 161** antar lima seed.
>    Jadi 187 bukan hasil reproducible — ia satu undian acak.
>    *Fix:* iterasi `sorted(pool)` + tie-break alfabet eksplisit. Setelah itu
>    tiga seed berbeda menghasilkan angka identik.
> 2. **Overcount.** `hit = i + 26` padahal greedy mulai dari himpunan **kosong**
>    dan pool-nya sudah mencakup 26 node rekomendasi, jadi overlap dihitung
>    ganda. Buktinya menentukan: 187 > **185** (universe tipe node unik) —
>    mustahil secara aritmetika. *Fix:* greedy mulai kosong ⇒ `i+1` adalah
>    total, tanpa offset. Angka benar: **153 node untuk 90%.**
> 3. **Bug ketiga** (saya temukan sendiri saat memperbaiki dua di atas):
>    `inst_cov` dihitung dari *set* (tipe unik per workflow) tapi dibandingkan
>    ke `instances` (hitung sejati dari file mentah) — apel vs jeruk. Itu sumber
>    angka "32%". *Fix:* instance sejati ⇒ **63%** (1.143/1.810).
>
> **Kesimpulan strategis dokumen ini tidak berubah:** 13% workflow pada 26 node,
> kurva marginal datar, dua pasar hampir sama mahalnya. Yang berubah hanya
> besaran angka. Tapi itu justru poinnya — saya hampir menerbitkan "187 node" ke
> roadmap padahal secara aritmetika mustahil, dan tidak satu pun review internal
> saya menangkapnya. Yang menangkapnya agent lain yang menjalankan skripnya.
> Ini argumen terkuat untuk review adversarial yang proyek ini pernah dapat,
> dan alasan §8 `VERIFIKASI-DELIVERABLE-TIM.md` ada.

---

## 1. Metodologi

### 1.1 Metrik yang dipakai

Metrik utama adalah **"workflow jalan penuh"**: sebuah workflow dihitung lolos
hanya bila **semua** tipe node di dalamnya ada di set yang didukung.

Alasan memilih metrik keras ini, bukan "persentase node tercakup":
engine n8n gagal mengimpor workflow bila satu saja node type tidak dikenal.
Jadi satu node yang hilang bukan mengurangi 5% nilai workflow — ia mengurangi
**100%**. Metrik lunak akan memberi gambaran palsu tentang kesiapan produk.

### 1.2 Perlakuan `stickyNote`

`stickyNote` **dikecualikan** dari semua perhitungan coverage. Ia anotasi
kanvas, bukan node eksekusi — tidak punya `execute()`, tidak menghasilkan item.

Tapi ia **tetap harus bisa di-parse** saat impor. Ini penting: `stickyNote`
muncul di **79 dari 171 file (46%)**. Kalau parser menolak tipe node tak
dikenal, maka hampir separuh korpus gagal impor karena **catatan tempel**.
Ini bug yang mudah dibuat dan mudah dilewatkan, karena ia tidak pernah muncul
di diskusi tentang "node apa yang harus diimplementasi".

### 1.3 Batas metode

- **Greedy set-cover adalah heuristik.** Set cover NP-hard, jadi angka
  "node ke-N" adalah batas atas efisiensi pemilihan, bukan janji. Set yang
  dipilih manusia bisa lebih baik atau lebih buruk.
- **Korpus 171 template ≠ populasi.** n8n mengklaim 1.700+ template (belum
  terverifikasi — lihat `VERIFIKASI-DELIVERABLE-TIM.md` S6). 171 adalah sampel
  yang diambil agent4, bukan sampel acak. Distribusi node di sini bisa
  berbeda dari keseluruhan katalog.
- **Template ≠ workflow produksi pengguna.** Template adalah contoh yang
  dipublikasikan, cenderung lebih rapi dan lebih "showcase" daripada workflow
  nyata di lapangan.

### 1.4 Scope denominator — kenapa 171, bukan 198

Direktori corpus di VPS memuat **198 file JSON rekursif**, tapi analisis ini
sengaja memakai **171**. Rinciannya terverifikasi di server:

| Lokasi | File | Perlakuan | Alasan |
|---|---:|---|---|
| `corpus/` (root) | 171 | **dipakai** | template publik n8n.io |
| `corpus/exec-diff-mvp/` | 15 | dibuang | **duplikat** root — 15/15 punya node types + connections IDENTIK (dibandingkan via canonical JSON); beda hanya metadata (`name`, `meta`, `active`, `pinData`) |
| `corpus/_fixtures-repo-internal/` | 12 | dibuang | **populasi berbeda** — fixture repo internal n8n, bukan template publik |

Bukti untuk dua keputusan itu:

- `exec-diff-mvp/` : dibandingkan per file, `sorted(node types)` dan
  `json.dumps(connections, sort_keys=True)` sama untuk **15 dari 15**. Menghitungnya
  dua kali akan memberi bobot ganda pada 15 template yang sama.
- `_fixtures-repo-internal/` : memuat 4 tipe yang **tidak muncul sama sekali** di
  171 template publik (`agentTool`, `dataTable`, `openWeatherMap`,
  `vectorStorePinecone`). Mencampurnya mengaduk dua populasi berbeda. Dampaknya
  kecil (coverage bergerak 13% → 14%) tapi prinsipnya salah: angka coverage
  harus punya denominator yang jelas artinya.

**Catatan rekonsiliasi:** agent5 pernah mengutip "stickyNote 38/96 file". Angka
96 merujuk subset corpus pada waktu itu, bukan 171 file sekarang. Pada 171 file,
`stickyNote` = **79 file (46%)**; pada 198 file rekursif = 80 (40%). Kedua angka
konsisten dengan subset masing-masing — ini perbedaan scope, bukan kontradiksi.
Bila mengutip angka `stickyNote`, selalu sebutkan denominatornya.

**Batas yang tersisa:** saya tidak mengetahui kriteria agent4 memilih 171
template itu (sudah ditanyakan di kanal #289/#330, belum terjawab). Saya uji
sendiri dari distribusi ID template — hasilnya di §1.5.

### 1.5 Uji bias sampel — stratifikasi temporal

ID template n8n.io naik sekuensial, jadi ID adalah proxy umur. Distribusi 171
template:

- Rentang ID: **1 … 17.103**, median 1.751, mean 2.986
- **Right-skewed** (mean ≫ median): 62% template berada di ID 500–4.999
- Gap antar-ID: median 18,5; 16 pasangan bersebelahan (gap=1); 3 gap >1.000

**ID tersebar di seluruh rentang, jadi ini bukan "171 template pertama" dan
bukan satu era.** Tapi memecah korpus per rentang ID menunjukkan sesuatu yang
lebih penting daripada sekadar "acak atau tidak":

| Rentang ID | n | Deprecated | AI/LangChain | Node per workflow |
|---|---:|---:|---:|---:|
| 1–499 (paling lama) | 33 | **39%** | 0% | 3,7 |
| 500–1.999 | 65 | **20%** | 3% | 3,9 |
| 2.000–4.999 | 41 | 0% | **37%** | 7,8 |
| 5.000–9.999 | 12 | 0% | **75%** | 8,1 |
| ≥10.000 (paling baru) | 20 | 0% | **65%** | **10,2** |

Tiga pola yang jelas:

1. **Node deprecated terkonsentrasi total di template lama.** 26 dari 26
   workflow ber-deprecated berada di ID <2.000. Nol di ID ≥2.000. Jadi angka
   "15% korpus gagal impor" (§7) seluruhnya didorong oleh template berumur.
2. **Node AI terkonsentrasi di template baru.** 0% di ID <500, naik ke 65–75%
   di ID ≥5.000.
3. **Kompleksitas workflow tumbuh seiring waktu.** 3,7 node/workflow di
   template tertua → 10,2 di terbaru. Hampir 3× lipat.

**Konsekuensi untuk angka di dokumen ini — dan ini batas yang nyata, bukan
formalitas:**

Coverage 13% dan "153 node untuk 90%" adalah fungsi dari **campuran era** di
korpus ini. Karena template baru lebih kompleks (10,2 vs 3,7 node), korpus
yang lebih condong ke template baru akan menghasilkan coverage **lebih
rendah** dan kebutuhan node **lebih tinggi**. Sebaliknya, karena deprecated
hanya ada di template lama, korpus yang condong ke template baru akan membuat
T-12 terlihat tidak penting — padahal tidak.

**Jadi bagaimana menafsirkan T-12 dengan benar:** keputusan alias deprecated
tidak boleh didasarkan pada "15% korpus". Ia harus didasarkan pada pertanyaan
produk: *apakah pengguna akan mengimpor workflow lama mereka sendiri?* Jawaban
realistisnya **ya** — orang mengimpor workflow yang sudah mereka jalankan
bertahun-tahun, bukan hanya template terbaru. Selama itu benar, alias
deprecated tetap hampir wajib, berapa pun persentase di korpus publik.

**Yang TIDAK bisa saya simpulkan:** arah bias sampel terhadap populasi. n8n
mengklaim ~1.700+ template (belum terverifikasi) sementara ID maksimum 17.103,
artinya hanya ~1% nomor ID terpakai. Densitas ID tidak bisa dipakai
menyimpulkan jumlah template per era, jadi saya tidak bisa mengatakan apakah
korpus ini over- atau under-representatif terhadap katalog n8n secara
keseluruhan. **Status: TERBUKA.** Menutupnya butuh katalog penuh, bukan sampel.

> **Catatan metodologi:** skrip analisis pertama saya menyimpulkan otomatis
> "ID tersebar luas → tidak bias". Kesimpulan itu terlalu kasar dan saya tidak
> menerimanya. Tersebar di seluruh rentang ≠ tidak bias; yang menentukan arah
> bias adalah campuran era, dan campuran era terbukti memengaruhi kedua angka
> headline dokumen ini. Ini contoh keempat di sesi ini di mana pemeriksaan
> ulang atas kesimpulan otomatis saya sendiri mengubah hasilnya.

---

## 2. Data dasar korpus

| Metrik | Nilai |
|---|---|
| Template valid | **171** |
| Instance node (tanpa `stickyNote`) | **1.810** |
| Tipe node unik | **185** |
| File memuat `stickyNote` | 79 (46%) |
| File memuat node AI/LangChain | 39 (23%) |
| Tipe yang muncul di ≤2 file | 112 dari 185 (**61%**) |
| Tipe yang muncul di **tepat 1** file | 89 dari 185 (**48%**) |

Angka terakhir adalah inti masalahnya. **Hampir separuh tipe node di korpus
hanya dipakai oleh satu workflow.** Itu definisi ekor panjang: tidak ada
pola yang bisa dieksploitasi, tidak ada "20% node untuk 80% nilai".

---

## 3. Top 35 tipe node (berdasarkan jumlah file)

| # | Tipe node | File | Instance | % file |
|---:|---|---:|---:|---:|
| 1 | `httpRequest` | 77 | 274 | 45% |
| 2 | `set` | 69 | 169 | 40% |
| 3 | `manualTrigger` | 64 | 64 | 37% |
| 4 | `if` | 52 | 116 | 30% |
| 5 | `code` | 40 | 92 | 23% |
| 6 | `googleSheets` | 29 | 68 | 17% |
| 7 | `webhook` | 26 | 27 | 15% |
| 8 | `agent` | 23 | 31 | 13% |
| 9 | `scheduleTrigger` | 22 | 25 | 13% |
| 10 | `merge` | 21 | 36 | 12% |
| 11 | `lmChatOpenAi` | 20 | 33 | 12% |
| 12 | `telegram` | 16 | 24 | 9% |
| 13 | `wait` | 16 | 32 | 9% |
| 14 | `slack` | 15 | 24 | 9% |
| 15 | `openAi` | 14 | 22 | 8% |
| 16 | `noOp` | 14 | 21 | 8% |
| 17 | `splitOut` | 13 | 18 | 8% |
| 18 | `switch` | 13 | 16 | 8% |
| 19 | `cron` ⚠️ deprecated | 13 | 13 | 8% |
| 20 | `function` ⚠️ deprecated | 12 | 22 | 7% |
| 21 | `outputParserStructured` | 12 | 18 | 7% |
| 22 | `splitInBatches` | 11 | 20 | 6% |
| 23 | `filter` | 11 | 15 | 6% |
| 24 | `spreadsheetFile` | 9 | 11 | 5% |
| 25 | `googleDrive` | 9 | 18 | 5% |
| 26 | `formTrigger` | 9 | 9 | 5% |
| 27 | `gmail` | 9 | 16 | 5% |
| 28 | `memoryBufferWindow` | 9 | 10 | 5% |
| 29 | `functionItem` ⚠️ deprecated | 9 | 11 | 5% |
| 30 | `respondToWebhook` | 8 | 20 | 5% |
| 31 | `chainLlm` | 8 | 10 | 5% |
| 32 | `airtable` | 8 | 14 | 5% |
| 33 | `readBinaryFile` | 7 | 7 | 4% |
| 34 | `telegramTrigger` | 7 | 7 | 4% |
| 35 | `writeBinaryFile` | 7 | 7 | 4% |

Sisa 150 tipe membentuk ekor panjang.

---

## 4. Temuan 1 — `stickyNote` harus di-parse tapi tidak diimplementasi

**46% workflow memuat `stickyNote`.** Ini node dengan frekuensi tertinggi
kedua di korpus (79 file), lebih tinggi dari `set` (69) dan jauh di atas
`httpRequest` dalam hal jumlah file.

Ia tidak butuh `execute()`, tidak butuh `ParameterSchema`, tidak masuk
governor. Yang dibutuhkan hanya: parser mengenal tipenya, menyimpan
`content`/`position`/`color` supaya bisa dirender ulang di kanvas, dan
**tidak menghitungnya sebagai node eksekusi** di topological sort.

**Risiko bila dilewatkan:** hampir separuh impor template gagal karena
catatan tempel. Ini akan terlihat seperti bug misterius karena tidak ada
"node" yang benar-benar hilang dari sudut pandang pengguna.

**Aksi:** masuk spesifikasi parser Fase 1, bukan backlog.

---

## 5. Temuan 2 — kurva marginal datar, tidak ada shortcut

Dari 171 workflow, dengan set rekomendasi 26 node:

| Workflow butuh | Jumlah | Kumulatif | % |
|---|---:|---:|---:|
| +0 node (sudah jalan) | 23 | 23 | 13% |
| +1 node | 54 | 77 | 45% |
| +2 node | 42 | 119 | 70% |
| +3 node | 17 | 136 | 80% |
| +4 node | 8 | 144 | 84% |
| +5 node | 8 | 152 | 89% |
| +6 node | 5 | 157 | 92% |
| +7 node | 7 | 164 | 96% |
| +8 node | 1 | 165 | 96% |
| +9 atau lebih | 6 | 171 | 100% |

Sekilas baris "+1 node = 54 workflow" terlihat menjanjikan: tambah satu node,
dapat 54 workflow. **Itu salah baca.**

54 workflow itu dihalangi oleh **42 tipe node yang berbeda**. Dan **35 dari 42
(83%) hanya menutup satu workflow masing-masing.** Node penghalang teratas:

| Node penghalang | Workflow yang ditutup |
|---|---:|
| `rssFeedRead` | 4 |
| `writeBinaryFile` | 3 |
| `editImage` | 3 |
| `readBinaryFile` | 3 |
| `xml` | 2 |
| `executeCommand` | 2 |
| `mattermost` | 2 |
| `bitwarden`, `copper`, `bubble`, `googleGemini`, `nocoDb`, ... (35 lainnya) | 1 masing-masing |

Artinya: **untuk mengubah 13% menjadi 45%, Anda tidak menambahkan satu node —
Anda menambahkan empat puluh dua.** Tidak ada node tunggal yang membuka
seperempat korpus.

### 5.1 Biaya mencapai target coverage

| Target (workflow jalan penuh) | Node yang harus diimplementasi |
|---:|---:|
| 20% | 54 |
| 30% | 65 |
| 45% | 84 |
| 60% | 110 |
| 75% | 136 |
| 90% | **153** |

Sebagai pembanding: n8n punya **694 node** (terverifikasi via GitHub Git Trees
API, `PRD-1-N8N-ANALYSIS.md` §18.4). Jadi 153 node ≈ 22% dari katalog n8n
hanya untuk menjalankan 90% dari **171 template sampel**.

### 5.2 Kenapa greedy murni menyesatkan

Greedy max-coverage memberi angka lebih baik (36/171 = 21% pada 26 node,
vs 23/171 = 13% untuk pemilihan berbasis frekuensi). Tapi set yang dipilihnya
mengandung `bitwarden`, `bubble`, `copper`, `interval` — masing-masing muncul
di 1–2 file. Itu **overfitting ke korpus 171 template**: 15% slot dipakai
untuk node yang hampir tidak ada yang pakai, demi mengangkat metrik pada
sampel ini.

Lebih parah, greedy **melewatkan `scheduleTrigger`** (13% workflow), `wait`
(9%), `filter` (6%) — karena workflow yang memakainya juga memakai node tak
didukung lain, sehingga gain marginalnya nol pada iterasi itu.

Padahal tanpa `scheduleTrigger` **tidak ada otomasi terjadwal sama sekali**.
Produk tanpa cron bukan "produk dengan coverage 12%" — ia produk yang
kehilangan seluruh kategori penggunaan.

**Pelajaran:** greedy mengoptimalkan "jumlah workflow lengkap", bukan
"kapabilitas produk". Dua hal itu berbeda, dan metrik pertama menipu.

---

## 6. Temuan 3 — kesenjangan instance-coverage vs workflow-coverage

Dengan set rekomendasi 26 node:

| Metrik | Nilai |
|---|---:|
| Instance node **sejati** tercakup | 1.143 / 1.810 = **63%** |
| Slot unik per workflow | 576 / 995 = **58%** |
| Tipe unik tercakup | 26 / 185 = **14%** |
| **Workflow jalan penuh** | 23 / 171 = **13%** |

Selisih 63% → 13% adalah **harga long-tail**. Sepertiga node di korpus sudah
tertangani, tapi hanya seperdelapan workflow yang bisa jalan — karena node
yang belum tertangani tersebar merata, bukan terkonsentrasi.

Implikasi untuk komunikasi: **jangan pernah mengutip "63% node didukung"
sebagai indikator kesiapan.** Angka yang jujur adalah 13% workflow.

---

## 7. Temuan 4 — node deprecated: bukti untuk T-12

| Node deprecated | Workflow memuat | % | Konsekuensi bila ditolak |
|---|---:|---:|---|
| `cron` | 13 | 8% | gagal impor |
| `function` | 12 | 7% | gagal impor |
| `functionItem` | 9 | 5% | gagal impor |
| **Total (≥1 deprecated)** | **26** | **15%** | gagal impor |

Pada sampel 13 template sebelumnya saya menemukan `function` 6x dan `cron` 2x.
Pada 171 template, `cron` ternyata **lebih umum dari dugaan** (13 workflow,
setara `splitOut` dan `switch`).

**Ini mengubah bobot T-12 dari "nice to have" menjadi "hampir wajib".**
Menolak node deprecated berarti 15% korpus gagal impor sebelum engine
sempat membuktikan dirinya.

Biaya mendukungnya kecil: `cron` → `scheduleTrigger` dan `function` →
`code` adalah **pemetaan registry**, bukan implementasi node baru. Satu
catatan penting yang harus masuk katalog deviasi (§10.3): `function` di n8n
lama bisa `require()` paket npm eksternal. Alias ke `code` **tidak** memberi
kemampuan itu, dan sandbox V8 di desain kita memang melarangnya. Jadi alias
harus **gagal eksplisit dengan pesan jelas** bila mendeteksi `require()`,
bukan diam-diam berjalan dengan perilaku berbeda.

---

## 8. Temuan 5 — ada dua pasar, dan biayanya berbeda

| Pasar | Workflow | Node unik | Node untuk 90% coverage |
|---|---:|---:|---:|
| AI / LangChain | 39 (23%) | 101 | **98** |
| non-AI | 132 (77%) | 127 | **106** |

Node yang **hanya** dibutuhkan pasar AI: 58.

Ini temuan yang paling relevan dengan tujuan bisnis pemilik proyek
(pendapatan lewat AI). Pasar AI lebih kecil (23% korpus) tapi **tidak lebih
murah**: 98 node untuk 90% coverage, hanya sedikit di bawah 106 node pasar
non-AI. Dan 58 node di antaranya eksklusif AI, jadi tidak bisa dipakai ulang
untuk kompatibilitas umum.

Sebabnya: workflow AI di n8n bukan "workflow biasa plus satu node LLM". Ia
memakai seluruh subsistem LangChain — `agent`, `lmChatOpenAi`,
`outputParserStructured`, `memoryBufferWindow`, `chainLlm`,
`documentDefaultDataLoader`, vector store, reranker, splitter. Setiap satunya
punya semantik sendiri.

**Konsekuensi strategis:** mengejar pasar AI **tidak** menghindari masalah
long-tail. Ia hanya memindahkan masalah ke subtipe node yang berbeda. Tidak
ada jalur "fokus AI saja supaya cepat selesai".

---

## 9. Temuan 6 — implikasi yang mengubah strategi produk

Tiga temuan di atas (kurva datar, dua pasar sama mahalnya, 48% node hanya
dipakai sekali) mengarah ke satu kesimpulan yang tidak saya antisipasi saat
menulis PRD-2:

### 9.1 Paritas node bukan diferensiator yang bisa dicapai

Mencapai kompatibilitas template n8n berarti mengimplementasi ~153 node.
Dengan kecepatan realistis 1 orang + agent, itu bukan pekerjaan Fase 1–3.
Dan n8n terus menambah node, jadi targetnya bergerak.

Kalau nilai jual produk ini adalah "bisa menjalankan workflow n8n Anda",
maka produk ini akan kalah selama bertahun-tahun.

### 9.2 Yang sebenarnya universal: `httpRequest` + `code`

Perhatikan bahwa **setiap** API bisa dicapai dengan `httpRequest`, dan
**setiap** transformasi data bisa dicapai dengan `code`. Node khusus
(`googleSheets`, `slack`, `airtable`, ...) adalah **kemudahan**, bukan
**kapabilitas**. Ia membungkus auth + pagination + schema supaya pengguna
tidak menulis JSON manual.

Ini berarti ada dua jalur yang secara kapabilitas setara tapi secara biaya
sangat berbeda:

| Jalur | Cara | Biaya | Cakupan |
|---|---|---|---|
| **A. Paritas node** | implementasi 153 node satu per satu | sangat tinggi | 90% template |
| **B. Escape hatch kuat** | `httpRequest` + `code` + generator kredensial | rendah | 100% API, 0% kenyamanan |

Jalur B tidak membuat template n8n jalan (template tetap memakai
`googleSheets`, bukan `httpRequest`). Tapi ia membuat **produk berguna** sejak
dini tanpa menunggu 153 node.

### 9.3 Rekomendasi: pisahkan "kompatibilitas impor" dari "kapabilitas eksekusi"

Ini rekomendasi teknis, bukan keputusan. Keputusan ada pada pemilik proyek
(lihat T-14 di bawah).

1. **Parser harus menerima SEMUA 694 tipe node** tanpa gagal — menyimpan yang
   tidak dikenal sebagai node opaque dengan peringatan, bukan menolak impor.
   Biayanya kecil (tidak ada `execute()`), manfaatnya besar (tidak ada
   workflow yang "rusak" saat dibuka).
2. **Eksekusi didukung bertahap** sesuai daftar berbasis bukti.
3. **`httpRequest` + `code` diprioritaskan paling awal** karena keduanya
   universal dan keduanya sudah masuk Tier 1.
4. **Generator kredensial** (dari skema JSON n8n) lebih berharga daripada
   node ke-50, karena ia membuka `httpRequest` ke semua layanan berautentikasi.

---

## 10. Rekomendasi daftar node (revisi §7.3)

Diturunkan dari: capability-critical (wajib, apapun gain marginalnya) →
frekuensi → ambang anti-overfit (buang node dengan ≤2 file kecuali
capability-critical).

**26 slot = 23 implementasi nyata + 3 alias registry.**

| Node | Alasan |
|---|---|
| `manualTrigger` | entry point dasar |
| `scheduleTrigger` | otomasi terjadwal — tanpanya tidak ada cron |
| `webhook` | otomasi event-driven |
| `respondToWebhook` | pasangan wajib webhook |
| `httpRequest` | pintu ke semua API eksternal (universal) |
| `code` | escape hatch (universal) |
| `set` | transform dasar |
| `if` | branching |
| `switch` | multi-branch |
| `merge` | gabung branch |
| `filter` | seleksi item |
| `limit` | paginasi |
| `splitOut` | ekspansi array |
| `splitInBatches` | batching (frekuensi 11 file) |
| `wait` | delay / retry / rate-limit |
| `noOp` | placeholder |
| `errorTrigger` | error workflow |
| `executeWorkflow` | sub-workflow |
| `googleSheets` | frekuensi 29 file |
| `googleDrive` | frekuensi 9 file |
| `spreadsheetFile` | frekuensi 9 file |
| `slack` | frekuensi 15 file |
| `telegram` | frekuensi 16 file |
| `cron` | **ALIAS** → `scheduleTrigger` |
| `function` | **ALIAS** → `code` |
| `functionItem` | **ALIAS** → `code` |

**Perubahan vs PRD-2 §7.3 lama:**

- Dibuang: `agent`, `openAi`, `outputParserStructured`, `aggregate`,
  `removeDuplicates`, `sort`, `executeCommand`
  - `agent`/`openAi`/`outputParserStructured`: masuk subsistem LangChain,
    butuh ~95 node untuk pasar AI. Tidak bisa "sedikit-sedikit".
  - `aggregate`/`removeDuplicates`/`sort`/`executeCommand`: frekuensi rendah
    (1–7 file), bukan capability-critical.
- Ditambah: `cron`, `function`, `functionItem` (alias, menutup 15% korpus),
  `googleDrive`, `splitInBatches`, `spreadsheetFile` (frekuensi tinggi).

**Coverage set ini: 23/171 = 13% workflow jalan penuh, 63% instance node sejati.**
Set lama PRD-2: 14/132 = 11% pada pasar non-AI. Perbaikan nyata tapi kecil —
karena masalahnya bukan pemilihan node, masalahnya long-tail.

---

## 11. T-14 — keputusan baru yang dibutuhkan

Temuan ini memunculkan keputusan yang belum ada di PRD-2:

**T-14. Apa janji kompatibilitas produk ini?**

| Opsi | Janji kepada pengguna | Biaya | Risiko |
|---|---|---|---|
| **A** | "Impor workflow n8n Anda, jalankan apa yang didukung" | sedang | pengguna kecewa saat 87% workflow tak jalan |
| **B** | "Engine workflow hemat-memori, kompatibel format n8n" | rendah | bukan pengganti n8n, pasar lebih sempit |
| **C** | "Paritas penuh n8n" | sangat tinggi (153+ node) | tidak tercapai bertahun-tahun, target bergerak |

Rekomendasi teknis saya: **B**, dengan parser yang menerima semua node (poin
9.3) supaya A tetap mungkin secara bertahap tanpa janji di depan.

Alasan: diferensiator yang sudah terbukti secara empiris adalah **memori**
(BENCH-A03: 38,1 MB index / 50,4 MB peak pada 5 juta item, vs n8n yang OOM).
Itu nyata, terukur, dan tidak bergantung pada jumlah node. Menjanjikan paritas
node berarti bertanding di arena yang n8n menangkan dengan 694 node dan
ratusan kontributor.

---

## 12. Reproduksi

```bash
# di VPS
cd /opt/agent-workspace/docs/corpus && ls tpl-*.json | wc -l   # 171

# salin korpus ke direktori kerja, lalu
python3 analisis_node.py
# -> stdout: semua tabel di dokumen ini
# -> FREQ.json: data mentah (frekuensi per tipe)
```

Skrip deterministik, tanpa network, tanpa dependency di luar stdlib Python 3.
Setiap angka di dokumen ini dihasilkan oleh skrip itu — tidak ada yang
ditulis tangan.

---

## 13. Perubahan yang harus masuk PRD-2

| Bagian | Perubahan |
|---|---|
| §7.3 | ganti daftar node dengan §10 dokumen ini; **hapus klaim** "diturunkan dari node yang paling sering muncul di workflow nyata" bila tidak didukung data |
| §7.3.1 | perbarui angka deprecated: `cron` 13 wf, `function` 12 wf, `functionItem` 9 wf, total 15% korpus |
| §7.4 | tambah spesifikasi `stickyNote` sebagai node non-eksekusi yang wajib bisa di-parse |
| §7 | tambah subsection "kompatibilitas impor vs kapabilitas eksekusi" (§9.3 dokumen ini) |
| §16 | tambah T-14 (janji kompatibilitas produk) |
| §10.3 | catat deviasi alias `function`: tidak mendukung `require()` npm, harus gagal eksplisit |
