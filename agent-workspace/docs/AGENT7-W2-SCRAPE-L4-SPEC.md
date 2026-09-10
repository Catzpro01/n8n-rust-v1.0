# AGENT7-W2-SCRAPE-L4 — Lapis 4 Scraping: Puppeteer Native Engine Node (low-level evaluate + hydration extraction)
**Versi:** 0.2 | **Tanggal:** 2026-09-09 | **Status:** DESAIN — DIKLAIM (W2-SCRAPE-L4, IN_PROGRESS)
**Penulis:** agent7 (ROLE_AI_MCP) · **Mandat:** fern #756/#767/#867 (W2-SCRAPE-L4 → agent7 + dukungan agent9), ARSITEKTUR-SCRAPING-BERLAPIS v1.0.0 (bcee7f0b), PRD-WORKFLOW-HUB-STANDALONE §3.2-butir-4
**Artefak kunci yang dirujuk:** NODES-CAMOFOX-SCRAPLING-SPEC v1.0 (f695f426), AGENT6-WCB-SCRAPLING-ADAPTER v0.2 (24247788), AGENT9-NODE-CAMOFOX-SPEC (b40ba543), AGENT7-W1-MCP-TOOLS-PLAN (08ecd4b3), AGENT7-JIT-ROUTING-REGISTRY v0.5 (458f1117), AGENT7-MCP-CONFORMANCE-TESTS v0.1 (e1e64aee)

---

## 0. Revisi
- **v0.1** — rilis pertama: kontrak node + jalur ekstraksi ganda + seam MCP authoring + substrat CDP + guardrail + gates + prototype hydration (14/14 gate).
- **v0.2** — serapan review admission: agent1 #831 (A-D) + gatekeeper agent10 #826 (a-d) + re-claim pasca-heartbeat-timeout (#867). Delta: error taxonomy +3 varian & pisah MISSING/EMPTY; partial_reason enum; propagasi truncated ke metadata Item; token CDP acak-per-sesi; no-retry kebijakan; G-L4-2 dipecah; prototype diperbarui & gate dijalankan ulang.

## 1. Posisi & boundary (hindari tumpang-tindih, pola Kunci-7 #573/#575)

| Pihak | Pemilik | Batas |
|---|---|---|
| Node n8n-nodes-puppeteer (skema/OpenAPI, kontrak operasi, fixture) | **agent7 (dokumen ini)** — queue & dekrit #867 menetapkan L4 ke ROLE_AI_MCP | node-level contract; hanya lapisan 4 |
| Eksekusi alur kerja | Engine (API-eksekusi) — **bukan MCP** (garis F-8/MCP-11; konsisten agent6 §1, agent9 #819, & D-5) | L4 adalah node biasa dalam workflow; dipanggil runtime engine, bukan via tool MCP |
| Permukaan MCP (authoring/validasi) | agent7 (TOOLS-PLAN: 5 tools authoring, tanpa eksekusi/fetch) | L4 menyumbang **nol** tool eksekusi baru; hanya validasi statis (lihat §5) |
| Substrat browser/CDP worker & anti-detect | agent9 (kontrak camofox/L1-L3, DONE b40ba543) / agent6 (adaptor WCB) / agent10 (gate) | L4 TIDAK meniru fingerprint masking; **berbagi pola worker** (127.0.0.1+token, auto-terminate, cgroup) — konsisten agent9 §CDP-worker |
| Katalog Hub & template (crypto ticker/orderbook memakai L4) | agent4 | registrasi intel-type; template menyebut node L4 |
| Guardrail kepatuhan | agent10 (#744, #826) | §7 verbatim + kondisi a-d #826 |
| Pengukuran performa/RSS | agent8 (BENCH-REPRO) | angka RAM/throughput L4 masuk bench pasca-substrat |

**Tidak ada perubahan dokumen lain.** Registry URI (v0.5) tidak berubah di tugas ini (catatan §5.3).

## 2. Dua jalur ekstraksi — keputusan arsitektur inti

Situs SPA/SSR modern menyimpan data asli (sebelum di-render DOM) di dua tempat:

1. **Jalur-A: DOM pasif (tanpa eksekusi).** Next.js/Nuxt menyisipkan `<script id="__NEXT_DATA__" type="application/json">{…}</script>` (atau `__NUXT__`) di HTML awal. Cukup **parse** — tanpa evaluate, tanpa browser penuh, murah, deterministik, dan aman (nol kode berjalan).
2. **Jalur-B: state live (via evaluate/CDP).** Data yang baru muncul **setelah** hydration/JS berjalan (state Redux/Vue, data pasca-fetch klien) hanya bisa dibaca dari memori browser via `evaluate` pada context halaman.

**Aturan pemilihan:** Jalur-A dicoba lebih dulu (HTTP/stealth, hemat); bila `__NEXT_DATA__` absen atau konten target tidak ada di dalamnya (mis. hanya props kosong + data di-fetch klien), naik ke Jalur-B (CDP worker, mahal, dibatasi). Perpindahan A→B direkam sebagai `partial_reason=fallback_b`. Selaras ARSITEKTUR §3: kasus ticker/orderbook = "Lapis 4 evaluate `__NEXT_DATA__` → cepat, instan, hemat RAM".

**Catatan produk:** "Native MCP Engine" pada ARSITEKTUR-L4 dimaknai: engine mengimplementasikan primitif bergaya-puppeteer (evaluate/hydration) secara **native sebagai node engine** — bukan menjadikan MCP server sebagai jalur eksekusi live (dilarang MCP-11/H-3). §5 merinci satu-satunya sentuhan MCP yang sah.

## 3. Kontrak operasi node (engine-native `n8n-nodes-puppeteer`, lapis 4)

Parameter standar semua operasi: `url` (wajib untuk operasi mandiri), `timeout_ms` (default 30_000, maks 60_000), `wait_until: domcontentloaded|networkidle0`, `delay_after_ms` (guardrail 2000–5000 acak bila 0). Output kanonik = ItemList dengan `input_digest` di-pin di ingress (pola agent6 §2-E / agent9 #819) + metadata degradasi (§8-E agent1 #831).

| Operasi | Parameter | Return (kanonik) | Catatan |
|---|---|---|---|
| `navigate` | url, wait_until, timeout | `{status, final_url, title, html_digest}` | HTML tidak di-materialisasi penuh; digest + ukuran |
| `extractHydration` | keys[] (default `["__NEXT_DATA__","__NUXT__"]`), selector_path (default `props.pageProps`), max_bytes (default 512_000) | `{source: script_tag\|live_global, key, payload, selector_path, partial_reason, truncated: bool}` | Jalur-A dulu, fallback B; MISSING vs EMPTY dibedakan (§3.2) |
| `evaluate` | expression (string, <16 KiB), await_promise: bool | `{result_json, execution_ms}` | Hanya via CDP worker; sandboxed, tanpa argumen kredensial (§7) |
| `click` | selector, delay_after | `{clicked: bool, matched: u32}` | `matched` = jumlah aktual elemen (bukan hardcode 1 — agent1 #831-D) |
| `dispatchEvent` | selector, event_type (click\|submit\|custom), detail (json) | `{dispatched, matched: u32}` | custom event utk framework listeners |

### 3.1 Error taxonomy (engine error-enum; pola E_SPILL_* agent1 #750/#751 + review #831)

| Varian | Makna | Status eksekusi | Retry |
|---|---|---|---|
| `L4_PARSE` | HTML/JSON tak terurai; kedalaman > 24 | FAILED | 0 (data korup — beda jalur) |
| `L4_HYDRATION_NOT_FOUND` | Kunci hydration absen di kedua jalur | FAILED | 0 |
| `L4_SELECTOR_MISSING` | **Path tidak ada** di payload (struktur berubah — sinyal layout-berubah → picu self-healing / gate stabilitas normalisasi) | PARTIAL→FAILED | 0 (naik Jalur-B bila diizinkan = fallback_b) |
| `L4_SELECTOR_EMPTY` | Path ada tapi bernilai null/[] — data legit nol | SUCCESS (payload kosong + partial_reason=empty) | — |
| `L4_OVERSIZE` | Melampaui max_bytes → potong batas objek + `truncated=true` | SUCCESS/PARTIAL (partial_reason=truncated) | — |
| `L4_TIMEOUT` | Navigasi/evaluate lewat timeout | FAILED | 1x + backoff |
| `L4_WORKER_FAIL` | **Infra**: worker CDP gagal spawn / biner chromium absen / cgroup gagal (pemisahan infra-vs-aplikasi, analog E_SPILL_IO vs CORRUPT — agent1 #831-B) | FAILED | 1x (setelah perbaikan infra) |
| `L4_NAV_BLOCKED` | Kebijakan SSRF/allowlist/robots menolak | FAILED | **0** (kebijakan ≠ transient — agent10 #826-a) |
| `L4_EVAL_DENIED` | Expression melanggar kebijakan §7 (termasuk ukuran >16 KiB saat authoring di-bypass — agent1 #831-D) | FAILED | **0** |

Keputusan retry per-varian (D14/D57 router) = penggerak determinisme; PARTIAL eksplisit + `input_digest` ingress konsisten garis #777-E.

## 4. Kontrak modul hydration (pure function — inti deliverable v0.1)

Modul **tanpa IO**, portabel: `extract(html: &[u8], keys: &[&str], selector_path: Option<&str>, max_bytes: usize) -> Result<HydrationItem, L4Err>`.

1. **Deteksi jalur-A:** scan `<script id="__NEXT_DATA__" type="application/json">` (dan varian `type=application/json; charset=utf-8`); tangkap sampai `</script>`; parse JSON. Kunci lain via registry: `__NUXT__` (objek JS — toleransi parse; `script_text` + best-effort JSON bila bentuk `window.__NUXT__={...}`).
2. **Penjaluran:** `HydrationItem { source, key, payload_json, selector_path, partial_reason: None|truncated|empty|fallback_b, truncated: bool, byte_len, script_text_hash }`.
3. **Selector:** path dot-notation, hanya objek/array. **MISSING vs EMPTY dipisah** (agent1 #831-C, pelajaran N-11): komponen path tak ditemukan di struktur → `L4_SELECTOR_MISSING` (struktur berubah); path lengkap ada tapi nilai null/[] → `L4_SELECTOR_EMPTY` (data nol legit → SUCCESS dgn payload kosong, bukan error).
4. **Batas:** `max_bytes` (potong deterministik di batas objek JSON + flag `truncated`), kedalaman ≤ 24 (anti bom rekursi), node_count ≤ 10_000.
5. **Propagasi degradasi (agent1 #831-E):** `truncated`/`partial_reason` WAJIB merambat ke metadata Item keluaran engine (`item.meta.degraded`), bukan hanya struct return — downstream (Token Condenser, audit) bisa melihat data terpotong.
6. **Determinisme:** output hanya fungsi (html, keys, path, max_bytes) — identik antar-eksekusi → aman direkam RecordSet & diputar ulang tanpa fetch ulang (W2-DETERM-ENFORCE).
7. **Keamanan:** output **hanya JSON data**; skrip/handler tidak pernah dieksekusi (bukan innerHTML/eval). Payload difilter PII (§7.3) sebelum disimpan log.

## 5. Integrasi MCP — authoring/validation only (D-5, MCP-11/H-3)

1. **Tidak ada tool eksekusi/fetch baru.** Tabel negatif TOOLS-PLAN butir 5 (eksekusi/fetch/kredensial → ditolak) berlaku penuh utk L4; conformance TC-30..36 tetap satu-satunya penguji negatif.
2. **Validasi statis authoring** (via tool `validate` eksisting): pemeriksaan konfigurasi node L4 — URL syntaks + kebijakan SSRF lint (domain/IP privat ditolak **saat authoring**, bukan saat run — agent10 #826-a), ukuran expression ≤16 KiB, keys ∈ registry, selector_path sintaks dot-notation. Semua murni lokal; `est_token` tanpa fetch (H-3b).
3. **Registry & resource:** v0.5 tidak berubah. Usulan (bukan bagian tugas ini): URI resource `mcp://engine/scrape/l4-preview` utk Wave-3 hub bila ada kebutuhan pratinjau deterministik berbasis fixture.

## 6. Substrat eksekusi & siklus hidup CDP worker (rancangan, eksekusi pasca-substrat)

- **Worker:** 1 instans browser aktif per engine; `--headless=new --remote-debugging-port=0 --user-data-dir=<tmp>`; binding **127.0.0.1 saja**; **token CDP acak per sesi** (bukan per-proses) — endpoint `http://127.0.0.1:<port>/json/version?token=…`, **token tidak pernah masuk log** (agent10 #826-b).
- **Lifecycle:** auto-terminate 30 detik idle (pola camofox §2.2 / agent9); cgroup 150 MB/worker; anggaran global 500 MB (hard-cap); fuel & timeout engine (#493 PRG meter). Gagal spawn/cgroup → `L4_WORKER_FAIL` (§3.1).
- **Biner browser:** **keputusan pemilik diperlukan** — VPS tidak punya chromium/chrome; opsi: bundel pinned headless chromium (sha256 tercatat, non-system-store) atau tunggu substrat. Non-blokir utk desain; blokir utk eksekusi Jalur-B & G-L4-9 (agent10 #826-d setuju: bukan blocker design).
- **Rust seam (port):** `crates/nodes-puppeteer` (baru) — modul pure §4 tanpa IO; `cdp_client` kecil (WS ke endpoint lokal) ditulis saat biner tersedia; `navigate/evaluate` memakai domain CDP `Page`, `Runtime`, `Input`.

## 7. Keamanan & kepatuhan (guardrail agent10 #744/#826 + arsitektur §4)

1. **SSRF:** blokir IP privat/lokal/loopback/link-local + DNS rebinding guard (resolve → cek → connect); tolak **saat authoring**; `L4_NAV_BLOCKED` saat run, tanpa retry.
2. **Polite delay:** acak 2.0–5.0 s antar-request ke domain sama; hormati `Retry-After` 429 dgn backoff+jitter 1–3 s.
3. **robots.txt:** cek sebelum scrape; override hanya eksplisit konfigurasi pengguna (per-URL, tercatat); pelanggaran = `L4_NAV_BLOCKED`.
4. **PII:** sanitasi email/nomor-telepon/kredensial pada hasil sebelum log/riwayat (modul bersama agent10); redaksi oracle #394 tetap di facade MCP (A7-6).
5. **evaluate:** expression dilarang memuat kredensial dari env/refs (D-6: refs-only creds; `$env` allowlist); hasil discan pola token (bearer/private key) sebelum disimpan; `L4_EVAL_DENIED` juga untuk ukuran runtime bila authoring di-bypass.
6. **Determinisme & replay:** `input_digest` di pin ingress; replay = baca RecordSet, **dilarang fetch ulang**; hydration payload + metadata degradasi ikut terekam.
7. **Atribusi:** item hasil membawa tag sumber + lisensi (CC-BY/situs) utk template Hub.

## 8. Gates & uji terukur (falsifiable)

| Gate | Kriteria lulus | Metode |
|---|---|---|
| G-L4-1 | Fixture hydrasi nyata: ≥8 varian HTML (Next 12/13/14 script-tag; charset varian; NUXT objek; tanpa hydrasi; malformed; huge) → klasifikasi benar 8/8 | fixture-gates v0.1 (PASS 8/8) |
| G-L4-2a | Selector path benar pada payload bertingkat + array-index; path ada-nilai-nol (null/[]) → `L4_SELECTOR_EMPTY` SUCCESS-payload-kosong (bukan panic) | unit test modul |
| G-L4-2b | Path komponen **hilang** → `L4_SELECTOR_MISSING` (beda kode dari EMPTY) | unit test modul (baru v0.2) |
| G-L4-3 | OVERSIZE: potong di batas objek, `truncated=true`, JSON tetap valid; `partial_reason=truncated` | unit test modul |
| G-L4-4 | JSON invalid → `L4_PARSE` dgn posisi; 0 panic pada fuzz 100 korpus pendek | fuzz ringan |
| G-L4-5 | Determinisme: byte-identik utk input sama (100 run) | hash run berulang |
| G-L4-6 | PII/redaksi: payload ber-email/token → tersanitasi sebelum output log (0 leak di 50 kasus) | saat integrasi (gate security, review agent10 #826-c) |
| G-L4-7 | SSRF/allowlist: 20 URL jahat (localhost/10.x/169.254) ditolak authoring & run; 0 egress privat | saat integrasi (review agent10 #826-c) |
| G-L4-8 | MCP negatif: tools authoring L4 tak pernah eksekusi/fetch (TC-30..36 konformansi tetap hijau) | conformance e1e64aee |
| G-L4-9 | Jalur-B: evaluate mengeksekusi ekspresi sandbox & event DOM; waktu ≤ timeout; worker terminator 30 s | pasca biner browser (menunggu keputusan pemilik — #826-d) |
| G-L4-10 | `L4_WORKER_FAIL` muncul saat biner absen/cgroup gagal; retry 1x; status FAILED (bukan TIMEOUT) | pasca substrat (baru v0.2, agent1 #831-B) |

## 9. Artefak deliverable v0.2

1. `docs/AGENT7-W2-SCRAPE-L4-SPEC.md` (dokumen ini) — sha header diumumkan di channel.
2. `qa/l4-hydration/` — `l4_hydration_ref.py` (referensi eksekutabel, stdlib-only; port ke Rust = seam §4 tanpa IO), `fixtures/` (8 korpus HTML + expected), `gates/run_gates.py` hasil G-L4-1..5 + G-L4-2a/2b.

## 10. Dependensi & status

| Dependensi | Status | Efek |
|---|---|---|
| Review admission agent1 #831 + agent10 #826 | DITERIMA → v0.2 | tertutup |
| W1-MCP-TOOLS / W1-MCP-TRANS (desain) | DONE (08ecd4b3/98d0f827) — fase kode belum ada baris queue (#733 belum dijawab fern) | authoring-tools rujukan §5; non-blokir utk modul murni |
| Node runtime engine (crates masih scaffold) | scaffold | integrasi eksekusi node = Wave lanjut |
| Biner Chromium pinned | **keputusan pemilik** (tak ada di VPS) | blokir jalur-B & G-L4-9 saja |
| Katalog Hub/template (agent4) | DONE (W2-HUB-CATALOG); katalog intel v1.2 agent9 (033924f6) | template L4 menyusul di bounty |
| Substrat scraping agent9/agent6 | agent9 L1-L3 + camofox DONE (b40ba543); adaptor WCB agent6 v0.2 | pola worker konsisten; tak tumpang tindih |
| Guardrail agent10 | aktif (#826: DESIGN-PASS bersyarat a-d) | §7 diterapkan saat integrasi |
| BENCH agent8 | pasca-substrat | angka RSS/throughput L4 |

**Status agent7:** BUSY W2-SCRAPE-L4 (1/1; re-claim 10:50 UTC pasca timeout — SOP #864). Review terbuka lanjutan: agent1 (v0.2 delta #831), agent10 (v0.2 delta #826), agent9 (dukungan eksekusi per #867).
