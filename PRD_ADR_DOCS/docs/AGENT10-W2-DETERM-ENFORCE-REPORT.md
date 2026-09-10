# W2-DETERM-ENFORCE — Laporan Implementasi (agent10, sesi B)

> **AMANDEMEN (2026-09-09 11:3x, Ruling #1000 matt):** (a) §2 `docs/BAHASA-BERSAMA.md` root = **NON-BINDING** (Ruling 3(a)) — kutipan di laporan ini bersifat HISTORIS (ditulis sebelum #1000), bukan otoritas; tipe kanonik = Ruling 3(c) (Item, ItemList::{Inline,Spilled}, SpilledList, ContentId, SpillStore, Checkpoint, SideEffect, ParameterSchema, PriorOutputs); `BAHASA-BERSAMA-v1.1-matt.md` = register temuan, **bukan kamus** (Ruling 3(d)). (b) Erratum F-1 FINAL: komentar `src/lib.rs` (sha 08a02715…) + README (sha b55669dc…) sudah merujuk tipe nyata & kutipan §2 diganti catatan NON-BINDING (Ruling 6 a/b); grep Ruling 6(c) = NOL hit PayloadRef/SpillId(uuid)/SpillRef di luar crate ini; test 9/9, clippy 0. Baris "kanonik = `uuid::Uuid`" (erratum F-1 §Temuan lama) = **SALAH pasca-#924/#1000** — dibantah Ruling 3(c).


| | |
|---|---|
| **Task** | `W2-DETERM-ENFORCE` (Wave 2, P0, ROLE_SECURITY_QA — diadopsi agent10 Plt.) |
| **Keputusan** | `#867`-3 (agent4 & agent10) `#903`-3 (Mitra C lanjutkan kontrak replay) |
| **Kontrak** | `docs/AGENT10-W2-DETERM-ENFORCE-SPEC.md` v1.1-PREP (implementasi apa adanya) |
| **Standar tipe** | `docs/BAHASA-BERSAMA.md` v1.0.0-CANONICAL (Blake3 `[u8;32]`/`blake3_hex`; SHA-256 = checksum berkas G-C7/#751; `PayloadRef` Inline\|Spilled) |
| **Tanggal** | 2026-09-09 (klaim 10:4x UTC; inti 11:0x UTC) |
| **Artefak** | `/opt/agent-workspace/w2-determ-enforce/` (crate `determ`, shadow worktree SOP Pilar 4) |

---

## 1. Yang diimplementasikan

- **`src/lib.rs`** — format `RecordSet` v1 kanonik: byte BE, u32/u64 panjang-prefix, `SCHEMA_VERSION=1`, `entry` 5 kind (`HTTP_RESPONSE 0x01`, `CLOCK_READ 0x02`, `RNG_OUTPUT 0x03`, `ITER_ORDER 0x04`, `SYS_ENV_READ 0x05`); builder payload kanonik per kind (spec §4.1); `redact_headers()` (kredensial → `[REDACTED-credential]`, kunci lowercase terurut); `body_hash()` BLAKE3 dgn domain separation `CTX_BODY`; `recordset_hash` = BLAKE3(`CTX_RECORD`‖prefix); `recordset_sha256` = SHA-256 berkas; encode/decode fail-closed (`HashMismatch`, `Truncated`, `BadVersion`, `BadKind`); KAT BLAKE3+SHA-256 vektor resmi.
- **`src/verify.rs`** — `Verifier` + tabel policy spec §5: `check_completeness` (fail-closed `MissingReplayRecord`), `check_body` (digest blob Inline; Spilled = verifikasi pemilik blob), `check_clock` (±`CLOCK_WINDOW_MS=300_000` eksplisit → `ClockDrift`), `enforce_deterministic` (`REJECTED_SOURCE_LIST`), `check_metadata_sha256` (`RecordsetHashMismatch`); `ReplayStatus` 8 status (REPLAY_OK/MISMATCH/BROKEN/CLOCK_DRIFT/RNG_UNSEEDED/SEED_DIFF/VIOLATION_LOGGED/REJECTED_SOURCE_LIST).
- **`tests/gates.rs`** — gate G-D1..G-D8 + KAT (semua BERJALAN, bukan klaim).
- **`README.md`** — keputusan D-D1..D-D4 dinyatakan eksplisit (bantahan ≤30 menit, DILEWATI tanpa bantahan dari agent4/agent6/agent2 pada 11:00+).

## 2. Hasil terukur

```
cargo test --release   -> 9 passed; 0 failed   (G-D1..G-D8 + KAT)
cargo clippy --all-targets --release -> 0 warning
forbid(unsafe_code)   -> aktif
```

| Gate | Hasil | Gate | Hasil |
|---|---|---|---|
| G-D1 byte-identik 2× seed sama + REPLAY_OK | ✅ | G-D5 +299 s OK; +301 s → CLOCK_DRIFT | ✅ |
| G-D2 entry hilang → MissingReplayRecord (FAIL) | ✅ | G-D6 urutan header ≠ ubah digest; K2/K6 dicatat jujur | ✅ |
| G-D3 1 byte body rusak → ReplayBodyMismatch | ✅ | G-D7 0 plaintext kredensial; header auto-redact | ✅ |
| G-D4 NON_DETERMINISTIC dipaksa → REJECTED | ✅ | G-D8 tamper seed/metadata → HashMismatch | ✅ |
| KAT BLAKE3("")/"abc" + SHA-256("")/"abc" | ✅ | | |

## 3. Keputusan kontrak — DINYATAKAN (spec §9)

- **D-D1 = YA dual**: BLAKE3 = digest RecordSet (CTX_RECORD); SHA-256 = checksum berkas. Satu primitif per tujuan (T-11a/T-11b, BAHASA-BERSAMA).
- **D-D2 = FAIL (fail-closed)** — hilang/rusak ⇒ error eksplisit; tidak pernah fallback diam-diam (budaya #751).
- **D-D3 = N = 300 s** eksplisit (bukan toleransi senyap).
- **D-D4 = TIDAK** — replay = reproduksibilitas, BUKAN bukti audit; hanya Envelope+anchor (ADDENDUM-A1, C-02).

## 4. Integrasi lintas agen (contract-first, Pilar 3 — path miliknya TIDAK disentuh)

- **@agent4/@agent6 (Mitra C, `AGENT4-AGENT6-DETERM-REPLAY-INTERFACE.md` v0.1):** crate ini = sisi kontrak RecordSet/Verifier; kontrak replay runtime = dokumen mereka — dua sisi satu jembatan, saya sinkron saat interface v0.1 (sudah dibaca; lihat catatan §5).
- **@agent1 (W3-TIMELINE-REPLAY):** dep terpenuhi setelah DONE ini → claim boleh; Q3 retensi (spek #910) saya jawab terpisah.
- **@agent2:** blob RecordSet = spill 0o600 + `SpillRef` (BAHASA-BERSAMA) — kontrak, bukan tulis ke storage.
- **@agent3:** RNG_OUTPUT = counter saja; `Date.now()`/`Math.random()` interception via EBC (kontrak W1 §3.2).
- **@agent8:** biaya transien sudah terkendali desain (payload streamed, counter 8B; `size_of` per Entry saat integrasi).

## 5. Batas jujur (tidak disembunyikan)

1. Replay ≠ bukti audit (spec §1) — jangan pernah dijual sebagai tamper-evidence.
2. Body TIDAK inline (spec §4.1); `Spilled` menyerahkan verifikasi digest ke pemilik blob — dicatat di README.
3. Fixture-only sekarang (tanpa engine) — 7-workflow RUNNABLE = gate integrasi pasca-hook engine (agent1/agent4).
4. K2/K6 (float presisi, UTF-16) TIDAK diklaim byte-identik lintas runtime — ranah DEVIATION-CATALOG (agent5); dicatat, bukan disembunyikan.
5. `SpillId` standalone = newtype String (hex UUID); kanonik = `uuid::Uuid`; kontrak serialisasi `{"type":"SpillRef","id":...,"bytes":N}` sama.

**Status: DONE (diverifikasi 9/9 + clippy 0 + forbid unsafe); handoff #896; menunggu review independen @matt (syarat merge #865: test hijau ✅, clippy 0 ✅, review independen ⏳).** — agent10 (ROLE_COMPLIANCE / Plt. ROLE_SECURITY_QA, sesi B)

---

## ERRATUM F-1 (per review independen #956, sesi A) — 2026-09-09 11:2x

- **Temuan:** komentar `src/lib.rs:67-69` + README §5 butir 4 menyebut "pada crate kanonik `SpillId` = `uuid::Uuid`" — bertentangan dengan fakta terverifikasi #924 (kernel TIDAK punya uuid; gate Cargo 4-dep; tipe nyata `ItemList::Spilled(SpilledList{SpillPath,len,total_bytes,codec})` + `ContentId(u64)`). Kelas C-05 (nama ≠ isi; dokumen lebih lemah/lebih kuat dari kode).
- **Fix (bukan kode fungsional):** komentar + README diperbarui merujuk tipe kernel nyata; kontrak serialisasi mengikuti #940 T-2 (nilai kanon `"inline" | "spilled"`).
- **Verifikasi ulang setelah fix:** `cargo test --release` = 9/9 PASS; `clippy --all-targets --release` = 0 warning. sha256 baru: `src/lib.rs` d279dec6…, `README.md` 77237830….
- Reviewer: terima kasih @agent10 (sesi A) — temuan tepat; verdict APPROVED-WITH-1-DOC-FIX kini terpenuhi penuh.


## AMANDEMEN-2 — TEMUAN-1/2 (#1033) & Ruling 10-12 (#1073) — 2026-09-09 11:5x
- TEMUAN-1: duplikat `lib.rs` di root crate (13.029 B, teks pra-perbaikan) DIHAPUS — kargo memakai `src/lib.rs`.
- TEMUAN-2: nama lokal fiktif-§2 dihilangkan sebagai vektor kebingungan: `PayloadRef`→`RecordBody`, `SpillId`→`SpillTag` (identifir Rust saja; **wire JSON + tag `"inline"|"spilled"` per #940 T-2 tidak berubah**; tipe lokal = SAH per matt #1073). 9/9 hijau, clippy 0. sha: src/lib.rs `6930bbe692d0222df8fb9d79b7f2f3ddd9a82c074be541fceac0dd59c0b22c63`, src/verify.rs `54d25355131d5f0ca3b2bde3969a0e154e46bab852dc3ff6f9ced0185b94ad9b`, tests/gates.rs `6b3f76c39155d0aae9ad2c6913fa1106a3f20862ff4329e25313f441b084d4ed`, README.md `12e523a79b08a663db5613f555487760276f6d9f2d6ae44827abc9b75785fc0f`.
- PEMETAAN (norm vs lokal): `RecordBody::Inline(Vec<u8>)` ≈ konten inline (kernel: `ItemList::Inline(Vec<Item>)`, bentuk beda lapisan); `RecordBody::Spilled(SpillTag)` ≈ penanda "payload di spill"; kernel padanan struktural = `ItemList::Spilled(SpilledList{path: SpillPath, len, total_bytes, codec})` — `SpillTag` hanyalah label wire lokal, BUKAN `SpillPath`.
- Catatan Ruling 10: uuid untuk nama berkas spill di rust-engine → ganti ke `ContentId` (keputusan port agent1/agent2); tidak menyentuh crate ini.
