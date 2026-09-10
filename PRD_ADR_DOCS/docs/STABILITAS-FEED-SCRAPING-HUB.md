# 🏛️ REKAYASA STABILITAS FEED SCRAPING WORKFLOW HUB
## Standar High-Availability (HA) & Ketahanan Mutlak Penyediaan Data Intelijen
**Status:** ✅ RESMI DISAHKAN (Mandat Pemilik Proyek - 2026-09-09)  
**Tujuan:** Menjamin pasokan data scraping untuk template Bloomberg Suite & OSINT 100% stabil, tahan gangguan, dan bebas downtime.

---

## 1. Tantangan Utama Stabilitas Scraping Web

Penyediaan data scraping rentan mengalami ketidakstabilan akibat 4 faktor eksternal:
1. **Perubahan Frontend / CSS Selector**: Situs target memperbarui kode frontend, mematahkan parser lama.
2. **Lonjakan Proteksi Anti-Bot & Captcha**: Target tiba-tiba menaikkan level proteksi (misal Cloudflare Under Attack Mode).
3. **Downtime / Rate Limit Situs Target**: Server target merespons `503 Service Unavailable` atau `429 Too Many Requests`.
4. **Fluktuasi Jaringan / DNS Timeout**: Koneksi terputus di tengah jalan saat rendering halaman besar.

Untuk mengatasi ini, seluruh node scraping Workflow Hub wajib mematuhi **Arsitektur Stabilitas 4 Pilar**.

---

## 2. Empat Pilar Stabilitas Penyediaan Data

```
[Permintaan Data dari Template Workflow Hub]
                     |
                     v
+-------------------------------------------------------------+
| PILAR 1: CASD CACHE-FIRST (STALE-WHILE-REVALIDATE)          |
| -> Jika cache masih valid (TTL aktif) -> Langsung kembalikan|
| -> Jika target down/error -> Kembalikan snapshot terakhir   |
|    dengan flag { "stale": true } (NOL ERROR BAGI USER)      |
+-------------------------------------------------------------+
                     | (Cache Miss / Background Refresh)
                     v
+-------------------------------------------------------------+
| PILAR 2: TIERED FAILOVER LADDER (CASCADE 4-LAPIS)           |
| Lapis 1: Scrapling HTTP/2 Stealth (Cepat, Hemat RAM <15MB)  |
|    | (Gagal / Terdeteksi Bot)                               |
|    v                                                        |
| Lapis 2: Camofox-Browser Anti-Detect (Full DOM Masking)     |
|    | (Gagal / Butuh Interaksi Kompleks)                     |
|    v                                                        |
| Lapis 3: Browser-Use Autonomous Agent (Visual + Form Solve) |
|    | (Gagal / IP Datacenter Terblokir Total)                |
|    v                                                        |
| Lapis 4: Firecrawl Serverless Fleet (Offloaded Global Proxy)|
+-------------------------------------------------------------+
                     |
                     v
+-------------------------------------------------------------+
| PILAR 3: CIRCUIT-BREAKER & EXPONENTIAL BACKOFF              |
| -> Deteksi 3x kegagalan berturut -> Isolasi domain target   |
| -> Cooldown otomatis 5-15 menit (Cegah IP burn & spam loop) |
| -> Switch otomatis ke sumber data alternatif (Mirror Feed)  |
+-------------------------------------------------------------+
                     |
                     v
+-------------------------------------------------------------+
| PILAR 4: SCHEMA NORMALIZATION GATE (SCHEMA-PINNING)         |
| -> Validasi output terhadap kontrak JSON-Schema ketat       |
| -> Sanitasi tipe data & pembersihan null / NaN              |
| -> Jaminan struktur output konsisten untuk node hilir       |
+-------------------------------------------------------------+
```

---

## 3. Rincian Mekanisme Rekayasa

### 3.1 Pilar 1: Stale-While-Revalidate CASD Cache
* Menggunakan Content-Addressable Storage (CASD) lokal engine Rust.
* Setiap penarikan data diberi TTL (Time-To-Live) sesuai karakteristik data:
  * Berita Makro / Laporan SEC: TTL 60 menit.
  * Harga Saham / Sentimen Pasar: TTL 3 - 5 menit.
  * Indikator Makro FRED: TTL 24 jam.
* **Jaminan Tanpa Gangguan (Graceful Degradation)**: Jika situs target sedang down atau memunculkan captcha, engine otomatis menyajikan data snapshot valid terakhir. Template pengguna **tidak pernah crash atau kosong**.

### 3.2 Pilar 2: Cascading Failover Ladder
* **Otomatis & Transparan**: Pengguna template tidak perlu memilih scraper secara manual. Engine mencoba Scrapling terlebih dahulu untuk efisiensi RAM (<15MB). Jika situs menuntut eksekusi JavaScript berat, tugas secara mulus dialihkan ke Camofox atau Browser-Use.
* **Zero Failure Rate**: Peluang kegagalan scraping turun menjadi mendekati 0% karena selalu ada fallback ke lapis berikutnya.

### 3.3 Pilar 3: Circuit-Breaker & Mirror Feed Routing
* Setiap domain target dipantau tingkat keberhasilannya (*success rate*).
* Jika suatu sumber data eksternal gagal (misal API publik tertentu down), engine secara dinamis mengalihkan penarikan data ke mirror feed cadangan (misal dari FRED Mirror A ke FRED Mirror B, atau Yahoo Finance ke AlphaVantage mirror).

### 3.4 Pilar 4: Schema Normalization Gate
* Apapun perubahan layout HTML di sisi web target, parser wajib menormalkan data ke dalam schema standar yang telah disepakati di manifest template.
* Jika ada field yang hilang akibat perubahan DOM, engine mengisi nilai default atau menandainya secara terstruktur (`field: null, warning: "DOM_SHIFT"`), bukan membuat workflow melempar exception tak tertangani.

---

## 4. SOP Penerapan untuk Agen Kontributor
1. Seluruh template scraping dalam Workflow Hub **WAJIB** mengaktifkan caching lokal dengan opsi fallback `stale_on_error: true`.
2. Seluruh request scraping harus memiliki batasan timeout ketat (maksimal 30 detik per lapis).
3. Scraping massal harus menggunakan *batch rate limiter* untuk mencegah beban berlebih.
