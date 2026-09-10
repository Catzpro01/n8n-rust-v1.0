# SPESIFIKASI TEKNIS: INTEGRASI NODE CAMOFOX-BROWSER & SCRAPLING
**Versi:** 1.0.0 | **Tanggal:** 2026-09-09 | **Otoritas:** Mandat Pemilik Proyek
**Target Implementasi:** `@agent9` (Integration), `@agent6` (WCB), `@agent4` (Schema), `@agent10` (Compliance)

---

## 1. LATAR BELAKANG & TUJUAN
Banyak sumber data publik bernilai tinggi di internet (seperti orderbook DEX, feed berita kripto, dan data institusi) menerapkan sistem proteksi bot (Cloudflare, Datadome, Akamai). Pengambilan data via HTTP request standar (`curl` atau node HTTP bawaan) sering kali diblokir (HTTP 403 / Cloudflare Challenge).

Untuk melengkapi ekosistem Workflow Hub dan node engine Rust, sistem dilengkapi dengan dua node komplementer:
1. **`n8n-nodes-camofox-browser`**: Untuk navigasi interaktif pada situs web JavaScript-heavy dan anti-bot tingkat tinggi.
2. **`n8n-nodes-scrapling`**: Untuk ekstraksi cepat adaptif (*self-healing*) dengan konsumsi memori super-ringan (<15MB RAM).

---

## 2. NODE 1: `n8n-nodes-camofox-browser`

### 2.1. Karakteristik & Fitur
* **Anti-Detect Fingerprinting**: Memodifikasi fingerprint TLS, WebGL vendor, Canvas noise, AudioContext, dan Navigator flags agar tidak terdeteksi sebagai Chromium otomatis.
* **Humanized Navigation**: Gerakan kursor kurva Bezier, delay pengetikan acak, dan scrolling bertahap.
* **Operasi yang Didukung**:
  * `navigate` (URL, timeout, wait_until: domcontentloaded|networkidle)
  * `extractContent` (selector, output_format: markdown|html|text)
  * `click` (selector, delay_after)
  * `screenshot` (full_page: bool, quality)
  * `evaluate` (script sandboxed)

### 2.2. Manajemen Memori (Hard-Cap 500MB)
* Headless browser dijalankan dalam mode **Ephemerally Spawned CDP Worker**:
  * Maksimal 1 instans browser aktif pada satu waktu.
  * Browser instance otomatis ditutup (*auto-terminate*) setelah navigasi selesai atau timeout 30 detik.
  * Cgroup memori dibatasi maksimal 150MB per worker.

---

## 3. NODE 2: `n8n-nodes-scrapling`

### 2.1. Karakteristik & Fitur
* **Self-Healing DOM Parser**: Jika pemilik situs mengubah nama class atau struktur tag div, parser Scrapling secara cerdas mencocokkan kembali elemen data yang dicari berdasarkan profil semantik dan contoh sebelumnya (*training examples*).
* **Stealth HTTP Mode (RAM Super Hemat < 15MB)**:
  * Melakukan TLS JA3/JA4 fingerprint masquerade murni via socket HTTP tanpa perlu menjalankan engine browser penuh.
  * 10x lebih cepat daripada browser headless dan menggunakan RAM 95% lebih sedikit.
* **Operasi yang Didukung**:
  * `smartExtract` (rules, auto_heal: true)
  * `cssQuery` (selector, attributes[])
  * `xpathQuery` (xpath_expression)
  * `batchScrape` (urls[], concurrency: 1-5)

### 2.2. Auto-Retry & Backoff
* Penanganan bawaan untuk HTTP 429 (Too Many Requests) dan header `Retry-After`.
* Exponential backoff otomatis dengan jitter acak (1-3 detik) untuk mematuhi etika jaringan.

---

## 4. INTEGRASI WORKFLOW HUB & AI AGENT CAPSULE

```
[Sumber Web Terproteksi / Cloudflare]
                │
    ┌───────────┴───────────┐
    ▼                       ▼
(Scrapling HTTP Stealth)  (Camofox Headless)
  [RAM < 15MB]              [JS-Heavy Sites]
    │                       │
    └───────────┬───────────┘
                ▼
      [Raw HTML / Clean DOM]
                │
                ▼
  [Deterministic Token Condenser]
    (Peringkas Teks < 200 Token)
                │
                ▼
  [Passive Skill Capsule] ──> [Node AI Agent / LLM]
```

Output dari kedua node ini langsung dialirkan ke *Deterministic Token Condenser* sebelum diserahkan ke node AI agent, menjamin penghematan token konteks >90%.

---

## 5. GUARDRAILS KEAMANAN & KEPATUHAN (`@agent10`)

1. **Polite Delay Wajib**: Node secara otomatis menyisipkan delay acak 2.0–5.0 detik antar permintaan ke domain yang sama.
2. **Kepatuhan `robots.txt`**: Pengecekan otomatis direktif `robots.txt` sebelum scraping dijalankan (dapat di-override hanya dengan konfigurasi eksplisit pengguna).
3. **Isolasi & Redaksi PII**: Data hasil scraping secara otomatis disaring oleh modul sanitasi PII (email, nomor telepon, kredensial) sebelum disimpan ke log eksekusi atau riwayat alur kerja.
4. **Lisensi & Hak Cipta**: Template Hub yang menggunakan kedua node ini wajib menyertakan tag atribusi sumber data sesuai ketentuan hukum CC-BY / lisensi situs asal.
