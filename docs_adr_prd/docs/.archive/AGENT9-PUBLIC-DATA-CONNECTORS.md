# AGENT9-PUBLIC-DATA-CONNECTORS — Pemetaan Konektor Sumber Data Publik Gratis (D2, Workflow Hub)

**Penulis:** agent9 (ROLE_INTEGRATION)
**Tanggal:** 2026-09-09 · **Versi:** v0.3
**Riwayat:** v0.1 batch-1 (8 uji). v0.2: batch-2 (12 uji) + katalog mesin `hub-intel-types.json`. v0.3: batch-3 (MET Norway lolos dgn UA jujur, USGS lolos; govtrack & data.go.id gagal final-jujur) + **adopsi SEC-HUB-01 agent10 #633**: field `pin.expected_sha256` per sumber (null = belum), `verify.status` enum baru `verified-tofu`/`verified-anchored`/`failed`, profil tetap single-source (manifest tanpa pin) — katalog naik schema_version 2.
**Mandat:** Tantangan WORKFLOW HUB dari Pemilik Proyek (diumumkan fern di #general, ~#525): tugas bidang agent9 = *"Pemetaan konektor sumber data publik gratis (Zero-API-Key feeds, RSS, public JSON endpoints)"* — untuk kategori Kripto & Web3, Politik & Berita Dunia, dan open data.
**Status kepatuhan freeze #386:** dokumen + pengujian read-only saja; **nol baris kode Rust**; semua uji = HTTP GET/HEAD publik tanpa autentikasi.

---

## 1. Posisi dalam arsitektur Workflow Hub

Workflow Hub = **pre-processing layer** sebelum input sampai ke AI Agent. Rantai kepemilikan (selaras pembagian agent1 #528 dan pengumuman fern):

```
[Konektor sumber data — DOKUMEN INI, agent9]
   → [fetch HTTP/RSS via engine — node HTTP-Request deklaratif / rssRead; ingress+admission agent1 §5.2]
   → [sanitasi & etika — agent5 (robots/ToS, timeout, prompt-injection)]
   → [kompresi ekstraktif transien — agent3 + agent1, <200 token]
   → [katalog MCP — agent7 (discovery; INVOKASI via API eksekusi terpisah, bukan via MCP — MCP-11)]
```

Kontrak lintas-agent yang dokumen ini penuhi: (a) metadata per-sumber untuk **manifest agent4** (§7); (b) kelas determinisme untuk **record-replay agent1 §6.2.1** dan inventaris agent5 (§4); (c) bahan **gate etika agent5** (§5); (d) seluruh output sumber = **TAINTED-EXTERNAL** — dilarang mengalir ke kredensial/partials (invarian-taint agent1 §3).

## 2. Kriteria inklusi & metodologi

Konektor masuk katalog v1 hanya bila: (1) **gratis & zero-API-key** (tanpa registrasi); (2) **alamat deterministik** (URL stabil, tanpa sesi); (3) lolos **uji live dari VPS tim** (status/ukuran/header dicatat); (4) lisensi/ToS mengizinkan konsumsi programatik (catatan agent5 §5). Kandidat yang belum diuji ditandai `[VERIFY]` — TIDAK dihitung sebagai terverifikasi (disiplin PRD-1 §18.4: klaim tanpa reproduksi = bukan klaim).

Reproduksi seluruh uji (dijalankan di VPS `master`, 2026-09-09 ~06:5x UTC):
`curl -sL -m 8 -D - -o /dev/null "<URL>"` (output lengkap per-URL di bawah tiap baris tabel).

## 3. Katalog konektor v1

### 3.1 Kripto & Web3

| Sumber | Endpoint contoh | Bukti uji (VPS) | Rate limit / cache | Volatilitas |
|---|---|---|---|---|
| **CoinGecko public** | `api.coingecko.com/api/v3/ping`, `/simple/price?ids=bitcoin&vs_currencies=usd` | ✅ 200, 34B, t=1.37s · `etag` + `cache-control: private, must-revalidate` | Free tanpa key ~10-30 req/min (header `x-ratelimit-*` saat ter-limit; docs) | harga: per-menit |
| **Binance public market** | `api.binance.com/api/v3/ticker/24hr?symbol=BTCUSDT` | ✅ 200, 556B, t=0.17s · **`x-mbx-used-weight-1m: 2`** (budget weight terlihat eksplisit) | weight-based per-IP (header teramati; docs: 6000 weight/menit) | ticker: detik-menit |
| **DefiLlama** | `coins.llama.fi/prices/current/coingecko:bitcoin`, `api.llama.fi/protocols` | ✅ 200, 115B, t=1.14s | open source, tanpa key; tidak terdokumentasi ketat `[VERIFY: batas praktis]` | protokol: jam |
| **DexScreener** | `api.dexscreener.com/latest/dex/search?q=WBTC` | ✅ 200, 25.386B, t=1.14s · `etag` + **`cache-control: public, max-age=60`** | docs API publik: 300 req/menit `[VERIFY: docs resmi]` | pair: detik |
| On-chain signals (kandidat) | mempool.space API, blockchain.info, public RPC (ethereum.publicnode.com) | `[VERIFY — belum diuji dari VPS]` | — | — |

### 3.2 Politik & Berita Dunia

| Sumber | Endpoint contoh | Bukti uji (VPS) | Catatan |
|---|---|---|---|
| **RSS media kredibel** | `feeds.bbci.co.uk/news/world/rss.xml` | ✅ 200, 23.188B, t=1.25s · `cache-control: public, max-age=4` | pola RSS standar; kandidat lain `[VERIFY]`: Guardian open RSS, Al Jazeera, NPR |
| **Agregator RSS** | `news.google.com/rss/search?q=<topik>` | `[VERIFY — belum diuji]` | ampuh utk "politik terbaru per-topik"; status ToS = input agent5 |
| **Wikipedia updates (stream)** | `stream.wikimedia.org/v2/stream/recentchange` | ✅ 200 SSE, ~391KB per 8 detik · `cache-control: no-cache` | event-firehose; utk Hub: konversi ke sinyal ringkas; alternatif lebih tenang: MediaWiki API recentchanges `[VERIFY]` |
| Open policy feeds (kandidat) | govtrack (US legislation) RSS/API, parltrack (EU) | `[VERIFY]` | cakupan politik kebijakan non-partisan |

### 3.3 Open data pemerintahan & keuangan

| Sumber | Endpoint contoh | Bukti uji (VPS) | Catatan |
|---|---|---|---|
| **Frankfurter (kurs, data ECB)** | `api.frankfurter.app/latest?from=USD` | ✅ 200, 433B, t=2.26s · **`etag` kuat + `cache-control: public, max-age=86400`** | open source; ideal utk cache kondisional; punya **OpenAPI spec resmi** (frankfurter.app) |
| **World Bank API** | `api.worldbank.org/v2/country/IDN/indicator/SP.POP.TOTL?format=json` | ✅ **PULIH (batch-2): 3× 200, t=0.08-0.11s**, `max-age=86400` | 2× timeout transien sejam sebelumnya dicatat jujur — flakiness masuk profil katalog |
| data.go.id (CKAN) & portal nasional | `/api/3/action/package_list` | `[VERIFY]` | relevan pasar ID (pemilik proyek di Surabaya) |

### 3.4 Cuaca & lingkungan (pencipta sinyal kontekstual)

| Sumber | Endpoint contoh | Bukti uji | Catatan |
|---|---|---|---|
| Open-Meteo | `api.open-meteo.com/v1/forecast?latitude=-7.25&longitude=112.75&current_weather=true` | ✅ **batch-2: 200, 477B, t=2.09s** (koordinat Surabaya) | zero-key; non-komersial + attribution — review agent5 utk lisensi |
| MET Norway | `api.met.no/weatherapi/locationforecast/2.0/compact?lat=..&lon=..` | `[VERIFY — batch-3]` | wajib User-Agent teridentifikasi (ToS) |

### 3.5 Batch-2 (2026-09-09, 12 uji tambahan)

| Sumber | Bukti uji (VPS) | Catatan |
|---|---|---|
| **Guardian world RSS** | ✅ 200, 158.658B, t=1.16s · `etag W/` + `max-age=60` + `stale-while-revalidate=6` + `stale-if-error=864000` | header paling ramah-cache di katalog |
| **Al Jazeera all RSS** | ✅ 200, 17.273B, t=1.18s · `etag W/` | |
| **NPR news RSS** | ✅ 200, 14.678B, t=1.18s · `max-age=216` | |
| **mempool.space** `api/blocks/tip/height` | ✅ 200, 6B, t=1.30s · `max-age=1` | sinyal on-chain termurah (6 byte!) |
| **blockchain.info ticker** | ✅ 200, 2.787B, t=0.09s · `max-age=60` | tercepat di katalog |
| **Google News RSS** (search, hl=id) | ✅ 200, 129.823B, t=1.10s · `no-cache` | **TEKNIS lolos, ETIKA TERBLOKIR**: terms status `grey-BLOCKED-agent5` — tidak masuk katalog live sebelum OPEN-INTEG-10 diputuskan |
| **Ethereum publicnode RPC** (POST JSON-RPC `eth_blockNumber`) | ✅ 200, 46B, result `0x18bc943` | POST tapi tetap zero-key & read-only → Idempotent |
| govtrack.us bills API | ❌ 400 (32B) — endpoint/param berubah | perlu koreksi query; bukan katalog |
| data.go.id CKAN | ❌ 404 (HTML) — path berubah / turun | kandidat portal data nasional; batch-3 |

### 3.6 Batch-3 (2026-09-09, 6 uji)

| Sumber | Bukti uji (VPS) | Catatan |
|---|---|---|
| **MET Norway locationforecast** | ✅ 200, 41.243B, t=2.04s · `expires` terjadwal | **dengan UA jujur teridentifikasi** (`8n-workflow-hub-connector/0.1 (proyek engine Rust; uji konektor; kontak: tim via VPS master)`) — patuh ToS UA MET |
| **USGS earthquakes all_hour** | ✅ 200, 6.117B, t=2.16s · `max-age=60` + `expires` + `last-modified` | **public domain** (pemerintah AS) — lisensi paling bersih di katalog; sinyal gempa jam-by-jam |
| govtrack.us (2 varian query baru) | ❌ 400 ×2 ("Cannot resolve keyword 'per_page' into field") | **gagal final** — API v2 govtrack param-nya berubah total; keluar dari antrean sampai ada baca docs baru |
| data.go.id (root + CKAN site_read) | ❌ root 200 tapi `site_read` 404 | **gagal final** — portal tidak lagi menyajikan CKAN API di path itu; alternatif portal data nasional ditunda ke batch-4 |

### 3.7 Batch-4 mini (2026-09-09, portal data nasional alternatif)

| Sumber | Hasil | Catatan |
|---|---|---|
| satudata.go.id (CKAN) | ❌ 000 (koneksi gagal cepat) | DNS/jaringan bermasalah dari VPS |
| data.jakarta.go.id (CKAN) | ❌ 000 (timeout 10s) | tidak merespons |
| opendata.jabarprov.go.id (CKAN) | ❌ 403 (HTML WAF 363KB) | diblok WAF utk akses programatik |

**Kesimpulan batch-4 (jujur): seluruh 3 portal open-data nasional/provinsi GAGAL diakses dari VPS** — tidak ada tambahan katalog; kategori opendata Indonesia tetap diwakili World Bank saja. Tidak di-retry buta; butuh investigasi terpisah (kemungkinan geo-blocking/WAF) bila prioritas naik.

**Ringkasan kejujuran (v0.3):** **18 sumber di katalog — 17 approved + 1 ethics-blocked (Google News)**, semuanya `verify.status = verified-tofu` (teruji live, belum berpin — jangkar pin = katalog, per SEC-HUB-01 agent10). 2 gagal final tercatat (govtrack, data.go.id). Katalog mesin resmi = `hub-intel-types.json` **schema_version 2** (field `pin.expected_sha256` null-able + enum status baru + tetap single-source). Klaim publik Hub hanya boleh memuat jumlah **terverifikasi & approved**.

## 4. Klasifikasi determinisme (interface record-replay agent1 §6.2.1 & DETERMINISM-SOURCE-INVENTORY agent5)

- **Semua** sumber ini = **input-eksternal** (kelas dominan korpus: 128/171 file menyerap input eksternal — inventaris agent5). Konsekuensi: fetch WAJIB lewat lapisan record-replay (indeks hash-request), BUKAN snapshot state; replay deterministik = respons terekam diputar ulang.
- **Volatilitas** (menentukan umur cache & jadwal auto-update manifest agent4): detik-menit (Binance, DexScreener) · menit (CoinGecko) · jam (DefiLlama, RSS) · harian (Frankfurter `max-age=86400`, World Bank) · stream (Wikimedia SSE — perlakuan khusus: potong jendela waktu, bukan konsumsi abadi).
- **Fetch kondisional:** CoinGecko/DexScreener/Frankfurter mengirim `ETag` → `If-None-Match` hemat bandwidth & ramah rate-limit; 304 = tanpa perubahan (sinyal "tetap" gratis).
- **Differential testing:** workflow Hub TIDAK ikut uji L3 byte-identik vs n8n live (konten berubah per-detik) — pakai mode record/skip ala N-22 (agent4 NORMALIZER): uji bentuk output (schema + ukuran), bukan nilai.

## 5. Etika, keamanan & rate limit (interface agent5)

1. **Zero credential by design** — seluruh katalog tanpa kredensial → permukaan serang minimal, selaras aturan L0 *zero credential leaks*; TIDAK ada rahasia yang bisa bocor lewat log/URL.
2. **Rate limit teramati di header** (bukti §3): Binance `x-mbx-used-weight-1m` (angka langsung terbaca per-response — bisa jadi sinyal backpressure deterministik), DexScreener `max-age=60`, Frankfurter `max-age=86400`. Rule: hormati `retry-after` bila ada; jarak antar-poll ≥ umur cache.
3. **User-Agent jujur & teridentifikasi** (MET Norway mewajibkannya) — default UA product+versi kita, no spoofing.
4. **ToS/robots:** BBC feed = untuk konsumsi wajar; CoinGecko free tier = batas rate; Google News RSS = area abu-abu → **putusan agent5 sebelum masuk v1**.
5. **Sanitasi prompt-injection:** seluruh konten web = TAINTED-EXTERNAL: di-strip/di-neutralisasi sebelum masuk kompresor & AI Agent (gate agent5); tidak pernah mengalir ke kredensial/partials (invarian-taint #528).
6. **Timeout guard:** semua fetch bounded (contoh uji: `-m 8`); kegagalan/timeout = sinyal "sumber tidak tersedia" pada output Hub, bukan kegagalan workflow — gagal-terbuka-informatif, gagal-tertutup-diam dilarang.

## 6. Pemetaan node & jalur integrasi engine

- **RSS** → node `n8n-nodes-base.rssRead` (paritas; korpus: node RSS muncul di template — frekuensi persis di ANALISIS-KORPUS-NODE) — poll terjadwal via `scheduleTrigger`.
- **JSON endpoint** → node HTTP-Request **deklaratif** (routing.send.query/path), semua GET publik → `SideEffect = Idempotent` trivially (tabel agent4 §9) → aman retry.
- **Sinergi codegen arah-B:** dua sumber terverifikasi punya **OpenAPI spec resmi** — Binance (spot API definitions) & Frankfurter → kandidat **golden spec INTEG-01** `openapi-codegen`: node hasil generate = jalur paling murah menyerap katalog ini. Sumber tanpa spec → parameter node tulisan-tangan (arah A, agent4).
- **Wikimedia SSE** → butuh dukungan streaming response/consumer khusus — dicatat sebagai kebutuhan baru utk `ResourceHint.buffering = Streaming` (PRD-2 §7.2), jangan dipaksa ke pola request-response.

## 7. Metadata manifest per sumber (interface skema agent4)

Usulan field yang saya sediakan dari katalog ini (format final milik agent4):

```json
{
  "source_id": "coingecko-simple-price",
  "kind": "http_json",
  "endpoint": "https://api.coingecko.com/api/v3/simple/price",
  "auth": "none",
  "params": {"ids": "string", "vs_currencies": "string"},
  "license": "CoinGecko free public API ToS",
  "rate_limit": {"observed_header": null, "documented": "~10-30/min"},
  "cache": {"etag": true, "max_age": null},
  "volatility": "per-minute",
  "category": ["crypto"],
  "verify": {"status": "verified-live", "date": "2026-09-09", "from": "VPS master",
             "evidence": "HTTP 200, 34B, etag present"},
  "sanitize": "tainted-external"
}
```

## 8. Keputusan yang diminta (OPEN-INTEG-8..10 — matt/fern/agent5)

- **OPEN-INTEG-8:** set final 10 sumber v1 — usulan: 7 terverifikasi (§3) + 3 dari kandidat `[VERIFY]` setelah diuji ulang. *Rekomendasi: uji ulang World Bank dari VPS (2× timeout) sebelum diputuskan.*
- **OPEN-INTEG-9:** kebijakan cache kondisional ETag/If-None-Match di engine (hemat + ramah rate-limit) — level engine (agent1) atau level template (Hub)? *Rekomendasi: level engine, supaya semua template mewarisinya.*
- **OPEN-INTEG-10:** Google News RSS masuk v1? (kekuatan agregasi politik tinggi, ToS abu-abu). *Rekomendasi: tunggu putusan etika agent5.*

## 9. Langkah berikutnya (agent9)

1. ~~Uji ulang World Bank + kandidat batch-2~~ **SELESAI (v0.2)** — World Bank pulih, 9 sumber baru lolos; sisa `[VERIFY]`: MET Norway, govtrack (koreksi query), data.go.id (batch-3).
2. ~~Emit metadata manifest~~ **SELESAI (v0.2)** — `hub-intel-types.json` = katalog resmi (hub://intel-types), profil 16 sumber; W3-HUB-INGRESS design terbit (AGENT9-W3-HUB-INGRESS.md).
3. Setelah freeze dicabut: Binance + Frankfurter OpenAPI spec masuk pipeline `openapi-codegen` (INTEG-01).
4. Batch-3 kecil: MET Norway (UA teridentifikasi), govtrack koreksi param, data.go.id retry.

*Bukti hidup di §3 diukur, bukan diasumsikan; kandidat tidak menyamar sebagai terverifikasi. — agent9*
