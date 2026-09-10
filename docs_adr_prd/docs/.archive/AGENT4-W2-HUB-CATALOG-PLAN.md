# W2-HUB-CATALOG — Rencana Wave-2: Katalog Workflow Hub, Manifest & Free Feeds

**Pemilik:** agent4 (ROLE_SCHEMA) · **Wave 2 · P1 · Status: DELIVERED via swarm-task**
**Patuh freeze #386:** dokumen/rencana — nol kode. Implementasi = setelah PRD-3 disahkan.
Catatan: judul queue menyebut "Manifest v0.3"; deliverable aktual = **v0.5** (7bb6cb32…).

## 1. Cakupan tugas (dari queue)

"Katalog Workflow Hub, Manifest v0.3 dan Free Feeds" — satu paket sisi SCHEMA atas Workflow
Hub (mandat #524/#526): skema manifest template + tata kelola katalog + integrasi sumber
gratis (free feeds) yg dipetakan agent9.

## 2. Deliverable & status

| Artefak | Status | Sha (2026-09-09) |
|---|---|---|
| AGENT4-WORKFLOW-HUB-MANIFEST.md v0.5 — skema manifest (identitas content-addressed, sources[] thin-ref + pin DI KATALOG SEC-HUB-01, update 5-field, documentation{} #547, i18n ref, license/attribution H-03, jangkar TOFU) + gates HUB-1..7 + konvergensi §7-§11 | FINAL-draf; review: agent3 #537/#545, agent1 #536/#540/#550, agent10 #561/#633 (SEC-HUB-01 diterima), agent7 #583 | 7bb6cb32 |
| AGENT4-W1-ROSETTA-PARSE-PLAN.md §4.5/§5 (kait Hub impor, receipt, deviation_id) | DONE (W1) | 5b2172eb |
| AGENT9-PUBLIC-DATA-CONNECTORS.md — katalog konektor free-feeds (7 terverifikasi live + [VERIFY]; kind rss/http_json/sse; terms) | agent9 (rujukan) | 7e49abaf |
| AGENT7-JIT-ROUTING-REGISTRY.md v0.2 — namespace hub://, pemetaan hub_id↔URI single-source | agent7 (rujukan) | e43ec6db |
| AGENT7-WORKFLOWHUB-MCP-PROPOSAL.md v0.2 — resource hub://skills/{hub_id}/manifest+receipt, hub_list/hub_inspect, H-3b | agent7 (rujukan) | 3ad9029f |

## 3. Keputusan arsitektur (dikunci lewat review silang, 2026-09-09)

1. **Manifest = payload; katalog = indeks; registri = single source id/URI** (agent7
   registry; prinsip agent4 #535 §4.3) — TIDAK ada dual-source-of-truth.
2. **Profil sumber lengkap di katalog agent9** (auth:"none", terms, rate_limit, cache,
   verify-evidence); sources[] manifest = referensi tipis {source_id, kind override?,
   url override?, rate_policy, etag_cache, robots, expected_sha256}.
3. **kind enum = rss|http_json|sse** (snake_case; Wikipedia/MediaWiki = http_json atau sse).
4. **Kontrak MCP Hub**: discovery+estimasi via MCP; eksekusi VIA API terpisah (D-7/F-8);
   est_token turunan metadata, tanpa fetch-live (H-3b); output = TAINTED-EXTERNAL.
5. **Lisensi**: license enum + license_url + attribution (agent10 H-03); jangkar keaslian:
   TOFU eksplisit + pin expected_sha256 (H-02).
6. **Self-doc/i18n** (#547): documentation{3 pilar + notes_binding}; terjemahan = tabel
   agent2 + i18n_rev terpisah dari content_version (tanpa invalidasi receipt palsu).

## 4. Acceptance (falsifiable)

- HUB-1: 10 manifest contoh valid 10/10 + sha byte-identik saat unduh-ulang.
- HUB-3: template node deprecated → receipt dgn deviation_id.
- HUB-5: 100 template korpus → manifest 100/100 valid + sha + receipt (integrasi Rosetta).
- HUB-6: 3 pilar documentation non-kosong pada template live 10/10.
- KAT-1: 18 konektor → 18/18 profil valid (kind enum, terms.status, verify.status verified-tofu, pin).
- Registri: tiap template live → 1 baris registri (hub_id, URI, content_version, status).

## 5. Kait lintas-agent

agent1 (boundary/exec API/H-3b), agent2 (L0 storage + workflow_i18n + registri katalog),
agent3 (strata injeksi H-4 + CASD), agent5 (gate etika/live + H-4/H-6), agent7 (MCP/registri),
agent9 (konektor/intel-types), agent10 (lisensi/attribution), agent8 (ukuran est_token H-8).

## 6. Riwayat

v0.1 2026-09-09: dibuat utk klaim swarm-task W2-HUB-CATALOG (agent4).
