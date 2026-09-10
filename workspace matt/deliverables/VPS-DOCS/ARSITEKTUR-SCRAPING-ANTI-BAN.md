# 🛡️ ARSITEKTUR SCRAPING ANTI-BAN ZERO-RISK (5-LAYER SHIELD)
## Standar Keamanan Mutlak Penarikan Data Tanpa Risiko Blokir IP / Anti-Bot
**Status:** ✅ RESMI DISAHKAN  
**Kompatibilitas:** Camofox, Scrapling, Browser-Use, Firecrawl, Puppeteer MCP

---

## 1. Filosofi & Masalah Utama Deteksi Bot

Situs modern (Cloudflare Turnstile, DataDome, Akamai, PerimeterX, AWS WAF) mendeteksi dan memblokir scraper melalui 4 vektor utama:
1. **Reputasi IP**: IP datacenter/VPS langsung ditandai berisiko tinggi (*high risk score*).
2. **Fingerprint TLS & TCP (JA3/JA4)**: Header HTTP bisa dipalsukan, tetapi urutan Cipher Suite TLS pada koneksi soket membocorkan bahwa klien adalah script otomatis (curl, python-requests, Puppeteer standar).
3. **Hardware & Browser Leaks**: Panggilan WebGL, Canvas render, AudioContext, `navigator.webdriver = true`, dan resolusi layar yang tidak wajar.
4. **Pola Perilaku (Behavioral Anomaly)**: Kecepatan klik instan, interval request statis, kursor melompat linear, dan volume request berlebih dalam waktu singkat.

Untuk mencapai **Zero-Risk Ban**, kelima vektor di atas dinetralisasi melalui **5-Layer Shield Architecture**:

---

## 2. Lima Lapisan Proteksi Anti-Ban (5-Layer Shield)

```
[Target Website (Cloudflare/DataDome/Akamai)]
                     ^
                     |
+-------------------------------------------------------------+
| LAYER 1: NETWORK & ROTATING RESIDENTIAL PROXY POOL          |
| -> IP Perumahan Asli (Bukan Datacenter), Rotasi Dinamis     |
| -> Automatic Cooldown jika terdeteksi 429/403               |
+-------------------------------------------------------------+
                     ^
                     |
+-------------------------------------------------------------+
| LAYER 2: TLS & BROWSER FINGERPRINT MASKING (CAMOFOX)        |
| -> Spoofing JA3/JA4 TLS Client Hello (identik Chrome asli)  |
| -> Masking WebGL, Canvas, AudioContext, Font List, WebRTC   |
| -> Menghapus flag navigator.webdriver = true                |
+-------------------------------------------------------------+
                     ^
                     |
+-------------------------------------------------------------+
| LAYER 3: HUMAN BEHAVIORAL JITTER (BROWSER-USE)              |
| -> Gerakan Mouse Kurva Bézier alami (Natural Acceleration)  |
| -> Penundaan Interval Gaussian (Jitter Acak 1.5s - 4.2s)    |
| -> Smooth Scroll berbasis viewport manusia                  |
+-------------------------------------------------------------+
                     ^
                     |
+-------------------------------------------------------------+
| LAYER 4: SELF-HEALING DOM & STEALTH HEADERS (SCRAPLING)     |
| -> Penyesuaian otomatis jika selector CSS diacak dinamis    |
| -> Rotasi header Accept-Language, Sec-Ch-Ua, Referer        |
+-------------------------------------------------------------+
                     ^
                     |
+-------------------------------------------------------------+
| LAYER 5: CONTENT-ADDRESSABLE CACHE & ISOLATION (DATA-PLANE) |
| -> CASD Dedup: Tidak pernah fetch ulang halaman statis      |
| -> Hard-Cap RAM <500MB, worker cgroup ephemeral (kill <=30s)|
+-------------------------------------------------------------+
```

---

## 3. Rincian Konfigurasi Zero-Risk per Node

### 3.1 Node Camofox-Browser (Anti-Detect Browser)
* **Canvas & WebGL Noise**: Menambahkan sedikit distorsi noise kriptografis pada piksel canvas sehingga hash fingerprint unik setiap sesi namun konsisten secara logis.
* **AudioContext Spoofing**: Mensimulasikan variasi resonansi hardware kartu suara pengguna asli.
* **WebRTC Leak Protection**: Mencegah kebocoran IP asli VPS melalui protokol WebRTC ICE candidate.

### 3.2 Node Scrapling (Self-Healing & TLS Masquerading)
* Mengadopsi library HTTP/2 dengan impersonasi TLS Chrome v128+ (JA4 matching).
* **Self-Healing Selector**: Jika situs target memperbarui frontend dan mengubah `<div class="price-x892a">` menjadi `<div class="p-91a">`, engine menggunakan keselarasan semantik teks untuk tetap menemukan elemen tanpa melempar error.

### 3.3 Node Browser-Use (Human Interaction Simulation)
* Menghindari aksi linear. Setiap pergerakan kursor mouse dihitung menggunakan persamaan kurva Bézier derajat 3 ($B(t) = (1-t)^3 P_0 + 3(1-t)^2 t P_1 + 3(1-t) t^2 P_2 + t^3 P_3$).
* Jeda pengetikan form karakter per karakter bervariasi antara 45ms hingga 180ms (kecepatan ketik natural manusia).

### 3.4 Node Firecrawl (Serverless Fallback)
* Jika target menerapkan proteksi tingkat militer (*extreme tier*), request didelegasikan ke Lapis 3 (Firecrawl serverless fleet). IP VPS Anda **100% terlindungi** dan tidak pernah menyentuh firewall target.

---

## 4. SOP & Aturan Operasional Anti-Ban
1. **Aturan 1 (No Static Delay)**: DILARANG menggunakan interval `setInterval(1000)` yang statis. WAJIB menggunakan jitter acak: `sleep(random(min, max))`.
2. **Aturan 2 (Proxy Gate)**: Untuk scraping massal (>50 halaman/domain), WAJIB menggunakan pool proxy residensial.
3. **Aturan 3 (Graceful Backoff)**: Jika menerima response status `429`, hentikan request ke domain tersebut secara otomatis selama minimum 5 menit (*exponential backoff*).
4. **Aturan 4 (Respect Session Lifespan)**: Buat sesi baru (cookie & fingerprint segar) setiap 50-100 request untuk menghindari akumulasi reputasi jejak bot.
