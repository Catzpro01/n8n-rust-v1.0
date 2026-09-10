# AGENT10 — REVIEW COMPLIANCE PRA-SIGN-OFF
## atas `PRD-3-PERFECTION-CHECKLIST.md` v3.0.0-FINAL

| | |
|---|---|
| **Versi** | v1.0 |
| **Penulis** | agent10 — ROLE_COMPLIANCE |
| **Tanggal** | 2026-09-09, observasi 07:04–07:1x UTC |
| **Subjek** | `/opt/agent-workspace/docs/PRD-3-PERFECTION-CHECKLIST.md` (8.356 byte, mtime 07:01, milik `fern`) |
| **Status subjek** | *"DRAFT FINAL UNTUK PERSETUJUAN PEMILIK PROYEK"* — §6: *"Pengesan final dilakukan secara eksplisit oleh Pemilik Proyek"* |
| **Mengapa sekarang** | Ini **satu-satunya** kesempatan murah. Setelah ditandatangani, Zero-Code Mandate terbuka dan Wave 1 berjalan; cacat gerbang baru terdeteksi saat implementasi, bukan saat dokumen. |
| **Kepatuhan proses** | Freeze #386 dipatuhi: nol kode implementasi, nol chmod, nol perubahan sistem, **nol suntingan ke file milik agen lain** — dokumen ini file baru milik saya. Semua perintah read-only. |
| **Dokumen pendamping** | `AGENT10-COMPLIANCE-CRYPTO-AUDIT.md` (v1.2 di server, 936 baris) — temuan C-01…C-10, gate G-C1…G-C10, keputusan D-1…D-7, bukti eksekusi §2.11 |

---

## 0. Ringkasan — 5 temuan, 1 di antaranya memblokir sign-off

| ID | Severity | Temuan | Aksi sebelum sign-off |
|---|---|---|---|
| **P3-01** | **BLOCKER** | Gerbang **L2 baris 97 salah label**: *"Keyed derive_key untuk Execution Envelope dan audit trail (`G-C6`)"*. `G-C6` adalah **known-answer test** (vektor uji hash), **bukan** uji keyed chain. Perbaikan C-01 (CRITICAL) diuji oleh **G-C1(b)**. Akibat: L2 bisa dinyatakan **lulus** dengan menjalankan KAT sementara total-rewrite tidak pernah diuji | **Wajib** dikoreksi |
| **P3-02** | **HIGH** | **Item Lineage tidak ada di PRD-3 sama sekali** — padahal fern #400 menerimanya sebagai pilar PRD-3, dan matriks resmi menugaskannya ke ROLE_COMPLIANCE. `grep -ic` atas PRD-3: `lineage`=**0**, `GDPR`=**0**, `crypto-shred`=**0**, `PII`=**0**, `retensi`=**0**, `lisensi`=**0**. Tidak ada tugas di `task_queue` (15 tugas, satu-satunya ROLE_COMPLIANCE = `W3-EXEC-ENVELOPE`) | **Wajib** dimasukkan, atau diputuskan sadar sebagai *out of scope* |
| **P3-03** | **HIGH** | **Kepemilikan Envelope ganda tanpa pembagian**: PRD-3 baris 59 menulis `(agent5/agent10)`, dan agent5 #530 berkomitmen *"Saya tulis ulang spesifikasi Envelope"*, sementara #500 butir 11 + handover #521(C) menugaskannya ke saya. Dua penulis, satu artefak, tanpa lane | **Wajib** diputuskan fern/matt |
| **P3-04** | **MEDIUM** | **SEC-08 di `AGENT5_QA_SECURITY_SPEC.md` (mtime 06:54:15) masih versi LAMA** — kriteria lulus (a) hanya menguji penyerang yang *menghapus*. agent5 sudah menerima di #530 bahwa ini tidak cukup, tapi teksnya belum diubah. PRD-3 akan mengonsolidasikan spec yang isinya sudah diketahui salah oleh penulisnya sendiri | Sebaiknya sebelum sign-off |
| **P3-05** | **MEDIUM** | **Gerbang `I18N-40` memuat batas `<500KB` yang belum diukur** sebagai kriteria lulus. Angka itu sudah saya flag di #567 butir (iii) sebagai kelas F15 (klaim tanpa pengukuran) + ERR-029 (satuan). Catatan: **provenance tag (`human`, `mt`, `glossary`) yang saya usulkan SUDAH diadopsi** — itu bagus, dan saya catat sebagai adopsi, bukan keberatan | Ukur dulu, atau tandai sebagai target bukan kriteria |

**Yang TIDAK saya persoalkan:** struktur 4-Wave, hard-cap 500MB, `SEC-TRANSIENT` (angka 1.920 byte yang saya ukur sudah terserap benar di baris 26 & 96), `HUB-6`/`HUB-7`, dan protokol sign-off §6. Semuanya konsisten dengan bukti yang ada.

---

## 1. P3-01 · BLOCKER · Gerbang L2 salah label, dan yang hilang adalah gerbang untuk temuan CRITICAL

### 1.1 Fakta

`PRD-3-PERFECTION-CHECKLIST.md:94–97`:

```
* **Gerbang L2 (Keamanan Sandboxing & Kriptografi)**:
  - Eksekusi WASM community node terisolasi ketat dalam linear memory 32MB (SEC-WCB-01..04).
  - BLAKE3 sequential hasher mematuhi batas transien 4KB (SEC-TRANSIENT).
  - Keyed derive_key untuk Execution Envelope dan audit trail (G-C6).        ← INI
```

Definisi `G-C6` yang sebenarnya — `AGENT10-COMPLIANCE-CRYPTO-AUDIT.md:866`:

```
| **G-C6** `known_answer_test` | vektor uji resmi BLAKE3 + SHA-256 | 100 % cocok. Gagal = dependensi hash salah/silently diganti |
```

### 1.2 Analisis

`G-C6` dan *keyed derive_key* adalah **dua hal yang tidak berhubungan**:

| | `G-C6` (known-answer test) | Perbaikan C-01 (keyed chain + anchor) |
|---|---|---|
| Yang diuji | apakah implementasi hash **adalah** algoritma yang diklaim | apakah riwayat **bisa dipalsukan** oleh penyerang ber-root |
| Gate saya yang benar | `G-C6` | **`G-C1(b)`**: "ubah 1 entri lalu rekomputasi seluruh rantai + root → **juga harus BROKEN**" |
| Kelas cacat yang ditutup | C-05 (hash palsu: `blake3_hash()` berisi `DefaultHasher`) | C-01 (total-rewrite tak terdeteksi) |
| Bukti empiris | §2.11 baris 5: KAT BLAKE3("")/SHA-256("") cocok | §2.11 baris 11–12: rewrite **lolos** tanpa kunci, **gagal** dengan kunci |

Menjalankan `G-C6` **tidak memberi informasi apa pun** tentang apakah rantai hash bisa dipalsukan. Jadi kalau L2 ditandatangani seperti sekarang, urutan kegagalannya:

1. Implementasi menulis Envelope **tanpa** `chain_key` (paling murah, dan tidak ada gate yang memaksa).
2. CI menjalankan `G-C6` → **lulus** (BLAKE3 memang BLAKE3).
3. L2 dinyatakan **hijau**.
4. C-01 — temuan **CRITICAL**, satu-satunya di audit saya — **lolos ke produksi dengan sertifikat gerbang**.

Ini bukan risiko teoretis: §2.11 baris 11 menunjukkan pemalsuan itu **berhasil saya lakukan dalam 20 baris**.

### 1.3 Perbaikan yang saya usulkan (satu baris, nol kode)

Ganti baris 97 menjadi:

```
  - Envelope chain WAJIB keyed (BLAKE3 derive_key(context, chain_key) dengan chain_key di luar
    filesystem) + anchor eksternal tiap N=1.000 entri. Gerbang: G-C1 (dua kasus: hapus-dan-rekomputasi
    KEDUANYA harus BROKEN) dan G-C6 (known-answer test) sebagai gate terpisah. (agent10 D-1/D-3/D-4)
```

`G-C6` **tetap** di L2 — ia memang gate L2 yang sah untuk C-05. Yang salah adalah ia **dipakai sebagai label untuk requirement lain**, sehingga requirement itu kehilangan gate-nya.

**Catatan kejujuran:** kesalahan label ini **bisa jadi berasal dari saya**. Di #555 saya menulis daftar gate dan menyebut `G-C6` di konteks "known-answer test menutup kelas C-05", tapi saya tidak pernah memasangkan `G-C6` dengan *keyed derive_key*. Penulis PRD-3 kemungkinan menggabungkan dua butir dari pesan saya yang panjang. Saya sebut ini supaya koreksinya tidak dibaca sebagai menyalahkan orang lain: **saya yang mengusulkan 10 gate dengan ID singkat, dan ID singkat memang mudah salah tempel.** Pelajarannya: usulan gate sebaiknya dikirim sebagai tabel (requirement → gate ID), bukan prosa — dan itu akan saya lakukan mulai sekarang.

---

## 2. P3-02 · HIGH · Item Lineage hilang dari PRD-3, padahal sudah diterima supervisor

### 2.1 Fakta

**Diterima secara eksplisit oleh fern, `#blockers` #400 (05:56) butir 2:**

> "- @agent1: **Item Lineage** (0 RAM hot-path) & Preflight Dry-Run (0 side-effects) **DITERIMA untuk masuk katalog fitur PRD-3**.
> - @agent5: Execution Envelope Rolling Hash Chain (32B RAM constant) **DITERIMA sebagai pilar audit enterprise**."

**Ditugaskan ke peran saya, dua kali:**

- fern `#500` butir 11: *"@agent10 (Enterprise Compliance - BARU): Execution Envelope (cryptographic rolling hash chain), **Item Lineage (GDPR audit)**."*
- `SISTEM-DISTRIBUSI-TUGAS-OTOMATIS.md` §2, baris agent10: *"Audit Trail, Execution Envelope, **Lisensi, PII** | **Crypto-shredding**, Hash chains"*

**agent5 #521(C)** juga menyerahkan keduanya ke saya: *"Item Lineage (agent1 #365) juga ke Anda; keduanya saling terkait (lineage-refs bisa masuk chain envelope)."*

**agent1 #522** sudah merevisi desain lineage-nya menyerap 4 kritik saya (ref pseudonim content-anchored, fan-in sebagai himpunan 16B, crypto-shredding `salt_exec`, dan **menurunkan klaim GDPR jadi "pseudonymised-by-construction, butuh review hukum"**).

### 2.2 Tapi di PRD-3

```
$ grep -ic <kata> PRD-3-PERFECTION-CHECKLIST.md
Lineage      : 0        crypto-shred : 0     GDPR      : 0
lineage      : 0        shredding    : 0     PII       : 0
pseudonim    : 0        retensi      : 0     lisensi   : 0
```

§1 butir 4 mendaftar 11 inovasi; **Item Lineage tidak ada di antaranya** (yang ada: Native MCP Server, Passive Skill Capsule, Workflow Hub, UI Self-Documenting, Translasi 40 Bahasa, Timeline Execution Replay, **Execution Envelope**, WCB, CASD Dedup, EBC QuickJS Cache, swarm-task).

`swarm-task list` = **15 tugas**; satu-satunya `ROLE_COMPLIANCE` adalah `W3-EXEC-ENVELOPE`. **Tidak ada tugas lineage.**

### 2.3 Mengapa ini matters — dan mengapa ini ranah compliance, bukan fitur

Tiga kewajiban yang hilang bersama Item Lineage:

1. **Jejak data subjek (GDPR Art. 15/30).** Tanpa lineage, produk tidak bisa menjawab "data subjek X mengalir ke node mana saja" — dan Workflow Hub (#524) justru menarik data publik yang **pasti** memuat nama orang (politik, berita).
2. **Hak penghapusan vs append-only (Art. 17).** Envelope adalah trail yang tidak boleh diubah. Tanpa crypto-shredding, menghapus data subjek berarti merusak trail — dua kewajiban yang saling membatalkan. Solusinya (`salt_exec` di key store terpisah) **sudah dirancang dan sudah disetujui agent1**, tapi tidak tercatat di PRD-3, jadi tidak akan ada yang mengimplementasikannya.
3. **Retensi.** PRD-2 §5.3 punya tabel retensi + mode `retain-outputs` (audit F21). PRD-3 tidak menyebut retensi sama sekali (`retensi`=0), padahal §5 gerbang L5 menguji Hub yang menarik konten web. Rekaman respons web publik **adalah data pribadi** bila feed-nya memuat nama orang — ini yang membuat agent1 (#557c) menyetujui `record-metadata-only` sebagai **default**. Keputusan itu belum punya tempat di PRD-3.

### 2.4 Perbaikan yang saya usulkan

Tambahkan ke §1 butir 4 (jadi 12 inovasi) **dan** ke Wave 3 §3 **dan** satu tugas baru:

```
§3 Wave 3:  - Item Lineage & Crypto-Shredding (GDPR Art.15/17/30) (agent10, depends: agent1)
task_queue: W3-ITEM-LINEAGE  wave=3  prio=P1  role=ROLE_COMPLIANCE  depends_on=W3-EXEC-ENVELOPE,W2-CASD-DEDUP
gerbang L5: LIN-1  setiap ItemRef memverifikasi ke content_hash target; ref yang targetnya sudah
                   di-GC -> status UNKNOWN eksplisit, bukan diam-diam salah   (gate G-C9)
            LIN-2  crypto-shredding: hancurkan salt_exec -> 0 ref bisa ditautkan ulang, DAN trail
                   tetap VERIFIED_ANCHORED (immutability tidak rusak)          (gate G-C10)
            LIN-3  klaim pemasaran: DILARANG menulis "GDPR-compliant"; yang boleh
                   "pseudonymised by construction" + catatan butuh review hukum (agent1 #522)
```

`LIN-3` sengaja saya masukkan sebagai **gerbang**, bukan catatan: klaim kepatuhan yang berlebihan adalah liabilitas yang bisa dipatahkan auditor, dan gerbang adalah satu-satunya cara tim ini menahan klaim (pola yang agent1 pakai sendiri di #463 untuk BENCH-A03).

**Kalau pemilik proyek memutuskan lineage di luar cakupan MVP, itu keputusan yang sah** — tapi harus tertulis sebagai keputusan sadar di PRD-3, bukan hilang diam-diam. Yang tidak boleh adalah fern #400 menerimanya, #500 menugaskannya, agent1 mendesainnya, lalu PRD-3 tidak menyebutnya.

---

## 3. P3-03 · HIGH · Kepemilikan Envelope ganda tanpa pembagian lane

### 3.1 Fakta

| Sumber | Bunyi |
|---|---|
| PRD-3 `:59` | `Execution Envelope & Rolling Hash-Chain Audit (agent5/agent10)` |
| fern `#500` butir 11 | agent10: *Execution Envelope (cryptographic rolling hash chain)* |
| agent5 `#521(C)` | *"fern #500 menugaskannya ke @agent10 … Saya DUKUNG pemindahan ini … **Saya yang usul (#327), agent10 yang bangun**"* |
| agent5 `#530` | *"**Saya tulis ulang spesifikasi Envelope** + SEC-08 menyerap SEMUA perbaikan agent10"* |
| agent5 `#578` | *"**Spec Envelope v2** + matriks gate QA saya akan serap semua ini"* |

#521 memindahkan ke saya; #530 dan #578 agent5 menulis spec-nya sendiri. **Keduanya terjadi setelah #521.** Ini bukan kesalahan siapa pun — ini konsekuensi wajar dari kepemilikan yang ditulis `(agent5/agent10)` tanpa pembagian.

### 3.2 Risiko konkretnya

Tim ini **sudah pernah** mengalami biaya dari dua kontrak yang berbeda untuk hal yang sama:

- matt, `KEPUTUSAN-YANG-DIBUTUHKAN.md` K-2: *"dua kontrak SpillStore yang berbeda adalah pola fork yang sama yang menyebabkan bencana 88-keputusan."*
- agent1 #445: registri `E-*` sempat **tidak sinkron** di dua dokumen dan butuh re-upload untuk memulihkan "satu sumber kebenaran".

Dua spec Envelope akan mengulang pola itu pada komponen yang **paling** tidak boleh bercabang: jalur kepercayaan audit.

### 3.3 Usulan pembagian (saya ajukan, fern/matt yang memutuskan)

| Lane | Pemilik | Artefak |
|---|---|---|
| **Konstruksi kriptografis** — encoding kanonik, keyed chain, checkpoint-root, anchor, disclosure tiers, anggaran memori, Item Lineage + crypto-shredding | **agent10** | `AGENT10-ENVELOPE-CHAIN-SPEC.md` (baru) |
| **Penegakan** — SEC-08 ditulis ulang, keluarga gate crypto/transient (`G-C1…G-C10`, `CASD-01`, `SEC-TRANSIENT`, `R1…R4`), matriks QA, differential testing | **agent5** | `AGENT5_QA_SECURITY_SPEC.md` (miliknya, sudah berjalan) |
| **Titik sambung** | keduanya | Envelope **tunduk** pada gate agent5; agent5 **tidak** mendefinisikan ulang konstruksi saya. Kalau ada sengketa → fern (`#462`, sudah disepakati) |

Ini persis pembagian `#462` yang sudah kami setujui (saya ranah **APA**, agent5 ranah **BAGAIMANA**) — hanya diterapkan ke artefak, bukan ke ranah abstrak. Saya **tidak** meminta lane agent5; SEC-08 dan matriks gate adalah miliknya dan ia sudah mengerjakannya lebih dulu.

---

## 4. P3-04 · MEDIUM · SEC-08 masih teks lama

`AGENT5_QA_SECURITY_SPEC.md` mtime **06:54:15**, baris 567–573, kriteria lulus masih:

> "(a) test: menghapus satu baris di tengah membuat verifikasi hash chain gagal; (b) akses kredensial selalu tercatat walau gagal; (c) log tidak memuat nilai kredensial."

agent5 di #530 sudah menyatakan: *"SEC-08 saya MENJANJIKAN tamper-evidence yang konstruksinya tidak punya … **SEC-08 WAJIB ditulis ulang**"*, dan di #530 butir 1 berkomitmen menambah kasus (b) *ubah-lalu-rekomputasi*. Jadi ini **bukan** ketidaksepakatan — hanya teks yang belum menyusul keputusan. Saya catat supaya PRD-3 tidak mengonsolidasikan spec yang penulisnya sendiri sudah menyatakan salah.

Tidak ada aksi untuk saya di sini; ini milik agent5. Saya hanya menandai statusnya.

---

## 5. P3-05 · MEDIUM · `<500KB` sebagai kriteria lulus padahal belum diukur

Gerbang L5: `I18N-40`: *"Bundle translasi 40 bahasa tersimpan rapi di SQLite FTS5 (**<500KB**) dengan provenance tag (`human`, `mt`, `glossary`)."*

**Yang saya akui sebagai kemenangan desain:** provenance tag itu **usul saya** di #567 butir (i) — terjemahan wajib menyatakan `human|mt|glossary` + model/kamus + versi, memakai pola `enc_key_id`+`enc_algo` (audit F20). Sudah masuk gerbang resmi dalam ~10 menit (#567 06:57 → PRD-3 07:01). Terima kasih @fern @agent7.

**Yang masih saya persoalkan:** angka `<500KB`.

- Sumber angka itu `#547` fern: *"Kamus istilah otomasi 40 bahasa terkompresi (<500KB total)"* — untuk **kamuss istilah**.
- Tapi `#547` juga mewajibkan *"deskripsi workflow, cara kerja, fungsi, tujuan, dan note node"* diterjemahkan. Itu **teks bebas** yang volumenya tumbuh dengan jumlah template, bukan kamus tetap.
- 500 KB / 40 bahasa = ~12,5 KB per bahasa **termasuk struktur**. Masuk akal untuk kamus; **tidak** untuk teks bebas × jumlah template × 40.

Ini kelas **F15** (klaim tanpa sumber, sudah diadjudikasi sebagai inkonsistensi-diri di PRD-1 §18) + kelas **ERR-029** (satuan tidak dipisah). Kalau `<500KB` jadi kriteria lulus, ia akan gagal pada template ke-sekian dan seseorang akan "memperbaikinya" dengan menaikkan angka — yaitu cara kriteria kehilangan makna.

**Usul:** pecah satuannya, dan tandai yang belum diukur:

```
I18N-40a  kamus istilah 40 bahasa            : <= 500 KB   (target desain, DIUKUR oleh agent8)
I18N-40b  teks deskripsi per template/bahasa : Y KB x jumlah template x 40  -> TIDAK diberi batas
          tetap; yang dibatasi adalah strategi muat (lazy-load per bahasa aktif, bukan seluruh bundle)
I18N-40c  provenance tag wajib per baris     : human | mt | glossary + model/kamus + versi   (sudah benar)
```

Yang penting bukan angkanya, tapi **memisahkan dua hal yang tumbuh dengan cara berbeda**. Pengukurannya tugas @agent8; menuntut satuannya dinyatakan tugas saya.

---

## 6. Dua catatan kecil (bukan temuan, supaya tidak ada yang tersandung)

1. **`task_queue` hidup di `/var/lib/agent-comm/comm.db`** (PRD-3 §4) — basis data yang sama dengan `msg`. Itu berarti **tugas agen bercampur dengan pesan agen** di satu DB yang riwayatnya pernah ber-mode 0666 (ERR-001) dan yang akses root-nya masih terbuka lewat catch-all sudoers (ERR-012/K-4). Konsekuensi praktis: status tugas **bukan** catatan yang tak terbantahkan. Untuk sekarang tidak masalah (belum ada yang bergantung padanya untuk kepatuhan), tapi kalau kelak `swarm-task complete --artifact` dipakai sebagai **bukti** penyelesaian gerbang, kita mengulang C-01 di lapisan proses. Usul satu kalimat di SOP: *status tugas adalah koordinasi, bukan bukti; bukti adalah artefak + checksum + gate yang bisa dijalankan ulang.*
2. **`swarm-task` membuat `comm.db` 0-byte di `$HOME` bila dijalankan tanpa path DB yang benar.** Ada satu di `/home/agent10/comm.db` (0 byte, 07:03) — kemungkinan besar dari pemanggilan `swarm-task list` saya sendiri di direktori home. Tidak berbahaya, tapi itu pola yang sama dengan ERR-002 (DB nyasar). Saya **tidak** menghapusnya tanpa izin karena itu home saya sendiri dan freeze melarang chmod/aksi sepihak; saya laporkan supaya pemilik CLI (@fern/@agent5) menambahkan guard.

---

## 7. Yang saya minta

| ID | Untuk | Permintaan |
|---|---|---|
| **P3-01** | @fern @matt | Koreksi baris 97 sebelum sign-off. **Ini yang memblokir.** |
| **P3-02** | @fern @matt | Masukkan Item Lineage (inovasi ke-12 + Wave 3 + `W3-ITEM-LINEAGE` + gerbang LIN-1..3), **atau** tuliskan keputusan sadar bahwa ia di luar cakupan |
| **P3-03** | @fern | Putuskan lane: konstruksi crypto = agent10, penegakan gate = agent5. Saya tidak meminta lane agent5 |
| **P3-04** | @agent5 | Tidak ada aksi dari saya; hanya penanda status bahwa teks SEC-08 belum menyusul #530 |
| **P3-05** | @fern @agent7 @agent8 | Pecah satuan `I18N-40` jadi a/b/c; angka `<500KB` diukur dulu sebelum jadi kriteria lulus |

Saya **bukan** approver PRD-3 — §6 menyebut pemilik proyek. Ini rekomendasi dari satu peran, dan kalau semuanya diterima tanpa ada yang membantah, itu kegagalan proses, bukan keberhasilan saya.

---

## 8. Reproduksi

Semua klaim di dokumen ini bisa dijalankan ulang read-only:

```bash
P=/opt/agent-workspace/docs/PRD-3-PERFECTION-CHECKLIST.md
sed -n '94,97p'  "$P"                                  # P3-01: label G-C6 di baris 97
grep -ic lineage "$P"; grep -ic GDPR "$P"               # P3-02: keduanya 0
sed -n '13p'     "$P"                                   # P3-02: daftar 11 inovasi
sed -n '59p'     "$P"                                   # P3-03: (agent5/agent10)
sed -n '567,573p' /opt/agent-workspace/docs/AGENT5_QA_SECURITY_SPEC.md   # P3-04
grep -n "I18N-40" "$P"                                  # P3-05
grep -n "G-C6" /opt/agent-workspace/docs/AGENT10-COMPLIANCE-CRYPTO-AUDIT.md  # definisi G-C6 asli
swarm-task list | grep -c '^W[0-9]-'                    # 15 tugas
```

— **agent10**, Enterprise Compliance & Cryptography Auditor (ROLE_COMPLIANCE)
