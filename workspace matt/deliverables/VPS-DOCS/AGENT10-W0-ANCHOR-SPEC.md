# W0-ANCHOR-SPEC — Anchor Eksternal untuk Root Hash-Chain (RFC 3161)

**Tugas:** `[R] W0-ANCHOR-SPEC` — PRD-3 v3.2 §4 Wave 0, pemilik **agent10** (ditetapkan matt, `e51bcc99…`)
**Penulis:** agent10 (ROLE_COMPLIANCE, sesi A) · **Tanggal:** 2026-09-09 · **Status:** DRAFT untuk review
**Dasar:** #646 butir 3 dan `AGENT10-AUDIT-ADDENDUM-A1-CORRECTION.md` — *keyed chain perlu tapi tidak cukup*
**Sifat:** dokumen desain + bukti eksekusi. **Nol kode produk** (freeze #386 dipatuhi).
**Wave 0 harus selesai sebelum Wave 1**, dan spec ini prasyarat bagi `W3-EXEC-ENVELOPE` + `W3-ITEM-LINEAGE`.

---

## 0. Ringkasan eksekutif (5 kalimat)

1. Anchor eksternal **layak dan sudah diuji berhasil** dari host ini: TSA RFC 3161 publik menjawab `Status: Granted` dan verifikasi lolos terhadap payload asli serta **gagal** terhadap payload yang diubah.
2. Anchor yang dipilih **tidak memerlukan kredensial apa pun** — ini properti terpenting, karena ia menghapus masalah yang membunuh keyed chain: tidak ada rahasia di host yang bisa dicuri penyerang ber-root.
3. Seluruh jaminan menyusut menjadi **satu hal: fingerprint 32-byte sertifikat CA TSA**, dan fingerprint itu **wajib berada di luar jangkauan root host** (diserahkan ke Pemilik Proyek, tertanam di verifier).
4. Dibuktikan secara adversarial: CA palsu dengan *Distinguished Name yang identik* menghasilkan TSR yang tak terbedakan mata — ditolak oleh pin, **diterima** bila trust anchor dikuasai penyerang. Nama/subject **tidak bernilai** sebagai bukti; hanya fingerprint.
5. Tanpa anchor ini, klaim produk maksimum yang jujur adalah `tamper-evident` dalam jendela tak-terjangkar; **C-01 (CRITICAL) tetap terbuka** dan tidak boleh dinyatakan ditutup.

---

## 1. Masalah yang ditutup spec ini

`G-C1b` terukur **GAGAL**: penyerang yang mengubah satu entri lalu **merekomputasi rantai + root** lolos verifikasi internal, karena verifier hanya punya data yang juga dikuasai penyerang. `keyed chain` menutup celah itu **hanya jika `chain_key` terlindungi**, dan di mesin ini tidak:

| Fakta terukur | Sumber |
|---|---|
| `agent-team ALL=(ALL) NOPASSWD:ALL` untuk 11 akun; bahkan `nobody` dapat root | ERR-012 / K-4, PRD-3 v3.2 §8 |
| Kunci dari `env`/file di host yang sama "memberi perlindungan nol terhadap penyerang ber-root" | PRD-2 §11.1 |
| Tidak ada mount read-only/WORM: `/` = `ext4 rw`; `chattr` butuh root | probe 2026-09-09 07:43 |

**Kesimpulan:** lapisan yang menahan penyerang ber-root hanya lapisan yang **tidak bisa ia tulis**. Itu definisi anchor eksternal.

---

## 2. Model ancaman

| Kelas | Kemampuan | Ditahan oleh |
|---|---|---|
| **A1** | Tulis DB/file lokal, **tanpa** root | keyed chain (A.1) + append-only |
| **A2** | **root penuh di host** (K-4) — baca `chain_key` dari memori/env, tulis ulang DB | **anchor eksternal (A.2)** — root yang sudah dipublikasikan ke TSA tidak bisa ditarik |
| **A3** | A2 **+** memasang CA sendiri ke `/usr/local/share/ca-certificates` lalu MITM | **pin CA + verifikasi `-CAfile`**, bukan trust store sistem |
| **A4** | A3 **+** mengganti berkas pin di host | **fingerprint dipublikasikan out-of-band ke Pemilik Proyek**; verifier menolak pin yang fingerprintnya tidak cocok |
| **A5** | Mengendalikan/membayar TSA, atau TSA berkolusi | **kuorum N-of-M TSA independen** (§7) + `ordering=yes` + nonce |

Batas jujur spec ini: **A4 adalah batas akhir.** Bila penyerang menguasai host *dan* Pemilik Proyek tidak pernah menerima fingerprint secara out-of-band, tidak ada mekanisme di dalam host yang bisa menyelamatkannya. Ini bukan kekurangan implementasi — ini sifat masalahnya: kepercayaan harus berakar di suatu tempat di luar jangkauan penyerang.

---

## 3. Bukti eksekusi (feasibility, terukur — bukan asumsi)

Lingkungan: master (Ubuntu 24.04.4), `openssl` 3.x sistem, `curl`. Workdir bukti: `/mnt/extra-storage/a10-work/anchor-probe/` (vdb — **wajib**, sebab `vda1` 88% penuh, sisa 2,3 G).

### 3.1 Egress
| Tujuan | Hasil | Arti |
|---|---|---|
| `https://api.github.com` | `200` | egress HTTPS keluar **ada** |
| `https://dns.google/resolve` | `200` | resolusi + HTTPS OK |
| `https://freetsa.org/tsr` (GET) | `403` | wajar: endpoint hanya menerima POST |
| `https://tsa.certum.pl` | timeout 12 s | tidak dapat diandalkan dari host ini |
| `http://timestamp.comodoca.com` (POST) | `200`, `Status: Granted`, signer **Sectigo Public Time Stamping Signer R37** | TSA komersial terjangkau |

### 3.2 Anchor nyata (TSA publik, TANPA kredensial)
```
payload.txt (104 B) sha256 = e17ef9d3db0bc0214321f8d98535d3282461f7769952a508cdf0c22b4dc38bbb
POST https://freetsa.org/tsr  -> HTTP 200, respon 4.633 byte
  Status        : Granted.
  Hash Algorithm: sha256
  Serial number : 0x07C1DB3F
  Time stamp    : Sep  9 07:43:20 2026 GMT
  Ordering      : yes
  TSA           : /O=Free TSA/OU=TSA/CN=www.freetsa.org/L=Wuerzburg/C=DE
resp-proof.tsr sha256 = ac4bf20bf45d293b29140f0b3c46e6691d4bbee884d88a71abf4b38ee91ce18d
```

### 3.3 Diskriminasi (sifat yang menentukan gate)
| Uji | Perintah | Hasil |
|---|---|---|
| Payload **asli** vs TSR asli, pin CA asli | `openssl ts -verify -data payload.txt -in resp-proof.tsr -CAfile freetsa-ca.pem` | **Verification: OK** |
| Payload **diubah** vs TSR asli | `-data fake.txt` | **FAILED — `message imprint mismatch`** |

### 3.4 Uji adversarial A3/A4 — **temuan terpenting spec ini**
Saya menerbitkan CA sendiri dengan **DN yang identik karakter-per-karakter** dengan CA Free TSA, plus `extendedKeyUsage=critical,timeStamping`, lalu membuat TSA lokal dan menandatangani root palsu:

```
subject CA palsu : O=Free TSA, OU=Root CA, CN=www.freetsa.org, emailAddress=busilezas@gmail.com, L=Wuerzburg, ST=Bayern, C=DE
subject CA asli  : O=Free TSA, OU=Root CA, CN=www.freetsa.org, emailAddress=busilezas@gmail.com, L=Wuerzburg, ST=Bayern, C=DE   <- IDENTIK
fingerprint palsu: CD:2F:46:6B:69:E5:38:9B:CE:D3:65:4C:38:A8:33:16:78:B3:09:FA:73:B5:96:44:10:6B:E2:9E:7A:22:64:57
fingerprint asli : A6:37:9E:7C:EC:C0:5F:AA:3C:BF:07:60:13:D7:45:E3:27:BB:BA:A3:8C:0B:9A:F2:24:69:D4:70:1D:18:AA:BC
TSR palsu        : Status: Granted. / Time stamp: Sep 9 07:45:46 2026 GMT / TSA: <DN identik>
evil.tsr sha256  = ba4c077cda2ab57d86a530de8d111e72285f107db9434f3e2369a8d1964d87df
```
| Uji | Hasil | Kesimpulan |
|---|---|---|
| **E1** TSR palsu vs **pin CA asli** | **FAILED — `self-signed certificate`** | pin menolak TSA palsu ✔ |
| **E2** TSR palsu vs **CA penyerang** | **Verification: OK** | bila trust anchor dikuasai penyerang, pemalsuan **lolos sempurna** |
| **E3** payload asli vs TSR palsu | **FAILED — `message imprint mismatch`** | TSR palsu tidak bisa menyamar sebagai bukti untuk data lain ✔ |

**Konsekuensi mengikat untuk implementasi:**
1. Verifier **DILARANG** memakai trust store sistem (`/etc/ssl/certs`). Wajib `-CAfile <pin>` eksplisit. Alasannya terukur di E2: penyerang ber-root dapat `update-ca-certificates` dan setiap verifikasi berbasis trust store akan menerima TSA palsunya.
2. **DN/subject/CN tidak boleh dipakai untuk identifikasi TSA.** DN identik berhasil dipalsukan. Satu-satunya pengenal yang sah = **fingerprint SHA-256 sertifikat**.
3. Berkas pin di host **bukan** akar kepercayaan. Akarnya = fingerprint yang diserahkan ke Pemilik Proyek secara out-of-band (§8).

### 3.5 Yang TIDAK feasible / ditolak
| Opsi | Status | Alasan |
|---|---|---|
| Mount WORM lokal, `chattr +i` | **tidak tersedia** | butuh root; semua agen root ⇒ tidak menambah jaminan |
| Repo git + signed commit ke remote | **belum ada** | `/opt/agent-workspace` bukan repo git; butuh token ⇒ **re-introduksi rahasia di host** (kalah pada §2 A2). Bisa jadi pelengkap, bukan anchor utama |
| OpenTimestamps (anchor ke Bitcoin) | **belum teruji** | `ots` tidak terpasang, modul Python tidak ada, `a.pool.opentimestamps.org/info` → `404`. Menarik karena tanpa kredensial & terdesentralisasi — diusulkan sebagai evaluasi terpisah (D-A4), **tidak saya klaim layak** |
| Verifikasi TSR Sectigo via trust store | **GAGAL** | `unable to get local issuer certificate` — chain tidak lengkap di respon; butuh unduh intermediate. Catatan: TSA komersial tetap perlu pin, hanya rantai-nya lebih panjang |

---

## 4. Rekaman anchor (anchor record)

Satu baris per peristiwa anchoring, **append-only**, disimpan di vdb.

| Field | Tipe | Isi |
|---|---|---|
| `anchor_id` | u64 | monoton, dari sequence |
| `chain_id` | text | mis. `exec-envelope/v1` |
| `chain_head` | blob(32) | root rantai saat anchoring |
| `prev_anchor_head` | blob(32) | `chain_head` anchor sebelumnya (membuat **log anchor sendiri berantai**) |
| `entry_from`,`entry_to` | u64 | rentang entri yang dijangkau |
| `entry_count` | u64 | jumlah entri |
| `imprint_algo` | text | `sha256` (wajib; jangan `sha1`) |
| `nonce` | blob(8) | **wajib diisi**, acak kriptografis |
| `tsa_id` | text | pengenal TSA (URL) |
| `tsa_cert_fp` | text | fingerprint SHA-256 CA TSA yang dipakai |
| `tsa_serial` | text | serial dari TSR |
| `tsa_time` | text | waktu yang dinyatakan TSA (UTC, ISO-8601) |
| `tsr_bytes` | blob | TSR DER mentah (bukti primer — ~3–7 KB) |
| `tsr_sha256` | blob(32) | hash TSR untuk indeks |
| `quorum` | text | mis. `2-of-3`; berapa TSA yang menjawab |
| `status` | enum | `ANCHORED` / `PARTIAL` / `FAILED` |

**Isi yang di-anchor = root hash SAJA.** Payload TSA adalah hash dari `{chain_id, chain_head, prev_anchor_head, entry_from, entry_to, entry_count, local_ts}` — **tidak pernah** isi item, prompt, atau data subjek.

Konsekuensi kepatuhan (penting, dan menguntungkan):
- Log anchor **tidak mengandung data pribadi** ⇒ tidak bersinggungan dengan GDPR Art. 17, tidak butuh `salt_exec`, boleh disimpan tanpa batas dan **boleh dipublikasikan**.
- Ini memisahkan dua masalah yang sering tercampur: *retensi bukti* (abadi, publik) vs *retensi konten* (terbatas, bisa di-shred). Lihat `AGENT10-ITEM-LINEAGE-SPEC.md` §6.

---

## 5. Protokol

### 5.1 Membuat anchor
1. Hitung `chain_head` saat ini; baca `prev_anchor_head` dari anchor terakhir.
2. Bangun payload kanonik (§4) → `imprint = SHA-256(payload)`.
3. `openssl ts -query -data <payload> -cert -sha256 -out req.tsq` — **dengan nonce** (tanpa `-no_nonce`). Terukur: req dengan nonce 70 B vs tanpa 59 B; `Nonce: 0xD6BF9FF70B4FC129`.
4. POST ke tiap TSA dalam daftar (§7), timeout 25 s, simpan TSR.
5. **Verifikasi sebelum mencatat** (§5.2). Hanya TSR yang lolos yang boleh menghasilkan baris `ANCHORED`.
6. Tulis baris ke `anchor_log` dalam satu transaksi; append `tsr` ke berkas bukti.

### 5.2 Verifikasi (semua butir WAJIB; gagal satu = gagal)
| # | Pemeriksaan | Alasan |
|---|---|---|
| V1 | `openssl ts -verify -data <payload> -in <tsr> -CAfile <pin>` → `OK` | integritas + tanda tangan |
| V2 | `Status: Granted.` | TSA menolak/tidak memproses |
| V3 | `Hash Algorithm: sha256` | cegah downgrade ke SHA-1 |
| V4 | `tsa_cert_fp` == fingerprint pin yang diharapkan | **menutup A3/A4** (§3.4 E2) |
| V5 | nonce di TSR == nonce yang kita kirim | cegah TSR pra-hitung/diulang |
| V6 | `Ordering: yes` | urutan waktu dapat dipercaya relatif |
| V7 | `tsa_time` dalam toleransi jam (mis. ±10 menit dari jam lokal) | deteksi TSA rusak/berkolusi |
| V8 | `entry_from` == `entry_to` anchor sebelumnya + 1 | tidak ada celah/loncatan rentang |

### 5.3 Verifier sisi-audit
Verifikasi ulang independen **dari artefak saja** (`anchor_log` + berkas TSR), tanpa koneksi jaringan, kecuali untuk mengunduh CA bila pin tidak tersedia lokal. Verifier **wajib** menampilkan status per entri: `ANCHORED` / `UNANCHORED` / `ANCHOR-MISMATCH`.

---

## 6. Cadence, jendela, dan degradasi jujur

| Parameter | Usul | Catatan |
|---|---|---|
| Interval anchoring | tiap **N = 1.000 entri** atau **60 detik**, mana lebih dulu | N menentukan jendela tak-terjangkar |
| Alarm | bila `UNANCHORED` > **2N** atau TSA gagal **3×** berturut-turut | kegagalan anchor **tidak boleh senyap** — sama dengan prinsip fail-open-informatif agent9 #654 |
| Retry | eksponensial 5/15/60 s, lalu tandai `FAILED` dan lanjut | jangan blokir eksekusi workflow karena TSA lambat |

**Aturan klaim (mengikat, bukan anjuran):**
- Entri dalam jendela sejak anchor terakhir = **`UNANCHORED`**. Verifier **DILARANG** menyebutnya "terverifikasi".
- Kalimat produk yang diizinkan: *"tamper-evident terhadap penyerang yang tidak mengendalikan sink anchor maupun akar kepercayaan verifier, dengan jendela tak-terjangkar ≤ N entri."*
- Kalimat yang **dilarang**: "tamper-proof", "tidak bisa dipalsukan", " immutable secara kriptografis".

**Keputusan yang belum diambil (D-A1):** apakah kegagalan anchor yang berkepanjangan harus **menghentikan** eksekusi (fail-closed) atau hanya melabeli `UNANCHORED` (fail-open-informatif). Usul saya: **fail-open untuk operasi, fail-closed untuk klaim** — mesin tetap jalan, tetapi laporan kepatuhan berhenti menyebut riwayat itu terverifikasi, dan alarm naik ke pemilik. Menghentikan produksi karena TSA hobi mati adalah keputusan bisnis, bukan keamanan, jadi milik Pemilik Proyek.

---

## 7. Kuorum TSA (menutup A5)

Free TSA adalah layanan hobi satu operator: risiko ketersediaan, kelangsungan, dan kolusi. Aturan:

- **Minimum 2 TSA independen**, kuorum **N-of-M** dengan M ≥ 3 bila tersedia. `status = ANCHORED` hanya bila kuorum tercapai; di bawah itu `PARTIAL` (tetap dicatat, tetap dihitung untuk jendela, tapi **tidak** boleh diklaim penuh).
- Daftar awal yang teruji terjangkau dari host ini: `https://freetsa.org/tsr` (tanpa kredensial, **teruji OK**) dan `http://timestamp.comodoca.com` (Sectigo, **teruji Granted**, tapi verifikasi butuh chain lengkap + **wajib** dinaikkan ke HTTPS bila tersedia — HTTP polos membuka A3 tanpa perlu root).
- **Produksi:** bila bobot hukum dibutuhkan (eIDAS *qualified timestamp*, sengketa, SOC2 Type II), TSA hobi **tidak mencukupi**. Butuh TSA terkualifikasi berbayar. Ini keterbatasan jujur, bukan cacat desain — dan biayanya keputusan pemilik (D-A3).
- Setiap TSA punya pin fingerprint sendiri; pin disimpan sebagai daftar, dan **penambahan TSA baru = peristiwa kepercayaan** yang butuh otorisasi manusia (pola yang sama dengan `re-pin` katalog Hub dan `erasure_log` lineage).

---

## 8. Bootstrap akar kepercayaan (masalah yang tidak bisa dihindari)

Pin di host bisa diganti root (A4). Maka:

1. **Serahkan fingerprint CA TSA ke Pemilik Proyek secara out-of-band** (pesan tertandatangani / dokumen sign-off), bersama: URL TSA, `notBefore`/`notAfter`, fingerprint SHA-256, dan tanggal penyerahan.
2. Fingerprint itu **masuk ke dokumen yang ditandatangani pemilik** (PRD-3 atau lampirannya). Setelah itu, mengganti pin di host tidak membantu penyerang: verifier membandingkan pin lokal terhadap fingerprint yang tercatat di dokumen sign-off.
3. **Anchor-kan log anchor itu sendiri**: `prev_anchor_head` membuat `anchor_log` berantai, dan tiap `anchor_id` ikut masuk ke rantai Envelope berikutnya (referensi silang). Memalsukan riwayat anchor berarti memalsukan TSR yang sudah dipegang pihak ketiga.
4. **Rotasi pin** (CA TSA kedaluwarsa/ganti) = peristiwa pemutusan kepercayaan: wajib dicatat `pin_change_log` dengan fingerprint lama, baru, alasan, dan identitas otorisator manusia. Tanpa ini, penyerang yang mengganti pin bisa berdalih "rotasi".

Fingerprint CA Free TSA yang terukur hari ini (untuk diserahkan, **bukan** untuk dipercaya dari dokumen ini saja — verifikasi ulang sendiri):
```
A6:37:9E:7C:EC:C0:5F:AA:3C:BF:07:60:13:D7:45:E3:27:BB:BA:A3:8C:0B:9A:F2:24:69:D4:70:1D:18:AA:BC
subject : O=Free TSA, OU=Root CA, CN=www.freetsa.org, L=Wuerzburg, ST=Bayern, C=DE
valid   : 2016-03-13 .. 2041-03-07
sha256(freetsa-ca.pem) = 2151b61137ffa86bf664691ba67e7da0b19f98c758e3d228d5d8ebf27e044438
```

---

## 9. Kontrak penyimpanan (untuk @agent2 — W2-STORAGE-L0)

Log anchor **wajib di vdb** (`/mnt/extra-storage`), bukan `vda1` (88% penuh, sisa 2,3 G; TSR ~3–7 KB per anchor ⇒ pada 1 anchor/menit ≈ 10 MB/hari).

```sql
CREATE TABLE anchor_log (
  anchor_id        INTEGER PRIMARY KEY,
  chain_id         TEXT    NOT NULL,
  chain_head       BLOB    NOT NULL CHECK (length(chain_head) = 32),
  prev_anchor_head BLOB    NOT NULL CHECK (length(prev_anchor_head) = 32),
  entry_from       INTEGER NOT NULL,
  entry_to         INTEGER NOT NULL,
  entry_count      INTEGER NOT NULL,
  imprint_algo     TEXT    NOT NULL DEFAULT 'sha256',
  nonce            BLOB    NOT NULL CHECK (length(nonce) >= 8),
  tsa_id           TEXT    NOT NULL,
  tsa_cert_fp      TEXT    NOT NULL,
  tsa_serial       TEXT    NOT NULL,
  tsa_time         TEXT    NOT NULL,
  tsr_sha256       BLOB    NOT NULL CHECK (length(tsr_sha256) = 32),
  tsr_path         TEXT    NOT NULL,
  quorum           TEXT    NOT NULL,
  status           TEXT    NOT NULL CHECK (status IN ('ANCHORED','PARTIAL','FAILED')),
  created_utc      TEXT    NOT NULL
);
CREATE TABLE pin_change_log (
  change_id     INTEGER PRIMARY KEY,
  tsa_id        TEXT NOT NULL,
  old_fp        TEXT,
  new_fp        TEXT NOT NULL,
  reason        TEXT NOT NULL,
  authorized_by TEXT NOT NULL,     -- manusia, bukan agen
  changed_utc   TEXT NOT NULL
);
```
`tsr_path` menunjuk berkas bukti (bukan BLOB inline) supaya TSR bisa diserahkan ke auditor apa adanya. **Tidak ada UPDATE/DELETE** pada `anchor_log`; koreksi = baris baru.

---

## 10. Gerbang (falsifiable, semuanya bisa dijalankan)

| Gate | Uji | Lulus bila |
|---|---|---|
| **ANC-1** | POST TSR ke TSA terdaftar, tanpa kredensial | `HTTP 200` + `Status: Granted.` |
| **ANC-2** | Verifikasi payload asli dengan pin | `Verification: OK` |
| **ANC-3** | Verifikasi payload yang diubah 1 byte | `FAILED` + `message imprint mismatch` |
| **ANC-4** | Verifikasi TSR dari CA **tidak** di-pin | `FAILED` (bukti: §3.4 E1) |
| **ANC-5** | Verifikasi **tanpa** `-CAfile` (trust store sistem) setelah CA penyerang dipasang | **harus ditolak oleh kebijakan**, bukan oleh crypto — uji bahwa verifier menolak berjalan tanpa pin eksplisit |
| **ANC-6** | Replay TSR lama untuk `chain_head` baru | `FAILED` (nonce/imprint tidak cocok) |
| **ANC-7** | TSA mati (timeout) | baris `status=FAILED`, alarm naik, jendela `UNANCHORED` bertambah, **eksekusi tidak berhenti**, klaim diturunkan |

**ANC-5 adalah gate yang paling sering dilupakan** dan justru yang membedakan desain ini dari "sudah pakai TSA". Sudah teruji di §3.4 E2 bahwa tanpa pin eksplisit, pemalsuan lolos sempurna.

---

## 11. Keputusan yang dibutuhkan

| ID | Pertanyaan | Usul saya | Pemutus |
|---|---|---|---|
| **D-A1** | Fail-closed (hentikan eksekusi) atau fail-open-informatif saat TSA mati? | fail-open untuk operasi, **fail-closed untuk klaim** + alarm | Pemilik Proyek |
| **D-A2** | N (jendela) = 1.000 entri / 60 detik? | ya untuk awal; ukur ulang setelah volume nyata | fern + agent1 |
| **D-A3** | TSA hobi cukup untuk MVP, atau butuh TSA terkualifikasi (berbayar, eIDAS)? | cukup untuk MVP **dengan keterbatasan tertulis**; produksi butuh TSA terkualifikasi bila ada bobot hukum | Pemilik Proyek |
| **D-A4** | Evaluasi OpenTimestamps sebagai TSA terdesentralisasi tanpa kredensial? | ya, tugas terpisah; **belum saya klaim layak** (belum teruji) | fern |
| **D-A5** | Siapa pemegang fingerprint out-of-band & di dokumen mana dicatat? | Pemilik Proyek, di lampiran sign-off PRD-3 | Pemilik Proyek + matt |
| **D-A6** | Anchor dipakai juga untuk **katalog Hub** (pin `expected_sha256` agent9/agent4) dan **manifest**? | ya — lihat butir veto/endorse saya di #6xx; pin di katalog yang root-writable = TOFU bernama lain | agent4 + agent9 + matt |

---

## 12. Yang spec ini TIDAK lakukan (batas jujur)

1. **Tidak** melindungi terhadap penyerang yang menguasai host **dan** Pemilik Proyek sekaligus (A4 adalah batas akhir).
2. **Tidak** membuat data "immutable". Yang immutable hanyalah *pernyataan bahwa root X ada pada waktu T*, dipegang pihak ketiga. Isi tetap bisa diubah di host; yang berubah adalah **ubahannya terdeteksi**.
3. **Tidak** memberi kepastian waktu absolut — `tsa_time` berasal dari TSA; akurasi ±1 s dinyatakan TSA, dan kolusi TSA tidak bisa dideteksi dari satu TSA saja (karena itu kuorum).
4. **Tidak** menyelesaikan `G-C5`/`G-C7` (checksum testkit) — itu `W0-HASH-FIX`/`W0-CHECKSUM-AGREE` milik agent3/agent2.
5. **Tidak** menggantikan keyed chain. Keduanya berlapis: keyed menaikkan biaya bagi A1, anchor menahan A2–A4.

---

## 13. Jejak

| Artefak | Nilai |
|---|---|
| Bukti mentah | `/mnt/extra-storage/a10-work/anchor-probe/` (`payload.txt`, `req.tsq`, `resp-proof.tsr`, `freetsa-ca.pem`, `comodoca.tsr`, `evil/`) |
| `resp-proof.tsr` | `ac4bf20bf45d293b29140f0b3c46e6691d4bbee884d88a71abf4b38ee91ce18d` |
| `freetsa-ca.pem` | `2151b61137ffa86bf664691ba67e7da0b19f98c758e3d228d5d8ebf27e044438` |
| `evil/evil.tsr` | `ba4c077cda2ab57d86a530de8d111e72285f107db9434f3e2369a8d1964d87df` |
| Berkaitan | `AGENT10-AUDIT-ADDENDUM-A1-CORRECTION.md`, `AGENT10-EXEC-ENVELOPE-SPEC.md` §3, `AGENT10-ITEM-LINEAGE-SPEC.md` §6, PRD-3 v3.2 §5 L2c |

---

# ADDENDUM v1.1 (08:4x) — hasil verifikasi independen implementasi + 2 perbaikan spec

Ditambahkan oleh agent10 setelah `W0-ANCHOR-IMPL` (agent1) terbit dan saya verifikasi sendiri.
Teks §1–§13 di atas **tidak diubah**; addendum ini yang mengoreksi.

## A. Verifikasi independen: 14/14 gate LULUS (direproduksi oleh pihak ketiga)

Implementasi: `/opt/agent-workspace/w0-anchor-impl/` (agent1, 08:40). Saya build sendiri dari sumber
dengan `CARGO_TARGET_DIR` saya di vdb (`--locked`, tidak menulis ke pohon penulis) lalu menjalankan
`tests/anc_gates.sh` terhadap TSA live:

```
biner  : /mnt/extra-storage/agent10-cargo-target/release/anchor  (2.693.760 B)
sha256 : bfceef8269cd1a3fc328938bea16477022e0447ce53e2f3510930aa78e9aebc0
build  : cargo build --release --locked -> Finished in 1m 52s
hasil  : RESULT: 14 passed, 0 failed   (workdir /tmp/anc-test-wISatN)
fp CA  : a6379e7cecc05faa3cbf076013d745e327bbbaa38c0b9af22469d4701d18aabc  == catatan §8 saya
```

| Gate | Hasil | Catatan |
|---|---|---|
| ANC-1 / 1b | PASS | `status=ANCHORED quorum=1-of-1 chain=TEST [0..99]`, 1 baris |
| ANC-2 | PASS | verifikasi ulang offline → `ANCHORED (re-verified)` |
| ANC-3 | PASS | TSR diubah → `ANCHOR-MISMATCH: tsr_sha256 mismatch` |
| ANC-3b | PASS | `chain_head` di DB diubah → `ANCHOR-MISMATCH` |
| ANC-4 / 4a | PASS | TSR palsu dari CA lokal: **OK vs CA palsu** (sanity, anti-vacuous), **FAILED vs pin FreeTSA** |
| ANC-5 / 5b | PASS | tanpa `--pin`: exit **2** + `POLICY REFUSAL` |
| ANC-6 | PASS | TSR lama diklaim untuk jendela baru → `ANCHOR-MISMATCH: message imprint mismatch` |
| V8 | PASS | celah rentang (`entry_from` != `entry_to` sebelumnya + 1) → exit 2 saat create |
| ANC-7 / 7b / 7c | PASS | TSA mati → `status=FAILED`, baris FAILED, `ALARM … UNANCHORED (claim downgraded, execution continues)`, **exit 0** |

**Pilihan desain yang saya ENDORSE** (semuanya lebih ketat atau lebih jujur dari spec saya):
1. **Nol kode crypto baru** — ASN.1/tanda-tangan/verifikasi diserahkan ke `openssl ts`, transport ke `curl`; crate hanya mengorkestrasi + menegakkan kebijakan. Ini memperkecil permukaan audit secara drastis.
2. **Kuorum MVP = M-of-M** (lebih ketat dari N-of-M usulan saya) → hanya bisa *under-claim*, tidak pernah *over-claim*. Relaksasi harus eksplisit via `--quorum K`.
3. **Verify-before-record** — TSR yang gagal V1–V7 tidak pernah dicatat `ANCHORED`.
4. **ANC-5 ditegakkan sebagai kebijakan**, bukan hanya crypto: verifier *menolak berjalan* tanpa pin eksplisit. Ini persis gate yang saya sebut paling sering dilupakan (§10).
5. **D-A1 default = fail-OPEN operasi / fail-CLOSED klaim** — usulan saya di §11 diadopsi sebagai default sementara pemilik belum memutuskan, dan ANC-7 membuktikan klaim benar-benar diturunkan (`claim downgraded`) sementara eksekusi lanjut.
6. Kata klaim terlarang tidak pernah dicetak; status persis `ANCHORED/PARTIAL/FAILED/UNANCHORED/ANCHOR-MISMATCH`.

## B. PERBAIKAN SPEC 1 — celah di DDL §9 saya (ditemukan agent1, saya akui)

Payload (§4) membutuhkan `local_ts`, tetapi DDL §9 saya **tidak punya kolom itu**. Implementasi
menyelesaikannya dengan menetapkan `local_ts == created_utc` (satu pembacaan jam), sehingga payload
tetap dapat direkonstruksi dari `anchor_log` saja — dan itu memang syarat verifikasi ulang offline
(§5.3). **Keputusan ini saya sahkan sebagai bagian spec.** §9 dianggap berbunyi: `created_utc`
merangkap `local_ts` payload; tidak ada pembacaan jam kedua.

## C. PERBAIKAN SPEC 2 — `pin_change_log` masih skema saja; bootstrap A4/D-A5 belum punya titik penegakan

Terukur di `src/main.rs`: tabel `pin_change_log` **dibuat** (satu-satunya kemunculan nama itu adalah
`CREATE TABLE`), **tidak pernah di-INSERT**. Hanya `anchor_log` yang ditulis (2 lokasi INSERT).
Akibatnya, rantai kepercayaan pin saat ini bergantung pada **disiplin operator**, bukan pada rekaman:

1. `--pin` boleh berupa **path ke PEM**, dan bila demikian tool **menurunkan** fingerprint dari PEM itu
   (`pin_fingerprint`, baris ~359). Di `anc_gates.sh`, PEM diambil dari
   `https://freetsa.org/files/cacert.pem` saat uji berjalan.
2. Artinya **bootstrap bersifat TOFU**: pihak yang mengendalikan jaringan/DNS/host pada pengambilan
   pertama yang menentukan pin, dan tool akan menghitung fingerprint dari PEM apa pun yang disodorkan.
3. V4 (`tsa_cert_fp` == pin) tetap benar dan berguna — ia menutup A3 (CA palsu ditolak, terbukti di
   ANC-4). Yang belum tertutup adalah **A4**: tidak ada konstanta di luar host yang dibandingkan.

**Wajib ditambahkan (menjadi bagian spec, gate ANC-8/ANC-9):**
- **Seeding bootstrap:** pada pemakaian pertama, tulis baris `pin_change_log`
  `(change_id=1, tsa_id, old_fp=NULL, new_fp=<fingerprint yang dipublikasikan pemilik>, reason='bootstrap', authorized_by=<manusia>)`.
- **Penegakan:** `verify` harus membandingkan pin yang dipakai dengan `new_fp` baris **terakhir**
  `pin_change_log`; bila berbeda → `PIN-MISMATCH`, exit 2, alarm. Dengan ini mengganti pin di host
  tidak lagi cukup: perubahan pin menjadi **peristiwa tercatat, append-only, dan teratribusi ke manusia**.
- **Opsi `--expected-fp <64hex>`**: konstanta fingerprint yang dipublikasikan pemilik (§8) dimasukan
  sebagai argumen/berkas konfigurasi terpisah; bila pin yang dimuat != `--expected-fp` → tolak.
  Ini titik penegakan D-A5 yang bisa diuji di CI.
- **ANC-8** (baru): `verify` dengan pin yang tidak cocok dengan baris terakhir `pin_change_log` → exit 2 + `PIN-MISMATCH`.
- **ANC-9** (baru): pin berubah tanpa baris `pin_change_log` baru → `verify` menolak.

Selama D-A5 belum dijawab pemilik, status jujur implementasi ini: **A1–A3 tertutup dan teruji;
A4 terbuka** (bergantung pada perbandingan manual terhadap fingerprint di §8).

## D. Penempatan (C-5) — masih terbuka

Crate berada di `/opt/agent-workspace/w0-anchor-impl/`, **belum** di pohon kanonik
(`kernel-asli-d3bcff0`, git bersih di `a7d0357`). agent1 sendiri menandai ini dan mengusulkan
`crates/anchor/`. Saya dukung usulan itu, dan menambahkan syarat: **masuk git dengan commit**, karena
saat ini satu-satunya rekaman perubahan kode di proyek ini adalah mtime (lihat #824 butir 2b).
Keputusan akhir di @matt / @fern.
