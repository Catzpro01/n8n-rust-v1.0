# BUKTI: Binance menarik OpenAPI spec resmi (untuk audit — diminta matt #1116 butir 5)

**Penyidik:** agent9 (ROLE_INTEGRATION) · **Tanggal pemeriksaan:** 2026-09-09, ~11:54–11:57 (waktu server) · **Metode:** curl langsung dari VPS produksi + GitHub API

## 1. Endpoint resmi — semuanya 404

| Host | URL | HTTP | Body |
|---|---|---|---|
| api.binance.com | `/api/v3/openapi.json` | **404** (nginx) | 0 B |
| api1.binance.com | `/api/v3/openapi.json` | **404** | 0 B |
| api2.binance.com | `/api/v3/openapi.json` | **404** | 0 B |
| api3.binance.com | `/api/v3/openapi.json` | **404** | 0 B |
| api4.binance.com | `/api/v3/openapi.json` | **404** | 0 B |
| data-api.binance.vision | `/api/v3/openapi.json` | **404** | 146 B |

(HEAD juga 404: `HTTP/2 404, server: nginx`.)

## 2. Repo dokumentasi resmi — markdown murni

Repo: `github.com/binance/binance-spot-api-docs` (branch `master`, diperiksa via GitHub Contents API + git-trees recursive):

- Listing root: hanya berkas markdown (`rest-api.md` 175 KB, `web-socket-api.md`, `user-data-stream.md`, `CHANGELOG.md`, `PROD-TERMS-OF-USE.md`, dsb.) + direktori `sbe/`, `testnet/`.
- `git/trees/master?recursive=1`: **0 (nol) berkas `.yaml`/`.yml`/`.json`** di seluruh tree.
- Repo di-update aktif (pushed_at 2026-09-09) — penarikan spec bukan repo mati.

## 3. Konsekuensi & keputusan

- Binance TIDAK LAYAK lagi jadi golden spec INTEG-01 (tidak ada artefak resmi yang bisa di-pin).
- Pengganti golden #2 (disetujui matt #1116 butir 5): **GitHub REST API** — `github/rest-api-description` `descriptions/api.github.com/api.github.com.json`, openapi 3.0.3, 12.927.758 B, sha256 `531b05749a9f86c7be01330e6d89d052fd3102ff31df4e0c765d9550b11bbad8`, no-key untuk endpoint publik, ToS memperbolehkan akses programatik.

## 4. GAP yang dicatat (tidak didiamkan — matt #1116)

**Cakupan node exchange/kripto dengan auth kompleks (API-key + signature HMAC, header khusus, rate-limit kelas exchange) HILANG dari golden set v1.** Produk akan membutuhkannya. Ditunda sampai:
1. ada exchange yang kembali menerbitkan OpenAPI spec resmi, ATAU
2. kurasi manual spec (dengan provenansi tercatat) disetujui komunitas.

Jangan cari pengganti ketiga sekarang (arah matt #1116).
