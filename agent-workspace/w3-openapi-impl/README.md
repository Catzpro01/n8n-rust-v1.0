# W3-OPENAPI-IMPL — openapi-codegen + nodes-openapi

**Pemilik:** agent9 (ROLE_INTEGRATION) · **Task:** W3-OPENAPI-IMPL (queue, fern #1074 §2) · **Spec acuan:** `docs/AGENT9-INTEGRATION-SPEC.md` v0.5.3 (sha256 `043af9c9…`, konsensus 2/2: agent4 #1017 + agent1 #1061; ruling matt #1116)

## Struktur

- `openapi-codegen/` — generator build-time. Pipeline INGEST→VALIDATE→TRIAGE→TRANSLATE→EMIT. v0.1 **tanpa HTTP client** (Ruling matt #1116); request kelak via trait kernel `HttpClient` (`context.rs:433`).
- `nodes-openapi/` — target EMIT. Domain eksklusif agent9 (SOP #864 Pilar 2). Semua tipe = tipe kernel nyata (`NodeDescriptor`/`ParameterSchema`) — tidak ada vocab paralel (R-A agent4).
- `specs/` — golden specs: `frankfurter.openapi.json` (9.777 B, sha `e3880496…`), `github-rest.openapi.json` (12.927.758 B, sha `531b0574…`).
- `tests/fixtures/` — rejection corpus M1 (kode CG-E eksak).
- `docs/EVIDENCE-BINANCE-SPEC-WITHDRAWAL.md` — bukti penarikan spec Binance (diminta matt #1116).

## Aturan yang diikuti (audit trail)

| Aturan | Sumber |
|---|---|
| Kernel kanonik READ-ONLY via path-dep | matt #1116 butir 2 |
| Workspace members sebelum test | matt #1116 butir 3 (= Ruling 10 b5) |
| `sha2 = "=0.10.8"` | matt #1116 butir 4 (preseden Ruling 7) |
| no-uuid; identifier via `ContentId(u64)` kernel | Ruling 10 b4 |
| digest framing u32-BE versi (C-02) | #988/#990 |
| `#![forbid(unsafe_code)]`, clippy 0 warning | syarat merge matt #865 |
| Tanpa HTTP client di v0.1 | matt #1116 butir 4 |
| Sort kanonik `urls[]`/list sebelum digest | OPEN-INTEG-4 (agent1 #1038) |
| INTEG-06 metric `+1` (duplikat pihak-kalah) | agent1 #1049 |

## Milestone

- **M1 (ini):** INGEST + VALIDATE — sha256 identitas, cek versi, servers (CG-E-102), operationId derive/duplikat (CG-E-103), sort kanonik.
- M2: TRIAGE + TRANSLATE (tabel §1.2 17 baris, serialisasi §1.3, rejection CG-E-201..205/301/302).
- M3: EMIT kode Rust deterministik + manifest verifikasi `{spec_sha256, hash_domain, ops_emitted, ops_rejected}`.
- M4: INTEG-01 golden diff + INTEG-02 rejection corpus ≥20 + INTEG-03 translation pairs.
- M5: pengukuran peak-RAM (GitHub 12,9 MB) + clippy + laporan.

Build: `CARGO_TARGET_DIR=/mnt/extra-storage/cargo-target cargo test --workspace`
