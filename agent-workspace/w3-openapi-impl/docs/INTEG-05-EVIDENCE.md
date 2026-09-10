# INTEG-05(a) — BUKTI zero-credential-leak (statis)

**Task:** W3-OPENAPI-IMPL · **Pemindai:** agent9 · **Tanggal:** 2026-09-09 · **Metode:** pola-regex atas seluruh artefak (49 file: `.rs` `.json` `.toml`) di salinan 38a (`/opt/agent-workspace/w3-openapi-impl/`, tree-hash `8a6b26f2…`)

## Pola yang discan

1. `sk-[a-zA-Z0-9]{16,}`
2. `api[_-]?key\s*[:=]\s*"<value>"` (≥8 char)
3. `bearer <token>` (≥16 char)
4. `password\s*[:=]\s*"<value>"` (≥6 char)
5. `-----BEGIN … PRIVATE KEY-----`

## Hasil

| Lokasi | Hit | Analisis |
|---|---|---|
| Kode generator (`openapi-codegen/src/**`) | **0** | bersih |
| Kode hasil-generate (`nodes-openapi/src/generated/*.rs`) | **0** | bersih |
| Manifest verifikasi (`*.manifest.json`) | **0** | bersih |
| `Cargo.toml` / config | **0** | bersih |
| Golden spec `github-rest.openapi.json` (baris 308.723) | **1** | **Contoh dummy resmi GitHub** di `components.examples` (payload contoh dokumentasi publik GitHub utk GitHub App webhook: `client_id Iv1.8a61f9b3a7aba766` = contoh dokumen mereka, pem dummy `-----BEGIN RSA PRIVATE KEY-----`). Bagian dari byte spec resmi ter-pin (sha256 `531b0574…`, upstream `github/rest-api-description`). **Bukan kredensial nyata, bukan injeksi pipeline.** |

## Bukti non-propagasi

`grep -c "PRIVATE KEY" nodes-openapi/src/generated/*` = **0 di semua file** — EMIT hanya menerjemahkan params/operasi, `components.examples` tidak pernah disalin ke node/manifest → contoh dummy tidak pernah mencapai runtime.

## Kesimpulan INTEG-05(a)

✅ **LULUS**: 0 literal rahasia di seluruh kode & artefak hasil pipeline; satu-satunya pattern-hit adalah contoh dokumentasi resmi penerbit spec (ter-pin apa adnya, tidak ter-propagasi). Bagian (b) scan **runtime** (request/log/metrics) menyusul saat jalur eksekusi ada (RFC AGENT9-RFC-OPENAPI-EXEC.md, post-merge).
