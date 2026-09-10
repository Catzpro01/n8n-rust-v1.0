# AGENT7-W1-MCP-TOOLS-PLAN — MCP Native Tools & JIT Resource Routing (Wave-1, P0)

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** REVIEW (task W1-MCP-TOOLS)
**Task:** `W1-MCP-TOOLS` — Implementasi MCP Native Tools dan JIT Resource routing · **Wave:** 1 · **Prioritas:** P0
**Dependensi:** `W1-MCP-TRANS` (DONE, agent7) — router/facade siap di AGENT7-W1-MCP-TRANSPORT-PLAN §3
**Amendemen v0.2:** (a) taint 3-level {EXTERNAL|SANITIZED|INTERNAL} + upgrade via sanitasi agent5 (adopsi agent3 #610.3); (b) staleness receipt dua kelas — manifest-level (H-7) & workflow-level (patch → wajib re-validate; agent3 #610.4).
**Patuh Zero-Code Mandate:** rencana implementasi & kontrak antarmuka (desain, bukan kode terkomit).
**Keputusan terkunci:** D-1…D-7; F-6 final (3E+5W — agent1 #503); pemetaan tool→service eksplisit; H-3b (est_token = metadata, tanpa fetch); kanon URI hub:// (agent4 #556); owner lengkap 9 URI (agent1 #594: limits → agent2).
**Dokumen induk:** AGENT7-MCP-INTEGRATION-SPEC v0.4 · AGENT7-JIT-ROUTING-REGISTRY v0.2 · AGENT7-PROTOCOL-ROSTER · AGENT7-WORKFLOWHUB-MCP-PROPOSAL v0.2 · AGENT7-MCP-I18N-RESOURCE v0.2 · AGENT7-W1-MCP-TRANSPORT-PLAN (DONE).

---

## 1. Lingkup & definisi selesai

**Lingkup:** isi permukaan MCP di atas transport (W1-MCP-TRANS): 5 tools authoring + JIT resource routing (9 URI `n8n://` + 4 URI `hub://` per REGISTRY v0.2) + pemetaan hasil/error terstruktur + label taint + paket konten ≤500 token. Transport & router sudah ada; tugas ini mengisi `facade` + `registry` + handler tipis.

**DoD saat freeze dibuka:**
1. 5 tools (agent1 spec): `inspect_workflow`, `patch_node`, `validate`, `get_receipt`, `preflight` — lolos skema JSON Schema 2020-12 & panggilan nyata ke mock facade.
2. JIT routing: tiap URI ter-resolve via registri (hub_id↔URI), payload ≤500 token + metadata `content_version`/`i18n_rev`; receipt-outdated flag (H-7); `?lang=` fallback deterministik (I-1…I-6).
3. Hasil validasi = DiagnosticReport terstruktur (E-*/W-* final F-6 + MIG-*/PARTIAL/DEV-*), autofix hint, `isError` benar (SEP-1303).
4. Taint: respons berisi data eksternal berlabel TAINTED-EXTERNAL; tak pernah tercuci (agent1 #490).
5. Uji negatif: eksekusi/fetch/nilai kredensial → tidak tersedia/ditolak (MCP-11/H-3); est_token tanpa fetch (H-3b).
6. Redaksi oracle #394 di facade (A7-6).

## 2. Permukaan tools (kontrak input/output; pemilik konten = agent1)

| Tool | Service | Input (ringkas) | Output sukses | Output gagal |
|---|---|---|---|---|
| `inspect_workflow` | WorkflowService | `{workflow_id}` | topo-map `nama:tipe->[succ]` + ≤1 baris/node; taint_flags | isError + kode |
| `patch_node` | WorkflowService | `{name, ops: RFC6902[]}` | `{applied:true}` + receipt ringkas | `{applied:false, diagnostics[]}` |
| `validate` | ValidateService | `{workflow_id?, patch?}` | `{valid:true, warnings[]?}` | `{valid:false, diagnostics[]}` |
| `get_receipt` | ReceiptService | `{workflow_id}` | receipt Rosetta ringkas | kode not-found |
| `preflight` | ValidateService (perluasan — agent1 #503) | `{workflow_id}` | estimasi eksternal + kredensial-butuh (refs only) + fanout | isError; **tanpa nilai kredensial** |

Aturan lintas tools: nama node dirender escaped sebagai DATA (T-2, tak pernah instruksi); kredensial = refs only (F-9/D-6); taint dari input workflow dipertahankan di output (P-4).

## 3. Pemetaan error (final, F-6 + registri agent1)

| Kelas | Kode | Perilaku |
|---|---|---|
| ERROR struktural | `E-REF-DANGLING`, `E-DAG-CYCLE`, `E-DUP-NODE-NAME`, `E-INGRESS-COLLISION` (delegasi agent9 #558/#559), `E-HUB-INPUT-OVERSIZE` (RESERVED), dll. | hard reject (agent1 registri E-*) |
| ERROR expression | `E-EXPR-REFERENCE`, `E-EXPR-SCOPE`, `E-EXPR-CREDENTIAL` | hard reject (F-6 final) |
| WARNING expression | `W-EXPR-FLOAT-PRECISION`, `W-EXPR-SURROGATE`, `W-EXPR-OBJECT-ORDER`, `W-EXPR-UNDEFINED-NULL`, `W-EXPR-ARRAY-METHODS` | lolos + tercatat di receipt (F-6: 5 W) |
| Rosetta | `MIG-*`, `PARTIAL`, `UNKNOWN` | per receipt impor (agent4) |
| Katalog | `DEV-*` | link by ID ke DEVIATION-CATALOG (agent5) |

Format: `{severity, code, node, path, message, hint, autofix|null}` — dibungkus content terstruktur + ringkasan ≤1 baris/node (kompresi agent1 §3).

## 4. JIT resource routing (registri → resolve → paket)

```
resources/read {uri: "n8n://templates/{id}?lang=id"}
  1. resolve template URI via REGISTRY (pola URI + owner + format kemasan)
  2. ambil konten sumber (agent4 manifest / Rosetta) → paket ≤500 token + metadata
  3. ?lang= : lookup workflow_i18n (agent2) → terjemahan ATAU fallback source_lang + flag (I-3)
  4. metadata: content_version (manifest) / i18n_rev (terjemahan) / updated_at / source_ref
  5. hub://: resolve hub_id↔URI → manifest/receipt; receipt-outdated flag (content_version < manifest → "re-execute needed") (H-7)
  6. redaksi (oracle) → respons
```

| URI namespace | Pemilik konten | Catatan khusus |
|---|---|---|
| `n8n://nodes/{name}/schema`, `n8n://nodes/{type}/alias`, `n8n://nodes/cron/mapping` | agent4 | schema compiler (K-1..K-4 pending) |
| `n8n://expression-rules` | agent3 | konten final 3E+5W |
| `n8n://errors/{code}` | agent1 (E-*), agent4 (MIG-*) | lookup dinamis |
| `n8n://deviation/{DEV-*}` | agent5 | link katalog |
| `n8n://limits` | agent2 — **owner dikonfirmasi #594** | allowlist `$env`, budget, retensi |
| `n8n://workflow/{id}/receipt`, `n8n://templates/{id}(?lang)` | agent4 (+i18n agent2) | corpus + workflow_i18n |
| `hub://skills/{hub_id}/manifest|receipt(?lang)` | agent4 (+agent2) | kanon #556; review_status live |
| `hub://intel-types` | agent9 | katalog konektor (PUBLIC-DATA-CONNECTORS) |

Semua payload ≤500 token inti + pointer; token diukur tokenizer #419a (menunggu matt). URI tak dikenal → error standar + (bila lang salah) daftar bahasa tersedia.

## 5. Alur validasi (loop perbaikan — AGENT4 §4)

`patch_node` → `validate` → diagnostik terstruktur (E/W) → klien perbaiki → ulang sampai bersih → `get_receipt` (receipt Rosetta tersimpan; resource receipt siap dibaca agent lain). Gate A7-5: 20 skenario sintetis (kode benar 20/20, ≤3 iterasi).

**Staleness receipt — dua kelas (agent3 #610.4):** (a) manifest-level: `receipt.content_version < manifest.content_version` → flag "re-execute needed" (H-7, template Hub); (b) workflow-level: workflow di-patch setelah receipt terakhir → `get_receipt` wajib re-lint/regenerasi (receipt basi = di-flag; tidak pernah menyajikan receipt lama tanpa penanda).

## 6. Taint & keamanan di tools (ringkas)

- Level taint (adopsi agent3 #610.3): `{EXTERNAL | SANITIZED | INTERNAL}` — output Hub/fetch/web = EXTERNAL (default; dilarang alir ke kredensial/partials); sanitasi pipeline agent5 menaikkan EXTERNAL→SANITIZED; data engine = INTERNAL. Label di metadata respons; invariant taint (agent1 #490): taint TIDAK tercuci antar-tools — bila Hub output mengalir ke Code node → HTTP Request node, node turunannya tetap berlabel tainted sampai sanitasi eksplisit.
- `preflight` hanya refs kredensial; allowlist `$env` dari `n8n://limits` (D-6).
- Nama node/workflow asing = data escaped (T-2).
- Redaksi oracle #394 di batas facade (sekali, bukan per-handler) — A7-6.

## 7. Checklist verifikasi (dijalankan saat freeze dibuka)

| # | Item | Gate |
|---|---|---|
| T-1 | 5 tools: skema valid + panggilan mock facade (tanpa engine penuh) | A7-1/MCP-02 |
| T-2 | 9+4 URI resolve; payload ≤500 token; metadata lengkap | A7-2/A7-3/H-1 |
| T-3 | `?lang=id` tersedia → terjemahan; tak tersedia → fallback+flag; 40/40 bahasa | I-1…I-6 |
| T-4 | receipt-outdated flag muncul saat basi (20 kasus) | H-7 |
| T-5 | DiagnosticReport: kode benar 20/20 skenario; ≤3 iterasi | A7-5/KLL-1 |
| T-6 | est_token dari metadata; 0 fetch saat list/inspect | H-3b |
| T-7 | 0 nilai kredensial di seluruh respons (50 sesi jahat) | A7-6 |
| T-8 | Taint TAINTED-EXTERNAL utuh (tak tercuci) 50 skenario berstrata | H-4 |

## 8. Catatan untuk Wave 2+ & PRD-3

- `hub://` dan `?lang=` aktif saat W2-HUB-CATALOG (agent4/agent9) & storage i18n (agent2) tersedia; antarmuka di sini sudah siap (mock di Wave 1).
- Permukaan konstan: tidak ada tool per-skill (keputusan Hub); eksekusi tetap API terpisah.
- Delegasi namespace: E-* milik agent1 (registri), MIG-*/PARTIAL milik agent4, DEV-* milik agent5, HUB-* milik manifest (agent4) — konsisten pola #559.

## 9. Artefak terkait

AGENT7-MCP-INTEGRATION-SPEC v0.4 · AGENT7-JIT-ROUTING-REGISTRY v0.2 · AGENT7-WORKFLOWHUB-MCP-PROPOSAL v0.2 · AGENT7-MCP-I18N-RESOURCE v0.2 · AGENT7-W1-MCP-TRANSPORT-PLAN (DONE, 98d0f827) · AGENT1-MCP-SKILL-SPEC (owner tools) · AGENT4-MCP-KNOWLEDGE-SPEC · AGENT3-EXPRESSION-RULES-MCP.

*Ditulis oleh agent7 (ROLE_AI_MCP) untuk task W1-MCP-TOOLS. Review dipersilakan (D-7).*
