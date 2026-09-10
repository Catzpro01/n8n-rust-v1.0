# MERGE REQUEST — W3-OPENAPI-IMPL (openapi-codegen + nodes-openapi)

**Pemilik:** agent9 (ROLE_INTEGRATION) · **Tanggal:** 2026-09-09 · **Status:** SIAP REVIEW MERGE (gate matt #865)
**Task queue:** W3-OPENAPI-IMPL (IN_PROGRESS, fern #1074 §2) · **Spec:** AGENT9-INTEGRATION-SPEC v0.5.3 (konsensus 2/2)

## 1. Ringkasan deliverable

| Artefak | Isi |
|---|---|
| `openapi-codegen/` | Generator build-time: INGEST→VALIDATE→TRIAGE→TRANSLATE→EMIT; CLI `--spec --translate --emit <dir>`; kode rejection CG-E-101..302 + CG-E-104 (depth guard Ruling 39b) |
| `nodes-openapi/` | 1.211 `NodeDescriptor` (frankfurter 5 + github_rest 1.206) memakai **tipe kernel nyata** (R-A agent4: nol vocab paralel); manifest verifikasi per vendor |
| `specs/` | Golden: frankfurter.openapi.json (e3880496…), github-rest.openapi.json (531b0574…) |
| `tests/fixtures/` | 26 fixture (21 rejection corpus + pasangan positif) |
| `docs/EVIDENCE-BINANCE-SPEC-WITHDRAWAL.md` | Bukti penarikan spec Binance + gap exchange-auth |

## 2. Status gate merge matt #865

| Syarat | Status | Bukti reproduksi |
|---|---|---|
| 100% test hijau | ✅ **59/59** (33 lib + 3 golden + 23 pairs/corpus-22) | `CARGO_TARGET_DIR=/mnt/extra-storage/cargo-target cargo test --workspace` |
| clippy 0 warning | ✅ 0 | `cargo clippy --workspace --all-targets` |
| Review independen | 🟡 1/2 — **agent1 APPROVE** (#1382: tipe kernel 100% terverifikasi, R40-bersih, aritmetika 59 tepat; 4 catatan dijawam #1386: R42 bukti 3-baris, angka depth 9/21, komitmen R41 switch ke fn kernel saat landing, rapikan path-dep/pin saat landing). Menunggu agent4 (EMIT). | — |
| Verifikasi agent10 | 🟡 berjalan — re-verifikasi tree 8a6b26f2 (#1413): kepatuhan pin/SHA/f39b7fe ✓, logika 59/59 ✓ di salinan path-fixed, **FINDING-1** (test depth menulis `/tmp/depth-{n}.json` path tetap → verifier non-owner 58/59 PermissionDenied) → **FIXED**: path unik per-proses + cleanup (`ingest.rs` test); menunggu re-run agent10. | `git diff`/SHA256SUMS |
| Ratifikasi matt | ✅ **Ruling 46 (#1408)** — RFC v0.1 EXEC diterima sebagai arah = ratifikasi dual-mode; (a) servers tanpa pemilihan diam-diam + (b) size-gate governor-settable/inkremental/spill = perubahan wajib → **sudah diterapkan di RFC-EXEC v0.2**; (c) pemetaan ke taksonomi §3.4 merged (89fdb3b). | `docs/AGENT9-RFC-OPENAPI-EXEC.md` v0.2 |
| `#![forbid(unsafe_code)]` | ✅ kedua crate | — |
| Workspace members | ✅ sebelum test (ruling #1116 b3) | `Cargo.toml` root |
| Kernel read-only | ✅ path-dep, nol edit kanonik | `nodes-openapi/Cargo.toml` + `openapi-codegen/Cargo.toml` |
| R41 kutover depth-guard | ✅ `check_depth` transisional DIHAPUS → `kernel::json::json_depth_exceeded(bytes, MAX_JSON_DEPTH)` (kanonik HEAD 5bc8ea8; otorisasi fern #1469; komitmen #1386 TEPATI). Batas 64 tetap kebijakan konsumen. 59/59 pasca-kutover. | `openapi-codegen/src/ingest.rs:26-39` |
| **COMMIT rust-engine** | ✅ **HEAD `077b4f3`** (lane-owner, Ruling 54g; 47 file, pesan bentuk-50g, manifest `dd5e36e1…`; pasca-commit HEAD hijau 59/59; sisa porcelain = lane lain) | `git log -1` |
| Kutover di salinan rust-engine | ✅ instruksi matt #1478-Papan + direktif fern #1526-5 dieksekusi: KEDUA crate lane terisi sumber pasca-kutover byte-identical (`openapi-codegen` 8 src + pairs + specs + 26 fixtures; `nodes-openapi` lib + generated ×2 vendor + manifest + golden) via workspace kernel-dep; **59/59 (33+23+3) + clippy 0 di rust-engine**; 50h dipatuhi pra-setiap-build. Pin serde_json workspace (1.0.114, empiris lolos) menunggu putusan matt. | `#1515`, `#1526-5` |
| Tanpa HTTP client / uuid / tokio | ✅ (ruling #1116 b4) | `Cargo.toml` deps: serde_json + sha2 =0.10.8 saja |

## 3. Gate INTEG (spec §4)

- **INTEG-01 golden diff** ✅ — regenerasi byte-identical vs artefak ter-commit (frankfurter + github 1,42MB); drift = fail + instruksi regenerate.
- **INTEG-02 rejection corpus** ✅ — **22 kasus** (≥20) semua gagal dengan kode CG-E eksak + format pesan spec §1.4; termasuk depth-batas Ruling 39b (64 diterima / 65 → CG-E-104).
- **INTEG-03 translation pairs** ✅ — tiap baris SUPPORTED tabel §1.2 punya pasangan terima (kind eksak) + polan tolak berpasangan.
- **INTEG-05 zero-credential-leak** ✅ parsial — grep artefak: 0 literal rahasia (kredensial hanya via `CredentialSpec` generik); scan runtime menyusul saat jalur eksekusi ada.

## 4. Angka pengukuran (M5)

- Peak-RAM pipeline penuh (12,9MB spec GitHub): **84 MB** — di bawah cap 500 MB.
- Registry runtime: 1.211 NodeDescriptor ter-build & teruji.
- Durasi test workspace: ~10s.

## 5. Temuan engineering (untuk engine / agent1)

1. Parse JSON 12,9MB **meluapkan stack thread 2MiB default** (test-thread libtest) — fixed via stack eksplisit 32MiB; catatan: parser dokumen besar runtime perlu stack >2MiB atau streaming.
2. Literal `vec![]` 1.206 elemen juga overflow di debug codegen — struktur EMIT diubah fungsi-per-node (pola codegen standar).
3. Header artefak memakai basename kanonik → golden reproducible lintas cwd.

## 6. Keputusan ratifikasi matt — ✅ SELESAI (Ruling 46 #1408)

Dual-mode translasi (#1168): STRICT fail-loud (default; INTEG-02) vs COLLECT (kegagalan level-operasi → `ops_rejected` manifest; dipakai INTEG-01 golden GitHub: 1.206 OK + 19 ditolak tercatat). **Diratifikasi matt #1408** sebagai bagian jawaban RFC-EXEC: "Empat jawaban di atas adalah ratifikasi dual-mode yang Anda tunggu." Perubahan wajib (a)+(b) diterapkan di RFC-EXEC v0.2; kode tetap menunggu agent4 + agent10.

## 7. Batasan jujur (bukan scope v0.1)

- Jalur EKSEKUSI (request nyata via trait kernel `HttpClient` context.rs:433) = task lanjutan (post-merge).
- N1: array-of-string → Json opaque sampai varian kernel string-array (agent1+matt).
- Gap cakupan exchange-auth (Binance menarik spec — lihat EVIDENCE).
- OAuth2 → kurasi manual (F-C5, konsensus).
