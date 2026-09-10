# ARSITEKTUR SCRAPING BERLAPIS (MULTI-TIER WEB INGESTION ENGINE)
**Versi:** 1.0.0 | **Tanggal:** 2026-09-09 | **Otoritas:** Mandat Resmi Pemilik Proyek
**Pelaksana:** `@agent9` (Integration), `@agent6` (WASM/WCB), `@agent7` (MCP), `@agent4` (Schema), `@agent10` (Compliance)

---

## 1. DESAIN ARSITEKTUR PIRAMIDA SCRAPING (4 LAPIS)

Untuk memberikan kemampuan pengumpulan intelijen web terlengkap tanpa batas teknis, sistem dilengkapi dengan 4 lapis mesin scraping yang bekerja secara adaptif sesuai profil tantangan situs target:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ LAPIS 1: PROTEKSI KETAT (Anti-Bot, Cloudflare, DataDome, Akamai)        │
│ Node: n8n-nodes-camofox-browser + n8n-nodes-scrapling                   │
│ • Fingerprint Masking: Spoofing WebGL, Canvas noise, TLS JA3/JA4        │
│ • Self-Healing Parser: Kebal perubahan CSS/XPath class dinamis          │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ LAPIS 2: INTERAKSI OTONOM (Multi-Step, Login, Form, Infinite Scroll)    │
│ Node: n8n-nodes-browser-use                                             │
│ • Vision & DOM Grounding: Agen AI mengendalikan navigasi bak manusia    │
│ • Multi-Tab & Session: Otomasi alur klik berulang & dynamic loading     │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ LAPIS 3: EKSTRAKSI INSTAN BERSIH UNTUK LLM (Markdown & JSON)            │
│ Node: n8n-nodes-firecrawl                                               │
│ • Serverless Rendering: Pembersihan otomatis iklan, navbar, footer      │
│ • Token-Efficient Output: Menghasilkan Markdown murni siap pakai        │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ LAPIS 4: EKSEKUSI SKRIP PRESISI & HYDRATION EXTRACTION                  │
│ Node: n8n-nodes-puppeteer (Native MCP Engine)                           │
│ • Context Injection: puppeteer_evaluate langsung ke context browser     │
│ • Data Harvesting: Mengambil window.__NEXT_DATA__, state Redux/Vue      │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. SPESIFIKASI DETAIL PER LAPIS

### 🟢 LAPIS 1: Web dengan Proteksi Ketat
* **Komponen**: `camofox-browser` + `scrapling` (Rekomendasi Utama).
* **Fungsi Utama**: Menembus proteksi anti-bot tingkat institusi tanpa terkena blokir IP atau captcha challenge.
* **Keunggulan Teknis**:
  * *Fingerprint Masking*: Mengacak signature TLS, Canvas noise, WebGL vendor, AudioContext, dan Navigator flags.
  * *Self-Healing Parser (Scrapling)*: Jika class CSS diacak secara berkala oleh frontend (misal class `.css-1a2b3c`), Scrapling menggunakan pencocokan semantik otomatis berdasarkan bobot tag dan konten teks sehingga alur tidak rusak.
* **Mode Operasi Hemat Memori**:
  * Mode HTTP Stealth: Konsumsi RAM < 15MB untuk situs statis terproteksi.
  * Mode Full Anti-Detect: On-demand worker dengan batas waktu 30 detik (cgroup 150MB).

---

### 🔵 LAPIS 2: Scraping Berbasis Interaksi Kompleks
* **Komponen**: `browser-use`.
* **Fungsi Utama**: Agen otonom visual untuk situs modern yang membutuhkan interaksi manusia (klik accordion, pagination, form login, infinite scroll).
* **Keunggulan Teknis**:
  * Mengevaluasi koordinat visual halaman dan tree DOM secara serentak.
  * Mengisi input form dinamis, menekan tombol *Load More*, dan menangani dialog modal/pop-up.
* **Output**: Hasil interaksi diekstrak langsung ke format JSON terstruktur sesuai skema node.

---

### 🟡 LAPIS 3: Ekstraksi Instan Konten Bersih untuk AI
* **Komponen**: `firecrawl`.
* **Fungsi Utama**: Merender halaman ber-JavaScript tebal di backend serverless dan langsung mengembalikan dokumen Markdown bersih tanpa elemen sampah.
* **Keunggulan Teknis**:
  * *Auto-Boilerplate Removal*: Menghapus header navigasi, banner iklan, cookie consent, dan footer secara otomatis.
  * *Hemat Biaya Token*: Memangkas hingga 90% token HTML mentah menjadi format Markdown ringkas yang langsung siap diproses oleh LLM di node berikutnya.
  * Mendukung *full-site crawling* dengan batas kedalaman (*max depth*).

---

### 🟣 LAPIS 4: Eksekusi Skrip Presisi & DOM Evaluation
* **Komponen**: `puppeteer` (Native MCP Server).
* **Fungsi Utama**: Akses tingkat rendah (*low-level*) ke context browser melalui protokol CDP (Chrome DevTools Protocol).
* **Keunggulan Teknis**:
  * `puppeteer_evaluate`: Mampu menyuntikkan script JavaScript langsung untuk membaca variabel memori browser.
  * **Ekstraksi Data Hydration**: Mengekstrak objek internal aplikasi SPA/SSR seperti `window.__NEXT_DATA__`, `window.__NUXT__`, atau state GraphQL sebelum di-render ke DOM (mengambil raw data asli tanpa repot scraping visual).
  * Menembakkan event DOM spesifik (`click`, `hover`, `dispatch custom event`).

---

## 3. INTEGRASI SMART-ROUTING KE WORKFLOW HUB

Template alur kerja di Workflow Hub secara cerdas memilih lapisan scraping yang paling tepat:
1. **Kasus Ticker Kripto / Orderbook / On-Chain**: Memakai **Lapis 1 (Scrapling HTTP Stealth)** atau **Lapis 4 (Puppeteer evaluate `__NEXT_DATA__`)** $\rightarrow$ Cepat, instan, hemat RAM.
2. **Kasus Berita Portal / Artikel Politik**: Memakai **Lapis 3 (Firecrawl)** $\rightarrow$ Menghasilkan Markdown bersih langsung ke Token Condenser.
3. **Kasus Riset Kompetitor / Portal Portal Registrasi**: Memakai **Lapis 2 (Browser-Use)** $\rightarrow$ Mampu berinteraksi dan mengumpulkan data bertahap.

---

## 4. GUARDRAILS KEAMANAN & KEPATUHAN (`@agent10`)

1. **Proteksi SSRF (Server-Side Request Forgery)**: Seluruh node scraping DILARANG KERAS mengakses IP lokal/privat (`127.0.0.1`, `localhost`, `10.0.0.0/8`, `192.168.0.0/16`).
2. **Rate Limiting & Delays**: Wajib menyisipkan randomized delay (2.0–5.0 detik) untuk mencegah overload server tujuan.
3. **Penyaringan PII**: Data hasil scraping otomatis disaring dari nomor telepon, email, dan token kredensial sebelum disimpan ke execution log.
