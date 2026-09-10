# W1-ROSETTA-PARSE — Rencana Wave-1: Compiler Rosetta (paritas 694 node + StickyNote)

**Pemilik:** agent4 (ROLE_SCHEMA) · **Wave 1 · P0 · Status: DELIVERED via swarm-task**
**Patuh freeze #386:** dokumen/rencana — nol kode. Implementasi = setelah PRD-3 disahkan.

## 1. Cakupan tugas (dari queue)

"Compiler Rosetta untuk paritas 694 nodes dan StickyNote" — fase WAVE-1 = spesifikasi &
rencana modul (belum kode). Deliverable = dokumen spesifikasi + pemetaan modul + acceptance.

## 2. Deliverable spesifikasi (sudah ada, dirujuk bukan diduplikasi)

| Modul (implementasi post-freeze) | Spesifikasi | Status |
|---|---|---|
| Parser workflow (node era mana pun, opaque-safe) | AGENT4-WORKFLOW-ROSETTA.md §2 | FINAL-draf (v1.1) |
| Rule registry migrasi + prosedur masuk (ADR + diff-test) | AGENT4-WORKFLOW-ROSETTA.md §5, §4.1-4.3 | v1 rules siap diverifikasi |
| Migration receipt (schema JSON, deviation_id wajib) | AGENT4-WORKFLOW-ROSETTA.md §3 | FINAL-draf |
| Normalisasi kanon utk diff-test | AGENT4-NORMALIZER-DESIGN.md + N-01..N-22 | FINAL-draf |
| Kompilasi schema -> tipe (kanon Rust) | AGENT4_SCHEMA_COMPILER_SPEC.md | FINAL-draf (deviasi DEV-A5 deferred) |
| 694-node catalog + alias/deprecation | AGENT4-NODE-ALIAS-DEPRECATION.md + corpus coverage | TERVERIFIKASI (penerimaan katalog = OPEN #418) |
| StickyNote parse + note-binding deterministik | AGENT4-WORKFLOW-ROSETTA.md §4.4 (baru) | FINAL-draf |
| Kait Hub template import | AGENT4-WORKFLOW-ROSETTA.md §4.5 + AGENT4-WORKFLOW-HUB-MANIFEST.md | FINAL-draf (v0.4) |

## 3. Pemetaan modul kode (post-freeze; struktur kasar, bukan implementasi)

```
rosetta/
  parse/        # workflow JSON -> grafik kanon; stickyNote dipertahankan + index posisi
  registry/     # aturan migrasi berversi (cron-v1->schedule, function->code, ...)
  migrate/      # penerap aturan -> workflow bersih + receipt {rules, assumptions, deviations}
  binding/      # note-binding geometri deterministik (cover-test) + override manifest
  kanon/        # normalisasi output utk diff-test (N-01..N-22, normalizer agent4)
test/
  fixtures/     # dari korpus 171 (tpl-159,175,199,1471,1771,1841 utk cron; dst.)
  diffgate/     # jalankan asli-vs-migrasi, bandingkan kanon (mode deterministik)
```

## 4. Acceptance W1 (falsifiable, satuan eksplisit)

- R-1: parse 171/171 file korpus tanpa gagal; 0 file ter-mutasi (hash asli utuh).
- R-2: tipe tak dikenal = opaque + masuk receipt (tidak menghentikan impor) — 100% kasus.
- R-3: stickyNote 46% template: binding deterministik (2× hitung = identik); coverage
  tercatat per template (HUB-7).
- R-4: tiap aturan migrasi v1 punya rule_version + fixture korpus + diff-test lolos.
- R-5: receipt memakai deviation_id dari DEVIATION-CATALOG (agent5) — tidak ada ID lokal.

## 5. Kait lintas-agent

- agent1 (engine): workflow bersih + receipt = input eksekusi; record-replay memakai receipt.
- agent2 (storage L0): penyimpanan workflow asli + cache migrasi + receipt (identitas = hash).
- agent5 (QA): DEVIATION-CATALOG + diff-test harness determinisme.
- agent7 (MCP): resource n8n://templates/{id}/receipt + stickyNote notes (i18n via agent2).
- agent9 (integration): impor template Hub = jalur sama; node deprecated -> deviation_id.

## 6. Riwayat

v0.1 2026-09-09: dibuat utk klaim swarm-task W1-ROSETTA-PARSE (agent4).
