# AGENT10 — SPESIFIKASI ITEM LINEAGE & CRYPTO-SHREDDING
## Pilar ke-12 yang hilang dari PRD-3 (P3-02) — deliverable `W3-ITEM-LINEAGE`

| | |
|---|---|
| **Versi** | v1.0 DRAFT (untuk review adversarial) |
| **Penulis** | agent10 — ROLE_COMPLIANCE |
| **Tanggal** | 2026-09-09, ~07:1x UTC |
| **Mandat** | fern `#500` butir 11 (*"Item Lineage (GDPR audit)"*), fern `#400` butir 2 (**DITERIMA untuk masuk katalog fitur PRD-3**), `SISTEM-DISTRIBUSI-TUGAS-OTOMATIS.md` §2 (*"Lisensi, PII \| Crypto-shredding"*), handover agent5 `#521(C)` |
| **Dasar desain** | agent1 `#365(A)` (proposal awal) + revisi agent1 `#522` (menyerap 4 kritik saya) + `AGENT10-COMPLIANCE-CRYPTO-AUDIT.md` C-06/C-08 |
| **Status kernel** | `PairedItem { item_id: u32, input_node_index: Option<u8> }` di `kernel/src/item.rs:179-189` — **posisional**, dan ini yang harus diganti kontraknya (bukan sekarang: freeze #386) |
| **Aturan** | Zero-Code Mandate dipatuhi: dokumen ini **bukan kode**. DDL di §7 adalah **kontrak kolom** untuk pemilik storage (agent2); implementasi menunggu sign-off. |
| **Prasyarat** | **C-06 harus beres lebih dulu** (`ContentId` = `u64` sekuensial, bukan content hash). Lineage yang berjangkar pada id sekuensial tidak bisa diverifikasi. Urutan: `BlobId`/`ContentHash` dipisah → baru spec ini bisa diimplementasikan. |

---

## 0. Mengapa dokumen ini ada

`PRD-3-PERFECTION-CHECKLIST.md` v3.0.0-FINAL **tidak menyebut lineage sama sekali**:

```
$ grep -ic <kata> PRD-3-PERFECTION-CHECKLIST.md
lineage 0 · Lineage 0 · GDPR 0 · crypto-shred 0 · shredding 0 · PII 0 · retensi 0 · lisensi 0 · pseudonim 0
```

`task_queue` = 15 tugas, satu-satunya `ROLE_COMPLIANCE` = `W3-EXEC-ENVELOPE`. Tidak ada tugas lineage.

Padahal supervisor menerimanya di `#400`, menugaskannya di `#500`, matriks SOP mencantumkan
"crypto-shredding" sebagai domain saya, dan agent1 sudah merevisi desainnya di `#522`. Dokumen ini
menutup lubang itu supaya P3-02 punya isi, bukan hanya keluhan. Rincian temuan: `AGENT10-PRD3-COMPLIANCE-REVIEW.md` §2.

---

## 1. Masalah: referensi berbasis posisi tidak bisa diaudit

### 1.1 Fakta di kode

```rust
// kernel-asli-d3bcff0/crates/kernel/src/item.rs:179-189
pub struct PairedItem {
    #[serde(rename = "itemId")]          pub item_id: u32,
    #[serde(rename = "inputNodeIndex")]  pub input_node_index: Option<u8>,
}
```

`item_id` adalah **indeks posisi** ke dalam daftar item node input. `input_node_index` adalah
`Option<u8>` — **satu** induk, maksimal 255 node input.

### 1.2 Empat kegagalan (C-08, sudah diterima agent1 di #522)

| # | Kegagalan | Konsekuensi |
|---|---|---|
| 1 | Posisi tidak bertahan terhadap **GC** (PRD-2 §5.3: *"Spill file — dihapus saat execution terminal"*), reorder, truncate | Ref menunjuk item yang **berbeda** dari maksud semula, dan **tidak ada cara mendeteksinya** |
| 2 | Lebar tipe: kernel `u32`, proposal agent1 `u64`, biaya dihitung 8 B/ref | `u32` membatasi ≤ 4.294.967.295 item/daftar — mungkin cukup, tapi **harus dinyatakan**, bukan diasumsikan (ERR-029) |
| 3 | **Satu** induk vs fan-in | Keluhan agent1 sendiri adalah `pairedItem` "lossy di merge/aggregate/split". Solusi yang juga menyimpan satu posisi akan **lossy dengan cara yang sama** |
| 4 | **Konflik GDPR**: graf lineage **adalah** data pribadi (re-identifikasi lintas node), dan Art. 17 (hak hapus) bertabrakan dengan append-only | Dua kewajiban saling membatalkan tanpa mekanisme crypto-shredding |

**Prinsip yang mengikat seluruh spec ini:**

> Untuk audit, referensi yang bisa **berubah makna secara diam-diam** lebih buruk daripada tidak ada
> referensi — karena ia menghasilkan bukti yang *terlihat* sahih.

---

## 2. Konstruksi: `ItemRef` pseudonim, berjangkar konten

### 2.1 Definisi

```
salt_exec  : 32 byte acak (CSPRNG) per execution
             disimpan di KEYRING, **bukan** di trail lineage, **bukan** di DB utama
             (disiplin kunci yang sama dengan chain_key Envelope — lihat spec Envelope §6)

ItemRef    : BLAKE3-128( salt_exec ‖ node_id ‖ seq ‖ content_hash )      → 16 byte
             framing: u32-BE length-prefix per field variabel (aturan C-02, WAJIB — tanpa itu
             node_id="ab"+ref("c…") dan node_id="abc"+ref("…") bertabrakan)

LineageEdge: { output_ref : ItemRef
               inputs     : LineageSet          ← §3, BUKAN satu ref
               kind       : DIRECT | DERIVED | AGGREGATED | UNKNOWN
               rule_id    : u32                  ← aturan Rosetta/alias yang menghasilkan edge (agent4)
               rule_ver   : u32 }
```

### 2.2 Sifat yang didapat

| Sifat | Mekanisme |
|---|---|
| **Verifiable** | tiap `ItemRef` disertai `content_hash` target; verifier **wajib menolak** ref yang hash-nya tidak cocok. Ref ke item yang sudah di-GC → status `UNKNOWN` **eksplisit**, bukan diam-diam salah |
| **Pseudonim** | tanpa `salt_exec`, ref tidak bisa ditautkan ke item, node, atau eksekusi tertentu |
| **Stabil lintas GC** | `content_hash` tidak bergantung pada posisi atau pada keberadaan file spill |
| **Bisa di-shred** | hancurkan `salt_exec` → **seluruh** ref eksekusi itu jadi tak-tertautkan, **tanpa** mengubah satu byte pun di trail |
| **Deterministik** | tidak ada wall-clock, tidak ada random setelah `salt_exec` dibuat → replay menghasilkan ref **identik** (syarat Envelope, C-04) |

### 2.3 Batas kejujuran

`16 byte` (BLAKE3-128) memberi birthday bound ≈ 2⁶⁴. Untuk **dedup dan penautan internal** itu
cukup. Untuk **bukti kriptografis ke auditor eksternal** itu **tidak** cukup — sama seperti putusan
D-2 di audit saya: dua ancaman, dua panjang. Karena `ItemRef` akan dipakai dalam jawaban atas
permintaan akses data subjek (Art. 15), saya usulkan **16 byte tetap** (biaya per-item dominan),
tapi `content_hash` yang menyertainya **32 byte** (BLAKE3-256) — itulah yang jadi jangkar bukti.

---

## 3. Masalah biaya yang belum dijawab siapa pun: fan-in tidak terbatas

Ini bagian yang saya temukan saat menulis spec, dan **belum** ada di proposal agent1 `#365` maupun
revisinya `#522`.

### 3.1 Masalahnya

agent1 `#365`: *"disk 8B × fan-in per item"* (sudah dikoreksi jadi 16 B di `#522`). Rumus itu
mengasumsikan fan-in **kecil**. Kenyataannya:

| Node | Fan-in per item output | Lineage per item (16 B/ref) |
|---|---|---|
| `Set`, `Move Binary Data` | 1 | 16 B — aman |
| `Merge` (combine by position) | 2 | 32 B — aman |
| `Split In Batches` | 1 (tapi banyak output) | aman |
| **`Aggregate` / `Summarize`** | **N** (seluruh daftar) | **16 N byte** |
| **`Compare Datasets`** | 2 sisi × M | 32 M byte |

Untuk `Aggregate` atas **5.000.000 item** — angka yang justru jadi target desain proyek ini (PRD-3
§2 butir 2, gerbang L4) — lineage **satu** item output = 5.000.000 × 16 B = **80 MB**.

Jadi klaim agent1 *"RAM jalur-panas +0"* tetap benar (lineage ditulis ke spill), tapi **disk tidak**:
lineage bisa **melampaui ukuran payload yang dilacakinya**. Fitur yang dimaksudkan untuk debug
presisi menjadi komponen disk dominan. Itu persis pola yang sudah ditangkap tim untuk index spill
(audit F4: *"Index spill adalah komponen memori DOMINAN pada N besar"*) — hanya kali ini di disk, dan
belum ada yang menghitungnya.

### 3.2 Solusi: tiga representasi `LineageSet`, dengan degradasi jujur

```
LineageSet ::=
  | EXACT      { refs : [ItemRef; k] }              k ≤ 16      → 16k byte
  | RANGE      { node_id, seq_base, count,          → ~40 byte APA PUN count-nya
                 content_hash_root }                              (blok kontigu: Aggregate, Sort)
  | DIGEST_SET { merkle_root : [32]byte,            → 40 byte
                 count       : u64,
                 sample      : [ItemRef; ≤4] }                    (fan-in besar, non-kontigu)
  | UNKNOWN    { reason : GC | RULE_GAP | LOSSY_UPSTREAM }        ← WAJIB ada, jangan ditebak
```

Aturan pemilihan (deterministik, per `rule_id`, bukan heuristic runtime):

| Kondisi | Representasi | Klaim yang boleh dibuat |
|---|---|---|
| fan-in ≤ 16 | `EXACT` | "output ini berasal dari **tepat** item-item ini" |
| fan-in besar **dan** kontigu | `RANGE` | "output ini berasal dari **blok** item ini" + `content_hash_root` membuktikan bloknya |
| fan-in besar **dan** tidak kontigu | `DIGEST_SET` | "output ini berasal dari **himpunan** berukuran `count` dengan root ini" — **tidak** bisa menyebut anggota tanpa membuka seluruh himpunan |
| aturan node tidak memetakan lineage | `UNKNOWN(reason)` | "lineage **tidak diketahui**" — dan ini **harus** terlihat di UI, bukan hilang diam-diam |

**Prinsip (mengikuti pola agent1 #365 sendiri, *"degradasi jujur: lineage-hilang = tandai UNKNOWN,
bukan tebak"*):** representasi yang lebih hemat **tidak boleh** mengklaim presisi yang tidak
dimilikinya. `DIGEST_SET` bukan `EXACT` yang dikompresi; ia klaim yang **berbeda** dan lebih lemah,
dan verifier serta UI wajib membedakannya.

### 3.3 Konsekuensi untuk gerbang

Gerbang agent1 (*"tiap output item query-nya kembali ke himpunan input eksak, 7/7"*) **harus
dinyatakan ulang per representasi**, karena "himpunan eksak" tidak terdefinisi untuk `DIGEST_SET`.
Lihat §8 `LIN-1a/1b/1c`.

---

## 4. Crypto-shredding: mendamaikan Art. 17 dengan append-only

### 4.1 Konflik

| Kewajiban | Bunyi | Bertabrakan karena |
|---|---|---|
| **GDPR Art. 17** | data subjek harus bisa dihapus | trail lineage **append-only** by design (itu nilainya) |
| **Audit trail** | tidak boleh ada baris yang diubah/dihapus | menghapus ref = mengubah trail |

### 4.2 Resolusi

```
erasure(execution_id, subject_request_id):
    1. hancurkan salt_exec[execution_id] dari keyring  (satu-satunya operasi destruktif)
    2. TAMBAH baris ke erasure_log: {subject_request_id, execution_id, ts, actor, authority}
       → append, bukan update. Trail lineage TIDAK disentuh.
    3. semua ItemRef eksekusi itu menjadi tak-tertautkan secara komputasional
```

| Setelah shredding | Masih bisa? |
|---|---|
| Verifikasi integritas Envelope (`VERIFIED_ANCHORED`) | ✅ **ya** — Envelope memakai `chain_key`, bukan `salt_exec` |
| Menautkan ref ke subjek data | ❌ tidak |
| Menghitung jumlah item / struktur graf | ✅ ya (bentuknya ada, maknanya tidak) |
| Membuktikan *kapan* penghapusan diminta & siapa yang mengotorisasi | ✅ ya (`erasure_log`) |

**Pemisahan kunci adalah inti desainnya:** `chain_key` (integritas, hidup panjang, di KMS) dan
`salt_exec` (penautan, per-eksekusi, bisa dihancurkan) **wajib** kunci berbeda. Kalau satu kunci
dipakai untuk keduanya, menghapus data subjek akan **merusak** trail — dan seluruh nilai Envelope hilang.

### 4.3 Batas kejujuran (wajib dinyatakan ke pemilik proyek)

1. **Pseudonimisasi ≠ anonimisasi.** GDPR Recital 26: data yang masih bisa diatribusikan = tetap data
   pribadi. Jadi klaim produk **dilarang** menulis *"GDPR-compliant"*; yang boleh
   **"pseudonymised by construction"** + catatan *butuh review hukum*. Ini sudah diterima agent1 di
   `#522` dan saya usulkan jadi **gerbang** `LIN-3`, bukan sekadar catatan — karena klaim kepatuhan
   yang berlebihan adalah liabilitas yang bisa dipatahkan auditor.
2. **Payload tidak ikut ter-shred.** `salt_exec` hanya memutus **penautan lineage**. Isi item
   (`json`) tetap ada di spill/blob dan tunduk pada retensi PRD-2 §5.3 + enkripsi at-rest SEC-01.
   Penghapusan payload yang sesungguhnya adalah masalah **retensi**, dan itu keputusan terpisah (D-L2).
3. **Rekaman respons web publik adalah data pribadi** bila feed memuat nama orang — dan Workflow Hub
   (`#524`) justru menarik feed politik/berita. Karena itu agent1 `#557(c)` menyetujui
   `record-metadata-only` sebagai **default** untuk feed bernama orang. Keputusan itu **harus** punya
   tempat di PRD-3; saat ini tidak ada (`retensi` = 0 hit).

---

## 5. Interaksi dengan Envelope (satu trail, dua kunci)

agent5 `#521(C)`: *"keduanya saling terkait (lineage-refs bisa masuk chain envelope)"* — benar, dan
ini cara tepatnya:

```
EnvelopeEntry_v1  (spec Envelope §2)  memuat:
    output_digest   : BLAKE3-256 atas payload output          ← sudah ada
  + lineage_root    : BLAKE3-256 atas LineageSet entri ini    ← BARU, 32 byte
  + lineage_repr    : u8  (EXACT|RANGE|DIGEST_SET|UNKNOWN)    ← BARU, 1 byte
```

Konsekuensi yang **menguntungkan**:

- Auditor bisa membuktikan *lineage tidak diubah* **tanpa** membuka lineage (cukup `lineage_root`).
- `lineage_repr` masuk digest → **penurunan representasi tidak bisa disembunyikan**. Kalau seseorang
  mengganti `EXACT` jadi `DIGEST_SET` untuk menghemat disk, digest berubah dan itu terdeteksi.
- Biaya: **33 byte per entri** Envelope. Terhadap anggaran Envelope (persisten 64 B + transien
  1.920 B) ini **diabaikan secara praktis**, tapi tetap wajib dicatat (ERR-029).

**Yang tidak boleh:** memasukkan `ItemRef` **langsung** ke digest Envelope. `ItemRef` bergantung pada
`salt_exec` yang **bisa dihancurkan**; kalau digest Envelope bergantung padanya, maka crypto-shredding
akan **membatalkan verifikasi Envelope** — dua fitur saling menghancurkan. `lineage_root` dihitung
atas `LineageSet` **apa adanya** (ref pseudonim), dan verifier Envelope tidak pernah perlu menautkan
ref ke subjek.

---

## 6. Anggaran (satuan eksplisit per ERR-029)

| Komponen | Persistent (jalur panas) | Transien | Disk |
|---|---|---|---|
| `salt_exec` | 32 B per eksekusi (di keyring, bukan di jalur panas) | — | 32 B per eksekusi |
| `ItemRef` | **0 B** — tidak ditahan di RAM saat eksekusi (klaim agent1 `#365` dipertahankan) | 16 B saat query | 16 B × fan-in **atau** ~40 B (RANGE/DIGEST_SET) |
| `lineage_root` | 32 B (ikut Envelope) | 1 hasher BLAKE3 — **dipakai ulang berurutan**, tidak boleh concurrent dengan hasher Envelope (aturan `R2` agent5 `#578`) | — |
| **Total tambahan RAM jalur panas** | **0 B persisten**, transien ≤ 1 hasher tambahan **bila** di-sequential | — | lihat §3.1 |

**Peringatan anggaran (kelas yang sama dengan C-09):** kalau lineage dan Envelope masing-masing
memegang `blake3::Hasher` **bersamaan**, transien = 2 × 1.920 B = **3.840 B = 96 % dari batas 4 KB**
yang dikunci fern `#431` — sebelum buffer entri. Jadi `R2` agent5 (*"hasher subsistem BERBEDA tidak
concurrent — sequential"*) **mengikat** spec ini, bukan hanya anjuran. Urutan: hash lineage dulu,
reset, lalu hash Envelope. @agent8 mohon masukkan ke transience-audit (`#500` butir 9).

---

## 7. Kontrak storage (untuk @agent2 — DDL milik agent2, ini hanya kolom wajib)

```sql
-- PSEUDONIM: tidak memuat node_id/seq/content_hash mentah
lineage_edge(
  execution_id   INTEGER NOT NULL,
  output_ref     BLOB    NOT NULL,              -- 16 byte BLAKE3-128
  repr           INTEGER NOT NULL CHECK (repr IN (0,1,2,3)),  -- EXACT|RANGE|DIGEST_SET|UNKNOWN
  inputs_exact   BLOB,                          -- [ItemRef] bila repr=0
  inputs_range   BLOB,                          -- {node_id,seq_base,count,hash_root} bila repr=1
  inputs_digest  BLOB,                          -- {merkle_root,count,sample} bila repr=2
  unknown_reason INTEGER,                       -- GC|RULE_GAP|LOSSY_UPSTREAM bila repr=3
  rule_id        INTEGER NOT NULL,
  rule_ver       INTEGER NOT NULL,
  content_hash   BLOB    NOT NULL,              -- 32 byte BLAKE3-256: jangkar bukti
  PRIMARY KEY (execution_id, output_ref)
);

-- Kunci TIDAK di tabel ini. salt_exec hidup di keyring (spec Envelope §6).
erasure_log(                                    -- APPEND-ONLY, tidak pernah UPDATE
  id               INTEGER PRIMARY KEY,
  subject_request_id TEXT  NOT NULL,
  execution_id     INTEGER NOT NULL,
  destroyed_at     TEXT    NOT NULL,
  actor            TEXT    NOT NULL,            -- siapa mengeksekusi
  authority        TEXT    NOT NULL,            -- dasar hukum/tiket
  salt_fingerprint BLOB    NOT NULL             -- BLAKE3(salt_exec) SEBELUM dihancurkan:
                                                -- membuktikan salt mana yang dimusnahkan
                                                -- TANPA menyimpan salt-nya
);
```

**`salt_fingerprint` adalah detail yang mudah dilewat dan penting:** tanpa itu, `erasure_log` hanya
mengklaim "sudah dihapus" tanpa bisa menunjukkan *salt mana* yang dimusnahkan — dan kalau ada dua
eksekusi, tidak ada cara membuktikan yang mana. Dengan fingerprint, penghapusan **bisa diaudit**
tanpa menyimpan rahasia yang baru saja dimusnahkan. Pola yang sama dengan `enc_key_id`/`enc_algo`
(audit F20): atribut penting tidak boleh implisit.

---

## 8. Gerbang (falsifiable, deterministik, tanpa LLM)

| Gate | Uji | Kriteria lulus |
|---|---|---|
| **LIN-1a** `lineage_exact_roundtrip` | 7 workflow RUNNABLE, semua edge `repr=EXACT` | tiap `output_ref` → himpunan input **eksak**, 7/7 |
| **LIN-1b** `lineage_range_proof` | `Aggregate`/`Sort` atas 1.000 dan 1.000.000 item | `RANGE` ~40 B apa pun `count`; `content_hash_root` memverifikasi blok; **ukuran lineage tidak tumbuh dengan count** |
| **LIN-1c** `lineage_digest_set_honesty` | fan-in > 16 non-kontigu | verifier **menolak** klaim "himpunan eksak" untuk `DIGEST_SET`; UI/API membedakan `EXACT` vs `DIGEST_SET` |
| **LIN-2** `crypto_shredding` | hancurkan `salt_exec`, coba tautkan ulang | **0** ref bisa ditautkan; `erasure_log` bertambah 1 baris; trail lineage **tidak berubah satu byte pun** (bandingkan hash tabel sebelum/sesudah); Envelope tetap `VERIFIED_ANCHORED` |
| **LIN-3** `no_overclaim` | scan dokumen produk, README, UI string, MCP resource | **0** kemunculan "GDPR-compliant"/"GDPR ready" tanpa kualifikasi; yang boleh hanya "pseudonymised by construction" + catatan review hukum |
| **LIN-4** `ref_target_verified` | acak 1.000 ref, bandingkan `content_hash` | 100 % cocok; ref yang targetnya sudah di-GC → `UNKNOWN(GC)`, **bukan** salah tunjuk diam-diam |
| **LIN-5** `lineage_root_in_envelope` | ubah satu `LineageSet` setelah Envelope ditutup | verifikasi Envelope **BROKEN**; ubah `repr` `EXACT`→`DIGEST_SET` → **juga BROKEN** |
| **LIN-6** `fan_in_budget` | `Aggregate` 5.000.000 item | total byte lineage ≤ 1 % dari total byte payload; kalau lebih, `RANGE`/`DIGEST_SET` **wajib** dipakai, bukan `EXACT` |

**Prasyarat urutan:** `G-C8` (content_id benar-benar content-addressed) harus lulus **sebelum**
LIN-1..LIN-6 bermakna. Menjalankan LIN di atas `ContentId` sekuensial akan menghasilkan gate yang
**lulus tapi tidak menguji apa pun** — persis kegagalan testkit C-05 (`15/15 hijau` tanpa menguji
implementasi nyata).

---

## 9. Keputusan yang diminta

| ID | Pertanyaan | Rekomendasi saya | Untuk |
|---|---|---|---|
| **D-L1** | Item Lineage masuk PRD-3 atau tidak? | **Masuk** sebagai inovasi ke-12 + Wave 3 + tugas `W3-ITEM-LINEAGE` + gerbang `LIN-1..LIN-6`. Kalau ditolak, tuliskan sebagai keputusan sadar | pemilik proyek + fern |
| **D-L2** | Retensi payload vs shredding penautan | Pisahkan eksplisit: `salt_exec` memutus **penautan**; penghapusan **payload** tunduk pada retensi PRD-2 §5.3 + `record-metadata-only` default untuk feed bernama orang | pemilik proyek + agent2 |
| **D-L3** | Batas `EXACT` (saya usulkan `k ≤ 16`) | 16 ref = 256 B/edge. Naik-turunkan berdasarkan pengukuran disk agent8, **bukan** selera | agent8 + agent10 |
| **D-L4** | Panjang `ItemRef` | 16 B (BLAKE3-128) untuk ref + **32 B** (BLAKE3-256) untuk `content_hash` jangkar bukti. Dua ancaman, dua panjang — sama seperti D-2 | matt + agent2 |
| **D-L5** | Siapa pemilik `erasure_log`? | Bukan engine. Penghapusan adalah **operasi kepatuhan**, butuh otorisasi manusia (`authority` NOT NULL). Engine hanya menyediakan mekanismenya | pemilik proyek + fern |

---

## 10. Reproduksi

```bash
P=/opt/agent-workspace/docs/PRD-3-PERFECTION-CHECKLIST.md
for k in lineage Lineage GDPR crypto-shred shredding PII retensi lisensi pseudonim; do
  printf "%-14s %s\n" "$k" "$(grep -ic "$k" "$P")"
done                                              # semuanya 0
swarm-task list | grep -i ROLE_COMPLIANCE          # hanya W3-EXEC-ENVELOPE
sed -n '179,189p' /opt/agent-workspace/kernel-asli-d3bcff0/crates/kernel/src/item.rs   # PairedItem
grep -n "Spill file" /opt/agent-workspace/docs/PRD-2-RUST.md                            # GC saat terminal
```

Angka §3.1 adalah **aritmetika**, bukan pengukuran: 5.000.000 × 16 B = 80.000.000 B = 80 MB.
Kalau ada yang membantahnya, bantah dengan rumus atau dengan pengukuran — bukan dengan intuisi.

---

## 11. Catatan proses

Dokumen ini saya tulis **setelah** menemukan bahwa pilar yang ditugaskan ke peran saya tidak ada di
dokumen yang akan ditandatangani pemilik proyek. Saya bisa saja hanya melaporkan lubangnya (itu
`P3-02`) dan menunggu ditugasi. Saya pilih menulis isinya, karena lubang yang dilaporkan tanpa isi
cenderung ditutup dengan cara menghapus laporannya.

Dua hal yang saya lakukan berbeda dari kebiasaan saya di dokumen sebelumnya:

1. **§3 adalah temuan saya sendiri, bukan koreksi atas orang lain.** Proposal agent1 `#365` dan
   revisinya `#522` menghitung biaya sebagai `16 B × fan-in` tanpa batas atas. Saya tidak menemukan
   itu dengan membaca kode — saya menemukannya dengan menanyakan "bagaimana kalau fan-in-nya lima
   juta?", yaitu angka yang proyek ini pakai sebagai target desain di tempat lain. Klaim biaya tanpa
   kasus terburuk adalah klaim yang belum selesai.
2. **Saya tidak menyunting file yang sedang ditulis sesi agent10 lain.** Ada dua sesi paralel memakai
   identitas `agent10` (bukti + usul penanganan di pesan `#628`). Saya menulis file baru milik saya,
   sesuai `#aturan` Pasal 7.3.

Mohon review adversarial. @agent1 terutama §3 dan §5 — §3 membatalkan sebagian klaim biaya proposal
Anda, dan saya lebih suka Anda yang membantahnya daripada saya yang benar tanpa diperiksa. @agent2
§7. @agent8 §6 dan D-L3. @agent5 (atau siapa pun Plt. Gatekeeper setelah keputusan `#628` butir 1)
§8.

— **agent10**, Enterprise Compliance & Cryptography Auditor (ROLE_COMPLIANCE)
