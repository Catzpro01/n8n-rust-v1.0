# MCP SKILL SPEC — sisi engine (agent1)

Status: draf pilar PRD-3, menjawab mandat #411 (pemilik proyek via fern).
Cakupan agent1: capsule-core, tools, budget token, gate. Skema node =
agent4 (schema compiler); perakitan PRD-3 = matt; receipt = Rosetta.
Dokumen, bukan kode (patuh freeze #386).

## 1. Engine Mastery Capsule (<500 token, passive skill)

Disuntik via MCP `instructions` + `prompts/get` saat connect. Isi padat,
lima blok berurutan (draf teks final di §1.1–1.5, dihitung ≤500 token
dengan tokenizer rujukan PRD-3):

1.  **DAG** — workflow = simpul bernama + koneksi bernama; alamat apa pun
    by-NAMA, tak pernah by-indeks/koordinat; posisi kanvas milik server.
2.  **EXPR** — `{{ }}` QuickJS-subset; `$json` item-kini; `$('Nama')` simpul
    lain; `$now` beku-per-task; `$env` allowlist; larangan: eval/Function/
    require/import/network/fs (diagnostik E_EXPR_FORBIDDEN).
3.  **PATCH** — ubah via JSON-Patch RFC 6902 per-node; respons selalu
    {applied | rejected+diagnostics}; kredensial = refs, tak pernah nilai.
4.  **VALIDATE** — tiap patch WAJIB validate sebelum simpan; loop sampai
    diagnostik kosong; terima receipt bila migrasi (Rosetta).
5.  **ERRORS** — kode E_NODE_REF/E_CONN_DANGLING/E_EXPR_* /E_CRED_VALUE +
    satu-baris arti + autofix? (ya/tidak).

## 2. JIT knowledge via MCP resources (hemat token ~90%)

| URI | Isi | Saat di-load |
|---|---|---|
| `n8n://nodes/{name}/schema` | parameter + versi + eval_scope (agent4) | patch/get simpul itu |
| `n8n://recipes/{pattern}` | pola-rangkaian teruji (branch/merge/batch/webhook-reply) | tugas cocok-pola |
| `n8n://expression-rules` | tabel K1-K10 ringkas (jebakan antar-runtime) | diagnostik expr gagal |
| `n8n://errors/{code}` | arti + contoh-betul + autofix | kode error tak dikenal AI |
| `n8n://limits` | budget 500MB/gate/guardrail eksekusi | preflight estimasi |

Aturan: server TAK PERNAH push resource tanpa diminta; AI pull per-URI.
Resep diindeks pola-tugas, bukan daftarButa.

## 3. Context-compact tools (permukaan minimal)

| Tool | Input | Output (padat) |
|---|---|---|
| `inspect_workflow` | id | topo-map: `nama:tipe -> [succ]` + ringkasan per-node ≤1 baris; target ≤2% token JSON-mentah |
| `patch_node` | nama + RFC6902 ops | applied \| rejected+diagnostics (§4) |
| `validate` | id (+patch kering) | bersih \| daftar-diagnostik |
| `get_receipt` | id | receipt Rosetta ringkas (node-diubah + aturan + deviation_id) |
| `preflight` | id | eksternal-API + kredensial-butuh + estimasi-fanout (proposal #365B) |

Larangan permukaan: baca/tulis kredensial-nilai; tulis posisi kanvas;
eksekusi-workflow dari MCP (eksekusi = API terpisah berautentikasi).
INVARIAN TAINT (#489): flag `tainted` pada data-workflow-asing diteruskan
utuh melintasi rantai tool (inspect→patch→validate); tak ada tool yang
mencuci-taint menjadi clean — batas-kepercayaan hanya dilewati eksplisit.

## 4. Diagnostik mesin-terbaca (kontrak Preflight-linter)

`{severity, code, node, path, message, hint, autofix: patch-ops | null}`.
`severity`: ERROR = hard-reject (ref-gantung, sintaks, DAG-siklik,
akses-nilai-kredensial); WARNING = lolos-dengan-catatan-eksplisit di
execution receipt (presisi-float, surrogate, ordering — keputusan fern
#431). Kode stabil antar-rilis (tambah-boleh, ubah-arti-dilarang —
disiplin versioning event-log). Ambigu = REJECT + `need_clarification`,
bukan tebak.

Namespace E-* dimiliki dokumen ini (kesepakatan #422/#438); DEV-* milik
DEVIATION-CATALOG agent5; MIG-*/PARTIAL/UNKNOWN milik Rosetta/KLL agent4.
Kode awal dari jebakan-AI agent3 (#419, format hyphen per #443):
E-EXPR-REFERENCE (ref by-nama di-enforce di validate, bukan sekadar
capsule), E-EXPR-SCOPE (ekspresi item-field di node trigger = cacat
TOPOLOGI, bukan sintaks — `$input.first()` adalah alias `$json` sehingga
bukan perbaikan; autofix = pindah-ekspresi-ke-node-berikut ATAU
properti-trigger-saja), E-EXPR-CREDENTIAL (akses nilai-kredensial via
expression = HARD-REJECT, bukan warning).

### 4.1 Format resource `n8n://errors/{code}` (kontrak registri agent7)

Payload mengikuti kontrak-kemasan JIT-REGISTRY §3 (metadata wajib +
`content_version`, payload ≤500 token, pointer-file utk detail).
`content_version` E-registry = 1.

```json
{ "uri": "n8n://errors/E-EXPR-SCOPE",
  "content_owner": "agent1", "content_version": 1,
  "source_ref": "AGENT1-MCP-SKILL-SPEC.md §4",
  "budget_tokens": 0, "updated_at": "2026-09-09",
  "body": { "code": "E-EXPR-SCOPE", "severity": "ERROR",
    "meaning": "ekspresi item-field di node tanpa-input (topologi)",
    "wrong": "trigger: {{ $json.field }}",
    "right": "pindah ke node-2 ATAU properti-trigger-saja",
    "autofix": { "action": "move-expression", "target": "next-node" },
    "test_pair": "kasus-salah DITOLAK / hasil-autofix DITERIMA (#445)" } }
```

Daftar v1 (7 kode + 1 delegasi + 3 struktural): TERDEFINISI — E-EXPR-REFERENCE,
E-EXPR-SCOPE, E-EXPR-CREDENTIAL; STRUKTURAL — E-REF-DANGLING (koneksi menunjuk
node-tak-ada; pair: koneksi-gantung DITOLAK / endpoint-valid DITERIMA),
E-DAG-CYCLE (siklus A->B->A; pair: bersiklus DITOLAK / DAG-valid DITERIMA),
E-DUP-NODE-NAME (dua node se-nama; pair: duplikat DITOLAK / unik DITERIMA)
— ketiganya diaktifkan #607 (Toohs-plan agent7 butuh hard-reject);
TERDEFINISI-DELEGASI — E-INGRESS-COLLISION (semantik milik agent9 #558:
aktivasi-kedua GAGAL HTTP-409 + (method,path)+pemilik-pertama;
tiebreak=urutan-aktivasi; re-register-idempotent=no-op; namespace milik agent1);
RESERVED (aktifkan dgn contoh+test-pair sebelum dipakai server) — E-NODE-REF
(format-ref-tak-valid; beda dari DANGLING), E-EXPR-SYNTAX, E-HUB-INPUT-OVERSIZE,
E-WCB-TRAP / E-WCB-FUEL / E-WCB-TIMEOUT (semantik agent6 #663/#664/#678: trap /
fuel-exhausted / timeout plugin → node-FAILED + routing-onError; namespace agent1).
E-CONN-DANGLING DIPENSIUNKAN (duplikat-makna E-REF-DANGLING; jangan-mint-lagi). `budget_tokens` = 0 = BELUM-UKUR (menunggu tokenizer
rujukan #419a/#438); server dilarang serve kode RESERVED
(resources/read → error standar per registri §2).
Kepemilikan (§3.6 registri, #479/#481): owner-tunggal URI = agent1;
konten MIG-*/PARTIAL/UNKNOWN = DELEGASI-RESMI ke agent4 (sumber:
Rosetta/KLL); tanggung-jawab `content_version` tetap di agent1.

## 5. Budget & gate (satuan eksplisit, aturan ERR-029)

| Metrik | Batas | Satuan | Cara ukur |
|---|---|---|---|
| Capsule | ≤500 | token (tokenizer rujukan) | hitung saat rilis; gagal-bila-lebih |
| inspect vs mentah | ≤2% | token/token per workflow | median atas 171 korpus |
| patch-atomik | 171/171 | workflow-tak-rusak | patch-1-node + impor-ulang + preflight-bersih |
| patch-salah | 50/50 | penolakan-tepat-node | 50 kasus sintetis |
| redaksi kredensial | 100% | nilai-tak-bocor | semua respons MCP discan |
| RAM MCP persisten | 0 | byte | handler stateless; ukur RSS idle ±MCP (standar #431: 0-persisten, transien O(workflow)/request dibebaskan usai respons) |
| CPU validate | O(V+E) | operasi/node+edge | bench atas workflow terbesar korpus |

## 6. Pembagian kerja pilar (usulan)

- agent1: capsule-core §1, tools §3, diagnostik §4, gate §5.
- agent4: `nodes/{name}/schema` (schema compiler), receipt ringkas.
- agent3: `expression-rules` (suite 215 kasus → tabel ringkas).
- agent2: `limits` (budget/guardrail) + retensi log-MCP — DIKONFIRMASI #588.
- agent5: redaksi-kredensial + ancaman-model MCP (tool-injection, prompt-
  injection via workflow-asing) + kriteria lulus keamanan.
- matt: rakitan PRD-3 + keputusan tokenizer-rujukan + autentikasi MCP.
