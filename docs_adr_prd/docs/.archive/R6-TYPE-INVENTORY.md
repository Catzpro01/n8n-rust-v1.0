# R-6 Baseline Inventory Tipe Korpus (bahan verifikasi katalog 694, #418)

**2026-09-09 · agent4 · read-only scan corpus/ (171 file)**
**Metode:** `python3` scan node.type di /opt/agent-workspace/docs/corpus/*.json (reproduksi di riwayat percakapan).

## Angka
| Kategori | Tipe unik | Instances |
|---|---|---|
| `n8n-nodes-base.*` | **147** | ~2.0xx |
| `@n8n/n8n-nodes-langchain.*` (first-party) | 31 | 172 |
| Komunitas (@scope & n8n-nodes-*) | 8 | 56 |
| **Total** | **186** | 2.260 |

## Catatan penting utk R-6 (verifikasi per-tipe → ParameterSchema)
1. TypeVersion SERING desimal & multi-versi per tipe (httpRequest: v1,2,3,4.1–4.4; googleSheets: v4.2–4.7; set: v3.2–3.5). R-6 mapping per (tipe, MAJOR) minimal — minor menyusul bila perlu.
2. Node deprecated yang TERTANGKAP korpus: cron 13, function 22, functionItem 11 → sudah ditangani registry v1 (35 migrasi; functionItem unresolved V-3b). splitInBatches v1-3 (20 inst) TIDAK deprecated (keputusan T-4).
3. Tool/langchain nodes (31 tipe, @n8n) = kandidat manifest WCB/agent6 hook — jangan dipetakan diam-diam ke ParameterSchema base.
4. Komunitas 8 tipe: blotato 36 (5 file!), firecrawlTool, buffer, browserAct, uploadPost, zaloBot — jalur OPAQUE (R-2) + kandidat manifest WCB (M6).
5. Tipe base tanpa korpus (sisanya dari 694) TIDAK diklaim — butuh verifikasi upstream per-tipe (V-1..V-3 pola) sebelum mapping; tanpa fixture = unit sintetis + [VERIFIKASI-UPSTREAM].

## Status
Baseline SAJA — belum verifikasi mapping apa pun. Butuh arahan matt (#1106 butir 3a) utk eksekusi R-6.

## KEPUTUSAN ENCODING VERSION (re note #1 di atas) — #tech-debate #1176
typeVersion desimal n8n → u16 kernel = **MAJOR*10+minor** (4.7 → 47), guard keras minor<10 else unmappable; total-order u16 = leksikografis (major,minor). PROVISIONAL agent1, override @matt/@fern terbuka. Mapping per (tipe, MAJOR) minimal; minor menyusul.

## VALIDASI KORPUS thd KEPUTUSAN #1176 (2026-09-09, scan read-only 171 file)
- minor>=10 di korpus: **0** → guard keras tidak pernah memicu (tetap layak utk masa depan).
- typeVersion non-numerik: 0; missing: 0 — semua node mem-pin version.
- Ukuran minimum tabel R-6: **217 pasangan (tipe,major)** — 161 tipe 1-major, 20 tipe 2-major, 4 tipe 3-major, 1 tipe 4-major (httpRequest v1–4.x).
- Konversi: integer "1"→10, "4.2"→42; u16 order = leksikografis (major,minor). Bukti publikasi #tech-debate #1199.
