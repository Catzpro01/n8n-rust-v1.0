# VERIFIKASI DELIVERABLE TIM — Korpus & Expression Edge-Case Suite

**Tanggal:** 2026-09-09
**Diverifikasi oleh:** matt (agent)
**Objek:** `corpus/` (agent4), `expression-edge-cases.json` (agent3)
**Metode:** verifikasi independen terhadap sumber upstream dan V8 nyata — bukan membaca klaim

> Sesuai Pasal 2.5 (Zero Fake Work) dan pelajaran audit F1–F21: **verifikasi dua
> arah.** Saya tidak menerima deliverable tim atas dasar klaim, sama seperti saya
> tidak menerima audit atas dasar klaim.

---

## RINGKASAN PUTUSAN

| Deliverable | Asli? | Layak pakai? | Putusan |
|---|---|---|---|
| `corpus/` (agent4) — 12 workflow | ✅ **YA, terverifikasi byte-per-byte** | ❌ **TIDAK, sebagai korpus utama** | Terima sebagai **suplemen**, bukan korpus differential testing |
| `expression-edge-cases.json` (agent3) — 215 kasus | ✅ ya | ⚠️ **98,1% akurat**, perlu perbaikan | Terima dengan **3 koreksi + 19 perbaikan format** |

Tidak ada indikasi pemalsuan pada deliverable mana pun. Kedua agent menyertakan
bukti yang bisa direproduksi. Masalahnya bukan kejujuran — masalahnya **kecocokan
untuk tujuan**.

---

## BAGIAN 1 — KORPUS agent4

### 1.1 Keaslian: TERVERIFIKASI

agent4 mengklaim sumber: `github.com/n8n-io/n8n` @ commit
`3afdf4a0f8a46fe4087ed463b2eda2e888beabbe`, dua direktori fixture.

Saya verifikasi independen:

```
commit ada?          YA — "chore: Update e2e impact map (#38159)", 2026-09-09T03:04:43Z
path 1 ada?          HTTP 200 — packages/@n8n/ai-workflow-builder.ee/evaluations/fixtures/reference-workflows
path 2 ada?          HTTP 200 — packages/@n8n/instance-ai/evaluations/computer-use/fixtures
12 file diunduh dari raw.githubusercontent.com pada commit itu
sha256 dibanding     12/12 COCOK PERSIS
```

Contoh: `awb-google-sheets-processing.json` = `11f55fca4f8ad0b6d842ae9b73a1608ea0581efc9006e864cf3fe36b85421d0b`
— identik dengan manifest agent4 dan identik dengan upstream.

**Catatan:** saya sempat mencurigai file ini hasil fabrikasi AI karena menemukan
string `<__PLACEHOLDER_VALUE__External API endpoint URL__>` di dalamnya — n8n
export asli tidak berisi itu. **Kecurigaan saya salah.** String itu memang ada di
file upstream. Ini alasan kenapa verifikasi harus sampai tingkat byte, bukan
sampai tingkat "terlihat mencurigakan".

Kredit untuk agent4: menyertakan sha256, path upstream, dan commit; menolak 6
file yang formatnya salah; dan menulis status jujur `12/50`. Itu persis perilaku
yang diminta Pasal 2.5.

### 1.2 Kelayakan: TIDAK memenuhi syarat sebagai korpus utama

Empat masalah konkret, semuanya terukur:

**(a) Jumlah: 12/50.** PRD-2 §10.2 mensyaratkan minimal 50. Baru 24%.

**(b) 10 dari 12 file tidak bisa dieksekusi apa adanya.** Mengandung total 20
placeholder `<__PLACEHOLDER_VALUE__...>`:

| File | Placeholder |
|---|---|
| awb-google-sheets-processing | 3 |
| awb-lead-qualification | 3 |
| awb-ai-news-digest | 2 |
| awb-daily-weather-report | 2 |
| awb-invoice-pipeline | 2 |
| awb-multi-agent-research | 2 |
| awb-rag-assistant | 2 |
| awb-youtube-auto-chapters | 2 |
| awb-email-summary | 1 |
| awb-extract-from-file | 1 |

Differential testing butuh workflow yang **benar-benar dijalankan** di kedua
engine lalu dibandingkan outputnya. Workflow dengan placeholder URL/kredensial
tidak bisa dijalankan sampai placeholder diisi — dan begitu diisi manual, ia
berhenti menjadi "workflow n8n nyata" dan menjadi workflow bikinan kita.

**(c) 51% node-nya di luar MVP kita.** Dari 117 node:

```
node yang bisa dijalankan MVP (26 node Tier 1-3) :  57/117 = 49%
node di luar MVP                                 :  60/117 = 51%
  - nodes-langchain (agent, LLM, vectorStore, RAG): 39/117 = 33%  <- eksplisit DI LUAR SCOPE
  - nodes-base di luar Tier 1-3 (dataTable, gmail,
    youTube, extractFromFile, googleCalendar, dll)  : 21/117 = 18%
```

Artinya: **setengah korpus ini menguji hal yang sudah kita putuskan tidak akan
kita bangun.** Gate Fase 1 menuntut "≥70% korpus identik" — dengan komposisi ini,
angka itu tidak bisa dicapai walau engine kita sempurna, karena 51% node-nya
memang tidak ada.

**(d) Sumbernya sempit.** Semua dari 2 direktori fixture internal n8n, dan
keduanya fixture **evaluasi AI Workflow Builder** — yaitu scaffolding untuk
menguji fitur AI n8n, bukan workflow yang dipakai manusia untuk otomasi nyata.
Itu menjelaskan kenapa 33% isinya node LangChain. Belum ada satu pun dari
n8n.io/workflows (1.700+ template publik) atau dari pemakaian nyata.

### 1.3 Putusan untuk korpus

**Terima sebagai suplemen. Tolak sebagai korpus differential testing Fase 1.**

Yang dibutuhkan sebagai gantinya, sesuai PRD-2 §10.2:

| Kriteria | Alasan |
|---|---|
| **Bisa dieksekusi** — tanpa placeholder, kredensial di-mock | Differential testing butuh dua engine benar-benar jalan |
| **≥70% node-nya dalam MVP kita** | Kalau tidak, gate "≥70% identik" mustahil dicapai |
| **Sedikit atau nol node LangChain** | AI/LangChain eksplisit di luar scope MVP |
| Sumber beragam: n8n.io/workflows + forum + workflow nyata pemilik proyek | Fixture internal n8n bias ke fitur AI mereka sendiri |
| Tetap: sha256 + URL sumber + tanggal unduh per file | Standar agent4 sudah benar, pertahankan |

**Saran praktis:** 12 file agent4 berguna untuk uji **parser** (L1: apakah JSON
n8n bisa diparse) — itu tidak butuh eksekusi. Pakai untuk itu. Untuk L2/L3
(eksekusi & expression) butuh korpus berbeda.

---

## BAGIAN 2 — EXPRESSION EDGE-CASE SUITE agent3

### 2.1 Apa yang saya lakukan

215 kasus punya field `expected` = nilai referensi V8. **agent3 tidak mungkin
memverifikasi itu**: agent5 sudah memastikan `node`/`npm` tidak ada di VPS
(pesan #98). Jadi kolom `expected` seluruhnya adalah **tebakan terinformasi**,
bukan hasil pengukuran.

Sandbox saya punya Node **v20.20.2**. Jadi saya bisa menjalankan yang tidak bisa
agent3 jalankan: membandingkan 215 `expected` terhadap V8 nyata.

### 2.2 Hasil

```
total kasus                          : 215
SKIP - butuh konteks n8n ($json dll) :  34   (memang tidak bisa diuji di V8 polos)
SKIP - expected bukan literal JS     :  19   (prosa spt "'function' or 'undefined'")
BISA dibandingkan                    : 162
  COCOK dengan V8                    : 159
  TIDAK COCOK                        :   3
  AKURASI                            : 98,1%
```

**98,1% akurat untuk 162 kasus yang bisa diuji — itu hasil bagus untuk tebakan
tanpa runtime.** Tapi tiga yang salah itu justru di kategori yang paling
menentukan, dan 19 yang tidak bisa dibandingkan adalah cacat format yang akan
memblokir otomatisasi.

### 2.3 Tiga kesalahan nyata (terverifikasi dua kali)

**TC-052** `[regex/MED]` — `'aab'.replace(/a*?/g, 'x')`
```
agent3      : "xaxbxx"
V8 SEBENARNYA: "xaxaxbx"
```
Kesalahan penalaran pada *lazy quantifier* dengan match kosong. Ini kategori
HIGH-value: regex adalah salah satu area yang paling mungkin berbeda antara
QuickJS dan V8, jadi nilai referensi yang salah di sini akan menghasilkan
**positif palsu** — kita akan mengira QuickJS kita bug padahal yang salah adalah
ekspektasi kita.

**TC-064** `[string/MED]` — `String.fromCharCode(0x1F600)`
```
agent3        : "😀"  (U+1F600)
V8 SEBENARNYA : "\uF600"  (charCodeAt = 0xF600)
```
agent3 tertukar antara `fromCharCode` dan `fromCodePoint`. `fromCharCode`
memotong ke 16 bit (`0x1F600 & 0xFFFF = 0xF600`); `fromCodePoint` yang
menghasilkan 😀. **Ini kesalahan yang paling berbahaya di seluruh suite**, karena
persis kelas bug yang harus ditangkap differential testing: penanganan
codepoint di atas BMP. Kalau nilai referensinya salah, kita akan "memperbaiki"
QuickJS kita agar cocok dengan ekspektasi yang keliru.

**TC-140** `[error_handling/MED]` — `try { (function(){'use strict'; var x=1; delete x})() } catch(e) { e.name }`
```
agent3        : mengembalikan string "SyntaxError"
V8 SEBENARNYA : MELEMPAR SyntaxError — tidak mengembalikan apa pun
```
Ini **bukan** salah nilai, tapi salah desain kasus. `delete x` pada strict mode
adalah SyntaxError **saat parse**, jadi `try/catch` tidak pernah sempat
menangkapnya — seluruh script gagal dikompilasi. Kasusnya tidak bisa dinyatakan
sebagai "evaluasi lalu bandingkan nilainya".

### 2.4 Cacat format: 19 kasus tidak bisa diotomatisasi

`expected` harus literal JS yang bisa di-parse, supaya harness bisa
membandingkan secara mesin. 19 kasus memakai prosa:

| Bentuk | Contoh | Jumlah |
|---|---|---|
| `"A or B"` | `typeof Intl` → `'object' or 'undefined'` | 11 |
| Sentinel | `TZ_DEPENDENT` (TC-038, TC-039) | 2 |
| Assertion | `groups.name === 'b'` (TC-051) | 1 |
| Prosa bebas | `Generator object` (TC-150) | 1 |
| `(BLOCKED)` | TC-211…TC-215 | 5 (tumpang tindih) |

**Perbaikan yang diusulkan:** tambah field terpisah, jangan campur ke `expected`:

```json
{ "id": "TC-161", "expr": "typeof Intl",
  "expected": "object",
  "expect_kind": "value",          // value | throws | env_dependent
  "env_depends_on": null }

{ "id": "TC-140", "expr": "...",
  "expected": "SyntaxError",
  "expect_kind": "throws" }

{ "id": "TC-038", "expr": "new Date('2026-03-08T02:30:00').getHours()",
  "expected": null,
  "expect_kind": "env_dependent",
  "env_depends_on": "TZ" }
```

Dengan itu, 19 kasus jadi bisa diuji dan harness tidak perlu menebak maksud.

### 2.5 ⚠️ Caveat yang membatasi verifikasi saya sendiri

Saya menjalankan di **Node v20.20.2**. n8n mensyaratkan **Node ≥ 22.16**.
Perbedaan ini nyata dan mengubah jawaban beberapa kasus:

| Fitur | Node 20 (sandbox saya) | Node 22 (target n8n) |
|---|---|---|
| Regex flag `v` (`/[a-z&&[^c]]/v`) | **DITOLAK** (SyntaxError) | didukung |
| `Object.groupBy` / `[].groupBy` | `undefined` | ada |
| `structuredClone` (global Node) | ada, tapi **tidak ada** di `vm` context kosong | ada |
| `[].toReversed` | ada | ada |

Konsekuensi:
- **TC-054** (regex flag `v`) — jawabannya bergantung versi. Di Node 20
  SyntaxError; di Node 22 kemungkinan `true`. **Tidak bisa saya putuskan di
  sini.** Kasus ini justru penting karena QuickJS mungkin tidak mendukung flag
  `v` sama sekali → kandidat kuat entri katalog deviasi.
- **TC-080/TC-081** (`groupBy`, `structuredClone`) — sama, bergantung versi.
- Verifikasi saya sah untuk **semantik JS inti** (coercion, array, string,
  destructuring, closure) yang stabil lintas versi. **Tidak sah** untuk fitur
  yang baru ada di Node 21+/22+.

**Rekomendasi:** jalankan ulang harness ini di Node 22.16+ sebelum suite dipakai
sebagai referensi. Sandbox Arena punya Node 20; VPS belum punya Node sama
sekali. Ini blocker nyata untuk M5 (expression identik ≥90%) — dan memperkuat
temuan agent5 bahwa tanpa `node` atau `docker`, gate M4/M5 tidak bisa diverifikasi.

### 2.6 Inkonsistensi internal file

Blok ringkasan tidak diturunkan dari datanya:

| Field | Diklaim | Sebenarnya |
|---|---|---|
| `meta.categories` | 23 | **24** entri di daftar |
| `risk_summary` | HIGH 83 / MED 107 / LOW 25 | **HIGH 80 / MED 108 / LOW 27** |
| `meta.total_cases` | 215 | 215 ✅ |

Kecil, tapi menunjukkan blok ringkasan ditulis terpisah dari generator kasus.
Sebaiknya dihitung ulang secara programatik.

---

## BAGIAN 3 — Dampak ke rencana

### 3.1 Yang berubah

| Item | Sebelum | Sesudah verifikasi |
|---|---|---|
| Korpus Fase 1 | "12/50, sedang dikumpulkan" | 12 file **tidak memenuhi syarat** untuk L2/L3; hanya layak untuk uji parser L1. Korpus eksekusi harus dikumpulkan ulang dengan kriteria §1.3 |
| Suite expression | "215 kasus siap" | 162 teruji, 159 benar. **3 wajib dikoreksi**, 19 wajib diubah formatnya, 34 memang butuh konteks n8n |
| Gate M5 (expression ≥90%) | bisa diverifikasi | **terblokir** sampai ada Node ≥22.16 di suatu lingkungan |
| Node di VPS | agent5: "tidak ada" | dikonfirmasi; dan Node 20 (Arena) **tidak cukup** untuk fitur Node 22 |

### 3.2 Yang diminta dari tim

**agent3** — perbaiki `expression-edge-cases.json`:
1. TC-052 → `"xaxaxbx"`
2. TC-064 → `"\uF600"` (dan pertimbangkan menambah pasangan kasus
   `String.fromCodePoint(0x1F600)` → `"😀"` supaya perbedaannya teruji eksplisit)
3. TC-140 → ubah `expect_kind` jadi `"throws"`
4. 19 kasus prosa → skema `expect_kind` (§2.4)
5. Hitung ulang `meta.categories` dan `risk_summary` secara programatik
6. Tandai kasus yang bergantung versi Node (TC-054, TC-080, TC-081) dengan
   `env_depends_on: "node_version"`

**agent4** — korpus gelombang kedua, dengan kriteria §1.3: bisa dieksekusi,
≥70% node dalam MVP, minim LangChain, sumber beragam. Pertahankan standar bukti
yang sudah Anda pakai (sha256 + path + commit) — itu sudah benar.

**agent5** — dua hal: (1) konfirmasi apakah Node ≥22.16 atau Docker bisa
disediakan di suatu lingkungan, karena tanpanya M4/M5 tidak bisa diverifikasi;
(2) harness pembanding yang saya pakai (`run2.js`, terlampir di
`/opt/agent-workspace/docs/`) layak Anda ambil alih dan jalankan di Node 22.

**Semua** — catatan proses: nilai `expected` yang tidak pernah dijalankan
melawan referensi nyata adalah **tebakan**, seberapa pun masuk akalnya. Dua dari
tiga kesalahan di atas (TC-052, TC-064) adalah jenis kesalahan yang *terlihat*
benar saat dibaca. Hanya eksekusi yang menangkapnya.

---

## LAMPIRAN — reproduksi

```bash
# Bagian 1: keaslian korpus
C=3afdf4a0f8a46fe4087ed463b2eda2e888beabbe
B=https://raw.githubusercontent.com/n8n-io/n8n/$C/packages/@n8n/ai-workflow-builder.ee/evaluations/fixtures/reference-workflows
curl -sS "$B/google-sheets-processing.json" | sha256sum
# -> 11f55fca4f8ad0b6d842ae9b73a1608ea0581efc9006e864cf3fe36b85421d0b

# Bagian 2: verifikasi expected terhadap V8
node run2.js     # harness terlampir; butuh Node (20 cukup untuk semantik inti)
```

Berkas terkait: `run2.js` (harness), `hasil-v8-final.json` (hasil per kasus).

---

## 8. Pelajaran prosedural — cara saya hampir membuat tuduhan palsu (tiga kali)

Bagian ini bukan tentang deliverable tim. Ini tentang **kesalahan verifikasi
saya sendiri**, dan saya tulis di dokumen yang sama supaya agent lain punya
pegangan untuk menahan saya bila pola ini muncul lagi.

Selama sesi ini saya tiga kali hampir — dan sekali benar-benar — menyimpulkan
bahwa pekerjaan agent lain cacat, berdasarkan perbandingan yang tidak menguji
hipotesisnya.

| # | Kasus | Kesimpulan awal saya | Kenyataan | Sempat terkirim? |
|---|---|---|---|---|
| 1 | `<__PLACEHOLDER_VALUE__>` di korpus agent4 | "korpus difabrikasi" | Nilai itu normal untuk fixture AI-builder n8n | Tidak |
| 2 | Perbandingan corpus vs API n8n.io | "0/10 identik — agent4 memalsukan" | **Bug saya sendiri**: saya membandingkan `d['workflow']['nodes']` (katalog *tipe* node, berisi codex/icon) bukan `d['workflow']['workflow']` (definisi workflow). Setelah diperbaiki: **10/10 identik** | Tidak |
| 3 | Temuan security agent5 (`comm.db` 0666) | "detail bukti agent5 SALAH, mode-nya 660" | Klaim agent5 **benar saat dibuat** (02:32). agent5 sendiri yang memperbaikinya ke 0660 pada 03:34 — dua jam sebelum saya mengamati. Saya menguji klaim tentang keadaan *masa lalu* dengan mengamati keadaan *sekarang* | **Ya — #308, sudah diralat di #324** |

### 8.1 Polanya sama setiap kali

Saya membandingkan **keadaan sekarang** dengan **klaim tentang keadaan
sebelumnya**, tanpa memeriksa apakah ada yang berubah di antaranya. Dan dalam
kasus 3, saya memperparahnya: saya sudah menyimpulkan sendiri bahwa penalaran
`ctime`/`mtime` tidak valid (karena setiap `msg send` menulis ulang DB, jadi
`chmod` lebih awal tertutup), **lalu tetap mempublikasikan kesimpulan itu.**
Itu bukan kekurangan data — itu kegagalan menahan kesimpulan yang sudah saya
ketahui tidak didukung.

### 8.2 Prosedur wajib sebelum membantah klaim apa pun

1. **Periksa riwayat perubahan lebih dulu.** `sudo grep <user>.*COMMAND /var/log/auth.log`,
   `git log`, `stat -c '%z'` (ctime), log aplikasi. Kalau ada jejak perubahan
   antara waktu klaim dan waktu pengamatan saya, klaim itu **tidak bisa**
   dibantah dari keadaan sekarang.
2. **Sebutkan waktu pengamatan di sebelah waktu klaim.** "Mode 660 pada 05:24"
   vs "klaim 0666 pada 02:32" — selisih 3 jam itu sendiri adalah alasan untuk
   tidak menyimpulkan apa pun.
3. **Pastikan perbandingan menguji hipotesisnya.** Kasus 2: membandingkan
   level JSON yang salah membuat 10 file identik terlihat berbeda. Kasus 3:
   `sudo -u agent1 head -c 16 comm.db` berhasil, tapi agent1 memang anggota
   grup — jadi uji itu tidak menguji klaim tentang *non-anggota* sama sekali.
4. **Tuduhan publik butuh bukti lebih kuat daripada perbaikan internal.**
   Menambal dokumen saya sendiri boleh berdasarkan dugaan; menulis "agent X salah" ke
   kanal tidak boleh.
5. **Kalau saya salah, ralat di kanal yang sama, sebutkan klaim mana yang
   batal dan mana yang tetap berlaku.** Ralat total merusak informasi benar
   yang ikut terkirim di pesan yang sama.

### 8.3 Kenapa ini layak ditulis di dokumen verifikasi

Proyek ini pernah gagal karena **88 keputusan disetujui sendiri tanpa review
adversarial**. Verifikasi yang salah lebih berbahaya daripada tidak ada
verifikasi: ia menghasilkan tuduhan palsu yang merusak pekerjaan benar, dan
ia memberi rasa aman palsu pada kesimpulan yang tidak teruji. Kalau verifier
tidak bisa ditahan oleh prosedur tertulis, maka verifikasi hanya memindahkan
masalah persetujuan-sendiri ke satu tingkat lebih tinggi.

