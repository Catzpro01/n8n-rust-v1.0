# W0-SPILL-TEST — run vs agent2 W0-SPILL-IMPL (#850)

- Impl: `/opt/agent-workspace/rust-engine/crates/storage/src/spill.rs` (302 lines,
  sha256 `71fafa07…` saat run; trait `SpillStore` kernel kanonik a7d0357).
- Harness: v0.2.0, 12 gate, kontrak kanonik (bukan PATCH), `#[tokio::test]`,
  `cargo test --release --offline`.
- Baseline (kode-shared): **11/12 HIJAU**, T-09 MERAH (= F1, di bawah).
- Baseline (salinan-trimmed /tmp/muteng): 11/12 identik → salinan setia.

## Mutasi (6/6 tepat — dibunuh TEPAT oleh covering-set-nya + baseline T-09)

| Mutan | Suntikan | Gate merah (ekspektasi) | Hasil |
|---|---|---|---|
| MU-A | bypass banding-checksum (2 situs) | T-02 | T-02 ✓ |
| MU-B | lewati header.validate (2 situs) | T-04 | T-04 ✓ |
| MU-C | hapus set_permissions 0600 | T-06 | T-06 ✓ |
| MU-D | disk_footprint → selalu 0 | T-01 | T-01 ✓ |
| MU-E | delete = no-op | T-01, T-08 | T-01, T-08 ✓ |
| MU-F | delete Err jika file hilang | T-08 | T-08 ✓ |

Mutasi hanya menyentuh SALINAN (`/tmp/muteng`); kode-shared tak tersentuh
(restore + re-sync per iterasi). Pola patch: `mutate.py` (assert count).

## Temuan

- **F1 (BLOCKING, milik agent2): T-09 MERAH** — drop writer tanpa finish
  membocorkan `<uuid>.spill.tmp`. Langgar trait-doc `SpillWriter` CONTRACT
  ("Dropping a writer … MUST release its partial file (audit A-21)") +
  penerimaan agent10 #814.3(d) "drop no-leak". Perbaikan: `impl Drop` pada
  `FileSpillWriter` yang menghapus `tmp_path` (abaikan error).
- **F2 (SPEC-DEBT, milik fern): delta PATCH-vs-kanonik** — dual-hash handle
  fields, `verify_integrity`, `gc_execution`, varian `ChecksumMismatch`/
  `InvalidMagic`/`OffsetOutOfRange` TAK ADA di kernel kanonik; agent2 tak bisa
  mengimplementasikan yang tak didefinisikan kernel. Putusan: perluas kernel
  (berat — kanonik) atau cabut syarat-PATCH.
- NIT: `spill.rs:13` impor tracing tak terpakai (warning).

## Verdict: CONDITIONAL — 11/12 + bukti-mutasi 6/6; sign-off setelah F1 fix + re-run 12/12.
FINAL: 12/12 GREEN vs spill.rs sha256 5bf974972567be7aeee056190d0f4b5dc1e663c8a4215bc9c9e997c6dd5bc9d2 (F1 Drop-fix landed by agent2); CONDITIONAL lifted.
