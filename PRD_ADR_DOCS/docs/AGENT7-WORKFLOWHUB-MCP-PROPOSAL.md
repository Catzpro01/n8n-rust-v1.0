# AGENT7-WORKFLOWHUB-MCP-PROPOSAL — Integrasi Workflow Hub sebagai AI Agent Skill via MCP

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** DRAFT v0.2 — entri sayembara, menunggu review
**Riwayat:** v0.1 `9b147357` (06:53) → **v0.2** (serap review: agent1 #550 (H-3b), agent1 #566 (exec_endpoint relatif), agent3 #545 (receipt-outdated + H-7), agent4 #556 (align manifest v0.2/v0.3), agent2 #554, agent3 #537 (uji injeksi berstrata))
**Mandat:** Pemilik proyek via fern #524/#525/#526 (Workflow Hub) + #498 (matriks 11 agen) + sayembara #298/#362
**Patuh freeze #386:** dokumen & proposal — nol kode.
**Bidang agent7 (mandat #524):** "Integrasi ke MCP Resource & Tools agar AI Agent bisa langsung pakai sebagai skill pasif."

---

## 0. Ringkasan proposisi

**Workflow Hub = repositori template workflow "Open-Web Intelligence" (kripto/politik/berita dari sumber publik gratis).** Agar AI agent bisa memakainya sebagai *skill*, agent7 mengusulkan **lapisan MCP Hub**: dua tool discovery (`hub_list`, `hub_inspect`) + resource katalog (`hub://skills/{hub_id}/…`) yang membuat template Hub **ditemukan, dinilai, dan dipilih** dengan biaya token rendah — sambil mempertahankan garis merah terkunci: **eksekusi TIDAK lewat MCP** (D-5/F-8/MCP-11; disetujui agent1 #550), dan output Hub = **data eksternal tainted**.

Diferensiator: permukaan discovery **tetap** (2 tool + resource template) + invokasi terpisah — katalog 100+ skill muat dengan biaya permukaan konstan (dipuji agent3 #545). Review agent1 #550 **menutup MCP-09** dan **menyetujui boundary Hub-MCP** dengan satu garis tajam yang kini diserap: `dry_run.est_token` wajib dari metadata manifest, **bukan fetch-live** (H-3b).

---

## 1. Tugas bidang & batas domain

| Bidang | Pemilik | Deliverable |
|---|---|---|
| Skema manifest (metadata, tags, schedule, zero-code, self-doc, i18n) | agent4 | `AGENT4-WORKFLOW-HUB-MANIFEST.md` (v0.3 `e235afeb…`) |
| Konektor sumber publik (Zero-API feeds, RSS, JSON endpoints) | agent9 | katalog konektor |
| Kompresi ekstraktif transien (<200 token) | agent1 + agent3 | pipeline deterministik (#528) |
| Etika scraping, robots, timeout, sanitasi prompt-injection | agent5 | gate SEC-HUB |
| Penyimpanan manifest/i18n/cache (Layer 0) | agent2 | tabel + FTS5 + bundle i18n <500KB |
| **Integrasi MCP Resource & Tools sebagai skill pasif** | **agent7** | **dokumen ini** |

Batas agent7: **tidak** membuat konektor/kompresor/skema manifest; **tidak** mengeksekusi workflow via MCP; **tidak** menghitung estimasi token dari eksekusi (hanya membaca metadata — H-3b).

## 2. Prinsip desain yang dipegang

| # | Prinsip | Konsekuensi untuk Hub |
|---|---|---|
| P-1 | MCP authoring/discovery; eksekusi = API terpisah (D-5, F-8, MCP-11) | MCP hanya memaparkan **katalog + estimasi-dari-metadata**; invokasi = API eksekusi ber-autentikasi (admission agent1 §5.2 + ingress agent9) — disetujui agent1 #550/#566 |
| P-2 | JIT pull-only; resource ≤500 token inti + pointer | Katalog = ringkasan; detail via pointer |
| P-3 | Capsule ≤500 token pasif | Bila "pasif" = pengetahuan disuntik: isi = indeks katalog + aturan pakai, bukan isi berita |
| P-4 | Output eksternal = `TAINTED-EXTERNAL` (agent1 #528/#490; agent3 #545) | Dilarang alir ke kredensial/partials; taint diteruskan antar-tool |
| P-5 | Redaksi: oracle #394 (agent5 #489) | A7-6 diperluas ke sesi Hub |
| P-6 | Stateless (MCP 2026-07-28) + RAM 0 persisten | Baca registri katalog per-request; tanpa state |
| P-7 | 1 owner/URI + content_version (REGISTRI agent7) | URI `hub://` terdaftar; pemetaan hub_id↔URI = single source (agent4 #556) |

## 3. Desain integrasi (v0.2 — selaras manifest agent4 v0.3)

### 3.1 Permukaan MCP Hub

**Tools (2, discovery):**

| Tool | Input | Output (ringkas) |
|---|---|---|
| `hub_list` | filter: tags[], query, lang? | daftar skill **status `live` SAJA** (agent4 #556): hub_id, skill_id, nama, deskripsi ≤120 token (tokenizer #419a), tags, freshness, `est_token` (dari metadata manifest — H-3b) |
| `hub_inspect` | hub_id, lang? | manifest ringkas: params ($ref skema agent4), jadwal, sumber (agent9), taint-class, **`receipt_outdated` flag** (agent3 #545.4: `receipt.content_version < manifest.content_version` → "re-execute needed"), `exec_ref` = **referensi RELATIF** (agent1 #566), `est_token` metadata |

**Resources (pull-only, kanon agent4 #556):**

| URI | Isi | Owner konten |
|---|---|---|
| `hub://skills/{hub_id}/manifest` | manifest (metadata, documentation{}, ai_skill, review_status, …) | agent4 |
| `hub://skills/{hub_id}/receipt` | receipt Rosetta ringkas `{template_sha256, content_version, deviation_ids, exec-verdict ref, ts}` | agent4/Rosetta |
| `hub://intel-types` | tipe intel + ringkasan sumber | agent9 |
| `n8n://templates/{id}?lang={code}` | 3 pilar deskripsi + node notes multibahasa (mandat #547 — lihat `AGENT7-MCP-I18N-RESOURCE.md`) | agent4 (sumber) + agent2 (i18n) |

**TIDAK ADA** tool eksekusi di MCP. Tool contoh mandat (`fetch_crypto_intel` dst.) = **entri katalog** (`ai_skill = {skill_id, description ≤120 token, invoke_ref}` per agent4) + endpoint API eksekusi terpisah — bukan tool MCP. `exec_ref` = referensi relatif; base URL diumumkan via kapabilitas server (agent1 #566: hindari URL absolut per-skill — tak portabel, bocor topologi).

### 3.2 Alur pemakaian skill oleh AI agent

```
1. tools/call hub_list {tags:["crypto"], lang:"id"}          (JIT; hanya status live)
2. tools/call hub_inspect {hub_id:"hub-0001", lang:"id"}     (≤500 token; receipt_outdated flag)
3. Invokasi via API EKSEKUSI terpisah (BUKAN MCP):
   POST /executions {workflow_id:<skill>, args}  [auth; admission agent1 §5.2; ingress agent9]
4. Hasil: ringkasan <200 token (kompresor agent1+agent3), label TAINTED-EXTERNAL,
   receipt diperbarui (agent4) → hub://skills/{hub_id}/receipt
5. Agent memakai hasil SEBAGAI DATA — sanitasi agent5; deskripsi terbaca dalam bahasa agent (mandat #547)
```

### 3.3 Skill pasif & kapsul

Indeks katalog (hub_id + 1-baris + kapan cocok) dapat menjadi bagian **capsule ≤500 token** — keputusan isi kapsul milik agent1 (F-3); agent7 menyediakan mekanik + batas token.

### 3.4 Siklus hidup konten

Fetch baru = **CANDIDATE** → verifikasi sha → diff-gate deterministik → live (agent4 #535: 0 template mati senyap). Katalog direfresh pipeline luar-MCP; MCP baca per-request (P-6). `review_status`: hanya `live` tampil di `hub_list`; gate agent5 sebelum live. Terjemahan (derived data) **tidak** menaikkan content_version manifest (agent4 v0.3 — hindari invalidasi receipt palsu).

## 4. Keamanan (agent5 pemilik gate)

- Output Hub = `TAINTED-EXTERNAL` (P-4); taint tak boleh tercuci antar-tool (agent1 #490).
- Uji injeksi **berstrata** (agent3 #537): 5 strata × 10 skenario = 50: data-driven (10), covert channel (10), tool poisoning (10), node-name (10), cache-basi (10) — tiap strata punya pass criteria eksplisit.
- Sanitasi prompt-injection web publik: agent5 (mandat #524).
- Redaksi: oracle #394; sesi Hub masuk cakupan A7-6.
- Etika scraping/robots/timeout: agent5; MCP tidak memulai fetch apa pun (hanya discovery).
- Allowlist sumber + `review_status` gate sebelum live.

## 5. Ketergantungan

| Dependensi | Pemilik | Untuk |
|---|---|---|
| Skema manifest (v0.3) + gates HUB-1..HUB-7 | agent4 | resource `hub://…/manifest`; kanon URI |
| Katalog konektor + tipe intel | agent9 | isi `hub://intel-types`; entri katalog |
| Kompresor <200 token | agent1 + agent3 | janji hemat token (hipotesis s/d ukur agent8) |
| Estimasi token di metadata manifest | agent4+agent9 (penulis), agent8 (ukur akurasi, H-8) | `est_token` (H-3b: tanpa fetch-live) |
| Gate etika/sanitasi + live-status | agent5 | sebelum `live` |
| API eksekusi terpisah | engine Fase 1+ (agent1 §5.2 + agent9 ingress) | langkah 3 §3.2 |
| Registri katalog + tabel i18n | agent2 (Layer 0) | manifest/receipt/terjemahan |
| Tokenizer rujukan #419a | matt | semua angka token |

## 6. Gate penerimaan (falsifiable, satuan eksplisit)

| ID | Gate | Target | Cara ukur |
|---|---|---|---|
| H-1 | Token permukaan Hub | hub_list + hub_inspect + 1 manifest ≤1.500 token (tokenizer #419a) | ukur tokenizer rujukan |
| H-2 | Conformance | tools + resource hub:// lulus MCP Inspector | Inspector |
| H-3 | Tak ada eksekusi via MCP | permintaan run/eksekusi → ditolak/tak tersedia 100% | uji negatif 20 kasus |
| **H-3b** | est_token = metadata, tanpa fetch-live | 0 panggilan jaringan saat hub_list/hub_inspect (agent1 #550) | audit jejaring sesi discovery |
| H-4 | Taint | 100% respons Hub berlabel TAINTED-EXTERNAL; 0 aliran ke kredensial | 50 skenario berstrata (agent3 #537) |
| H-5 | Freshness | katalog = manifest terbaru (content_version match) 100% | diff registri vs manifest |
| H-6 | Redaksi | 0 kecocokan oracle #394 pada sesi Hub sintetis | scan (agent5) |
| H-7 | Receipt-outdated | `hub_inspect` menampilkan flag saat receipt.content_version < manifest (agent3 #545.4) | 20 kasus sintetis (10 basi/10 segar) |
| H-8 | Akurasi est_token | deviasi est vs aktual tercatat (diukur agent8) | bench agent8 (target angka menyusul) |

## 7. Koordinasi & status

- Selaras manifest agent4 v0.3 (`AGENT4-WORKFLOW-HUB-MANIFEST.md`, sha `e235afeb…`): kanon URI `hub://skills/{hub_id}/…`, `ai_skill` schema, review_status live-only, receipt-outdated, i18n linkage. Registri URI agent7 diperbarui (v0.2) sebagai pemeta hub_id↔URI single-source.
- Masukan PRD-3 (matt); keputusan: Hub INCLUDE awal Fase 1 (spike stdio + mock katalog) vs pasca-MVP — mengikuti pola WCB #502; agent1 #565: katalog+manifest+manual = Wave-2/3, auto-update+Envelope = Wave-4, value-cepat = katalog-dulu.
- Review masih terbuka: agent9 (konektor↔hub_list), agent5 (H-4/H-6 + gate live), agent2 (registri Layer 0).

## 8. Referensi

Mandat #524–#526, #547; review: agent1 #550/#566/#528/#490/#565, agent3 #545/#537, agent4 #556/#535, agent2 #554, agent5 #489; `AGENT4-WORKFLOW-HUB-MANIFEST.md` v0.3; fondasi MCP agent7 (INTEGRATION-SPEC v0.4, PROTOCOL-ROSTER, JIT-ROUTING-REGISTRY, SESSION-THREATMODEL).

*Ditulis oleh agent7 (ROLE_AI_MCP). Koreksi & review dipersilakan.*
