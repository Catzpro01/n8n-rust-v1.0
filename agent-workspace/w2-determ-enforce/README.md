# determin — W2-DETERM-ENFORCE `RecordSet` v1 (Record-Replay Enforcement)

**Task:** `W2-DETERM-ENFORCE` (Wave 2, P0, ROLE_SECURITY_QA) — agent10 (Plt. ROLE_SECURITY_QA, mandat #617/#621; klaim swarm-task 2026-09-09 10:4x UTC).
**Kontrak:** `docs/AGENT10-W2-DETERM-ENFORCE-SPEC.md` v1.1-PREP (spec §1–§9) — diimplementasikan apa adanya.
**Standar tipe:** tipe kernel NYATA per Ruling 3(c) #1000 (Item, ItemList::{Inline,Spilled}, SpilledList, ContentId, SpillStore, PriorOutputs); Blake3 = `[u8;32]`/`blake3_hex` 64-hex; checksum berkas = SHA-256 per G-C7/#751. §2 `docs/BAHASA-BERSAMA.md` root = NON-BINDING (Ruling 3(a) #1000), TIDAK dikutip sebagai otoritas tipe.
**Lokasi (SOP #864 Pilar 4 — shadow worktree):** `/opt/agent-workspace/w2-determ-enforce/` — merge ke `kernel-asli-d3bcff0/crates/determ/` = pintu single-gatekeeper @matt (syarat #865: test hijau + clippy 0 + review independen).

## Keputusan kontrak (spec §9 D-D1..D-D4) — dinyatakan + siap dibantah 30 menit

| ID | Pertanyaan | Keputusan | Dasar |
|---|---|---|---|
| D-D1 | RecordSet: BLAKE3 (digest) + SHA-256 (checksum blob) | **YA — dual, satu primitif per tujuan** | T-11a/T-11b, G-C7 (#751) |
| D-D2 | Replay miss policy | **FAIL (fail-closed)** | #751 budaya fail-closed; warn membuat replay tak dipercaya |
| D-D3 | Jendela clock replay N | **300 s** (eksplisit, bukan toleransi senyap) | spec §5; disiplin ERR-029 |
| D-D4 | Replay = bukti audit? | **TIDAK** — hanya Envelope+anchor | ADDENDUM-A1, C-02 domain separation |

## Implementasi

- `src/lib.rs` — format `RecordSet` v1 (BE, u32/u64, tidak ada Display/JSON bebas), EntryKind (HTTP_RESPONSE/CLOCK_READ/RNG_OUTPUT/ITER_ORDER/SYS_ENV_READ), builder payload kanonik, `redact_headers` (G-D7), `body_hash` (CTX_BODY), `recordset_hash` = BLAKE3(CTX_RECORD‖prefix), `recordset_sha256`, KAT BLAKE3+SHA-256.
- `src/verify.rs` — `Verifier` + tabel policy §5: completeness (fail-closed), body-hash, clock window ±300 s, reject NON_DETERMINISTIC forced, metadata sha256; `ReplayStatus` (REPLAY_OK/MISMATCH/BROKEN/CLOCK_DRIFT/RNG_UNSEEDED/SEED_DIFF/VIOLATION_LOGGED/REJECTED_SOURCE_LIST).
- `tests/gates.rs` — **G-D1..G-D8 semua dijalankan** (lihat Hasil).

## Hasil verifikasi (diukur, bukan diklaim)

```
cargo test --release   -> 9 passed; 0 failed (G-D1..D8 + KAT; clippy 0 warning)
cargo clippy --all-targets -> 0 warning
```

| Gate | Uji | Hasil |
|---|---|---|
| G-D1 | 2× run seed sama → byte-identik + REPLAY_OK | ✅ |
| G-D2 | hapus 1 entry → MissingReplayRecord / REPLAY_BROKEN (FAIL, bukan refetch) | ✅ |
| G-D3 | 1 byte body berubah → ReplayBodyMismatch / REPLAY_MISMATCH | ✅ |
| G-D4 | NON_DETERMINISTIC dipaksa → REJECTED_SOURCE_LIST | ✅ |
| G-D5 | +299 s OK; +301 s → CLOCK_DRIFT (N=300_000 ms eksplisit) | ✅ |
| G-D6 | urutan header ≠ ubah digest; K2/K6 (float/UTF-16) TIDAK diklaim byte-identik lintas runtime (catat, bukan sembunyi) — ranah DEVIATION-CATALOG agent5 | ✅ jujur |
| G-D7 | scan byte RecordSet → 0 plaintext kredensial; header auth di-redact | ✅ |
| G-D8 | ubah seed tanpa recompute → HashMismatch; metadata sha256 salah → RecordsetHashMismatch | ✅ |
| KAT | BLAKE3("")/"abc" + SHA-256("")/"abc" = vektor resmi | ✅ |

## Integrasi lintas agen (contract-first, Pilar 3 — MENUNGGU, bukan menulis di path mereka)

- **@agent4 (W1-DETERM-SPEC):** verdict enums `DETERMINISTIC|CONDITIONAL|NON_DETERMINISTIC` sudah diserap; `determinism_record` perlu field `recordset_sha256` (+`seed`, `content_version`) — usul kolom SQL siap.
- **@agent1 (kernel/executor §6.2.1):** Engine hook = 2 mode (normal → violation log; deterministic → kunci semua). Trait `Recorder/Replayer/Verifier` di crate ini = titik kontrak.
- **@agent3 (QuickJS):** interception `Date.now()`/`Math.random()` deterministic-mode via EBC (`RNG_OUTPUT`, `CLOCK_READ`).
- **@agent9 (HTTP):** `HTTP_RESPONSE` dihasilkan dari jalur ingress sama (ETag/If-None-Match → captured_at).
- **@agent2 (storage):** blob RecordSet = spill mode 0600; `SpillRef` kontrak wire (#940 T-2); metadata tabel `determinism_record`.
- **@agent8:** biaya transien RecordSet = streamed; `size_of` per Entry diukur saat integrasi.

## Batas jujur

1. Record-replay membuktikan **reproduksibilitas**, bukan keaslian; replay ≠ bukti audit (spec §1).
2. Body tidak disimpan inline (spec §4.1) — verifikasi digest via blob (Inline/Spilled); `Spilled` menyerahkan verifikasi digest ke pemilik blob (storage).
3. Fixture-only saat ini (tanpa engine) — sesuai spec §8 pra-engine; G-D1 memakai 2 run dari fixture yang sama (byte-identik), bukan eksekusi engine penuh (7 workflow RUNNABLE = gate integrasi pasca-hook engine).
4. (erratum F-1, #956) `SpillTag` di crate ini = penanda payload spill (bukan kanonik); tipe KANONIK kernel: `ItemList::Spilled(SpilledList{path: SpillPath, len, total_bytes, codec})` + `ContentId(u64)` — NOL `uuid` di kernel (gate 4-dep, #924). Kontrak serialisasi mengikuti #940 T-2 (nilai kanon "inline"|"spilled").

— agent10 (ROLE_COMPLIANCE / Plt. ROLE_SECURITY_QA)
