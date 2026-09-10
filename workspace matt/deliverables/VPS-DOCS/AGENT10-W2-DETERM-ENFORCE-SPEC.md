# AGENT10 — W2-DETERM-ENFORCE: PENEGAKAN DETERMINISME VIA RECORD-REPLAY ENGINE
## Deliverable tugas `W2-DETERM-ENFORCE` (Wave 2, P0, ROLE_SECURITY_QA — dipegang agent10 Plt. per mandat #617/#621) — v1.0-PREP

| | |
|---|---|
| **Versi** | v1.1-PREP — v1.0 + **serapan keputusan 07:56–07:59**: R1/R2 disahkan (#742 — peak terukur ≤ 4 KB, sekuensial ATAU paralel), dual-hash interim SHA-256+BLAKE3 (W0-CHECKSUM-AGREE, #735/#737/#739), T-11a/T-11b terkunci. Menunggu W2-STORAGE-L0. |
| **Penulis** | agent10 — ROLE_COMPLIANCE, **Plt. ROLE_SECURITY_QA** (mandat #617/#621; agent5 limitasi sesi) |
| **Basis** | W1-DETERM-SPEC (agent4, `AGENT4-DETERMINISM-CONTRACT-SPEC.md` v1.1, DONE); DETERMINISM-SOURCE-INVENTORY (agent5); DEVIATION-CATALOG §6 K1–K10; PRD-3 v3.0.0-FINAL L2/L3; C-04 audit (kelas DETERMINISTIC vs OBSERVATIONAL) |
| **Aturan** | Nol kode (freeze #586); dokumen ini = kontrak, implementasi pasca sign-off |
| **Status pengerjaan** | W1-DETERM-SPEC DONE (#635); W2-STORAGE-L0 = dependensi terbuka untuk agent2 (saya TIDAK merebutnya — prinsip #600/#601) |

---

## 1. Tujuan & batas ranah (W1 vs W2)

| | W1-DETERM-SPEC (agent4, DONE) | **W2-DETERM-ENFORCE (saya)** |
|---|---|---|
| Pertanyaan | **Bagaimana MENENTUKAN** kelas determinisme workflow (DETERMINISTIC / CONDITIONAL / NON_DETERMINISTIC) + kontrak per-workflow | **Bagaimana MENEGAKKAN** saat runtime: record & replay input eksternal/clock/RNG + verifikasi byte-identik + kebijakan kegagalan |
| Output W1 | DETERMINISM REPORT (per import, JSON) | **RecordSet + Recorder/Replayer/Verifier contract + gates + policy** |
| Output bersama | DETERMINISM RECORD (agent1 §6.2.1) — record digest wajib masuk di sini (lihat §6) | |

**Batas jujur (wajib dibaca sebelum mengutip):** record-replay membuktikan **reproduksibilitas**, **bukan keaslian**. Penyerang ber-root yang dapat menulis RecordSet + menjalankan ulang menghasilkan output byte-identik palsu — replays **tidak pernah** diklaim sebagai bukti audit. Bukti audit = Execution Envelope (spec terpisah; lihat ADDENDUM-A1: hanya anchor eksternal yang menjamin). Jangan mencampur dua domain ini (C-02: domain separation).

## 2. Prinsip

1. **Fail-closed, bukan warn-only**: record hilang/rusak saat eksekusi deterministik → **GAGAL** dengan alasan eksplisit (`MissingReplayRecord`), bukan fallback diam-diam ke nilai baru.
2. **Byte-identik = target**: dua run seed sama → output kanon identik (paritas gate L3 agent5).
3. **Verdict jujur**: `NON_DETERMINISTIC` + diminta paksa → tolak (W1 §3.3) — ditegakkan, tidak hanya dilaporkan.
4. **Yang direkam minimal**: hanya apa yang benar-benar nondeterministik (eksternal input + clock + RNG + urutan iterasi), bukan seluruh log — biaya & permukaan PII terkendali.
5. **Domain separation** (C-02): RecordSet digest ≠ Envelope digest ≠ checksum spill — satu BLAKE3 dengan konteks berbeda `CTX_RECORD = "n8nrust/determinism/recordset/v1"`.
6. **Zero-assumption**: semua policy dinyatakan eksplisit; tidak ada "biasanya", "wajar", "tidak mungkin terjadi".

## 3. Komponen kontrak (trait-level; nol implementasi)

```
Recorder   : intercept external_points + clock + RNG + iterasi (per node) → tulis RecordSet
Replayer   : serve RecordSet pada run kedua (seed sama) → nilai yang DIKUNCI
Verifier   : (1) RecordSet digest vs determinism-record digest; (2) output_eq kanon 2 run;
             (3) violation log → receipt; (4) status { REPLAY_OK | REPLAY_MISMATCH | REPLAY_BROKEN | CLOCK_DRIFT | RNG_UNSEEDED }
Engine hook: titik masuk = 2 mode per W1: normal (violation log) & deterministic (kunci semua)
```

**Kontrak recorder → storage (agent2, menunggu spec final):** RecordSet disimpan sebagai blob biner (streaming, seperti spill) + metadata di tabel `determinism_record (exec_id, workflow_id, seed, recordset_sha256 B32, status, created_at)`. `recordset_sha256` = digest RecordSet kanonik. (SHA-256 untuk checksum file blob = konsisten keputusan spill; BLAKE3 untuk digest yang masuk record — setara T-11a/T-11b, jangan dirancukan.)

## 4. Format RecordSet v1 (kanonik)

```
RecordSet_v1 ::=
  schema_version    u8 = 1
  exec_id           [u32 len][bytes]
  workflow_sha      bytes[32]
  seed              bytes[8]                    ; 0 = unseeded (non-deterministic mode)
  entries           [u32 count] Entry*
  recordset_hash    bytes[32] = BLAKE3(CTX_RECORD, byte stream SEBELUM field ini)

Entry ::=
  kind              u8      ; 0x01=HTTP_RESPONSE 0x02=CLOCK_READ 0x03=RNG_OUTPUT 0x04=ITER_ORDER 0x05=SYS_ENV_READ
  node_id           [u32 len][bytes]
  seq               u64 BE  ; urutan dalam eksekusi (pinjam disiplin seq Envelope)
  payload           [u32 len][bytes]            ; kanon per kind (lihat §4.1)
  captured_at       u64 BE  ; wall-clock capture (metadata only — tidak pernah jadi nilai replay)
```

### 4.1 Kanon payload per kind
- **HTTP_RESPONSE**: `{url, method, status, headers[tersusun: key lowercase sorted], body_hash B32, body_len u64}` — **body TIDAK disimpan inline** (PII & ukuran; lihat §7). Replay: verifikasi `body_hash` dari blob referensi; mismatch → `REPLAY_BROKEN`.
- **CLOCK_READ**: `{now_ms_utc u64, tz_scope}` — nilai jam yang dikunci.
- **RNG_OUTPUT**: `{consumed u64}` (counter) + seed reconstructible → tidak menyimpan output RNG, hanya counter.
- **ITER_ORDER**: `{item_ids: [u32 len][bytes] utk tiap batch}` — urutan iterasi yang dikunci.
- **SYS_ENV_READ**: `{key, value_hash B32}` (env = secret → tidak pernah inline).

**Aturan kanonisasi:** `u32` BE untuk panjang; `u64` BE; array di-sort bila semantiknya set (headers); tidak ada `Display`/JSON bebas di digest. Satu fungsi encode, satu test.

## 5. Policy enforcement (eksplisit, falsifiable)

| Skenario | Perilaku WAJIB | Status dilaporkan |
|---|---|---|
| Record hilang utk external_point | **FAIL** `MissingReplayRecord` (tidak fetch ulang diam-diam) | REPLAY_BROKEN |
| Record digest ≠ determinism-record digest | **FAIL** `RecordsetHashMismatch` | REPLAY_BROKEN |
| Body hash mismatch (blob berubah) | **FAIL** `ReplayBodyMismatch` | REPLAY_MISMATCH |
| Seed beda pada run ke-2 | diizinkan, tapi WAJIB dicatat (output = run baru; BUKAN replay) | RNG_UNSEEDED / SEED_DIFF |
| Clock melampaui jendela ±N saat replay | **FAIL** `ClockDrift` (N = 300 s, SATUAN EKSPlisit) | CLOCK_DRIFT |
| Violation access clock/RNG liar di mode normal | **CATAT** (bukan gagal) → receipt `violation[]` | VIOLATION_LOGGED |
| NON_DETERMINISTIC + diminta deterministic | **TOLAK** dengan daftar sumber (W1 §3.3) | REJECTED_SOURCE_LIST |

## 6. Integrasi (kontrak lintas agen)

- **agent1 (determinism-record §6.2.1):** `(template_sha256, content_version, recordset_sha256, seed)` = field wajib record — turunan langsung dari prinsip yang sudah disepakati di H-01/agent1 #536; **recordset_sha256** = jembatan replay ↔ determinisme.
- **Envelope (saya, spec terpisah):** entry membawa `class_flags bit0 = HAS_UNVERIFIED_EXTERNAL_IO`; `input_digest` = digest input kanon SEBELUM replay, **bukan** digest RecordSet (keduanya di-hash dalam entry — lihat §3.1 Envelope: field `input_digest`/`output_digest` dinegerai dari payload; RecordSet digest masuk `determinism_record`, TIDAK sebagai pengganti `input_digest`). Awas: jangan pernah mengganti `input_digest` dengan `recordset_sha256` — dua makna berbeda (C-02).
- **agent3 (QuickJS):** interception `Date.now()`/`Math.random()` deterministic-mode via bytecode/EBC — titik tunggal, tidak ada jalur kedua (kontrak W1 §3.2).
- **agent9 (HTTP):** RecordSet kind HTTP_RESPONSE dihasilkan dari jalur ingress yang sama (ETag/If-None-Match → `captured_at`); katalog source (hub://intel-types) tidak berubah.
- **agent8:** biaya RecordSet = BLOB streamed (tidak materialisasi penuh) → ≤ batas transien; angka `size_of` per Entry diukur, bukan asumsi (disiplin ERR-029).

## 7. Keamanan & GDPR (ranah gatekeeper)

1. **RecordSet berisi data pribadi potensial** (respons API publik berisi alamat/email, header). Diperlakukan seperti spill: mode `0600`, direktori per-instance, tidak pernah masuk log teks.
2. **Retensi & penghapusan (Art. 17):** RecordSet ikut kebijakan retensi `retain-outputs`; penghapusan = hapus blob + tandai `recordset_sha256 = NULL` di determinism-record (ubah status menjadi `RETENTION_ERASED`) — digest di Envelope TIDAK berubah (buffer: Envelope tidak bergantung pada RecordSet untuk verifikasinya).
3. **Tidak ada kredensial di RecordSet:** header auth/signature DI-REDACT (hash dengan context `REDACTED` + flag) — gate: sekuel test memastikan 0 kredensial di konten RecordSet (G-D7).
4. **Replay ≠ bukti audit** (lihat §1): jangan pernah menjual replay sebagai tamper-evidence.

## 8. Gates (falsifiable; CI — usul masuk matriks QA sebagai G-D1…D8)

| Gate | Uji | Lulus bila |
|---|---|---|
| G-D1 | run deterministik 2× seed sama (7 workflow RUNNABLE, subset exec-diff) | output kanon byte-identik 7/7; `REPLAY_OK` |
| G-D2 | hapus 1 RecordSet entry | `REPLAY_BROKEN` + `MissingReplayRecord` (FAIL, bukan fetch ulang) |
| G-D3 | ubah 1 byte body blob | `ReplayBodyMismatch` (digest/body hash menangkap) |
| G-D4 | NON_DETERMINISTIC dipaksa deterministic | ditolak + daftar sumber (return `REJECTED_SOURCE_LIST`) |
| G-D5 | clock digeser +N pada replay | `ClockDrift` (dalam N=300 s) |
| G-D6 | output_eq lintas runtime — kelas K2/K6 (angka, UTF-16/8) | tercatat di violation/deviation catalog; **tidak** diklaim byte-identik bila deviasi lintas-runtime (jujur) |
| G-D7 | scan RecordSet konten | TODO 0 plaintext kredensial; headers auth redacted |
| G-D8 | `recordset_sha256` ≠ digest (ubah metadata) | `RecordsetHashMismatch` |

Reproduksi & satuan: dapat dijalankan pra-sign-off HANYA jika ada fixture record kecil buatan (tanpa engine) — untuk itu butuh storage (W2-STORAGE-L0) → dependensi tetap.

## 9. Keputusan yang diminta

| ID | Pertanyaan | Rekomendasi | Kepada |
|---|---|---|---|
| D-D1 | RecordSet: hash BLAKE3 (digest record) + SHA-256 (checksum blob) | **YA** — konsisten T-11a/T-11b; satu primitif per tujuan | agent2 + matt |
| D-D2 | Replay Miss policy: FAIL vs WARN | **FAIL (fail-closed)** — warn membuat replay "byte-identik" tidak dapat dipercaya | matt + pemilik |
| D-D3 | Jendela clock replay N | **300 s** — di atasnya = ClockDrift, bukan toleransi senyap | agent3 + agent8 |
| D-D4 | Replay sebagai bukti audit? | **TIDAK** — hanya Envelope (anchor) yang bisa diklaim; replay = reproduksibilitas (A1.4) | pemilik |

---

**Status:** PREP — kontrak lengkap; dep `W2-STORAGE-L0` (agent2) & sinkronisasi agent1/agent3/agent9. Bantahan: baris + timestamp. — **agent10** (ROLE_COMPLIANCE / Plt. ROLE_SECURITY_QA)
