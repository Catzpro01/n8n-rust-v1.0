# CHECKLIST CONFORMANCE — W3-ITEM-LINEAGE terhadap norm v1.0 + Ruling 14

| | |
|---|---|
| **Untuk** | @agent2 (implementor, `crates/storage/`, #1076/#1098) |
| **Dari** | agent10 sesi A (penulis norm v1.0 `eae5cbc2…`) |
| **Norm** | `AGENT10-ITEM-LINEAGE-SPEC.md` v1.0 = BINDING; `AGENT10-W3-ITEM-LINEAGE-SPEC-v0.3.md` (`beb9d636…`) = rencana implementasi |
| **Putusan hash** | **Ruling 14 @matt `#1102` = opsi (a) salted-only.** Usulan opsi (c) saya (`#1097` butir 3) **DITOLAK dan saya terima** — lihat §6 |
| **Sifat dokumen** | *Conformance check*, bukan review independen penuh (saya penulis norm). Reviewer independen kode: @agent1 / @agent5 |

Setiap butir ditulis sebagai **bisa diuji**, bukan sebagai nasihat.

---

## A. Tipe & identitas

| # | Butir | Orakel |
|---|---|---|
| A1 | `execution_id` = **`ExecutionId`** (u64, `numeric_id!` di `id.rs:35`). **BUKAN `ContentId`** (`id.rs:41`) — keduanya u64, jadi compiler tidak akan menangkap pertukaran ini | grep DDL + tinjau tipe di Rust |
| A2 | `attempt` = otoritas **`task.attempt` engine**, SATU sumber penomoran (`#951` cat.2). Jangan buat counter kedua | satu kolom, tidak ada `retry_no`/`attempt2` |
| A3 | NOL `PayloadRef` / `SpillId` / `SpillRef` / `uuid` di kode lineage (Ruling 3(a), Ruling 10 butir 4). Bila butuh identifier unik → `ContentId(u64)` | `grep -rn 'uuid\|PayloadRef\|SpillId' crates/storage/src` → 0 |
| A4 | `SpilledList` = `{path, len, total_bytes, codec}` (`item.rs:212`). `content_id` ada di **`BinaryLocation::Ref{content_id,size_bytes}`** (`item.rs:146,159-162`) — **mekanisme terpisah**, jangan digabung (Ruling 13) | tinjau pemakaian tipe |

## B. Hash — Ruling 14(a) + syarat kerasnya

| # | Butir | Orakel |
|---|---|---|
| B1 | `content_hash` = `BLAKE3-256(salt_exec ∥ kanon(meta))`. **TIDAK ADA hash polos atas isi di tabel lineage** | baca kode + uji G-I1 di §E |
| B2 | **Syarat keras Ruling 14:** `casd_ref` = SHA-256 **lintas-instance → wajib tanpa salt**. Konsekuensinya: **`casd_ref` HARUS `NULL` untuk isi yang terklasifikasi personal.** Kalau `casd_ref` polos disimpan untuk isi yang sama, men-salt `content_hash` tidak memberi apa-apa — penyerang tinggal brute-force `casd_ref` | uji G-I2 di §E |
| B3 | Klasifikasi **wajib terekam**: `data_class` + `class_rule_id` + `class_rule_ver`. Tanpa ini, perbedaan "baris punya hash polos vs tidak" tidak bisa dijelaskan ke auditor, dan salah klasifikasi terjadi secara senyap ke arah paling berbahaya | kolom ada di DDL |
| B4 | Kolom plain-hash (`casd_ref`) **NULL-able dan TANPA default yang mengisi otomatis**. Default yang mengisi = jalur senyap menuju hash polos atas PII | baca DDL |
| B5 | Checksum **berkas fisik** tetap SHA-256 (G-C7, `#751`, Piagam `#922` §A) — jangan dicampur dengan digest isi | satu primitif per tujuan |
| B6 | Dedup **baris lineage = NON-GOAL** (v0.3 butir 5). Baris = bukti, bukan konten | tidak ada UNIQUE atas content |

## C. Crypto-shredding (v1.0 §4/§7)

| # | Butir | Orakel |
|---|---|---|
| C1 | `salt_exec` hidup di **keyring**, TIDAK di tabel lineage | grep DDL: tidak ada kolom salt |
| C2 | `erasure_log` **append-only**, tidak pernah UPDATE; `authority NOT NULL`; `salt_fingerprint` = `BLAKE3(salt_exec)` **sebelum** dimusnahkan (membuktikan salt mana yang dihancurkan tanpa menyimpannya) | DDL + uji LIN-2 |
| C3 | **TTL otomatis tanpa baris `erasure_log` = DILARANG** (D-L5). Penghapusan adalah operasi kepatuhan berotorisasi manusia | tidak ada job TTL yang menghapus tanpa insert erasure_log |
| C4 | Pasca-shred: tautan **mati**, bentuk graf **tetap ada**, dan Envelope **tetap `VERIFIED_ANCHORED`** (v1.0 §4.2, LIN-2) | uji LIN-2 |
| C5 | Retensi **terpisah per kelas**: metadata lineage = panjang (audit); payload/`event_json` = pendek (TTL bernama). Purge payload TIDAK boleh menghapus jejak lineage (D-L7) | partial index, bukan DELETE cascade ke lineage |

## D. Larangan overclaim (LIN-3)

| # | Butir | Orakel |
|---|---|---|
| D1 | **0 kemunculan** `"GDPR-compliant"` / `"GDPR ready"` di kode, README, UI string, resource MCP — tanpa kualifikasi | `grep -rni 'gdpr-compliant\|gdpr ready'` → 0 |
| D2 | Yang boleh: **"pseudonymised by construction"** + catatan review hukum. Pseudonimisasi ≠ anonimisasi (Recital 26) | tinjau teks |
| D3 | Lineage **bukan bukti audit**. Bukti audit hanya Envelope + anchor eksternal (ADDENDUM-A1). Jangan jual lineage sebagai tamper-evidence | tinjau klaim dokumen |

## E. Gates wajib — termasuk kontrol negatif

Definisi `DONE-CODE` di kamus baris 40-48 kini **mewajibkan "uji negatif/mutan"**, jadi gate berikut bukan opsional.

| Gate | Klaim | Orakel (harus BISA gagal) |
|---|---|---|
| **LIN-1** | Query "item ini lahir dari mana" tanpa table scan | `EXPLAIN QUERY PLAN` → nol `SCAN` |
| **LIN-2** | Hancurkan `salt_exec` → **0** ref bisa ditautkan; `erasure_log` +1 baris; trail lineage **tidak berubah satu byte pun** (bandingkan hash tabel sebelum/sesudah); Envelope tetap `VERIFIED_ANCHORED` | bandingkan digest tabel |
| **LIN-3** | Scan dokumen/README/UI → 0 overclaim | §D1 |
| **G-I1 (baru, anti-plain-hash)** | Sisipkan item terklasifikasi **personal** → `casd_ref IS NULL`, dan **tidak ada kolom di baris itu yang memuat `SHA-256(isi)` polos** | hitung kandidat hash polos di uji, lalu pastikan tidak muncul di byte baris mana pun |
| **G-I2 (baru, anti-vacuous)** | Kontrol positif untuk G-I1: item terklasifikasi **non-personal** → `casd_ref` TERISI dan dedup lintas-instance benar-benar terjadi (2 insert → 1 blob) | tanpa ini G-I1 bisa lulus karena kolom selalu NULL |
| **G-I3** | `repr=UNKNOWN` eksplisit pasca-GC (`unknown_reason` ∈ {GC, RULE_GAP, LOSSY_UPSTREAM}) — tidak pernah diam-diam jadi EXACT | hapus payload → baca repr |

**G-I2 sama pentingnya dengan G-I1.** Tanpa kontrol positif, G-I1 lulus secara vacuous bila `casd_ref` selalu NULL karena bug — persis pola yang saya temukan di gate anchor (ANC-8e) dan yang membuat matt nyaris tertipu `test result: ok. 0 passed` (`#960` koreksi diri (a)).

## F. Batas yang tidak boleh dilewati

- Lineage = **metadata non-PII + digest** (C-06). PII tinggal di payload/`event_json`.
- Bukan replay engine (itu `#910`, @agent1) · bukan RecordSet (DETERM, konteks beda C-02) · bukan checksum berkas.
- Tidak menulis ke tabel milik `#910`/migrasi-003: index `lineage_lookup` (T-1) bentuknya turunan, koordinasi @agent2 sebagai pemilik DDL.

## 6. Catatan jujur: usulan saya ditolak, dan penolaknya benar

Di `#1097` butir 3 saya mengusulkan **opsi (c)**: dual-hash bersyarat klasifikasi (salted selalu + plain SHA-256 untuk non-personal). @matt memutuskan **opsi (a)** salted-only (Ruling 14) dan membantah keberatan saya dengan tepat:

> Keberatan saya — "bukti Art. 15 atas isi ikut hilang pasca-shred" — **bukan bug, itu definisi Art. 17**. Setelah crypto-shredding kita memang seharusnya tidak bisa membuktikan isi. Hak akses Art. 15 dilayani dari payload/`event_json` selama ia ada, bukan dari hash.

Saya terima. Saya keliru karena mencampur dua hal: kewajiban membuktikan isi **selama data ada** (Art. 15, terlayani dari payload) dengan keinginan membuktikan isi **setelah kunci dimusnahkan** (yang justru harus tidak mungkin). Poin 3 beliau juga benar: +32 byte per baris pada log yang tidak pernah mendedup baris adalah biaya permanen untuk manfaat yang sudah terbantahkan.

Yang **tetap berlaku** dari usulan saya, dan diserap oleh syarat keras Ruling 14 sendiri: aturan klasifikasi (B2/B3/B4) dan gate anti-plain-hash (G-I1/G-I2). Tanpa keduanya, salt di `content_hash` bersifat dekoratif — karena `casd_ref` polos akan membocorkan isi yang sama.
