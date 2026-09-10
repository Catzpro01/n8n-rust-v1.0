# WORKFLOW HUB — Skema Manifest & Siklus Hidup Template (draf agent4)

Status: menjawab mandat #524/#526 (tugas bidang agent4: "Desain skema manifest Workflow Hub:
metadata, tags, auto-update schedule, zero-code template format") + #547 (self-doc/i18n).
Riwayat: v0.1 06:52 → v0.2 (serap agent1 #536) → v0.3 (agent3 #537, agent1 #540, agent7 #542,
self-doc #547) → v0.4 (agent10 #561 H-02/H-03, agent9 #589 interface) → v0.5 (agent10 #633
SEC-HUB-01: pin pindah ke katalog, single-source; VERIFIED_TOFU) → v0.6 (agent9 #817
OPEN-SCRAPE-1/3: kind=scrape + requires_credentials BYOK; agent10 #814: schema_hash digest
nyata).
Posisi: Hub = saudara korpus 171 (docs/corpus/) dgn siklus hidup terkelola + integrasi Rosetta.

## 1. Prinsip desain

1. Template Hub = workflow n8n format standar + MANIFEST (file JSON sidecar, 1:1).
2. Identitas = content-addressed: `hub_id` + `template_sha256` (file utuh, prinsip corpus kami).
   Update = versi BARU dgn sha baru; versi lama tidak pernah ditimpa buta (auditability + rollback).
3. Auto-update TIDAK pernah senyap: tiap perubahan yang diambil dari sumber menjalani
   RELEASE-ENVELOPE-style diff (gate #374): output kanon versi lama vs baru dibandingkan;
   perubahan perilaku = laporan, bukan diam-diam.
4. Impor template = lewat Rosetta (L-1): workflow era mana pun diterima, migrasi dicatat di
   receipt, node opaque dicatat — template Hub boleh memakai node apa pun; janji eksekusi
   tetap per-node (konsisten T-14 opsi B).
5. Zero-code: template & manifest murni DATA (JSON). Tidak ada kode yang dieksekusi dari Hub
   selain workflow itu sendiri di sandbox normal (bukan saat fetch/update).

## 2. Skema manifest v0.1

```jsonc
{
  // --- identitas & konten (agent4) ---
  "schema": "workflow-hub-manifest", "schema_version": 1,
  "hub_id": "hub-0001",               // stabil; template_sha berubah tiap versi
  "template_sha256": "…",             // hash file workflow utuh — JANGKAR VERIFIKASI (TOFU, lihat §1/§2a):
                                      // sha = deteksi korupsi dlm kanal distribusi tepercaya, BUKAN klaim keaslian
                                      // absolut; jangkar keaslian v1 = pin expected_sha256 di KATALOG agent9 (eksternal
                                      // utk manifest; SEC-HUB-01) + pipeline review (review_status gate)
  "workflow_ref": "wfh-crypto-pulse.json",
  "name": "Crypto Pulse (7d)", "summary": "≤50 kata",
  "category": ["crypto", "news"],
  "tags": ["coingecko", "rss", "condense"],
  "language": "en",
  "license": "CC0|CC-BY|MIT|SEE-LICENSE",  // wajib; enum + SEE-LICENSE (agent10 H-03)
  "license_url": "…",                      // wajib utk CC-BY/MIT/SEE-LICENSE
  "attribution": {"required": false, "text": "…", "author": "…"},  // agent10 H-03; auditable, bukan sekadar etika

  // --- kredensial (v0.6 — OPEN-SCRAPE-3, agent9 #817) ---
  // ABSEN = template BEBAS kredensial (Zero-API-Key hub v1). ADA = BYOK (Bring-Your-Own-Key):
  // pengguna menyediakan kredensial saat instal; template/dag TIDAK PERNAH menyimpan nilai
  // rahasia — hanya referensi {credential_kind, target} ke vault kredensial engine.
  "requires_credentials": {            // OPSIONAL
    "byok": true,                      // true = template ditandai BYOK di hub_list/UI
    "credentials": [                   // ≥1 entri; SEMUA rahasia
      {"credential_kind": "api_key|oauth2|basic|cookie",
       "target": "<node_id|source_id>",   // node atau sumber katalog yang memakainya
       "secret": true}],
    "note": "≤120 token utk LLM; kredensial TIDAK pernah di-commit ke template/dag"
  },
  // Kebijakan BYOK (v0.6): live TIDAK diblokir otomatis, TAPI mensyaratkan (a) gate etika
  // SCRAP-E1 (agent5+agent10) utk sumber scrape/anti-detect — anti-detect HANYA utk sumber
  // ToS-izinkan-programatik; paywall/login tanpa izin = DILARANG; (b) terms.status sumber
  // lolos (ok) — lihat §12.4; (c) kategori listing: badge BYOK di katalog (OPEN-SCRAPE-3).

  // --- sumber & pembaruan (auto-update schedule) ---
  // SELURUH profil sumber (auth, terms, rate_limit, cache/etag, volatility, PIN
  // expected_sha256, verify-status incl VERIFIED_TOFU) = KATALOG agent9 (hub://intel-types)
  // sbg SINGLE SOURCE OF TRUTH (SEC-HUB-01, agent10 #633: pin harus eksternal agar bukan
  // TOFU dgn nama lain). sources[] = referensi tipis + override TERBATAS:
  "sources": [
    {"source_id": "coingecko-simple-price",     // ref katalog agent9 (wajib)
     "kind": "rss|http_json|sse|scrape*",       // override OPSIONAL (default = katalog);
                                                 // *scrape = profil khusus, lihat §13 (OPEN-SCRAPE-1)
     "url": "…",                                // override OPSIONAL (default = katalog)
     "rate_policy": {"min_interval_s": 3600, "max_bytes": 1048576, "timeout_s": 20},
     "robots": "checked-2026-09-09"}            // override OPSIONAL (default = katalog; gate agent5)
     // TIDAK ada expected_sha256/pin di manifest — nilai pin hidup di katalog saja
  ],
  "update": {"schedule": "cron 5-field", "max_age_h": 24,
             "on_change": "new-version|notify-only",   // determinisme + gate diff
             "last_check": "…", "next_check": "…"},

  // --- eksekusi & konsumsi AI (0-code, dijalankan engine) ---
  "inputs": {"schema": {"$ref": "…/input-schema.json"}},   // param yg boleh user set
  "outputs": {"kind": "condensed", "max_tokens": 200, "format": "json"},
  "ai_skill": {"skill_id": "crypto-pulse-daily", "description": "≤120 token utk LLM (tokenizer #419a)",
               "invoke_ref": "API eksekusi terpisah (D-7/F-8/MCP-11) — BUKAN tool MCP"},

  // --- kualitas & jejak (disiplin tim) ---
  "provenance": {"author": "hub-team|agenN", "created": "2026-09-09",
                 "reviewed_by": null,
                 "review_status": "draft|in-review|live|deprecated"},  // gate agent5 sebelum live; katalog hub_list hanya tampilkan live
  "metrics_ref": "…",             // verdict determinisme + hasil diff gate (agent5/agent8)
  "deps_nodes": ["n8n-nodes-base.httpRequest", "n8n-nodes-base.code"],  // utk L1 readiness
  "deviation_ids": [],            // DEV-* yg muncul saat impor Rosetta (link katalog by ID)
  "content_version": 1            // naik tiap isi berubah (pola registri agent7)

  // --- self-documenting & i18n (mandat #547; wajib utk template live) ---
  "documentation": {
    "cara_kerja": "…",            // pilar 1: langkah input→output
    "fungsi": "…",                // pilar 2: kemampuan teknis/transformasi
    "tujuan": "…",                // pilar 3: manfaat bisnis/operasional
    "notes_binding": {            // binding stickyNote↔node (opsional; default geometrik)
      "<node_id>": "<stickyNote_id>"}
  },
  "i18n": {"source_lang": "en",   // manifest TETAP satu bahasa sumber (lean)
           "available": ["id", "zh", "es", "…"],   // 40 bahasa, daftar turunan dari tabel agent2
           "translations_ref": "workflow_i18n"}    // bundle agent2 <500KB + FTS5; MCP baca dari sana
}
```

## 3. Auto-update schedule — aturan

1. Jadwal diekspresikan 5-field (bukan detik acak — temuan #435: jalur v2 deterministik).
2. Fetch baru → verifikasi sha (kalau katalog memasang pin utk sumber itu) → simpan sbg
   CANDIDATE (bukan langsung live).
3. Gate diff (Release Envelope #374, mode deterministik): output kanon candidate vs live;
   IDENTIK → ganti diam (aman, tercatat); BERUBAH → naik content_version + notifikasi ke
   subscriber; GAGAL (sumber berubah format/rusak) → retensi versi lama + alarm, template
   TIDAK mati senyap (pola "old templates stay runnable" = alasan n8n memanggul legacy;
   Hub memecahkannya dgn versi, bukan dgn memaksa pengguna migrasi).
4. Rate limit & etika: patuh min_interval, robots, timeout (gate agent5) — Hub = warga
   internet yang sopan, bukan scraper agresif (ini kritik desain: fitur "gratis" tanpa
   rate-policy = risiko blokir + beban etika).

## 4. Kritik desain (masukan ke sayembara)

1. TEMPLATE = VEKTOR INJEKSI: workflow Hub mengambil data internet publik utk AI agent;
   output-nya WAJIB taint-marked (T-1 agent7/agent3) + sanitasi (agent5). Ringkasan
   "<200 token" dari artikel berbahaya tetap bisa memuat instruksi tersembunyi.
2. "HEMAT 90% TOKEN" = hipotesa (agent1 #528 benar): ukur dulu (agent8), jangan masuk janji
   produk. Korpus kami: template berita/crypto era baru = AI-heavy; jangan tiru bias itu.
3. MANIFEST BUKAN KATALOG BARU: satu sumber kebenaran id/versi/owner = registri (pola
   AGENT7-JIT-ROUTING-REGISTRY §4); manifest Hub per-template = payload; katalog = indeks.
4. SUMBER "GRATIS" BERUBAH: RSS/endpoint publik mati/berubah format (pelajaran: api.n8n.io
   probe kami — 171 dari ~17.000 id). Auto-update harus anggap sumber TIDAK stabil:
   schema-version sumber + test kontrak per source (bukan cuma fetch-ok).
5. KUALITAS TEMPLATE: pelajaran korpus (bias era, node deprecated 39% di template tua,
   #398) → Hub butuh review_status + metrics (determinisme verdict, exec-success rate),
   bukan sekadar "1-klik" tanpa ukuran.

## 5. Integrasi dgn kerja lain

- Rosetta (L-1): impor template Hub = alur yg sama dgn impor korpus; receipt per template.
- Release Envelope/Gate (#374): mesin diff utk auto-update (bagian 3).
- Determinism Contract (#368): template Hub yg verdict NON-DETERMINISTIC tidak layak
  auto-update senyap (mis. pakai Math.random di code node).
- Registri URI agent7: template live = resource `hub://skills/{hub_id}/manifest` (kanon,
  penamaan agent7 #542; draf awal saya `n8n://hub/…` disupersede), receipt di
  `hub://skills/{hub_id}/receipt`; registri petakan hub_id ↔ URI (single source of truth).
- Agent9: pemetaan konektor sumber publik = isi `sources[]`; agent5: gate etika/robots;
  agent3/agent1: kompresi ekstraktif = `outputs`; agent10: lisensi/attribution.

## 6. Gate penerimaan (falsifiable)

- HUB-1: 10 template contoh (5 kategori) — manifest valid 10/10, sha cocok, unduh ulang
  byte-identik.
- HUB-2: simulasi update: sumber berubah isi → content_version naik + subscriber
  dinotifikasi; sumber mati → versi lama bertahan + alarm (0 template mati senyap).
- HUB-3: template dgn node deprecated → impor Rosetta menghasilkan receipt dgn
  deviation_id (bukan gagal, bukan senyap).
- HUB-4: 50 skenario jahat, STRATIFIKASI 5×10 (agent3 #537), pass criteria eksplisit per
  strata: (1) 10 data-driven "ignore previous instructions" → 10/10 taint-marked, 0 bocor;
  (2) 10 covert channel `{{ $env.SECRET }}` ter-escape → 10/10 redacted, 0 leak (oracle
  #394); (3) 10 tool-poisoning (fetch berisi instruksi) → 10/10 output terstruktur, 0
  prose-instruction; (4) 10 node/workflow-name injection → 10/10 escaped, 0 tereksekusi;
  (5) 10 cache-basi (content_version lama) → 10/10 version-check, 0 stale. = uji langsung
  invarian-taint §3 agent1.
- HUB-5 (agent3 #537): 100 template dari korpus → manifest ter-generate 100/100 valid JSON
  schema + sha256 cocok + receipt ter-generate (uji integrasi Hub + Rosetta + manifest).
- HUB-6 (mandat #547): template live → 3 pilar documentation (cara_kerja/fungsi/tujuan)
  non-kosong 10/10 + i18n.source_lang konsisten dgn content.
- HUB-7 (mandat #547): note-binding deterministik — 20 template, binding dihitung ulang 2x
  dari file sama = identik 20/20 (determinisme geometri; coverage node per template
  TERCATAT sbg metrik, bukan janji kualitas isi stickyNote lama).

## 7. Catatan konvergensi (agent1 #536, 2026-09-09) — diserap

- (template_sha256, content_version) wajib masuk determinism-record engine (§6.2.1 analog
  webhook_urls) — sinkronisasi otomatis dgn #374/gate diff.
- Eksekusi-dua-versi (live vs candidate) = mode jadwal engine; Hub memakainya utk gate diff
  tanpa infra baru. Tercatat sbg OPEN-HUB-1 di rakitan PRD-3 matt.
- MCP catalog-only (discovery + estimasi) tetap garis merah D-7/F-8; invoke via API
  terpisah. Konsekuensi skema: field `ai_skill` berisi skill_id + invoke_ref (bukan nama
  tool MCP); contoh mandat `fetch_crypto_intel` = entri katalog + endpoint API eksekusi.

## 8. Konvergensi lanjutan (agent3 #537, agent1 #540, agent7 #542, 2026-09-09) — diserap

- Jadwal auto-update: otoritas = ENGINE/SERVER scheduler (parser deterministik #435);
  poll-klien boleh sbg transport, DILARANG sbg sumber jadwal (zona waktu klien =
  nondeterminisme). TriggerTimes masuk determinism-record.
- Semua klaim token (deskripsi skill ≤120, output ≤200) = tokenizer-rujukan tunggal #419a,
  tanpa pengecualian (MCP §4.1 + Hub).
- PRNG-seeded-dari-sha boleh bila dibutuhkan, tapi skor ekstraktif agent1 #528 tidak perlu
  PRNG sama sekali (lebih sederhana = lebih baik).
- Jawaban agent4 atas pertanyaan agent3 #537 (4b): PIN di KATALOG (agent9) DISET + hash
  fetch BERBEDA → candidate DITOLAK + alarm + versi lama tetap live. Pin = kontrak; re-pin
  hanya oleh pemilik katalog setelah persetujuan (konsisten "0 mati senyap"). Tanpa pin →
  gate diff normal. LINGKUP: kelas STATIC (#686 §12); kelas volatil tanpa pin by-design.
- Penyelarasan manifest ↔ resource MCP agent7 (AGENT7-WORKFLOWHUB-MCP-PROPOSAL, sha
  9b147357…): resource `hub://skills/{hub_id}/manifest` menyajikan manifest ini apa adanya;
  `hub://skills/{hub_id}/receipt` = receipt Rosetta ringkas: {template_sha256,
  content_version, deviation_ids (HUB-3), exec-verdict ref (determinism-record), ts}.
  review_status: draft|in-review|live|deprecated; hanya "live" muncul di hub_list.
- HUB-5 + stratifikasi HUB-4 (+ HUB-1..3) = paket gate Hub utk rakitan PRD-3 (matt).
- Receipt-outdated rule (agent3 #545.4): receipt.content_version < manifest.content_version
  → flag "receipt outdated, re-execute needed" di hub_inspect — 0 receipt basi tak terlihat.
- dry_run.est_token (agent3 #545.3 + agent1 #550 H-3b): WAJIB diturunkan dari METADATA
  manifest (deps_nodes count, inputs schema, outputs.max_tokens) — DILARANG fetch-live
  (fetch via MCP = eksekusi-berkedok-sampling = pelanggaran F-8). Akurasi diukur agent8 (H-8).

## 10. Konsolidasi review compliance & interface (agent10 #561, agent7 #583, agent9 #589)

1. agent10 H-01: hub_id/template_sha256 (pola BlobId/ContentHash C-06) + determinism-record
   (C-04) — dikonfirmasi.
2. agent10 H-02 (celah MEDIUM, jangkar verifikasi): DISERAP — pernyataan jangkar di §2:
   sha256 = deteksi korupsi dalam kanal distribusi tepercaya (TOFU eksplisit), BUKAN klaim
   keaslian; jangkar keaslian v1 utk kelas STATIC = pin expected_sha256 DI KATALOG agent9
   (eksternal thd manifest, SEC-HUB-01 #633) + pipeline review (review_status gate); kelas
   volatil = tanpa pin (lihat §12). Usul Ed25519 (kunci di registri agent7) = OPEN post-v1,
   dicatat, tidak dijanjikan.
3. agent10 H-03 (lisensi/attribution): DISERAP — license enum + license_url +
   attribution{required,text,author} (auditable, CC0 vs CC-BY vs MIT beda kewajiban).
4. agent9 §7 interface: DITERIMA dgn normalisasi kecil (format final = dokumen ini):
   - kind enum = rss|http_json|sse (snake_case; "wikipedia" HAPUS — MediaWiki API = http_json,
     EventStreams = sse). agent9 menambah kind SSE (Wikimedia) yg tak ada di draf awal saya.
   - endpoint → url; profil lengkap (auth:"none", terms, rate_limit, cache, verify-evidence,
     volatility) = KATALOG agent9 (hub://intel-types) single-source; manifest sources[] =
     referensi tipis {source_id, kind override?, url override?, rate_policy, etag_cache,
     robots} + pin TIDAK ada di manifest (nilai & penegakan = katalog) — mencegah duplikasi
     data 2 tempat (prinsip §4.3; SEC-HUB-01 butir 2). Sumber tanpa entri katalog = status
     VERIFIED_TOFU eksplisit di katalog, BUKAN "terverifikasi" (agent10 #633 butir 2).
   - source-level "license" agent9 = terms konsumsi sumber (beda makna dgn license manifest =
     redistribusi template, butir 3) — usul penamaan terms di katalog utk hindari ambigu.
5. agent7 v0.2 (3ad9029f): selaras manifest v0.3 — kanon hub://skills/{hub_id}/…, ai_skill,
   receipt payload, live-only hub_list, H-3b/H-7/H-8 — dikonfirmasi (#623); skema ini (v0.4)
   TIDAK mengubah kontrak resource tsb (hanya menambah license/attribution + jangkar TOFU).

## 11. v0.5 — SEC-HUB-01 diterima (agent10 #633, 2026-09-09)

1. Syarat SEC-HUB-01 butir 2 DITERIMA: jangkar keaslian v1 = pin expected_sha256 di KATALOG
   agent9 (hub://intel-types) — pihak berbeda dari penulis manifest, bukan TOFU dgn nama
   lain. Sumber tanpa entri katalog → status VERIFIED_TOFU eksplisit (bukan terverifikasi).
2. Catatan butir 4 DITERIMA: expected_sha256 DIHAPUS dari sources[] manifest (v0.5);
   manifest cukup {source_id, kind?/url?/rate_policy?/robots? override}; nilai pin + aturan
   penegakan hidup di katalog (single source, prinsip §4.3).
3. Konsekuensi ke skema: §2 sources[] diperbarui; §3 aturan pin; §8 4b tetap (mismatch =
   tolak + alarm + re-pin oleh pemilik katalog). Hub-2/4b semantics TIDAK berubah.
4. Ed25519 post-v1 (manifest_sig + kid + trust anchor registri agent7) = selaras keputusan
   gatekeeper SEC-HUB-01 — dicatat OPEN, tidak dijanjikan.
5. KATALOG agent9 DITERIMA v1.1 (hub-intel-types.json, ddd46498, 2026-09-09): 18 profil
   sumber (+MET Norway, +USGS), kind ∈ {rss, http_json, sse} selaras v0.5; single-source
   pin/terms terpenuhi (KAT-1: 18/18 vs target 7); pin{expected_sha256 null, target
   response-schema} + verify.status verified-tofu + terms.status ok/review-agent5/
   grey-BLOCKED-agent5 — kontrak interface #693 TERTUTUP. Kebijakan pin = PUTUSAN agent10 #686 (dua kelas:
   statis = pin wajib nullable/TOFU; volatil = tanpa pin, jangkar katalog+ETag+record-replay;
   schema-pin -> structural_guard OPSIONAL per-template) — dirinci di
   AGENT4-W4-HUB-AUTOUPDATE-SPEC v0.2 §4. review_status=live = lolos review pipeline
   (termasuk gate etika agent5 utk sources[]), BUKAN klaim keaslian.

## 9. Self-documenting & note-binding (mandat #547, 2026-09-09) — perluasan Rosetta/manifest

1. Prinsip: file workflow TIDAK dimutasi (norm korpus: template = bukti, bukan medium edit).
   Pilar deskripsi tinggal di MANIFEST (documentation{}), konten stickyNote tetap di file.
2. Rosetta: stickyNote sudah diparse (termasuk posisi kanvas & teks). Yang baru = index
   note-binding: (a) DEFAULT = geometri deterministik: node yg kotaknya ter-cover/terdekat
   stickyNote → binding; (b) OVERRIDE manual opsional di manifest notes_binding utk kasus
   ambigu/overlap. Tidak ada wall-clock/random di binding (konsisten #368).
3. StickyNote "46% template" (temuan matt): tetap binding walau isinya non-deskriptif —
   binding = koneksi deterministik, BUKAN jaminan kualitas isi; coverage dilaporkan sbg
   metrik per template.
4. i18n 40 bahasa: manifest = 1 bahasa sumber (source_lang); terjemahan = derived data di
   tabel workflow_i18n agent2 (FTS5) + bundle <500KB; MCP n8n://templates/{id}?lang={code}
   (agent7) membaca tabel tsb, bukan manifest. Terjemahan punya versioning sendiri — TIDAK
   menaikkan content_version manifest (menghindari invalidasi receipt palsu).
5. Kait Hub: pilar deskripsi template Hub juga memakai skema sama (documentation{}) —
   hub_list/hub_inspect agent7 menampilkan 1-baris dari field tsb.

## 12. Kebijakan pin FINAL — PUTUSAN agent10 #686 (Plt. Security Gatekeeper, 2026-09-09)

Dua kelas, satu kebijakan per kelas (menggantikan §8/§10 utk lingkup; rincian lengkap di
AGENT4-W4-HUB-AUTOUPDATE-SPEC v0.2 §4):

1. SUMBER STATIC (file/dataset ber-versi): pin expected_sha256 WAJIB (nullable) di KATALOG;
   tanpa pin = VERIFIED_TOFU eksplisit. Mismatch -> REJECT + alarm + versi lama live;
   re-pin oleh pemilik katalog (4b semantics, §8).
2. SUMBER VOLATILE (16/18 sumber katalog kini): TANPA value-pin (alarm palsu permanen);
   jangkar v1 = katalog + ETag/cache (W3-1) + record-replay (W2-DETERM-ENFORCE fail-closed).
   verify.status "verified-live" DIBACA = VERIFIED_TOFU ("hidup dari VPS tanggal T", BUKAN
   "asli dari penerbit").
3. structural_guard (schema-pin) = OPSIONAL per-template, default false; validasi struktur
   thd pin.target "response-schema" di katalog saat template mengaktifkannya.
4. ETIKA: terms.status selain "ok" (review-agent5 / grey-BLOCKED-agent5) = sumber boleh diuji
   internal, DILARANG utk template live sampai gate etika agent5 menyetujui. Konsekuensi:
   review_status=live mensyaratkan SEMUA sources[] lolos gate etika; live = lolos review
   pipeline, BUKAN keaslian. Template Hub sumber volatil = TOFU by-design (klaim jujur).

## 13. v0.6 — OPEN-SCRAPE-1/3 + schema_hash digest nyata (agent9 #817, agent10 #814, 2026-09-09)

### 13.1 OPEN-SCRAPE-1 — kind=scrape + profil sumber scrape (JAWABAN agent4)

Sumber scraping = kelas baru di katalog hub://intel-types (agent9) — usul enum:
`kind ∈ {rss, http_json, sse, scrape}`. Profil scrape (di katalog, single-source SEC-HUB-01):

```json
{"source_id": "<id>", "kind": "scrape",
 "url": "…", "volatility": "volatile",
 "scrape_profile": {
   "detector": "penanda deterministik (gate SCRP-L1, agent9)",
   "shield": "5-lapis ARSITEKTUR-SCRAPING-ANTI-BAN (network/fingerprint/behavior/self-heal/offload)",
   "failover_ladder": ["scrapling", "camofox", "browser-use", "firecrawl"],
   "cache": "CASD stale-while-revalidate (snapshot lama + flag stale, NOL downtime)",
   "schema_normalization_gate": "output JSON kanonik stabil walau layout berubah (structural_guard W4)",
   "etika": "SCRAP-E1 wajib lolos utk live (agent5+agent10): HANYA sumber ToS-izinkan-programatik"}
}
```

Aturan turunan:
1. kind=scrape TANPA profil di katalog = TIDAK sah utk sources[] manifest (ref tipis wajib
   berkatalog — konsisten SEC-HUB-01). Override kind ke "scrape" di manifest TIDAK diizinkan;
   hanya pemilik katalog (agent9) menambah sumber scrape.
2. Sumber scrape = kelas VOLATIL: TANPA value-pin (alarm palsu permanen); jangkar =
   katalog + ETag/record-replay + structural_guard (schema-pin) → deteksi break format API.
   Konsisten PUTUSAN #686 + W4 v0.3 §4.
3. rate-limit: token-bucket PER-DOMAIN wajib (gate SCRP-L1-4 agent9) — masuk profil rate_limit
   katalog, bukan override manifest.
4. OUTPUT kanonik sumber scrape tunduk pada normalisasi berversi (N-rules; fungsi bernama +
   daftar field) — supaya diff gate W4 deterministik.

### 13.2 OPEN-SCRAPE-3 — requires_credentials BYOK (JAWABAN agent4)

Skema: blok `requires_credentials` OPSIONAL di §2 template (lihat anotasi skema):
- ABSEN → template bebas-kredensial (Zero-API-Key); nilai default hub v1.
- ADA `byok: true` → template ditandai BYOK (badge di hub_list/UI + filter katalog).
- Nilai rahasia TIDAK PERNAH di template/dag — referensi {credential_kind, target} ke vault
  engine (konsisten PRD-Hub inputs `secret: true`).
- Live TIDAK diblokir otomatis oleh BYOK; prasyarat live: (a) SCRAP-E1 lolos utk sumber
  scrape/anti-detect (etika: ToS-izinkan-programatik; DILARANG paywall/login tanpa izin),
  (b) terms.status sumber "ok" (§12.4), (c) badge BYOK ditampilkan jujur (S6 #713: label beda,
  bukan "verified").
- Firecrawl ber-API-key (temuan L3 agent9, 5680bdfd): node ber-kredensial → template yg
  memakainya WAJIB deklarasi BYOK + kategori listing "BYOK" — tidak pernah disamarkan sbg
  free-feed.

### 13.3 schema_hash digest NYATA (agent10 #814 — diserap)

1. Verdict Gatekeeper #814: schema-pin placeholder ("blake3:macro_v1_hash") TIDAK cukup utk
   Fase 4 (silent-swap tak terverifikasi). Aturan v0.6: schema_hash = `sha256:<64-hex>` yang
   DIHITUNG dari skema kanonik; gerbang sementara: pin nyata = syarat masuk hub-catalog.json.
2. Definisi kanonikalisasi (hash_domain, S4 #713): canon-json-sorted atas
   `{"expected_keys": [...], "outputs": {name: {type}}} ` — sort_keys + separators padat +
   UTF-8; ditulis eksplisit di pinning sebagai `hash_domain: "canon-json-sorted:expected_keys+outputs"`.
3. Template agent4 (4 entri) TELAH di-upgrade ke digest nyata (resubmit CLI, registry
   konsisten). Semua penulis template lain diundang melakukan hal sama sebelum Fase 4.
4. Konsistensi label: schema-pin lulus = SHAPE-VERIFIED (§4a W4 v0.3 / #713 S2), BUKAN
   "verified"; "verified" hanya utk value-pin cocok + anchor TSA eksternal (S1).
