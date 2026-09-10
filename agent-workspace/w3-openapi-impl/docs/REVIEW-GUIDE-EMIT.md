# REVIEW-GUIDE EMIT — paket panduan review `openapi-codegen/src/emit.rs` (untuk agent4)

**Penyusun:** agent9 · **Tanggal:** 2026-09-09 · **Tujuan:** mempercepat gerbang review EMIT (matt #865) — bukan pengganti review independen; implementor≠reviewer tetap.

## 1. Peta file (total pipeline 2.010 baris)

| File | Baris | Peran |
|---|---|---|
| `ingest.rs` | 136 | baca+sha256+**guard depth via kernel** (`kernel::json::json_depth_exceeded`, R41 kutover) + versi 3.0/3.1 → `SpecDoc` |
| `resolve.rs` | 112 | resolusi `$ref` internal → IR (eksternal → CG-E-204) |
| `validate.rs` | 163 | aturan spec (server-var wajib default, security-undefined, dsb.) |
| `translate.rs` | 722 | IR → `TranslatedOp` (tabel SUPPORTED §1.2 spec; N1 array-of-string → Json opaque — keputusan agent1) |
| `emit.rs` | ~290 | `TranslatedOp` → kode Rust `generated/<vendor>.rs` + manifest JSON |
| `error.rs` | 66 | `CodegenError` + kode CG-E-10x (104 = depth) |

## 2.emit.rs — fungsi kunci

- `emit()` :29 — entry; basename kanonik :36 (header/manifest tak tergantung cwd → golden reproducible)
- header template :51 — `#![allow(clippy::vec_init_then_push)]` **terdokumentasi di header file hasil-generate**: push-per-node = anti stack-overflow (literal `vec![]` 1.206 elemen meluapkan stack debug — temuan M3)
- `field_expr()` :159, `value_expr()` :204 — ekspresi Rust dari IR
- `node_kind()` :211 — pemetaan kind kernel (nol vocab paralel, R-A Anda)
- `vendor_of()` :222, `sanitize()` :229 (stem module-safe), `rs()` :244 (escape string literal)

## 3. Invariant yang layak ditegakkan reviewer (dengan cara cek)

| # | Invariant | Cara verifikasi cepat |
|---|---|---|
| V1 | Nol vocab paralel: hanya tipe kernel (`NodeDescriptor`, `ParameterSchema`, `NodeKind`, `SideEffect`…) | `grep -n "use kernel::" nodes-openapi/src/generated/*.rs` (5 import, semua kernel) |
| V2 | Golden byte-identical (determinisme EMIT) | `cargo test -p nodes-openapi --test golden` (3 test, drift = fail otomatis) |
| V3 | Header memuat sha256 raw-bytes spec + jumlah emitted/rejected | `head -3 nodes-openapi/src/generated/*.rs` |
| V4 | Manifest: `{sha256, raw-bytes, ops_rejected[], nodes[]}` per vendor | `python3 -m json.tool nodes-openapi/src/generated/frankfurter.manifest.json \| head -20` |
| V5 | Dual-mode: STRICT default (INTEG-02 fail-loud) vs COLLECT (github 1.206 OK + 19 rejected tercatat di manifest — bukan senyap) | lihat `ops_rejected` di github_rest.manifest.json (19 entri berkode CG-E) |
| V6 | Kode ter-generate = murni data (tanpa parse input, tanpa I/O) | `grep -c "from_str\|from_slice\|read\|write" nodes-openapi/src/generated/*.rs` = 0 |
| V7 | allow-list clippy hanya 1 (`vec_init_then_push`) + alasannya tertulis | V3 di atas |

## 4. Batas yang sudah diakui (jujur, jangan dihitung sebagai cacat baru)

- N1 array-of-string → Json opaque (menunggu varian kernel string-array; keputusan agent1+matt).
- Gap exchange-auth (Binance menarik spec — `docs/EVIDENCE-BINANCE-SPEC-WITHDRAWAL.md`).
- Eksekusi = task terpisah `W3-OPENAPI-EXEC-IMPL` (RFC v0.3; sudah di queue, depends_on merge ini).

## 5. Jalankan bukti (satu blok)

```
cd /opt/agent-workspace/w3-openapi-impl
CARGO_TARGET_DIR=/mnt/extra-storage/cargo-target cargo test --workspace   # 3+33+23 = 59/59
CARGO_TARGET_DIR=/mnt/extra-storage/cargo-target cargo clippy --workspace --all-targets  # 0
sha256sum -c SHA256SUMS   # integritas salinan 38a
```

Verifikasi independen Anda adalah gerbang terakhir selain re-run agent10. Terima kasih.
