# 🏛️ PRD MANDIRI: WORKFLOW HUB & EKOSISTEM TEMPLATE
## The "Bloomberg-Grade" Intelligence & Turn-Key Automation Play Store
**Versi Dokumen:** 1.0.0-CANONICAL  
**Status:** ✅ RESMI DISAHKAN (Ratifikasi Pemilik Proyek - 2026-09-09)  
**Arsitektur Mesin:** Rust Engine Drop-in Replacement for n8n (<500MB RAM Hard-Cap)  
**Otoritas:** Pemilik Proyek via Chief Supervisor & Technical Lead (`fern`)

---

## 1. Visi Produk & Filosofi Dasar

### 1.1 Visi Inti
Menghadirkan **Workflows Play Store** yang menyediakan akses instan ke ratusan workflow siap pakai (*turn-key workflows*) lintas domain. Pengguna dapat memperoleh kemampuan otomasi setara **Bloomberg Terminal pribadi** secara mudah, gratis/hemat biaya, aman, dan tanpa perlu merakit logika kompleks dari nol.

### 1.2 Tiga Pilar Pengalaman Pengguna (UX)
1. **Instan & Tanpa Gesekan (Play Store Model)**: Unduh, atur parameter minimal (API key/kredensial), dan jalankan dalam hitungan detik.
2. **Kapasitas Intelijen Bloomberg-Grade**: Agregasi data pasar global, makroekonomi, laporan keuangan SEC EDGAR, sentimen NLP pasar, metrik on-chain, serta investigasi web otomatis.
3. **Efisiensi Ekstrem Rust**: Seluruh eksekusi berjalan di atas engine Rust berkapasitas memori ketat (<500MB RAM), *zero memory leak*, dan *content-addressable storage* (CASD).

---

## 2. Arsitektur Dualitas Workflow (Aktif vs. Pasif)

Untuk mempermudah variasi kebutuhan pengguna, setiap template di Workflow Hub diklasifikasikan ke dalam **Dua Model Distribusi**:

```
+-------------------------------------------------------------------------+
|                        WORKFLOW HUB PLAY STORE                          |
+-------------------------------------------------------------------------+
       |                                                 |
       v                                                 v
+-----------------------------+   +---------------------------------------+
|  [TIPE A] WORKFLOW AKTIF    |   |       [TIPE B] WORKFLOW PASIF         |
|  (Full-Canvas Project Graph)|   |    (Single-Node Function Capsule)     |
+-----------------------------+   +---------------------------------------+
| - Kanvas visual terbuka     |   | - 1 Node Tunggal di kanvas utama      |
| - Semua node & wires tampak |   | - Black-box sub-workflow terenkapsulasi|
| - Pengguna bebas mengedit   |   | - Input sederhana -> Output data bersih|
| - Ada Sticky Notes panduan  |   | - Nol kerumitan graf (Zero Clutter)   |
| - Cocok untuk kustomisasi   |   | - Cocok untuk konsumsi fungsi instan  |
+-----------------------------+   +---------------------------------------+
```

### 2.1 Tipe A: Workflow Aktif (Full-Canvas Project Template)
* **Definisi**: Template proyek komprehensif yang mengimpor seluruh graf Directed Acyclic Graph (DAG) ke kanvas visual pengguna.
* **Perilaku**:
  * Ketika diunduh, seluruh node, wire, dan percabangan terbuka penuh.
  * Dilengkapi dengan `stickyNote` panduan yang memuat 3 pilar: `[CARA KERJA]`, `[FUNGSI]`, dan `[TUJUAN]`.
  * Pengguna memiliki kebebasan 100% untuk memodifikasi parameter, mengganti webhook, menambah logika transformer, atau menyambungkannya ke database lokal.
* **Target Pengguna**: Developer, sysadmin, dan analis yang ingin membangun sistem modular kustom di atas fondasi yang sudah teruji.

### 2.2 Tipe B: Workflow Pasif (Single-Node Function Capsule / Black-Box Sub-Workflow)
* **Definisi**: Workflow fungsional yang dikemas menjadi **Satu Node Tunggal** di palet n8n/Rust.
* **Perilaku**:
  * Pengguna hanya perlu menyeret 1 node ke kanvas (misal node: `Bloomberg Macro Scanner` atau `Stealth Web Scraper`).
  * Seluruh kompleksitas multi-node di dalamnya (misal 15 node: HTTP Request -> TLS Fingerprint Spoof -> HTML Parsing -> NLP Sentiment -> Formatting) dieksekusi secara terisolasi sebagai sub-workflow di data-plane.
  * **Interface Sederhana**: Pengguna hanya mengisi parameter form tingkat tinggi (misal: `Ticker Symbol: NVDA`, `Analyze Sentiment: True`).
  * **Output Bersih**: Node langsung menghasilkan JSON matang siap pakai untuk node berikutnya.
* **Target Pengguna**: Trader, analis bisnis, dan pengguna awam yang hanya menginginkan hasil akhir tanpa ingin dipusingkan oleh kabel logika yang rumit (*zero clutter*).

---

## 3. Matriks Domain & Katalog Template Bloomberg-Grade

Workflow Hub menargetkan 4 domain spesialisasi utama:

### 3.1 Domain 1: The Bloomberg Financial & Macro Intelligence Suite
Menyediakan akses informasi komprehensif pasar keuangan global:
1. **Global Macro & Central Bank Radar**:
   * Penarikan otomatis indikator FRED (Federal Reserve Economic Data): Yield Curve 10Y-2Y inversion, suku bunga Fed Funds, laju inflasi CPI/PCE, dan neraca M2.
   * Format Pasif: Node `Macro-Health-Score` -> Mengeluarkan metrik probabilitas resesi dan tren likuiditas global.
2. **Institutional Equities & SEC EDGAR Parser**:
   * Penarikan dan ekstraksi otomatis laporan berkala SEC 10-K dan 10-Q perusahaan publik AS.
   * Parser Item 1A (Risk Factors) dan MD&A (Management Discussion and Analysis) dengan ringkasan otomatis.
3. **Real-time Financial NLP Sentiment Scanner**:
   * Model FinBERT lokal/serverless memindai sentimen berita finansial (Reuters, Bloomberg RSS, CNBC) dan media sosial (Reddit r/wallstreetbets, Twitter/X FinTwit).
   * Menghasilkan skor sentimen terukur (-1.0 hingga +1.0) per emiten.
4. **Crypto On-Chain & Order Book Microstructure**:
   * Pelacakan pergerakan *whale wallet*, likuiditas DEX via DefiLlama, dan kedalaman buku pesanan (*order book depth & funding rates*) via CCXT.

### 3.2 Domain 2: Deep Web Intelligence & Layered Scraping (OSINT)
Mengintegrasikan arsitektur scraping 4 lapis yang telah diratifikasi:
1. **Lapis 1 (Evasive Anti-Bot)**: `camofox-browser` + `scrapling` untuk menembus situs dengan proteksi Cloudflare Turnstile, DataDome, dan Akamai dengan self-healing DOM parser.
2. **Lapis 2 (Interactive Navigation)**: `browser-use` untuk aksi interaktif (login otomatis, form-filling, infinite scroll, multi-step navigation).
3. **Lapis 3 (JS-Render to Clean Markdown)**: `firecrawl` untuk ekstraksi instan artikel, dokumentasi, atau portal berita ke Markdown/JSON bebas iklan.
4. **Lapis 4 (DOM Script & Hydration Injection)**: `puppeteer` MCP tool untuk mengekstrak state internal React/Next.js (`__NEXT_DATA__`) secara instan.

### 3.3 Domain 3: Quantitative Trading & Risk Management
1. **Capital Preservation Shield**:
   * Kalkulator Fractional Kelly Criterion untuk ukuran posisi optimal.
   * Adaptive ATR Trailing Stop-Loss dan Value-at-Risk (VaR) 95%/99% portfolio guard.
2. **Vectorized Strategy Backtester**:
   * Pengujian strategi kuantitatif historis dengan kalkulasi instan metrik Sharpe Ratio, Sortino Ratio, dan Max Drawdown.

### 3.4 Domain 4: Business Automation & Daily Operations
1. **Multi-Channel Dispatcher**: Otomasi notifikasi cerdas ke Telegram, Discord, Slack, WhatsApp, dan Webhook perusahaan.
2. **Atomic Data Ingress**: Penanganan webhook berkecepatan tinggi dengan validasi schema instan dan deduplikasi data (CASD).

---

## 4. Format Paket & Spesifikasi Teknis Template

Setiap template di Workflow Hub disimpan dalam format arsip standar:
* Nama berkas: `<template-id>-v<version>.hub.json`
* Struktur Skema Manifest:
```json
{
  "id": "bloomberg-macro-health",
  "version": "1.0.0",
  "name": "Global Macro and Liquidity Scanner",
  "type": "passive", 
  "category": "finance_macro",
  "author": "Swarm Community / Financial Engineering SIG",
  "license": "MIT",
  "engine_compatibility": ">=0.1.0",
  "documentation": {
    "cara_kerja": "Mengambil data yield spread 10Y-2Y dan CPI dari FRED API, menghitung dispersi likuiditas, dan memetakan rezim makro.",
    "fungsi": "Menghitung Macro Health Index (0-100) dan probabilitas ekspansi/kontraksi ekonomi.",
    "tujuan": "Memberikan sinyal risiko makro bagi pengelola portofolio sebelum mengambil keputusan alokasi aset."
  },
  "pinning": {
    "type": "schema-pin",
    "schema_hash": "blake3:9f83ab...",
    "expected_keys": ["recession_prob", "liquidity_index", "yield_spread", "timestamp"]
  },
  "inputs": {
    "fred_api_key": { "type": "string", "required": true, "secret": true },
    "lookback_months": { "type": "integer", "default": 24 }
  },
  "outputs": {
    "macro_status": { "type": "string" },
    "health_score": { "type": "number" }
  },
  "dag": {
    "nodes": [],
    "connections": {}
  }
}
```

---

## 5. SAYEMBARA KHUSUS TEMPLATE WORKFLOW HUB
### "The Grand Workflow Template Bounty"

### 5.1 Latar Belakang & Tujuan
Dengan ditutupnya sayembara arsitektur fondasi, kompetisi baru difokuskan 100% pada **ekspansi katalog workflow siap pakai**. Sayembara ini terbuka untuk seluruh agen swarm dan kontributor komunitas guna menciptakan ekosistem otomasi terkaya di dunia.

### 5.2 Kategori Sayembara
1. **Kategori A: Bloomberg Financial & Market Intelligence** (Bobot: 35%)
   * Template yang menyediakan analitik fundamental, sentimen berita saham, radar makro, dan analitik on-chain.
2. **Kategori B: Deep Scraping, Anti-Bot & Data Harvester** (Bobot: 30%)
   * Template berbasis Camofox, Scrapling, Browser-Use, dan Firecrawl yang mampu mengekstrak situs tersulit secara andal.
3. **Kategori C: Algorithmic Trading & Risk Shield** (Bobot: 20%)
   * Template strategi kuantitatif, kalkulator sizing Kelly, dan automasi proteksi portofolio.
4. **Kategori D: General Automation & Business Operations** (Bobot: 15%)
   * Template integrasi enterprise, CRM sync, multi-agent AI pipeline, dan notifikasi pintar.

### 5.3 Kriteria Penilaian (Evaluation Gate)
Setiap submission template harus lolos 5 gerbang pengujian ketat:
1. **Turn-Key Usability (Kemudahan Pakai)**: Pengguna dapat mengoperasikannya dalam <= 60 detik setelah unduh.
2. **Self-Documentation (3 Pilar Wajib)**: Memiliki uraian eksplisit `[CARA KERJA]`, `[FUNGSI]`, dan `[TUJUAN]` di manifest dan sticky note.
3. **Determinisme & Ketahanan Error**: Menangani rate-limit, timeout, dan network drops secara elegan tanpa mematikan engine.
4. **Efisiensi Memori (Hard-Cap <500MB)**: Eksekusi template tidak boleh memicu lonjakan memori (*memory spike*) >50MB di worker data-plane.
5. **Dualitas Tersedia**: Template nilai tambah tinggi dianjurkan menyediakan versi **Aktif** (full canvas) dan versi **Pasif** (1-node capsule).

### 5.4 Sistem Penghargaan & Meritokrasi
* Submission yang lolos verifikasi Gatekeeper `@agent10` dan QA akan dimasukkan ke katalog resmi `hub-catalog.json`.
* Kontributor dicatat secara permanen dalam metadata template dan papan kehormatan repositori.

---

## 6. Rencana Implementasi & Jadwal Rilis

* **Fase 1 (Wave 2)**: Standardisasi Skema Manifest Template & Runtime Invoker 1-Node (`HubCapsuleNode`).
* **Fase 2 (Wave 2-3)**: Kurasi dan penerbitan 25 Template Inti (10 Bloomberg/Finance, 8 Deep Scraping, 4 Trading Risk, 3 Business).
* **Fase 3 (Wave 3)**: Peluncuran UI Katalog Workflow Hub (pencarian, filter kategori, preview aktif/pasif, 1-click install).
* **Fase 4 (Wave 4)**: Auto-Update Engine (Diff-Gate, silent-swap untuk template berbasis schema-pin).

---
**Dokumen ini sah dan mengikat sebagai acuan resmi pengembangan ekosistem Workflow Hub.**
