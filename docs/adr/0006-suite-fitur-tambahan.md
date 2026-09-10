# ADR-0006: Suite fitur tambahan (keputusan Pemilik, fase 2–3)

**Status:** DITERIMA sebagai scope; prioritas & urutan detail di spec. 2026-09-10.

Prinsip atas: **n8n-rust unggul dalam segala hal vs n8n asli** — keamanan terjamin,
error mudah dilacak + auto-solve, tanpa pemangkasan fitur asli (MCP & tampilan tidak
dipotong; boleh ditingkatkan).

## 1. Scrape Orchestrator (node)
Mode **otomatis / manual** dipilih per eksekusi. Backend: Crawlee+Playwright,
Firecrawl, Scrapy, Bright Data, OxyLabs, Camofox. (Prioritas backend → menunggu
konfirmasi ronde 2.)

## 2. AI Agent Node (5 cabang)
- **Engine**: default bawaan, atau AI agent luar (Hermes, OpenClaw, opencode CLI,
  Claude, dst.) **melalui MCP**.
- **Memory**: multi-layer (bisa >1 layer); backend n8n-native bawaan atau
  Obsidian/graphify/dll; **otomatis tercatat tanpa habiskan token** (local-first,
  token hanya saat query; layer = working/session → project → long-term).
- **Skill**: tempel link GitHub langsung, atau dari folder skills global/project/local.
- **MCP**: sama dengan tools n8n biasa (tool = callable).
- **Output**: bentuk keluaran configurable.

## 3. Workflow Hub (fase AKHIR)
Konsep playstore: daftar workflow publik (GitHub) → klik → download otomatis ke
canvas/list project.

## 4. HTTP
Node HTTP Request bawaan + **HTTP Orchestrator** (beberapa client/router, mudah
ditambah/dikurangi).

## 5. Konektor
GitHub, Gmail (pola konektor = template, mudah ditambah).

## 6. Custom node by workflow
Sub-workflow sebagai node (komposisi workflow). (Termasuk tier ADR-0007.)

## 7. Paritas tanpa pemangkasan
- MCP: **standar** di n8n-rust (bukan fitur ekstra).
- Tampilan: kanvas gaya n8n (familiar), boleh ditingkatkan.
- Katalog node n8n 2.39.0 = target parity bertahap (core → konektor populer →
  long-tail via tier custom/MCP). Keputusan final batas ini = konfirmasi ronde 2.

## 8. Keamanan & error (melintang semua)
- Sandboxing node (WASM/JS/Python terisolasi); kredensial AES-GCM; secrets tak pernah
  di log (redaksi, patokan skill diagnosing-bugs); audit log; budget per-eksekusi.
- Error: envelope terstruktur (node/item/stage/rantai sebab), journal eksekusi
  (replay), auto-retry per-node (jumlah, backoff, kondisi), error workflow
  (Error Trigger n8n parity). "Auto-solve" = auto-retry + fallback, bukan magis.

## Amendemen ronde 2 (2026-09-10, keputusan Pemilik)

1. **7 fitur melintang disetujui semua** (budget guard, replay, workflow-as-tool,
   benchmark harness, WASM hot-reload, queue lokal, metrics).
2. **Budget Guard BUKAN pemangkasan fitur** (klarifikasi eksplisit): guard adalah
   pembatas *konfigurabel* per-eksekusi — default = batas aman mesin, dapat dinaikkan
   per workflow, dapat dinonaktifkan per eksekusi (dengan catatan audit). n8n asli
   justru tidak punya ini: saat memori habis, OS membunuh proses (OOM-kill) tanpa
   laporan per-workflow. Guard = versi terkontrol, per-workflow, dan dapat dipulihkan.
3. **Workflow-as-tool dengan batasan jelas agar tak error** (permintaan Pemilik):
   - maks kedalaman rekursi = 5 (default, konfigurabel 1–10)
   - maks pemanggilan nested per eksekusi = 1.000
   - budget diwarisi (anak tidak boleh melebihi sisa budget induk)
   - siklus (A panggil B panggil A) terdeteksi saat compile graph → error compile, bukan runtime
   - setiap pelanggaran = error envelope dengan rantai panggilan lengkap (node.workflow.depth)
4. **Scrape Orchestrator — daftar backend & kebutuhan API** (permintaan Pemilik:
   "masukkan daftar rencana yang menggunakan API"):

   | Backend | Jenis | Butuh API key / biaya |
   |---|---|---|
   | Crawlee + Playwright | open-source, lokal | **tidak** (unduh browser; resource lokal) |
   | Camofox | open-source, anti-detect browser | **tidak** |
   | Scrapy | open-source, lokal | **tidak** (python) |
   | Firecrawl | hosted API | **YA — API key berbayar** (free tier terbatas) |
   | Bright Data | proxy/data API berbayar | **YA — akun + API key, berbayar** |
   | OxyLabs | proxy/data API berbayar | **YA — akun + API key, berbayar** |

   Urutan build: (1) Crawlee+Playwright, Camofox, Scrapy (fungsi tanpa key) →
   (2) Firecrawl → (3) Bright Data, OxyLabs. Node bekerja dari hari pertama dengan
   backend tanpa key; backend berbayar aktif setelah key dikonfigurasi.
