# CONTEXT.md — Glosarium & Konteks Proyek n8n-rust

> Dibuat oleh skill `grill-with-docs` (2026-09-10). Istilah di sini adalah istilah
> resmi proyek; gunakan istilah ini di tiket, test name, dan dokumen.

## Istilah inti
- **Engine** — runtime eksekusi workflow Rust (`agent-workspace/rust-engine`), target: 100rb–1jt node di 500 MB–2 GB RAM.
- **Spill Store** — mekanisme state/item di luar memori (disk) agar RSS kecil; bukti: 753 MB payload @ peak RSS 50,4 MB.
- **Graph statis** — representasi workflow (node + edge) sebagai adjacency ringkas di arena memori; bukan objek per-node.
- **Item** — satu unit data yang mengalir antar node (patuh skema item n8n).
- **Tracer Bullet** — tiket yang menembus semua lapisan (storage→exec→node→keluaran); tiap selesai = terlihat bekerja (TB-01…).
- **Budget Guard** — pembatas keras per eksekusi: max RSS, max waktu, max item; dilanggar = kill + laporan.
- **Journal** — log eksekusi deterministik; dasar replay/time-travel.
- **Tier custom node** — 8 lapisan cara menambah node baru (ADR-0007).
- **Paritas n8n** — kompatibilitas perilaku/JSON dengan n8n asli versi paku **2.39.0**.
- **Divergensi Sadar** — perbedaan dari n8n yang diputuskan Pemilik & didaftarkan di CLI-PARITY-SPEC §DIVERGENSI SADAR; selain itu = cacat paritas.
- **Workflow Hub** — fitur playstore-style: workflow dari GitHub → klik → masuk canvas (fase akhir).
- **Scrape Orchestrator** — node scraping multi-backend (Crawlee+Playwright, Firecrawl, Scrapy, Bright Data, OxyLabs, Camofox), mode otomatis/manual.
- **AI Agent Node** — node 5-cabang: engine (bawaan/luar via MCP), memory (multi-layer), skill (github link/folder), mcp (tools), output.
- **HTTP Orchestrator** — varian HTTP Request dengan beberapa client/router, mudah ditambah/dikurangi.

## Keputusan arsitektur
Lihat `docs/adr/` (0001–0007). Keputusan operasional skill: `docs/agents/` (issue tracker = GitHub, label triage, layout domain docs).
