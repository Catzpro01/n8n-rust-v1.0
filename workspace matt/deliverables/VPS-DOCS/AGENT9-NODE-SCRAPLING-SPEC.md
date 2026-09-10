# AGENT9-NODE-SCRAPLING-SPEC — Desain Node `n8n-nodes-scrapling` (Lapis 1, mode hemat)

**Penulis:** agent9 (ROLE_INTEGRATION) · **Tanggal:** 2026-09-09 · **Versi:** v0.1 (DESIGN — freeze #386 dipatuhi, 0 kode)
**Mandat:** fern #744 · ARSITEKTUR-SCRAPING-BERLAPIS.md Lapis 1 · NODES-CAMOFOX-SCRAPLING-SPEC.md §3 · queue W2-NODE-SCRAPLING (P1, ROLE_INTEGRATION).
**Posisi:** node komunitas jalur 5; Lapis 1 — **mode default L1** (HTTP stealth <15MB); eskalasi ke camofox hanya bila situs butuh JS penuh (smart-routing L1 → AGENT9-SCRAPE-L1-DESIGN.md).

---

## 1. Model parameter (→ `ParameterSchema`, arah-A agent4)

| # | Field | Tipe IR | Wajib | Default | Catatan |
|---|---|---|---|---|---|
| 1 | `resource` | `OptionsField` | ✅ | `scraper` | |
| 2 | `operation` | `OptionsField` | ✅ | `smartExtract` | `smartExtract` \| `cssQuery` \| `xpathQuery` \| `batchScrape` (NODES-SPEC §3) |
| 3 | `url` / `urls[]` | `StringField` / `StringArrayField` | ✅ | — | batchScrape: 1–50 URL; **SSRF-guard §4 per URL** |
| 4 | `rules` (smartExtract) | `JsonField` OPAQUE | ✅ (smartExtract) | — | aturan ekstraksi deklaratif `{selector, fields[], output}`; opaque = tidak dikompilasi jadi fixedCollection (pola INTEGRATION-SPEC §1.2 #9) |
| 5 | `autoHeal` | `BooleanField` | ⬜ | `true` | self-healing parser (NODES-SPEC §3) |
| 6 | `selector` / `attributes[]` (cssQuery) | `StringField`/`StringArrayField` | ✅ (cssQuery) | — | |
| 7 | `xpath` (xpathQuery) | `StringField` | ✅ (xpathQuery) | — | |
| 8 | `concurrency` (batchScrape) | `NumberField` | ⬜ | `2` | 1–5 (NODES-SPEC §3) |
| 9 | `mode` | `OptionsField` | ⬜ | `httpStealth` | `httpStealth` \| `browserFallback` (browserFallback = delegasi camofox) |
| 10 | `options.respectRetryAfter` | `BooleanField` | ⬜ | `true` | |
| 11 | `options.ignoreRobots` | `BooleanField` | ⬜ | `false` | sama semantik camofox §4.3 |

## 2. ResourceHint & memory

| Operasi | SideEffect | Catatan |
|---|---|---|
| `smartExtract` / `cssQuery` / `xpathQuery` | `Idempotent` | read-only |
| `batchScrape` | `Idempotent` | read-only; rate-policy host-global per-domain (W3-INGRESS §3) |

- `weight`: ringan — mode `httpStealth` tanpa browser: **RAM < 15MB** (NODES-SPEC §3; mandat fern #744: prioritas mode ini "agar aman dalam budget 500MB").
- `max_concurrency`: `concurrency`×domain berbeda ≤5; **per domain sama = 1** (polite delay §4).
- `buffering`: `Batch`.
- TLS JA3/JA4 masquerade = fungsi **host** (bukan guest) — interface `wcb:net/request` agent6 dengan profil TLS; node hanya mendeklarasikan `mode`.

## 3. Auto-retry & backoff (NODES-SPEC §3.2 — kontrak eksak)

- HTTP 429 / 5xx: backoff eksponensial **1.5^n detik** + jitter acak 1–3s (n = attempt), max 3 attempt; `respectRetryAfter=true` (default) → `Retry-After` server **menang** atas rumus (min(Retry-After, backoff) tidak dibatasi — nilai server diikuti penuh).
- Gagal total → item output `{url, status: 'failed', error_code}` (fail-open-informatif — template tidak mati senyap, konsisten W3 §4).
- **Determinisme**: jitter & delay = sumber non-determinisme → di-seed dari determinism record (agent1 §6.2.1) supaya replay deterministik; dicatat di kelas inventaris agent5 (seperti cron-v1 generated).

## 4. Guardrails (identik camofox §4 — dibagikan sebagai modul umum L1)

SSRF blocklist pra-fetch per-URL (E-SSRF-BLOCKED) · polite delay 2.0–5.0s per domain sama · robots.txt auto-check (ignoreRobots = eksplisit + audit) · PII redaction pra-log · atribusi lisensi template (H-03). **Catatan khusus scrapling**: self-healing parser BUKAN alasan menembus ToS — target tetap harus lolos gate etika (SCRAP-E1, lihat camofox §5; berlaku sama).

## 5. Output kanon

```
{ url, status, content_format: json|markdown|html,
  data_ref / content_ref,   // hasil ekstraksi via data-plane; batchScrape = array per-URL
  auto_healed: bool,        // true bila parser melakukan re-match semantik (sinyal kualitas utk gate diff W4)
  pii_redacted: true, took_ms, attempts }
```
`auto_healed=true` + struktur berubah → di-hubungkan ke **structural_guard** manifest Hub (opsi agent4 W4 §4.3): sinyal "format sumber berubah" → diff struktural W4, bukan silent.

## 6. Gate verifikasi (post-freeze)

| Gate | Kriteria PASS | Reproduksi |
|---|---|---|
| SCRP-SL-1 | RAM mode httpStealth <15MB pada 20 halaman uji (ukur RSS proses host-handler) | uji bebar + /proc RSS |
| SCRP-SL-2 | SSRF corpus 20/20 ditolak; batchScrape 50 URL → semua di-guard | sama SCRP-CF-1 |
| SCRP-SL-3 | 429 + Retry-After: 7s → tidak ada retry sebelum 7s; tanpa header → backoff 1.5^n+jitter(1-3s) terverifikasi dari log seed | mock server |
| SCRP-SL-4 | self-heal: 10 halaman dengan class diacak (css-1a2b3c → css-9z8y7x) → field tetap terekstrak ≥8/10; `auto_healed=true` | fixture repo |
| SCRP-SL-5 | PII scan output & log = 0 kebocoran (SCRP-CF-5 reuse) | check-secrets |
| SCRP-SL-6 | replay deterministik: 2 run seed sama → output byte-identik (termasuk urutan & jitter) | harness differential |

## 7. Terbuka

- **SCRAP-E1** berlaku juga di sini (stealth TLS = kategori sama dengan fingerprint masking).
- Kontrak `wcb:net/request` profil TLS (JA3/JA4) = agent6 — detail teknis masquerade di luar dokumen ini.
- `verifications/scrapling.json` (golden) post-freeze; status katalog `verified-tofu` sampai itu.
