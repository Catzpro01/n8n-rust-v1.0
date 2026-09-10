# AGENT9-SCRAPE-L3-DESIGN — Desain Lapis 3 Scraping: `n8n-nodes-firecrawl` (Ekstraksi Bersih utk LLM)

**Penulis:** agent9 (ROLE_INTEGRATION) · **Tanggal:** 2026-09-09 · **Versi:** v0.1 (DESIGN — freeze #386 dipatuhi, 0 kode)
**Mandat:** ARSITEKTUR-SCRAPING-BERLAPIS.md §2 Lapis 3 (render serverless → Markdown bersih) · queue W2-SCRAPE-L3 (P1, ROLE_INTEGRATION) · turunan AGENT9-SCRAPE-L1-DESIGN.md.

---

## 1. Temuan desain utama: firecrawl BUKAN zero-API-key (jujur, kategorikan benar)

Firecrawl (layanan) = **ber-API-key & berbayar** (ada free tier terbatas). Ini **melanggar prinsip "Zero-API-Key" katalog Hub** (D2 §2 kriteria inklusi #1). Konsekuensi yang benar:

1. Node `firecrawl` TETAP dibuat — tapi sebagai node **ber-kredensial** (`NodeDescriptor.credentials: firecrawlApi`), BUKAN sumber katalog zero-key.
2. **Katalog Hub v1 TIDAK memuat sumber firecrawl** — template Hub yang memakainya = kategori terpisah "bring-your-own-key" (BYOK), ditandai eksplisit di manifest (usul field `requires_credentials: [firecrawlApi]` — koordinasi agent4).
3. **Alternatif self-host**: firecrawl open-source bisa self-host `[VERIFIKASI-UPSTREAM: lisensi & batas self-host firecrawl]` → instance self-host tim = opsi future utk template Hub gratis (offline-friendly); belum keputusan (biaya operasi).

## 2. Model parameter (→ `ParameterSchema`, arah-A agent4)

| # | Field | Tipe IR | Wajib | Default | Catatan |
|---|---|---|---|---|---|
| 1 | `resource` | `OptionsField` | ✅ | `scraper` | |
| 2 | `operation` | `OptionsField` | ✅ | `scrape` | `scrape` \| `crawl` \| `map` |
| 3 | `url` | `StringField` | ✅ | — | SSRF-guard §5 (tetap di-enforce walau render di server firecrawl — cegah target internal kami) |
| 4 | `formats` | `MultiOptionsField` | ⬜ | `["markdown"]` | `markdown` \| `html` \| `json` \| `screenshot` |
| 5 | `onlyMainContent` | `BooleanField` | ⬜ | `true` | auto-boilerplate-removal (ARSITEKTUR L3) |
| 6 | `maxDepth` (crawl) | `NumberField` | ⬜ | `1` | 1–5 (full-site crawl terbatas) |
| 7 | `limit` (crawl/map) | `NumberField` | ⬜ | `50` | max 500 |
| 8 | `waitFor` | `NumberField` | ⬜ | `0` | ms tunggu render JS |
| 9 | `baseUrlOverride` | `StringField` | ⬜ | — | utk self-host (§1.3) — default endpoint resmi |

## 3. ResourceHint

- **SideEffect: `Idempotent` semua operasi** (read-only render+fetch).
- `weight`: ringan di sisi kita (render terjadi di server firecrawl) — network-bound; `max_concurrency`: ikut rate-policy firecrawl (header `X-RateLimit-*`/429 dihormati; pola backoff scrapling §3 reuse).
- `buffering`: Batch; crawl besar → Streaming (potongan per-halaman, hindari akumulasi).

## 4. Output kanon & rantai Hub

```
{ url, status, format: markdown|json|html,
  content_ref,           // via data-plane — Markdown bersih siap kondenser
  main_content_only: true, took_ms }
```
Peran L3 di Hub (ARSITEKTUR §3): **berita portal/artikel politik** → Markdown bersih → Token Condenser <200 token. Output = tainted-external; sanitasi PII tetap (firecrawl membersihkan boilerplate, BUKAN PII — dua hal berbeda, jangan diasumsikan).

## 5. Guardrails

Modul-bersama L1 penuh (SSRF, polite delay ke API firecrawl, robots, PII, atribusi) + tambahan:
1. **Kredensial**: apiKey via credential store (encripsi at-rest PRD-2 §11; redaksi log — INTEG-05); TIDAK PERNAH di parameter URL/log.
2. **crawl limit**: maxDepth×limit dibatasi admission (biaya kuota API berbayar — anggaran per eksekusi, usul default 100 halaman/run; kelebihan = `E-L3-CRAWL-BUDGET`).
3. BYOK template: manifest wajib `requires_credentials` + dokumentasi biaya (anti kejutan tagihan).

## 6. Gate (post-freeze)

| Gate | Kriteria PASS | Reproduksi |
|---|---|---|
| SCRP-L3-1 | SSRF 20/20 ditolak SEBELUM request ke firecrawl | uji unit |
| SCRP-L3-2 | kredensial: 0 apiKey di log/URL/snapshot pada 100 call (INTEG-05 reuse) | check-secrets |
| SCRP-L3-3 | crawl budget: maxDepth=3, limit=500 → berhenti di budget + kode eksak | mock API |
| SCRP-L3-4 | 429/Retry-After firecrawl → backoff pola scrapling; 3 attempt max | mock |
| SCRP-L3-5 | output 10 artikel uji → Markdown bersih (0 navbar/footer marker) + PII scan 0 | fixture |

## 7. Terbuka

- **OPEN-SCRAPE-3** (agent4): field manifest `requires_credentials` + kategori BYOK di katalog/registry Hub.
- **OPEN-SCRAPE-4** (matt/fern): self-host firecrawl — layak atau tidak utk template gratis (biaya VPS vs zero-key promise).
- `[VERIFIKASI-UPSTREAM: param & header rate-limit firecrawl asli]` — baca docs firecrawl saat implementasi.
