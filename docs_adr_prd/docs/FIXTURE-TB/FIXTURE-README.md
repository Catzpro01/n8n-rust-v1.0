# FIXTURE-TB — fixture tracer-bullet utk TB-02 & TB-03 (agent9, Ruling 63)

**Tugas (matt R63 #1664):** dua workflow nyata hasil ekspor n8n 2.39.0 + golden set minimal 5 ekspresi dengan keluaran terkunci.
**Sumber:** `/opt/agent-workspace/upstream/n8n-2.39.0` — anchor **fcf21f5efc633fa59034893cf10a73ae6c2177cb** (diverifikasi `git rev-parse HEAD`).

## Isi

| File | Isi | Dipakai oleh |
|---|---|---|
| `workflow-a-manual-to-set.json` | WF-A: Manual Trigger → Set (manual mapping, price=21, include:none). Bentuk TB-01 (2 node) dalam FORMAT EKSPOR penuh | TB-02 |
| `workflow-b-set-chain-expressions.json` | WF-B: Manual Trigger → Set(price=21) → Set(ekspresi: total=`{{ $json.price * 2 }}`, label, ternary). Rantai 3 node + ekspresi | TB-02 + TB-03 |
| `expressions-golden.json` | **7 ekspresi** (E1–E7) dengan keluaran TERKUNCI + tipe + konteks evaluasi; semua murni-deterministik (tanpa $now/$random) | TB-02/TB-03 (evaluator) |

## Keluaran terkunci WF-B (test MERAH siap utk TB-03 — BUKAN scope TB-02)

Per Ruling-67 §2: batas resmi = **WF-A wajib hijau di TB-02**; bagian EKSPRESI WF-B **boleh tetap merah** di TB-02 (ekspresi `{{ }}` = lingkup TB-03 — "TB-03 sekarang lahir dengan test merah yang sudah siap"). Item akhir WF-B `{ "total": 42, "label": "n8n-rust", "ok": "yes" }` = kontrak untuk **TB-03**, bukan TB-02. TIKET YANG MEMERINTAH, bukan README ini.

## Provenance format (jujur — bukan klaim "diekspor dari instance hidup")

Tidak ada instance n8n hidup di server; workflow ini adalah **rekonstruksi format-eksak** dari SERIALISER EKSPOR + skema node di source 2.39.0 (setiap field dapat dilacak):

- Assembly ekspor: `packages/cli/src/commands/export/workflow.ts` + `packages/cli/src/workflow-helpers.ts` (pinData/staticData handling) — field payload: `name, nodes, connections, settings` (+ `staticData/pinData/meta/versionId` bila ada; dikosongkan = bentuk minimal sah).
- ManualTrigger: `packages/nodes-base/nodes/ManualTrigger/ManualTrigger.node.ts` — 1 output Main (typeVersion 1, parameters kosong).
- Set V2: `packages/nodes-base/nodes/Set/v2/SetV2.node.ts` — `mode: manual` (default :51), `duplicateItem` (:55), `include: none` (:94, opsi none/all/selected), `assignments.assignments[] {id,name,value,type}`; typeVersion 2.
- Ekspresi dievaluasi evaluator JS n8n: titik panggil `packages/core/src/execution-engine/node-execution-context/node-execution-context.ts` (evaluateExpression).
- id node/assignment = UUID-stabil bentuk v4 yang DITETAPKAN (fixture butuh id deterministik lintas-run; ekspor nyata memakai uuid acak — bentuknya identik, asal-usul angkanya diungkap di sini).

## Cara verifikasi (R63: perintah + argumen)

```bash
cd /opt/agent-workspace/docs/FIXTURE-TB
python3 -m json.tool workflow-a-manual-to-set.json > /dev/null && echo WF-A valid
python3 -m json.tool workflow-b-set-chain-expressions.json > /dev/null && echo WF-B valid
python3 -m json.tool expressions-golden.json > /dev/null && echo GOLDEN valid
sha256sum *.json *.md
```

## Catatan konsumsi TB-02/03

- TB-02 (eksekusi+storage): muat WF-A/WF-B via importer, eksekusi, bandingkan item akhir WF-B terhadap kontrak di atas.
- TB-03 (ekspresi): evaluator harus memenuhi 7/7 kasus E1–E7 (keluaran + tipe eksak).
- Semua kasus bebas kredensial & bebas jaringan — sesuai aturan tracer-bullet.

— agent9 (ROLE_INTEGRATION), 2026-09-09
