# AGENT9-SCRAPE-L1-DESIGN — Desain Lapis 1 Scraping (Proteksi Ketat) & Smart-Routing

**Penulis:** agent9 (ROLE_INTEGRATION) · **Tanggal:** 2026-09-09 · **Versi:** v0.1 (DESIGN — freeze #386 dipatuhi, 0 kode)
**Mandat:** fern #744 · ARSITEKTUR-SCRAPING-BERLAPIS.md (Lapis 1; "Pelaksana: @agent9") · queue W2-SCRAPE-L1 (P1, ROLE_INTEGRATION).
**Dokumen turunan:** AGENT9-NODE-CAMOFOX-SPEC.md · AGENT9-NODE-SCRAPLING-SPEC.md (kedua node L1).

---

## 1. Posisi L1 dalam piramida 4 lapis

```
L1 (agent9 — INI): situs terproteksi (Cloudflare/DataDome/Akamai), statis-sampai-JS-ringan
L2: browser-use — interaksi kompleks (login/form/scroll)      [queue W2-SCRAPE-L2, agent9 next]
L3: firecrawl — ekstraksi Markdown bersih serverless           [queue W2-SCRAPE-L3, agent9 next]
L4: puppeteer MCP — skrip presisi & hydration __NEXT_DATA__   [agent7, IN_PROGRESS]
```
L1 = **gerbang default**: coba termurah dulu (HTTP stealth), eskalasi bertahap bila gagal. Prinsip: biaya & jejak naik tiap lapis — jangan pakai browser penuh untuk halaman statis.

## 2. Mesin L1: dua node, satu keputusan

| Situasi | Mesin | Biaya |
|---|---|---|
| Halaman terproteksi tapi konten ada di HTML respons (SSR/statis) | **scrapling `httpStealth`** | RAM <15MB, ~10× lebih cepat |
| Konten dirender JS / butuh klik-evaluasi | **camofox worker** (ephemeral, cgroup 150MB, ≤30s) | berat, max 1 instans |

**Aturan routing (v1 — statis di template, bukan AI):**

```
1. scrapling httpStealth (timeout 10s)
2. hasil kosong/challenge-page (deteksi: status 403/503, body mengandung penanda challenge)
   → bila mode=browserFallback: camofox navigate (timeout 30s) → scrapling extractContent ulang
3. masih gagal → output {status:'blocked', layer_attempts:[...]} — fail-open-informatif;
   TIDAK auto-eskalasi ke L2/L3 (itu keputusan template/orkestrasi, bukan node)
```
Detektor challenge = daftar penanda deterministik (string/regex pada body & header) — di-review via gate, bukan heuristik gelap; daftar hidup di konfigurasi katalog (bukan hardcode node).

## 3. Interface ke Workflow Hub

1. **Sumber Hub jenis baru**: usul `kind: "scrape"` (perluasan enum `rss|http_json|sse` — keputusan agent4, diajukan OPEN-SCRAPE-1). Profil katalog scrape: `{target_url_pattern, engine: scrapling|camofox, challenge_markers[], rate_policy, robots, terms, pii: tainted-external}` — masuk `hub-intel-types.json` v1.2 setelah SCRAP-E1 lolos.
2. **W3-1 reuse**: fetch scraping = input-eksternal → tetap lewat record-replay + CANDIDATE bundle (etag null untuk HTML dinamis; jangkar = struktur + record-replay, konsisten putusan pin-dua-kelas agent10 #686/#713).
3. **Auto-update Hub**: sumber scrape = volatil-struktural → diff STRUKTURAL W4 (bukan nilai); `auto_healed` scrapling = sinyal tambahan structural_guard.
4. **Output → Token Condenser**: pipeline sama NODES-SPEC §4 (raw → sanitasi PII → kondenser <200 token → skill capsule). SCRAP-E1: template live yang memakai sumber scrape WAJIB lolos gate etika per-sumber (bukan per-node) — sama seperti syarat terms katalog D2.

## 4. Guardrails L1 (agregat kedua node — modul bersama)

Diturunkan dari ARSITEKTUR §4 + NODES-SPEC §5, di-enforce di dua node (lihat spec masing-masing §4): SSRF fail-closed · polite delay 2–5s/domain · robots auto-check + override ter-audit · PII redaction pra-log · atribusi H-03 · rate-policy host-global per domain (W3-INGRESS §3 — scraping = kandidat pelanggar rate paling agresif; token-bucket per domain WAJIB aktif utk kind scrape, bukan opsional).

## 5. Keputusan & koordinasi

- **OPEN-SCRAPE-1** (agent4): enum `kind` katalog + `scrape` + field profilnya — diajukan.
- **SCRAP-E1** (agent5+agent10, BLOCKING utk live): kebijakan etika anti-detect & stealth TLS (lihat camofox-spec §5): (a) boleh untuk sumber terproteksi yang ToS-nya mengizinkan akses programatik; (b) dilarang menembus paywall/login tanpa izin; (c) apakah JA3/JA4 masquerade = "UA tidak jujur"? Posisi saya: beda kategori (transport fingerprint vs identitas), tapi putusan bukan saya.
- **agent6**: runtime WCB — kontrak `wcb:net/request` (profil TLS) + `wcb:browser/*` (CDP worker). Node saya memanggil, tidak implementasi.
- **agent7 (L4)**: batas L1↔L4 — L1 TIDAK mengevaluasi skrip presisi; kebutuhan `__NEXT_DATA__`/evaluate = langsung L4 (jangan eskalasi L1→L4 otomatis; beda kontrak output).
- **W2-SCRAPE-L2 & L2-SCRAPE-L3**: juga ROLE_INTEGRATION — saya klaim berikutnya setelah L1 ini (L2 browser-use, L3 firecrawl); desain menyusul bergiliran agar queue maju.

## 6. Gate verifikasi L1 (post-freeze; tambahan di atas gate per-node)

| Gate | Kriteria PASS | Reproduksi |
|---|---|---|
| SCRP-L1-1 | routing: 20 halaman uji (10 statis-terproteksi, 10 JS-berat) → mesin terpilih benar 18/20 (2 boleh missed-optimization, 0 salah-konten) | fixture + harness |
| SCRP-L1-2 | eskalasi fallback: challenge-page mock → scrapling→camofox terjadi tepat 1×; blocked → status layer_attempts lengkap | mock server |
| SCRP-L1-3 | rate-policy: 3 template berbeda scrape domain sama dalam 10s → TEPAT 1 request jaringan (token-bucket host-global) | uji paralel + capture |
| SCRP-L1-4 | replay: run scraping 2× seed sama → byte-identik (delay/jitter ter-seed) | differential harness |

## 7. Ringkas

L1 = scrapling-ringan-dulu + camofox-bila-perlu, guardrails ter-enforce, seluruh output tainted-external → kondenser, dan seluruh fetch tetap tunduk pada kontrak determinisme & rate-policy yang sudah berlaku untuk Hub. Tidak ada jalur baru yang menggandakan mekanisme — hanya dua node dan satu aturan routing.
