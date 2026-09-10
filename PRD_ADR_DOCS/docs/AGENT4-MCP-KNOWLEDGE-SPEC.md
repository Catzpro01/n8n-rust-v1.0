# MCP KNOWLEDGE & LINT LAYER (KLL) — spesifikasi agent4 untuk mandat #405/#411

Status: dokumen (freeze #386; pilar PRD-3 per instruksi fern #411 "@agent1 @agent4 @matt").
Komplemen AGENT1-MCP-SKILL-SPEC.md (#413): agent1 = transport/tools/capsule sisi-engine;
dokumen ini = LAPISAN PENGETAHUAN (resources JIT) + SEMANTIK VALIDASI/LINT yang memberi AI
"tahu semua fungsi, cara merangkai, semua fitur" (#411) dan feedback error terstruktur (#405).

## 1. Prinsip
1. Satu MCP server (agent1). KLL = konten + kontrak hasil-validasi, bukan server kedua.
2. Pengetahuan = data TERVERIFIKASI yang sudah dimiliki tim (bukan manual baru):
   registri alias-deprecation (V-1/V-2 verified #264), katalog deviasi (agent5), spesifikasi
   cron parser (agent4), schema compiler v0.2, korpus 171 + receipt.
3. Token hemat: resources pull-only JIT (AI load saat butuh); tool hasil = kompresi semantik.
4. Rosetta & lint berbagi mesin: lint = Rosetta yang dipanggil per-edit, hasilnya = daftar
   diagnostic TERSTRUKTUR + receipt (instance katalog by deviation_id).

## 2. RESOURCES (pull-only, JIT — usulan konten agent4)
CATATAN 2026-09-09: daftar URI resmi & 1-owner-per-URI kini dipegang AGENT7-JIT-ROUTING-REGISTRY.md
(9 URI; 5 milik agent4: schema, alias, cron/mapping, workflow/receipt, templates). Dokumen ini hanya
memegang KONTRAK SEMANTIK + pengemasan; jangan duplikasi daftar URI di sini (KLL-4).
| URI | Isi | Sumber terverifikasi |
|---|---|---|
| n8n://nodes/{type}/alias | status deprecation + aturan konversi + referensi bukti | AGENT4-NODE-ALIAS-DEPRECATION.md (V-1/V-2) |
| n8n://nodes/cron/mapping | tabel mode cron v1 -> ScheduleTrigger v2 + default D1-D4 | AGENT4-CRON-PARSER-SPEC.md |
| n8n://deviation/{DEV-Ax} | entri katalog: kelas, alasan, contoh, referensi silang | DEVIATION-CATALOG.md (agent5) |
| n8n://expression-rules | aturan ekspresi: kanon N-rules + divergensi K2/K3/K6 (kanonik: agent1 spec; konten: agent3 tabel 215 kasus; pemilik kurasi: agent3) | normalizer N-01..N-22 + catalog §6 + AGENT3 expression-edge-cases |
| n8n://schema/{nodeType} | field/parameter schema (partial/defer DEV-A5) | schema compiler v0.2 (K-1..K-4) |
| n8n://templates/{id} | recipe: struktur + verdict determinisme/coverage | korpus 171 + manifest |
| n8n://workflow/{id}/receipt | receipt terakhir (migrasi + determinisme) | Rosetta + Determinism Contract |

Aturan pengemasan: tiap resource <= 500 token inti (ringkas), detail panjang via pointer file.

## 3. TOOL VALIDASI (semantik; hidup DI BAWAH tool agent1)
validate_workflow(workflow|patch) -> DiagnosticReport:
```json
{
  "valid": false,
  "errors": [{"code": "E-REF-DANGLING", "node": "http-2", "path": "/connections",
              "fix_hint": "hubungkan ke node sebelum-nya (Set)"}],
  "warnings": [{"code": "DEV-A2", "deviation_id": "DEV-A2", "node": "code-1",
                "detail": "function + require() npm -> gagal eksplisit, bukan beda perilaku"}],
  "receipt_summary": {"rules": 0, "assumptions": 0, "opaque": 1}
}
```
Kode diagnostic = alfabet stabil: E-* (error struktural), MIG-* (akan dimigrasi saat impor),
PARTIAL (field diabaikan, DEV-A5), UNKNOWN (node opaque), DEV-* (link katalog by ID).
Feedback ini yang diminta #405: LLM salah edit -> dapat error terstruktur, bisa loop perbaiki.

## 4. CONTOH ALUR (AI edit workflow lama)
1. AI load resource n8n://nodes/cron/mapping (JIT) -> tahu cron v1 -> ScheduleTrigger v2.
2. AI patch node cron (via JSON-patch atomik agent2/agent1) -> validate_workflow dipanggil.
3. Report: warning DEV-A4 (detik acak dibuang, detik=0 eksplisit; menunggu keputusan matt) +
   MIG-1 (typeVersion 1->2). AI perbaiki; loop sampai valid.
4. Simpan -> impor via Rosetta -> receipt final tersimpan -> resource n8n://workflow/{id}/receipt
   siap dibaca AI lain. Rantai pengetahuan tertutup.

## 5. Efisiensi token (target, falsifiable)
- Capsule mastery: <=500 token (agent1, sudah).
- inspect_workflow: topo-map 1-2% token vs raw JSON (agent1) — KLL menambah kelas kompresi:
  "kelas node + deviasi + verdict determinisme" sbg 1 baris per node.
- 90% manual tidak pernah dimuat: resources pull-only.

## 6. Gate penerimaan (falsifiable, tanpa engine penuh)
- KLL-1: 20 skenario edit (10 valid/10 rusak sintaksis/ref/tipe) -> validate_workflow memberi
  kode benar 20/20 (dievaluasi thd data korpus).
- KLL-2: total token muat capsule+resource utk kasus cron = < 900 token (terukur; tokenizer rujukan menyusul keputusan agent1 #419).
- KLL-4: tidak ada resource duplikat dgn AGENT1-MCP-SKILL-SPEC (cek URI 1:1).
- KLL-3: tiap DEV-* di report bisa di-resolve ke entri katalog (link by ID, 1 sumber kebenaran).

## 7. Batas & ketergantungan
- Bergantung AGENT1-MCP-SKILL-SPEC (transport, capsule, 5 tools) — KLL mengisi resources + schema
  diagnostic; tidak menduplikasi tool.
- Skema field resource n8n://schema menunggu K-1..K-4 schema compiler (matt).
- Data korpus "recipes" versi publik: hindari memuat template utuh; cukup struktur + pola.
