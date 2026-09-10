# AGENT7-MCP-INTEGRATION-SPEC — Lapisan Integrasi Server MCP untuk Coding Agents

**Penulis:** agent7 (ROLE_AI_MCP — AI & MCP Integration Specialist)
**Tanggal:** 2026-09-09 · **Status:** DRAFT v0.4 — menunggu review adversarial (agent1–5) + keputusan pemilik/matt/fern
**Revisi:** v0.1 `621e6dcf` (06:32, rilis awal #469) · v0.2 `0c5ad285` (06:35, D-1/D-2 dikunci) · v0.3 `50480828` (06:39, D-4…D-7 dikunci) · **v0.4 (serap review: agent1 #503, agent5 #489, agent1 #490 — F-6 3E+5W, pemetaan tool→service, oracle A7-6, taint/authz)**
**Mandat:** fern #452 (ekspansi swarm → "tuntaskan PRD-3 bersama"), mandat #411 (via fern, pilar PRD-3), fondasi tersahkan #431.4 (agent1 #457)
**Handoff masuk:** agent1 #457, agent3 #458, agent4 #459, agent2 #460, agent5 #462 (onboarding keamanan)
**Patuh freeze #386:** dokumen & analisis saja — nol kode produk.

---

## 0. Ringkasan

Tiga spesifikasi MCP sudah ada sebagai fondasi pilar PRD-3 yang disahkan (#431.4):

| Dokumen | Pemilik | Isi |
|---|---|---|
| `AGENT1-MCP-SKILL-SPEC.md` | agent1 | Capsule mastery ≤500 token, 5 tools, registri kode E-*, gate |
| `AGENT4-MCP-KNOWLEDGE-SPEC.md` | agent4 | Lapisan knowledge JIT (resources), DiagnosticReport, gate KLL-1…4 |
| `AGENT3-EXPRESSION-RULES-MCP.md` | agent3 | Isi resource `n8n://expression-rules` (215 kasus → tabel) |

**Peran agent7:** pemilik desain **lapisan integrasi** yang menyatukan ketiganya menjadi satu blueprint server MCP — protokol/transport, mekanik passive skill injection, tabel routing JIT, kontrak hasil & keamanan ke klien (coding agents), serta gap/keputusan yang belum tercakup. Bukan pemilik isi konten (milik agent1/3/4/5), bukan pemilik rakitan PRD-3 (matt).

Dokumen ini mengidentifikasi **1 temuan utama** (protokol MCP berubah drastis: revisi 2026-07-28 menghapus sesi & handshake) dan **9 gap integrasi**, lalu mengusulkan arsitektur, mekanik injeksi/routing, gate terukur, dan **7 keputusan (D-1…D-7)** untuk pemilik proyek.

---

## 1. Fondasi sah yang dipakai (tidak diulang di sini)

Yang sudah dikunci tim dan menjadi batasan desain server:

| # | Fondasi | Sumber |
|---|---|---|
| F-1 | **SATU server MCP** (bukan per-lapisan) | AGENT4 §1.1 |
| F-2 | Tools engine = 5: `inspect_workflow`, `patch_node`, `validate`, `get_receipt`, `preflight` | AGENT1 §3 |
| F-3 | Capsule mastery ≤500 token; content milik agent1 | AGENT1 §1, #411 |
| F-4 | Resources **pull-only, JIT**; server tak pernah push | AGENT1 §2, AGENT4 §1.3 |
| F-5 | Resource ≤500 token inti + pointer file untuk detail | AGENT4 §2 |
| F-6 | ERROR vs WARNING dikunci kelasnya (#431): **3 E** (`E-EXPR-REFERENCE`, `E-EXPR-SCOPE`, `E-EXPR-CREDENTIAL`) + **5 W** (`W-EXPR-FLOAT-PRECISION`, `W-EXPR-SURROGATE`, `W-EXPR-OBJECT-ORDER`, `W-EXPR-UNDEFINED-NULL`, `W-EXPR-ARRAY-METHODS` — daftar W mengikuti versi final AGENT3 §3 pasca #425–#432; sinkronisasi dicatat di REGISTRY content_version); tambahan E-* struktural milik agent1 (E-REF-DANGLING dll.) | AGENT3 §3, #431, AGENT1 §4, review agent1 #503 |
| F-7 | RAM persisten MCP = 0 byte; transien O(workflow)/request dibebaskan usai respons | AGENT1 §5, #431/ERR-029 |
| F-8 | Eksekusi workflow TIDAK pernah lewat MCP (API terpisah ber-autentikasi) | AGENT1 §3 |
| F-9 | Kredensial = refs, tak pernah nilai; redaksi 100% respons | AGENT1 §3/§5 (agent5 sebagai pemilik gate) |
| F-10 | Diagnostic: `{severity, code, node, path, message, hint, autofix}`; kode stabil antar-rilis | AGENT1 §4, AGENT4 §3 |

Batas: seluruh pengukuran token menunggu **tokenizer rujukan** (matt, #419, "TBA" di AGENT3/AGENT4). Dokumen ini tidak mengunci angka token sebelum tokenizer ada.

---

## 2. Temuan utama: protokol MCP bergerak — asumsi "saat connect" perlu dipetakan ulang

### 2.1 Fakta (diverifikasi 2026-09-09 dari modelcontextprotocol.io changelog)

| Revisi | Perubahan relevan |
|---|---|
| 2024-11-05 | Awal; transport HTTP+SSE |
| 2025-03-26 | **Streamable HTTP** menggantikan HTTP+SSE (yang jadi deprecated); OAuth 2.1; `instructions` = field opsional hasil `initialize` |
| 2025-06-18 | Elicitation; struktur hasil tool |
| 2025-11-25 | Ikon tool/resource/prompt; OIDC discovery; sampling bertools; JSON Schema 2020-12 |
| **2026-07-28** | **Stateless total:** sesi & handshake `initialize`/`notifications/initialized` DIHAPUS; tiap request membawa `protocolVersion`/client capabilities di `_meta`; RPC baru `server/discover` (iklan versi+capabilities); `subscriptions/listen`; **Roots/Sampling/Logging di-deprecate**; HTTP+SSE resmi Deprecated |

### 2.2 Implikasi untuk dokumen fondasi

- AGENT1 §1 & AGENT4 §2/§4 menulis "disuntik saat connect via MCP `instructions` + `prompts/get`". Ini konsisten dengan revisi 2024–2025 (`instructions` ada di hasil `initialize`, lihat lifecycle 2025-03-26). **Pada 2026-07-28 tidak ada `initialize`** → mekanik "saat connect" harus dipetakan ulang ke salah satu dari: (a) per-request (capabilities/versi di `_meta`, `server/discover`), (b) `prompts/get` eksplisit, (c) resources pull, (d) sisi-klien (config/skill coding agent — pola dominan untuk Claude Code/Cursor dkk.).
- Konsep **capsule pasif ≤500 token tetap valid sebagai artefak**, hanya saluran suntiknya yang menjadi keputusan implementasi (lihat D-1).
- `diagnostics` lama (E-*/W-*) dan konsep tools/resources/prompts TIDAK berubah secara konseptual — yang berubah adalah siklus hidup koneksi. Dampak nyata ke desain: **handler server harus stateless murni** (kebetulan sejalan F-7) dan **tidak boleh menyimpan state sesi di RAM/DB** — justru menyederhanakan arsitektur yang agent1/agent4 sudah rancang.

### 2.3 Rekomendasi agent7

Jangan mengunci versi protokol di dokumen sekarang (implementasi masih Fase 1+). Rancang **adapter protokol** (satu permukaan internal, binding per-revisi), dan putuskan saat implementasi berdasarkan SDK/klien aktual. Detail di D-1.

---

## 3. Gap integrasi (yang belum tercakup tiga spec fondasi)

| ID | Gap | Dampak | Pemilik solusi |
|---|---|---|---|
| G-1 | Versi protokol & strategi negosiasi belum diputuskan (perubahan 2.1) | Server dibangun di atas asumsi usang | agent7 + matt (D-1) |
| G-2 | Transport & deployment belum ditentukan (stdio/HTTP; posisi proses; OAuth) | Arsitektur proses & keamanan beda drastis | agent7 (D-2/D-3) |
| G-3 | Mekanik passive skill injection belum dijabarkan (saluran, urutan muat, larangan) | "≤500 token" tanpa cara sampai ke model | agent7 (§4) |
| G-4 | Belum ada tabel routing JIT tunggal + 1-owner per URI + siklus hidup versi | Resource ganda/kedaluwarsa; KLL-4 hanya cek duplikasi antar-2 dokumen | agent7 (§4, W3) |
| G-5 | Belum ada kontrak pengemasan DiagnosticReport → hasil MCP (isError, structured content) + titik redaksi | Klien tidak bisa loop perbaiki secara terstruktur | agent7 + agent5 |
| G-6 | Service facade engine belum didefinisikan sebagai antarmuka | Server MCP menunggu engine; padahal bisa diuji dgn mock | agent7 (W7) |
| G-7 | Belum ada gate end-to-end (token/latensi/FP per sesi, bukan per-komponen) | Lolos per-komponen ≠ lolos terpasang | agent7 (§6) |
| G-8 | Belum ada contoh sesi nyata + threat model integrasi utk coding agents | Agent3 §6 sudah mulai; belum utuh | agent7 + agent5 (W4) |
| G-9 | Siapa penerima/versi tiap resource saat sumber naik versi (schema compiler v0.2→v0.3 dll.) | Resource basi diam-diam | agent7 + pemilik konten (D-4) |

---

## 4. Arsitektur yang diusulkan (desain, bukan kode)

### 4.1 Posisi & lapisan

```
Klien (coding agent: Claude Code/Cursor/agent internal)
   │  stdio (v1) │  Streamable HTTP (Fase 2+, ber-OAuth — HTTP+SSE TIDAK dibangun)
   ▼
┌─ TRANSPORT ADAPTER (stdio | streamable-http) ─────────────┐
├─ PROTOCOL ADAPTER (binding per revisi MCP; permukaan sama) │ ← G-1
├─ CAPABILITY HANDLERS (tools | resources | prompts | discover/initialize)
├─ SERVICE FACADE (trait, bukan implementasi):               │ ← G-6
│    WorkflowService · ValidateService · SchemaService ·
│    ReceiptService · LimitsService
├─ CONTENT REGISTRY (routing JIT §4.3; 1 owner per URI)      │ ← G-4
└─ (Fase 1+) → engine services (executor/scheduler/registry via kontrak kernel)
```

- **Posisi proses:** subcommand/mode binary tunggal (`engine mcp --transport …`) — konsisten prinsip satu-binary PRD-2 §2 & T-3. Proses terpisah saat product (Fase 2+). (D-3)
- **Stateless murni:** tidak ada sesi; semua state transien per-request (F-7; sejalan revisi 2026-07-28). Kredensial hanya refs.
- **Handler tipis:** tidak ada logika bisnis di handler; hanya (de)serialisasi + pemanggilan facade → semua redaksi & validasi di facade (G-5).
- **Pemetaan tool→service EKSPLISIT** (syarat review agent1 #503): `inspect_workflow` → WorkflowService · `patch_node` → WorkflowService · `validate` → ValidateService · `get_receipt` → ReceiptService · `schema` (bila dibuka sbg tool) → SchemaService · **`preflight` → ValidateService (perluasan kontrak)** — ditetapkan sebelum PRD-3 dikunci.

### 4.2 Permukaan v1 (usulan; isi = milik agent1/3/4)

| Permukaan | Isi | Pemilik konten |
|---|---|---|
| Tools (5) | `inspect_workflow`, `patch_node`, `validate`, `get_receipt`, `preflight` | agent1 |
| Resources | URI `n8n://` per tabel §4.3 | agent4/agent3/agent1/agent2/agent5 |
| Prompts | 1 template "authoring loop" (dipanggil eksplisit) — tidak ada prompt push | agent7 merakit dari agent1/3/4 |
| Capsule | Artefak ≤500 token; saluran sesuai D-1 | agent1 (isi) |

### 4.3 Routing JIT — tabel konsolidasi v0 (verifikasi 1:1 dengan pemilik sebelum dikunci)

Aturan: pull-only; ≤500 token inti; pointer file utk detail; **1 owner per URI**; perubahan = pemberitahuan ke pemilik katalog (norma KLL-3/KLL-4).

| URI | Isi | Sumber | Owner | Status |
|---|---|---|---|---|
| `n8n://nodes/{name}/schema` | parameter+versi+eval_scope | AGENT1 §2, AGENT4 §2 | agent4 (schema compiler v0.2) | menunggu K-1…K-4 |
| `n8n://nodes/{type}/alias` | deprecation+konversi | AGENT4 §2 (V-1/V-2) | agent4 | verified |
| `n8n://nodes/cron/mapping` | cron v1→v2 + default D1–D4 | AGENT4 §2 | agent4 | verified upstream (#435) |
| `n8n://expression-rules` | tabel E/W (215 kasus) | AGENT3 | agent3 | draft, review #425–#432 |
| `n8n://errors/{code}` | arti+contoh+autofix | AGENT1 §2/§4 | agent1 (E-*); agent4 (MIG-*/PARTIAL) | draf |
| `n8n://deviation/{DEV-*}` | entri katalog by ID | AGENT4 §2, DEVIATION-CATALOG.md | agent5 | ada |
| `n8n://limits` | budget/guardrail/allowlist `$env` | AGENT1 §2; PRD-2 §8.3 | agent2 (usulan; konfirmasi) | belum ada dokumen |
| `n8n://workflow/{id}/receipt` | receipt Rosetta | AGENT4 §4 | agent4 | kontrak |
| `n8n://templates/{id}` | struktur+verdict determinisme | AGENT4 §2 | agent4 (manifest korpus) | ada (corpus/) |

### 4.4 Passive skill injection — mekanik (G-3)

Urutan muat yang disarankan (bukan push; setiap langkah atas inisiatif klien):

1. **Identitas & versi** — `server/discover` (2026-07-28) / `initialize` (revisi lama): server mengiklankan capabilities + `instructions`/setara.
2. **Capsule mastery** — artefak ≤500 token (isi agent1 §1.1–1.5) dikirim sebagai `instructions`/setara pada langkah 1, atau via `prompts/get` saat sesi authoring dimulai (keputusan D-1).
3. **Resources JIT** — klien menarik per-URI dari tabel §4.3 saat task membutuhkan (pola tugas, bukan daftar buta — AGENT1 §2).
4. **Diagnostik loop** — tiap `patch_node`/`validate` mengembalikan DiagnosticReport terstruktur (G-5) → klien loop perbaiki sampai bersih (AGENT4 §4).

**Larangan tegas:** server tak pernah push resource; tak menyisipkan instruksi tersembunyi di output tool; tak mengeksekusi workflow; tak menyentuh nilai kredensial.

---

## 5. Keamanan integrasi (sambungan agent5; detail gate milik agent5)

- Eksekusi non-MCP (F-8). Tools authoring/validasi saja.
- Redaksi di service facade (sekali, bukan per-handler): 100% nilai kredensial tak pernah keluar (F-9); uji sintetis 50 sesi (A7-6).
- Threat model yang diadopsi dari AGENT3 §6: data-driven injection (workflow asing → output ditandai `tainted`), covert channel `$env` (allowlist eksplisit di `n8n://limits`). Agent7 menambahkan: prompt-injection lewat nama node/workflow asing saat `inspect_workflow` — nama/deskripsi dari workflow tak-terpercaya diperlakukan sebagai data, dirender escaped, tidak pernah sebagai instruksi. (Review agent5 #489: disetujui; SEC-MCP-01 terintegrasi.)
- **TAINT-PROPAGATION = invariant rantai tool** (adopsi agent1 #490): `inspect → patch → validate` TIDAK boleh menghilangkan taint; taint = flag di respons yang wajib diteruskan ke respons berikutnya.
- **AUTHZ-stateless = dependensi transport** (adopsi agent1 #490): tiap request diotorisasi independen (tak ada sesi tepercaya); capsule = data publik-ke-LLM.
- **Definisi LEAK untuk A7-6** (disepakati agent5 #489 + agent1 #490): oracle pola scan #394 (BEGIN…PRIVATE KEY, password=/passwd=, api_key/bearer/authorization, blob base64/hex ≥32 char) + daftar credential-value fixture; "0 leak" = 0 kecocokan pola pada 50 sesi sintetis.
- stdio: stdout = protokol murni; semua log ke stderr (juga berlaku umum per spec). HTTP (nanti): OAuth resource server + Origin check (403 utk Origin invalid, spec 2026-07-28).
- Keputusan auth MCP HTTP = milik matt/PRD-3 (AGENT1 §6 "matt: … autentikasi MCP") → D-7 koordinasi.
- Output Hub/eksternal (bila terintegrasi via Workflow Hub, proposal AGENT7-WORKFLOWHUB-MCP-PROPOSAL) = **TAINTED-EXTERNAL**: dilarang mengalir ke kredensial/partials (invarian agent1 #528).

---

## 6. Gate penerimaan integrasi (falsifiable; satuan eksplisit — ERR-029)

| ID | Gate | Target | Cara ukur |
|---|---|---|---|
| A7-1 | Conformance protokol | Lulus daftar metode versi dipin (tools/resources/prompts/discover) | MCP Inspector (atau setara saat implementasi) |
| A7-2 | Tanpa duplikasi | URI 1:1 dgn AGENT1/3/4; tak ada konflik semantik | Diff tabel §4.3 vs 3 dokumen (otomasi) |
| A7-3 | Token | tiap resource ≤500 token; kasus cron (capsule+resource) <900 token (KLL-2) | tokenizer rujukan #419 (TBA) |
| A7-4 | Overhead | handshake/discover stdio ≤ 50 ms p50 (dev); RAM persisten 0 byte (RSS idle ±server) | hyperfine / RSS sampling 100 ms |
| A7-5 | Kualitas loop | 20 skenario edit sintetis: kode error benar 20/20; perbaikan ≤3 iterasi (sambung KLL-1) | corpus + skenario sintetis |
| A7-6 | Redaksi — definisi leak = oracle agent5 #489 (pola #394 + fixture) | 0 kecocokan pola dari 50 sesi sintetis (workflow jahat) | scan output (gate agent5) |

Catatan: A7-4 perlu kalibrasi ulang di mesin target 2 CPU (PRD-2 §6.1); angka dev-machine hanya pagar awal.

---

## 7. Rencana kerja agent7

**Sekarang — freeze #386 (dokumen):**

| ID | Artefak | Ketergantungan |
|---|---|---|
| W1 | Dokumen ini (integrasi + gap + keputusan) | — (selesai) |
| W2 | ROSTER protokol & permukaan: matriks versi MCP × capability × transport, utk masukan PRD-3 (matt) | review W1 |
| W3 | Finalisasi tabel routing §4.3 + kontrak content-owner (versi & siklus hidup per URI) | konfirmasi agent1/2/3/4/5 |
| W4 | Contoh sesi end-to-end (coding agent mengedit workflow) + threat model integrasi | agent5 (review), kontrak facade |
| W5 | Masukan ke rakitan PRD-3 / PRD-3-PERFECTION-CHECKLIST (matt) | jadwal matt |

**Fase 1+ (setelah freeze dicabut & keputusan K/T):**

| ID | Artefak | Ketergantungan |
|---|---|---|
| W6 | Spike: transport stdio + conformance (MCP Inspector) + SDK rujukan | keputusan D-1/D-2 |
| W7 | Handler + service facade + mock engine (uji tanpa engine penuh) → kontrak utk agent1 | G-6, engine service boundary |

Dependensi eksternal: tokenizer rujukan #419 (semua angka token), K-1/K-2/K-3 & T-14 (scope produk: authoring internal vs publik), keputusan auth (matt), mount vdb 30GB / CARGO_TARGET_DIR (#455, fern/pemilik) utk build nanti, rotasi SSH key & sudoers (K-5/K-6 — rekomendasi tetap: pemilik eksekusi).

---

## 8. Keputusan yang diminta (untuk pemilik proyek / matt / fern)

| ID | Pertanyaan | Opsi | Rekomendasi agent7 |
|---|---|---|---|
| D-1 | Strategi versi protokol MCP | (a) baseline 2025-06-18/2025-11-25 (ekosistem klien luas), (b) langsung 2026-07-28 (stateless, terbaru), (c) **adapter ganda, pin saat implementasi** | **c — DIKUNCI pemilik 2026-09-09** (rincian: AGENT7-PROTOCOL-ROSTER §3). Catatan kompatibilitas utk AGENT1/4 (asumsi "saat connect") wajib diberi anotasi |
| D-2 | Transport v1 | (a) **stdio dulu**, (b) stdio + Streamable HTTP | **a — DIKUNCI pemilik 2026-09-09** — HTTP butuh auth/oauth (keputusan matt) & deployment; stdio cukup utk authoring agent lokal |
| D-3 | Posisi server | (a) **mode binary tunggal** (`engine mcp`), (b) crate/proses terpisah | **a** — konsisten satu-binary PRD-2 §2 |
| D-4 | Scope resource v1 | (a) **subset authoring inti** (schema, alias, expression-rules, errors, receipt), (b) semua URI §4.3 | **a — DIKUNCI pemilik 2026-09-09** — sisanya (limits, templates, deviation) menyusul; tiap URI butuh owner+versi dulu (G-9) |
| D-5 | Batas kewenangan MCP | (a) **authoring/validasi saja**, (b) termasuk eksekusi terbatas | **a — DIKUNCI pemilik 2026-09-09** — F-8 garis merah; eksekusi = API terpisah |
| D-6 | `preflight` & kredensial | (a) **refs-only + allowlist `$env` dikunci**, (b) longgar dulu | **a — DIKUNCI pemilik 2026-09-09** — E-EXPR-CREDENTIAL HARD REJECT (#431); jangan buka celah via tool |
| D-7 | Alur review | (a) **review adversarial agent1–5 dulu** sebelum masuk PRD-3, (b) langsung ke matt | **a — DIKUNCI pemilik 2026-09-09** — budaya tim (VERIFIKASI §8); agent7 baru, butuh jaring pengaman |

---

## 9. Sumber & catatan verifikasi

Dibaca langsung di VPS (2026-09-09, sesi agent7): `/opt/agent-workspace/docs/{AGENT1-MCP-SKILL-SPEC, AGENT4-MCP-KNOWLEDGE-SPEC, AGENT3-EXPRESSION-RULES-MCP, AGENT1_CORE_ENGINE_SPEC, KEPUTUSAN-YANG-DIBUTUHKAN, PRD-2-RUST(§12–16), SINTESIS-SPILLSTORE, VERIFIKASI-DELIVERABLE-TIM(§8)}.md`; comm.db pesan #452–#464; #aturan Pasal 1–7; `mem core ROLE_AI_MCP` (role card L0). Fakta protokol MCP: modelcontextprotocol.io changelog revisi 2025-03-26…2026-07-28 (diakses 2026-09-09).

Kualifikasi jujur (norma tim): dokumen ini DRAFT penulis tunggal yang belum melalui review adversarial penuh; setiap klaim punya sumber di atas; bila ada yang keliru, ralat akan dipublikasikan di kanal yang sama dengan menyebut bagian yang batal.

> **Catatan integritas checksum (agent5 #489):** agent5 melaporkan checksum #469 (`621e6dcf1e70f005…`) ≠ checksum file saat ia baca (`5a1c7c1d448d258c…`). Akar: dokumen ini di-**edit in-place** setelah rilis (v0.1→v0.2→v0.3), sementara checksum yang dikutip tidak diperbarui — kesalahan proses, dan koreksi agent5 tepat (budaya ERR-029: angka yang dikutip harus reproduktif). **Perbaikan permanen mulai v0.4:** (1) setiap revisi memublikasikan checksum baru + riwayat versi di header (§0) dan di pesan kanal; (2) isi konten tidak pernah diklaim dengan checksum lama. Riwayat: v0.1 `621e6dcf` → v0.2 `0c5ad285` → v0.3 `50480828` → v0.4 (serap review #489/#490/#503) dihitung & diumumkan saat rilis. Dokumen turunan REGISTRY/ROSTER/THREATMODEL mengikuti aturan yang sama.

*Ditulis oleh agent7 (ROLE_AI_MCP). Koreksi & review dipersilakan — makin cepat, makin murah.*
