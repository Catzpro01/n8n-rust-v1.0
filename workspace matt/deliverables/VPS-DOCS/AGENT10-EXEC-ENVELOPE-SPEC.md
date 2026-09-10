# AGENT10 — SPESIFIKASI EXECUTION ENVELOPE & ROLLING HASH-CHAIN
## Deliverable W3-EXEC-ENVELOPE (Wave 3, ROLE_COMPLIANCE) — v1.0 PREP

| | |
|---|---|
| **Versi** | v1.2-PREP — v1.1 (Merkle dicabut per #555/#628 §6; deklarasi dua-sesi §13) + **v1.2: SERAP ADDENDUM-A1 (07:16, sesi A — koreksi diri: keyed chain PERLU tapi TIDAK CUKUP; anchor eksternal = satu-satunya lapisan yang MENJAMIN; chain_key_id per segmen; klaim produk wajib "tamper-evident", bukan "tamper-proof")** |
| **Penulis** | agent10 — ROLE_COMPLIANCE (Enterprise Compliance & Cryptography Auditor). **Sesi kedua** (sesi yang menulis v1.0 dokumen ini; sesi pertama memegang `AGENT10-PRD3-COMPLIANCE-REVIEW.md` per pembagian lane #628 §5 → usulan (b) diterima, menunggu ratifikasi fern/matt) |
| **Tanggal** | 2026-09-09, ~07:05 UTC |
| **Induk bukti** | `AGENT10-COMPLIANCE-CRYPTO-AUDIT.md` v1.2 (§2.12) — temuan C-01…C-09, uji runtime §2.11 (13 baris, rustc 1.98.1) |
| **Gerbang PRD-3** | L2: *"BLAKE3 sequential hasher ≤ 4KB transien (`SEC-TRANSIENT`)"* & *"Keyed derive_key untuk Execution Envelope dan audit trail (`G-C6`)"* |
| **Aturan** | Zero-Code Mandate dipatuhi: dokumen ini **bukan kode**. Semua DDL/SQL di §7 adalah **kontrak kolom** untuk pemilik storage (agent2); implementasi menunggu sign-off Pemilik Proyek. |

---

## 0. Tujuan & batas ranah

Dokumen ini menentukan **kontrak kriptografis** Execution Envelope: apa yang di-hash, bagaimana di-hash, apa yang boleh diklaim dari hasilnya, dan bagaimana membuktikannya. Envelope **tidak** menyimpan data eksekusi yang bisa dibaca — ia menyimpan bukti. Kontrak diserahkan:

| Kepada | Untuk |
|---|---|
| agent2 (storage) | kolom/tabel wajib (§7) |
| agent5 (SEC) | gate CI G-C1…G-C10 + `SEC-TRANSIENT` + penulisan ulang SEC-08 sesuai C-01 |
| agent1 (timeline) | keterkaitan seq/head dengan Timeline Replay (Wave 3) |
| agent4 (Hub) | pola trust anchor yang sama untuk `manifest_sig` (H-02) |
| agent8 (perf) | angka RAM di §4 |
| pemilik proyek | keputusan D-1…D-4 (§12) |

## 1. Prinsip (non-negotiable)

1. **Klaim jujur**: verifier mengembalikan `VERIFIED_ANCHORED | VERIFIED_UNANCHORED | BROKEN`, tidak pernah "VERIFIED" polos (C-01).
2. **Domain separation**: satu konteks BLAKE3 per subsistem; digest Envelope tidak boleh bermakna di subsistem lain (C-02).
3. **Framing kanonik**: semua field panjang-variabel memakai prefix `u32` big-endian; tidak ada jalur encoding kedua (C-02).
4. **Satu hasher**: persisten 32 B + satu `blake3::Hasher` dipakai berurutan (`SEC-TRANSIENT` ≤ 4 KB, C-09).
5. **Jangkar eksternal**: tanpa anchor di luar kendali penulis storage, tidak ada klaim tamper-evidence (C-01).
6. **Replay deterministik**: hanya field DETERMINISTIC masuk digest (C-04).

## 2. Entri kanonik — `EnvelopeEntry_v1`

```
EnvelopeEntry_v1 ::=                                len  |  catatan
  schema_version   u8 BE   = 1                       1   |  versi skema, naik hanya bila format berubah
  exec_id          [u32 BE len][bytes]              4+n   |
  seq              u64 BE                             8   |  urutan eksplisit (C-03: Merkle tidak memberi urutan)
  node_id          [u32 BE len][bytes]              4+n   |
  input_digest     bytes[32]                         32   |  BLAKE3-256 payload input kanonik
  output_digest    bytes[32]                         32   |  BLAKE3-256 payload output kanonik
  status           u8                                 1   |  tabel nilai beku berversi (lihat §2.1)
  engine_version   [u32 BE len][bytes]              4+n   |  wajib (C-04: atribusi hasil ke versi engine)
  config_digest    bytes[32]                         32   |  wajib (C-04: replay lintas konfigurasi terdeteksi)
  class_flags      u8                                 1   |  bit0 = HAS_UNVERIFIED_EXTERNAL_IO
```

**Aturan encoding:** integer big-endian; string = `u32 len` + byte; `Display`/`Debug`/JSON dilarang masuk digest; encoding adalah **satu fungsi**, satu test (G-C2). Tidak ada jalur kedua.

### 2.1 Tabel `status` (nilai beku, berversi)

| u8 | Makna | Catatan |
|---|---|---|
| 0x01 | SUCCEEDED | |
| 0x02 | FAILED_ERROR | error runtime di-capture di metrik, bukan di digest |
| 0x03 | SKIPPED | kondisional false |
| 0x04 | TIMEOUT | |
| 0x05 | CANCELLED | |
| 0x10… | OPACUE_NODE_* | reservasi node opaque (Rosetta), per-era |

Nilai baru wajib menaikkan `schema_version` **atau** terdaftar sebagai ekstensi berversi — tidak boleh "bebas".

### 2.2 Field yang DIKELUARKAN dari digest (kelas OBSERVATIONAL, C-04)

`durasi_ms`, `started_at`, `finished_at`, `worker_id`, `rss_bytes` → tabel `execution_metrics` terpisah, **tidak dirantai**, dan tidak pernah dipakai sebagai bukti integritas. (Replay byte-identical hanya mungkin bila field ini di luar digest.)

## 3. Konstruksi chain + Merkle (perbaikan §3.2 audit)

```
CTX     = "n8nrust/envelope/v1"
ckey    = blake3::derive_key(CTX, chain_key)          ; DUA argumen — lihat KOREKSI-1
h_0     = blake3::keyed_hash(&ckey, 0x00 || 0x00 || entry_0)   ; genesis ber-flag, bukan string kosong
h_i     = blake3::keyed_hash(&ckey, h_{i-1} || entry_i)        ; framing 32 B tetap, tidak ambigu (C-02)
head    = h_{n-1}                                     ; chain head — BUKAN "Merkle root" (istilah #327 butir 3 salah; C-03)
checkpoint_k = h_{(k+1)*K - 1}  untuk tiap K=1.000     ; skip-ahead verification (agent5 #521), tanpa mengubah chain jadi tree
envelope= { head, checkpoint[], n, schema_version, chain_algo, anchor[] }
```

> **⚠️ REKONSILIASI v1.1 (sesi kedua):** `merkle_root` dan `leaf_i` **DICABUT**. Alasan (konsisten
> #555 + #628 §6 sesi pertama): consensus 4-agen #379/#383 memilih **chain** (deteksi penyisipan/
> pengurutan-ulang di tengah secara alami), checkpoint-root tiap K untuk skip-ahead; tier-3 disclosure
> sudah membocorkan seluruh isi semantik kepada auditor yang sama, sehingga inclusion proof O(log n)
> tidak melayani use-case yang ada (tidak ada range-proof parsial — agent1 #631 butir (b) mengonfirmasi
> dari sisi engine; penggunaan paralel satu-satunya yang diusulkan = bukti sebagian entri tanpa seluruh
> rantai, dan tidak ada konsumennya). Yang TETAP wajib dari C-03: (1) istilah `envelope_root` = chain
> head `h_{n-1}` (bukan akar Merkle) di seluruh dokumen; (2) `seq` eksplisit di entri (sudah ada).
> Jika kelak muncul use-case range-proof parsial yang dapat ditunjukkan, Merkle tree masuk kembali
> sebagai ekstensi `schema_version` — satu kalimat ini di spec cukup.

> **KOREKSI-1 (dari §3.2 audit v1.1):** `BLAKE3::derive_key(context)` **tidak kompilasi**. Signature blake3 1.8.7: `derive_key(context: &str, key_material: &[u8]) -> [u8; OUT_LEN]`. `chain_key` (rahasia) wajib diberikan eksplisit sebagai `key_material`. Alternatif setara teruji: `Hasher::new_derive_key(CTX).update(chain_key).finalize_xof().fill(&mut ckey)` — hasil identik (diuji `true`, §2.11 baris 6–7).
>
> **Peringatan keamanan:** konteks saja **bukan rahasia** — tanpa `chain_key`, "kunci" = fungsi dari string publik, dan C-01 tidak tertutup.

`envelope_root` yang dipublikasikan = **`head = h_{n-1}`** (chain head; istilah "Merkle root" di #327 butir 3 adalah salah — C-03; `root` Merkle TIDAK ada sejak v1.1). `head` disimpan bersama checkpoint (C-03: head ≠ akar Merkle; keduanya tidak boleh dirancukan).

## 4. Anggaran memori (C-09, satuan eksplisit per ERR-029)

| Komponen | Persistent | Transien | Sumber |
|---|---|---|---|
| `h_prev` | 32 B | — | rolling state |
| `ckey` | 32 B | — | hidup selama eksekusi |
| `blake3::Hasher` | — | **1.920 B** (terukur) | blake3 1.8.7, `size_of` §2.11 baris 3 |
| buffer entri kanonik | — | ≤ 256 B + len(node_id) | field tetap |
| sha2::Sha256 (banding) | — | 112 B (terukur) | §2.11 baris 4 |

**Aturan:** **satu** `blake3::Hasher` dipakai ulang berurutan (reset, bukan instansiasi baru). Dua hasher paralel = 3.840 B = 96 % dari 4 KB (C-09) → dilarang tanpa persetujuan eksplisit. Total persistent **64 B/eksekusi** — klaim #327 yang menulis "32 byte" wajib direvisi ke angka ini (angka itu hanya `h_prev`).

## 5. Checkpoint, anchor & verifier (C-01, selaras agent5 #521)

- `K = N = 1.000` — **satu interval, dua sinkronisasi**: checkpoint (verifikasi skip-ahead internal) dan anchor (keaslian eksternal).
- Tiap checkpoint → baris `envelope_checkpoint` (append-only) **dan** publikasi ke sink eksternal: Object Lock S3/Glacier, atau RFC 3161 TSA. Pilihan sink = **D-3** (pemilik proyek).
- Verifier: `{ VERIFIED_ANCHORED \| VERIFIED_UNANCHORED \| BROKEN }` + **indeks entri pertama yang menyimpang**. Entri sejak anchor terakhir berstatus `UNANCHORED` — jendela jujur, tidak pernah diklaim "terverifikasi" penuh.

## 6. Kebijakan kunci (keyring) — satu disiplin, tiga kunci

| Kunci | Panjang | Keperluan | Sumber (WAJIB di luar filesystem host, C-01/D-4) |
|---|---|---|---|
| `chain_key` | 32 B | Envelope chain (keyed) | KMS/HSM eksternal atau envelope-encryption; **bukan** file di host (prinsip PRD-2 §11.1 yang sudah menolak `ENGINE_ENCRYPTION_KEY`) |
| `hub_signing_key` | 32 B (Ed25519 seed) | manifest Hub (H-02) | sama; publik di registri agent7 sebagai `trust_anchor` |
| `salt_exec` | 32 B acak/eksekusi | crypto-shredding Item Lineage (C-08) | key store terpisah dari trail; hancurkan = hak lupa GDPR |

**Rotasi (H-02, ranah saya + agent5):** kunci punya `kid` + `valid_from`/`valid_until`/`revoked` di registri; rotasi memakai **dual-key overlap** (dua kid valid, penandatangan lama diverifikasi sampai `valid_from` baru terlewat); klien **wajib** fetch registri ulang sebelum verifikasi; tanda tangan harus menyertakan `kid` yang dipakai. Dua elemen wajib di spek skema.

> **⚠️ A1.5 untuk `chain_key` (sesi A, addendum):** rotasi `chain_key` **bukan** sekadar dual-key —
> ia adalah **pemutusan rantai** (chain break): segmen rantai di bawah key lama tidak bisa diverifikasi
> dengan key baru. Wajib: (a) `chain_key_id` dibawa di setiap entri/checkpoint; (b) verifier menolak
> segmen yang `chain_key_id`-nya tidak dikenal; (c) rotasi = catat `chain_key_id` baru + validasi
> segmen lama SEBELUM memutus; (d) `chain_key` hanya dari secret-manager (bukan env/file — alasan
> diperkuat: A1.2, root lokal dapat membaca env/file/memori proses tanpa "membobol" apa pun).

**Manifest Hub — kontrak canonical (konsensus #602/#605/#612):**

```
manifest_canon_v1 ::= { schema_version, hub_id, template_sha256, content_version }
                      || BLAKE3( sorted(deviation_ids) )        ; deviasi DI-IKAT (agent3 #605)
manifest_sig      ::= Ed25519( hub_signing_key, manifest_canon_v1 )
```

Alasan: `deviation_ids` = metadata penerbit (alias Rosetta) yang TIDAK boleh terbuka terhadap manipulasi diam-diam; di-sort sebagai set (tidak peduli urutan) → BLAKE3 = satu primitif (G-C6). Penandatangan menandatangani **kanon penerbit ansambel**, bukan hash tunggal.

**TOCTOU (agent1 #603) — garis keras:** verifikasi `manifest_sig` di **install-gate** (agent5) **DAN** diulang saat **first-execution** (agent1), TIDAK dalam salah satu saja. Alasan: install-gate dapat dilewati/out-of-band; first-execution = titik kontrol tanpa asumsi. Biaya 1× Ed25519 verify + 1× lookup registri — di bawah batas transien.

## 7. Kontrak storage (untuk agent2 — kolom WAJIB, DDL milik agent2)

Entri envelope: **append-only file** terpisah dari DB utama (gate SEC-08 agent5), satu record per baris `EnvelopeEntry_v1` biner; nama file `envelope-{exec_id}.env`:

```
kolom logis yang WAJIB ada per record:
  exec_id, seq, node_id, input_digest, output_digest, status,
  engine_version, config_digest, class_flags, h_i (32 B), created_at
```

Tabel DB (kontrak kolom):

```sql
envelope_checkpoint ( id INTEGER PK, exec_id TEXT, seq INT, root B32, head B32,
                      n INT, anchor_kind TEXT, anchor_ref TEXT, anchor_ts INT,
                      schema_version INT, chain_algo TEXT, created_at INT )
execution_metrics   ( id INTEGER PK, exec_id TEXT, node_id TEXT, durasi_ms INT,
                      started_at INT, finished_at INT, worker_id TEXT, rss_bytes INT )
keyring_meta        ( kid TEXT PK, purpose TEXT, valid_from INT, valid_until INT,
                      revoked INT, pubkey B32, source TEXT )   -- metadata SAJA; material kunci TIDAK di host
```

Kontrak: tidak pernah UPDATE/DELETE di `envelope_checkpoint`; `execution_metrics` boleh dirotasi sesuai retensi (bukan bukti).

## 8. Gate penerimaan (falsifiable — usul masuk `AGENT5_QA_SECURITY_SPEC.md`)

| Gate | Uji | Lulus bila | Dijalankan |
|---|---|---|---|
| G-C1 | hapus entri tengah; **dan** ubah + rekomputasi seluruh rantai | keduanya BROKEN; yang kedua sekarang TIDAK ada di SEC-08 | CI agent5 |
| G-C2 | fuzz 10.000 pasangan entri berbeda + kasus pergeseran batas (`node_id="abc"` vs `"ab"‖'c'`) | 0 tumbukan; kasus geser **wajib** berbeda | CI |
| G-C3 | ~~inclusion proof~~ | **WITHDRAWN** — Merkle dicabut (#555/#628 §6, konsensus #379/#383). Kembali aktif HANYA bila ada use-case range-proof parsial yang ditunjukkan (agent1 #631: tidak ada dari sisi engine) | — |
| G-C4 | replay 2× fixture deterministik | head & root identik; cek statis: `durasi_ms` tidak di-hash | CI |
| G-C5 | scan identifier `blake3\|sha256\|sha2\|md5` | tiap hit memanggil implementasi algoritmanya (daftar putih direview) | CI |
| G-C6 | KAT resmi BLAKE3 + SHA-256 | 100 % cocok (vektor di §11) | CI |
| G-C7 | blob sama via testkit & data-plane | checksum identik (menutup 16-hex vs 64-hex, C-05) | CI |
| G-C8 | `put` konten identik 2× | `ContentHash` sama (akan **gagal** sekarang — itu maksudnya) | CI |
| G-C9 | tiap `ItemRef` di 7 workflow RUNNABLE | verifikasi ke `content_hash` target; GC → `UNKNOWN`, tidak diam-diam | CI |
| G-C10 | hancurkan `salt_exec` | 0 ref tertaut ulang; trail tetap VERIFIED_ANCHORED | CI |
| G-C11 | **TRUST-01** dual-key overlap (usul agent3 #605): sign dengan kid lama selama overlap | verify LULUS (kid dipetakan ke pubkey valid, overlap berfungsi) | CI |

Prasyarat urutan: **G-C8 → G-C9/G-C10** (C-06 → C-08). G-C6 = persis gerbang L2 PRD-3 (*"Keyed derive_key… (G-C6)"*) — konfirmasi ke agent5 bahwa formulisinya: KAT **keyed** + `derive_key` dua-argumen + kebutuhan `chain_key` eksplisit. Penomoran gate SEC-* adalah milik agent5; G-C11 saya catat di spec compliance sebagai gate yang diagendakan (menunggu nomor resmi dari agent5).

## 9. Pemetaan kepatuhan

| Klaim | Standar | Dipenuhi oleh |
|---|---|---|
| Tamper-evidence terhadap penyerang ber-root | SOC 2 CC7.2, CC6.1/CC6.3 | **HANYA** anchor eksternal (A.2, di luar kontrol root) yang menjamin; keyed chain (A.1) menaikkan biaya & mengurangi jendela — **bukan** jaminan; per A1.3 klaim dibatasi ke "tamper-evident … jendela ≤ N entri berlabel UNANCHORED" |
| Perubahan otorisasi tercatat | SOC 2 CC8.1 | proses di luar kripto — lihat audit C-10/ERR-030 (butuh K-4 diputus) |
| Tanggung jawab akuntabilitas | GDPR Art. 5(2), Art. 30 | §3–§5 |
| Hak penghapusan tanpa merusak trail | GDPR Art. 17 | crypto-shredding `salt_exec` (§6, C-08) + `retain-outputs` menyimpan digest+salt saja (§3.5 audit) |
| Data protection by design | GDPR Art. 25 | pseudonimisasi di level struktur data |
| Keterlacakan subjek data | GDPR Art. 15 | hanya selama `salt_exec` hidup (disclosure-tier 2/3 agent5 #521) |

## 10. Interaksi lintas agen (disepakati/kontrak)

- **agent1 (Timeline Replay)**: `seq` Envelope = urutan timeline; verifikasi `head/root` sebelum reversion — keduanya dirancang di Wave 3.
- **agent2 (storage)**: kontrak §7; dependensi `W2-STORAGE-L0` = prasyarat claim W3-EXEC-ENVELOPE (task system).
- **agent5 (SEC)**: SEC-08 ditulis ulang sesuai C-01; gate G-*; `SEC-TRANSIENT`; postur klaim Envelope.
- **agent4 (Hub)**: `manifest_sig` (Ed25519) atas `{schema_version, hub_id, template_sha256, content_version}` (H-02 §2.12 audit) + `attribution` field (H-03).
- **agent7 (registri)**: `trust_anchor {hub_id, kid, pubkey_ed25519, rotasi}` di AGENT7-JIT-ROUTING-REGISTRY v0.2 (posisi agent7 #587 diterima).
- **agent8 (perf)**: angka `size_of` + aturan satu-hasher diukur ulang saat profiling nyata.

## 11. Known-answer test (vektor resmi; §2.11 baris 5 terverifikasi cocok)

```
BLAKE3("")   = af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
SHA256("")   = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```
Wajib ditambah KAT domain-separation (§2.11 baris 7, 9–10): `keyed_hash(ckey,"abc")` ≠ `hash("abc")`; `BLAKE3("abc"‖"X")` = `BLAKE3("ab"‖"cX")` (ambiguitas nyata) sedangkan versi ber-prefix `u32` BE **berbeda**. Reproduksi: `/mnt/extra-storage/a10-work` (cargo run, rustc 1.98.1, blake3 1.8.7).

## 12. Keputusan yang diminta (mengulang D-1…D-4 agar tak hilang di thread)

| ID | Pertanyaan | Rekomendasi | Kepada |
|---|---|---|---|
| D-1 | Algoritma Envelope: BLAKE3 keyed; spill checksum: SHA-256 | pecah T-11 → T-11a/T-11b | matt + pemilik proyek |
| D-2 | Panjang `ContentHash` | 32 B (Envelope/bukti auditor); 16 B boleh (CASD dedup) | matt + agent2 + agent3 |
| D-3 | Sink anchor eksternal | Object Lock **atau** RFC 3161 TSA (pilih sebelum Wave 3) | pemilik proyek |
| D-4 | Sumber `chain_key`/`hub_signing_key` | di luar filesystem host (KMS/HSM) | matt + agent5 |
| D-7 | `chmod 0700 /home/agent10` vs freeze #386 | pengecualian home sendiri atau provisioning pemilik mesin | fern + agent5 |

---

## 13. Deklarasi dua sesi & pembagian lane (integritas atribusi — laporan #628 §5)

- **Sesi pertama (agent10 asli, aktif 06:21 UTC):** penulis `AGENT10-COMPLIANCE-CRYPTO-AUDIT.md` v1.0/v1.1, `AGENT10-PRD3-COMPLIANCE-REVIEW.md` (P3-01…P3-05, #628), pesan #519/#567, posisi #555 (pencabutan Merkle), dan pemegang lane **Item Lineage / crypto-shredding** + review PRD-3.
- **Sesi kedua (saya — sesi yang memulihkan kunci SSH atas permintaan pemilik; menulis v1.2 audit + §2.12, #561/#602/#604/#612/#618/#630/#633, dokumen ini, amandemen SEC):** pemegang lane **Execution Envelope / spec kripto**, **Plt. Security Gatekeeper** (mandat #617/#621), sinkronisasi gate SEC-*.
- **Konsekuensi yang saya akui:** atribusi "agent10" kini memuat dua penulis sampai fern/matt meratifikasi lane (usul (b) #628 §5) atau provisioning `agent11` (usul (c)). Semua dokumen di atas menyebut penulis sesinya; tidak ada file sesi lain yang saya timpa; kunci SSH = satu identitas (K-6, kredensial bersama — keputusan ada di pemilik).
- **Kontrak anti-tabrakan:** setiap rilis dokumen baru oleh sesi mana pun WAJIB `ls -lat` dulu + deklarasi "dokumen X = sesi Y" di header. Sesi pertama menahan diri menyunting file sesi kedua dan sebaliknya.

---

**Status:** PREP — spec siap; tugas `W3-EXEC-ENVELOPE` masih menunggu `W2-STORAGE-L0`. Bantahan dipersilakan dengan baris + timestamp. — **agent10** (ROLE_COMPLIANCE, sesi kedua per §13)
