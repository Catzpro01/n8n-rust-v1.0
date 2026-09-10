# [SPEC] n8n-rust v3 — single-user, 100rb–1jt node, 8 tier custom node

**Status:** DISETUJUI — disintesis dari sesi `grill-with-docs` ronde 1–2 dengan Pemilik Produk (2026-09-10). Keputusan rinci: `docs/adr/0001–0007` + amendemen. Ganti status PRD v0.2 (publik+multi-tenant).

## 1. Produk & target
Engine workflow automation **Rust-native, n8n-compatible**, **single-user self-hosted** (untuk diri sendiri). Dikerjakan di sandbox agent (build) + di-host di VPS (2 vCPU/2 GB/50 GB, **swap 4 GB**) dan harus jalan di **PC 1 vCPU/500 MB**. Hukum: **Correctness → Measurable Performance → Compatibility**; tanpa benchmark tidak ada klaim.

## 2. Target skala (bertahap & terukur)
| Fase | Target | Mesin | Kriteria lulus |
|---|---|---|---|
| B0 | workflow panjang-berat harian | 1 vCPU/500 MB | tanpa OOM/swap-thrash, guard aktif |
| B1 | 100.000 node graph tunggal | 1 vCPU/500 MB | peak RSS < 500 MB |
| B2 | 500.000 node | VPS 2 vCPU/2 GB | RSS terukur & dilaporkan |
| B3 | 1.000.000 node | VPS + 4 GB swap (guard) | RSS terukur; swap bukan jalan tol |
| B4 | konkurensi 100rb+ node simultan | VPS | throughput + RSS terukur |

Strategi: adjacency ringkas di arena (bukan objek per-node), state per-node via **spill store** (bukti ada: 753 MB payload @ RSS 50,4 MB vs n8n asli OOM di 1,68 GB), eksekusi streaming per-item. Alat bukti: **benchmark harness permanen** (crate `bench`, RSS via `/proc`, graph deterministik).

## 3. Fitur dasar (baseline, fleksibel)
1. **Round-trip JSON n8n asli 2.39.0** (import → jalankan → export; fixture dari ekspor NYATA)
2. Node inti fase 1: Manual/Cron/Webhook Trigger, Set, IF, Switch, Merge, SplitInBatches, HTTP Request, Code (JS), NoOp, Respond, **MCP tool node (standar, bukan ekstra)**
3. Kredensial (AES-GCM, kunci dari env/file) · Schedule + Webhook
4. CLI (6 perintah paritas Stage-1) + REST API + **UI kanvas gaya n8n** (daftar/buat/sunting/jalankan/hasil)
5. Riwayat eksekusi + retry + **error workflow (Error Trigger, n8n parity)**
6. Berjalan di 1 vCPU/500 MB (B0) sebagai fitur dasar permanen

## 4. Custom node — 8 TIER (disetujui Pemilik)
① JS Script (rquickjs, tanpa build) · ② Python tertanam (PyO3) · ③ **WASM** (hot-load, sandbox wasmtime kuat; crate `nodes-wasm` sudah ada) · ④ Native Rust crate (tercepat) · ⑤ External process/microservice (bahasa apa pun, IPC lokal) · ⑥ **AI-Generated Node** (deskripsi → engine tulis kode+test+node) · ⑦ Sub-workflow node · ⑧ MCP tool as node.
Urutan build: ① → ③ → ⑦/⑧ → ② → ④ → ⑤ → ⑥. Lintas tier: budget guard, kontrak item sama, satu contoh jadi per tier.

## 5. Suite fitur tambahan (fase 2–3)
- **Scrape Orchestrator** (mode otomatis/manual) — backend & API:

  | Backend | Butuh API key/biaya |
  |---|---|
  | Crawlee+Playwright, Camofox, Scrapy | TIDAK (fase 1) |
  | Firecrawl | YA (fase 2) |
  | Bright Data, OxyLabs | YA, berbayar (fase 3) |
- **AI Agent Node (5 cabang):** engine (bawaan / Hermes, OpenClaw, opencode CLI, Claude… via MCP) · memory (multi-layer, backend bawaan/Obsidian/graphify, local-first, auto-catatan tanpa bakar token) · skill (link GitHub / folder global-project-local) · mcp (tools n8n biasa) · output (configurable)
- **HTTP Orchestrator** (bawaan + multi-client, mudah tambah/kurangi)
- **Konektor:** GitHub, Gmail (pola template, mudah ditambah)
- **Workflow Hub (fase AKHIR):** playstyle GitHub → klik → masuk canvas/list project
- **Tanpa pemangkasan fitur asli n8n:** MCP standar, tampilan kanvas n8n (boleh ditingkatkan), katalog node 2.39.0 = paritas bertahap (core → konektor populer → long-tail via tier/MCP)

## 6. Melintang (7, semua disetujui)
1. **Budget Guard** (bukan pemangkasan — pembatas konfigurabel per-eksekusi: max RSS/waktu/item; default batas aman, bisa dinaikkan/nonaktifkan per eksekusi + audit; n8n tidak punya ini — OOM-kill-nya buta)
2. **Replay/time-travel** (journal deterministik → putar ulang sampai node-N dengan input berbeda)
3. **Workflow-as-tool** + batasan jelas: kedalaman rekursi maks 5 (conf 1–10), maks 1.000 panggilan nested, budget diwarisi, siklus terdeteksi saat compile → error compile + rantai panggilan
4. **Benchmark harness** permanen
5. **WASM hot-reload**
6. **Queue lokal zero-broker** (SQLite WAL; Redis opsional)
7. **Metrics endpoint** (`/metrics`: RSS, antrean, item/s)

## 7. Keamanan & error
Sandboxing per tier (WASM/JS/Python terisolasi) · kredensial AES-GCM · secrets tak pernah di log (redaksi) · audit log · guard budget · auth single-user (loopback/LAN/VPN).
Error: **envelope terstruktur** (node/item/stage/rantai sebab) + **journal** (replay) + **auto-retry per-node** (jumlah/backoff/kondisi) + **error workflow**. "Auto-solve" = auto-retry + fallback, bukan magis.

## 8. Build & deploy
Build di **sandbox** (2 vCPU/4 GB) → binary ke **VPS** (runtime: systemd + swap 4 GB + guard). **GitHub = source of truth; 1 tiket selesai = 1 commit + push** (branch sesi), tag per milestone. GitHub Actions = CI gate opsional belakangan.

## 9. Diferensiasi vs n8n asli (jujur: yang perf hanya sah setelah benchmark lulus)
1. Skala 100rb–1jt node @ RAM kecil (n8n: OOM — bukti empiris ada)
2. Jalan di 1 vCPU/500 MB (n8n praktis butuh ≥1 GB)
3. 8 tier custom node (n8n: 1 tier JS/TS, butuh rebuild)
4. Budget guard per-eksekusi (n8n: tidak ada)
5. Replay/time-travel (n8n: retry = ulang dari awal)
6. Sandbox WASM sejati (n8n: Code node in-process)
7. Queue tanpa broker di mesin kecil (n8n queue mode wajib Redis)
8. Scrape orchestrator multi-backend built-in (n8n: node komunitas terpisah)
9. AI agent node 5-cabang + memory multi-layer local-first (n8n: lebih terbatas)
10. Workflow Hub pribadi, bisa offline (n8n: template library online)
11. Hot-reload node tanpa restart (n8n: restart)
12. Zero GC pause (Rust vs Node.js) — latensi prediktif

## 10. Out of scope
Multi-tenant, RBAC multi-pengguna, tim/sharing, mobile, marketplace publik. (Pintu arsitektur tenant_id dibiarkan.)

## 11. Acceptance criteria utama
- TB-01…TB-10 (TRACER BULLET) selesai berurutan
- B0 lulus di mesin 1 vCPU/500 MB (bukti output harness)
- Round-trip n8n 2.39.0 lulus (fixture ekspor nyata)
- 1 contoh jadi per tier custom node yang dibangun, lolos budget guard
- VPS: systemd + swap 4 GB + service hidup, metrics endpoint respons
