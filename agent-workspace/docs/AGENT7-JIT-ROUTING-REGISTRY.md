# AGENT7-JIT-ROUTING-REGISTRY — Registri Routing JIT & Kontrak Content-Owner

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** v0.5
**Riwayat:** v0.1 (06:35) → v0.2 (hub://, ?lang=, H-3b, hub_id↔URI) → v0.3 (owner limits dikonfirmasi #594; referensi W1-MCP) → v0.4 (manifest Hub v0.4; kesiapan trust-anchor Ed25519) → **v0.5 (07:50, sinkronisasi pemantauan)**: expression-rules → DRAFT v1.0 content_version 2 (agent3, finalisasi integrasi agent7); manifest Hub → v0.5 (agent10 #633 SEC-HUB-01 — pin pindah ke katalog single-source, VERIFIED_TOFU); AGENT1-MCP-SKILL-SPEC re-upload (limits DIKONFIRMASI #588; E-WCB-TRAP/FUEL/TIMEOUT RESERVED — server dilarang serve kode RESERVED)
**Induk:** `AGENT7-MCP-INTEGRATION-SPEC.md` §4.3 · Patuh freeze #386.

---

## 1. Tujuan

Registri ini = **satu sumber kebenaran daftar URI resource MCP** + kontrak kepemilikan: tiap URI punya **tepat 1 owner konten**, versi sumber, dan status siklus hidup. Server MCP v1 membaca registri ini untuk `resources/list` dan routing `resources/read`. Mulai v0.2 registri juga memetakan `hub_id ↔ URI` (single source of truth — kesepakatan agent4 #556) dan parameter kueri resmi per URI.

## 2. Daftar URI resmi (konsolidasi AGENT1 §2 + AGENT4 §2/§4 + AGENT3 + Hub #524/#547)

**Namespace `n8n://` (authoring workflow):**

| URI | Isi inti | Sumber kebenaran | Owner konten | Status | Token target* |
|---|---|---|---|---|---|
| `n8n://nodes/{name}/schema` | parameter+versi+eval_scope | AGENT1 §2, AGENT4 §2 | agent4 (schema compiler v0.2) | dirancang (menunggu K-1..K-4) | ≤500 |
| `n8n://nodes/{type}/alias` | deprecation + konversi | AGENT4-NODE-ALIAS-DEPRECATION.md (V-1/V-2) | agent4 | verified | ≤500 |
| `n8n://nodes/cron/mapping` | cron v1→v2 + default D1–D4 | AGENT4-CRON-PARSER-SPEC.md (#435 verified upstream) | agent4 | verified | ≤500 |
| `n8n://expression-rules` | tabel E-3/W-5 (215 kasus; 3E+5W final #503) | AGENT3-EXPRESSION-RULES-MCP.md **DRAFT v1.0, content_version 2** | agent3 | finalisasi integrasi agent7 (autofix 3-level #446/#449) | ≤500 |
| `n8n://errors/{code}` | arti+contoh+autofix | AGENT1 §2/§4 (E-*), AGENT4 (MIG-*/PARTIAL) | agent1 (E-*), agent4 (MIG-*) | dirancang | ≤500 |
| `n8n://deviation/{DEV-*}` | entri katalog by ID | DEVIATION-CATALOG.md | agent5 | ada | ≤500 |
| `n8n://limits` | budget/guardrail/allowlist `$env` | **owner DIKONFIRMASI agent2** (#588/#594; AGENT1-MCP-SKILL-SPEC §6 + riwayat re-upload) | agent2 | kontrak disetujui | ≤500 |
| `n8n://workflow/{id}/receipt` | receipt Rosetta (ringkas) | kontrak Rosetta (AGENT4-WORKFLOW-ROSETTA.md) | agent4 | kontrak | ≤500 |
| `n8n://templates/{id}` | struktur + verdict determinisme; **`?lang={code}` = 3 pilar + node notes multibahasa (mandat #547; fallback deterministik, i18n_rev; desain: AGENT7-MCP-I18N-RESOURCE)** | corpus 171 + manifest + `workflow_i18n` (agent2) | agent4 (sumber); terjemahan: agent2 | ada (corpus/); i18n dirancang | ≤500 |
| `n8n://templates/{id}?lang=*` | daftar bahasa tersedia + coverage (metadata) | workflow_i18n (agent2; skema final milik agent2 — granularity node-level sedang dikonfirmasi, #609) | agent2 | dirancang | ≤100 |

**Namespace `hub://` (Workflow Hub — kanon agent4 #556, mandat #524/#547):**

| URI | Isi inti | Sumber kebenaran | Owner konten | Status | Token target* |
|---|---|---|---|---|---|
| `hub://skills/{hub_id}/manifest` | manifest utuh (metadata, documentation{}, ai_skill, review_status, est_token) | AGENT4-WORKFLOW-HUB-MANIFEST.md **v0.5** (SEC-HUB-01: pin pindah ke katalog single-source; VERIFIED_TOFU; kontrak resource tak berubah) | agent4 | v0.5 | ≤500 |
| `hub://skills/{hub_id}/receipt` | receipt Rosetta ringkas {template_sha256, content_version, deviation_ids, exec-verdict ref, ts} | Rosetta + manifest | agent4 | kontrak | ≤500 |
| `hub://skills/{hub_id}/manifest?lang=` | deskripsi multibahasa template Hub (aturan sama dgn n8n://templates) | workflow_i18n (agent2) | agent4 + agent2 | dirancang | ≤500 |
| `hub://intel-types` | tipe intel (crypto/politics/…) + ringkasan sumber | katalog konektor agent9 | agent9 | dirancang | ≤500 |

\* Semua angka token final menunggu **tokenizer rujukan #419a** (matt). URI di luar daftar → error standar.

## 3. Kontrak kemasan konten (berlaku semua URI)

1. Payload inti **≤500 token** (tokenizer #419a); detail via pointer file/baris.
2. Metadata tiap payload: `uri`, `content_owner`, `source_ref`, `content_version` (naik saat isi SUMBER berubah), `updated_at`, `budget_tokens`. Khusus konten terjemahan: `i18n_rev` (versi baris terjemahan agent2) — **perubahan i18n TIDAK menaikkan content_version** (hindari invalidasi receipt palsu; agent4 v0.4).
3. Payload = data; larangan prosa instruksional tersembunyi; dirender escaped.
4. Perubahan isi = owner menaikkan versi + kabari agent7 (registri).
5. Kredensial/rahasia tak pernah jadi isi resource (F-9).
6. Satu URI → satu owner konten (terjemahan: pemilik tabel agent2, skema milik agent4).

## 4. Prosedur siklus hidup URI

| Aksi | Siapa | Syarat |
|---|---|---|
| Daftar URI baru | agent7 (registri) | kebutuhan nyata + owner + cek duplikasi + sumber kebenaran |
| Ubah isi | owner | naikkan versi; kabari agent7 |
| Ubah pola URI / kueri | agent7 + owner | cek referensi dokumen fondasi; pemberitahuan katalog |
| Nonaktifkan | owner + agent7 | alasan tertulis |
| Konflik kepemilikan | agent7 + pihak | eskalasi matt/fern |

**Pemetaan hub_id ↔ URI** (agent4 #556): dipetakan DI REGISTRI INI (single source); manifest cukup memuat `hub_id`; server MCP resolve via registri.

## 5. Matriks konfirmasi yang diminta (to-do lintas agent)

| URI | Butuh | Kepada | Status |
|---|---|---|---|
| `n8n://limits` | agent2 bersedia jadi owner + isi awal | agent2 | menunggu |
| `n8n://expression-rules` | versi final + content_version 1 | agent3 | final 3E+5W terkonfirmasi (#503) — tinggal angka token |
| `n8n://errors/{code}` | daftar kode v1 + format stabil | agent1 | sebagian (E-INGRESS-COLLISION #559) |
| `n8n://deviation/{DEV-*}` | format lookup by ID | agent5 | ada |
| `n8n://templates/{id}?lang` | skema `workflow_i18n` + `i18n_rev` | agent2 | dirancang (I-1..I-6 di AGENT7-MCP-I18N-RESOURCE) |
| `hub://…` | kanon URI hub_id + review_status gate | agent4 + agent5 | v0.4 manifest; gate live agent5 |
| `hub://intel-types` | struktur awal | agent9 | dirancang |
| seluruh URI | review adversarial registri v0.2 | agent1–5 | D-7 |

## 6. Verifikasi (gate A7-2, I-1..I-6, H-1..H-8 terkait)

- Diff URI vs penyebutan di AGENT1/3/4 + proposal Hub → 0 duplikasi/konflik pola.
- Uji fallback bahasa 40/40 (I-1); uji receipt-outdated 20 kasus (H-7); uji no-fetch saat discovery (H-3b).
- (Utilitas diff ditulis di Fase 1+ atau /tmp milik agent7 — bukan sekarang, freeze #386.)

## 7. Referensi

AGENT1-MCP-SKILL-SPEC §2 (re-upload, E-WCB RESERVED); AGENT4-MCP-KNOWLEDGE-SPEC §2/KLL-3/4; AGENT3-EXPRESSION-RULES-MCP §5 (v1.0, cv 2); AGENT4-WORKFLOW-HUB-MANIFEST.md v0.5; `AGENT7-WORKFLOWHUB-MCP-PROPOSAL.md` v0.2; `AGENT7-MCP-I18N-RESOURCE.md` v0.2; `AGENT7-W1-MCP-TRANSPORT-PLAN.md` (98d0f827); `AGENT7-W1-MCP-TOOLS-PLAN.md` (69cbe2fe); `AGENT7-MCP-INTEGRATION-SPEC.md` §4.3.
