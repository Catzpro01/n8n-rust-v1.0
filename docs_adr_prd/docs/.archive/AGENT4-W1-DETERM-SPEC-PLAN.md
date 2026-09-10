# W1-DETERM-SPEC — Rencana Wave-1: Spesifikasi & Kontrak Determinisme Eksekusi

**Pemilik:** agent4 (ROLE_SCHEMA) · **Wave 1 · P0 · Status: DELIVERED via swarm-task**
**Patuh freeze #386:** dokumen/rencana — nol kode. Implementasi = setelah PRD-3 disahkan.

## 1. Cakupan tugas (dari queue)

"Spesifikasi dan Kontrak Determinisme Eksekusi" — spesifikasi kontrak determinisme yang
dibuat saat impor (statis, 0 biaya runtime), ditegakkan saat eksekusi, dilaporkan ke
receipt. Diterima pemilik sbg inovasi (fern #399, wave-2); ditag W1 utk kelengkapan rencana.

## 2. Deliverable

| Artefak | Status |
|---|---|
| AGENT4-DETERMINISM-CONTRACT-SPEC.md v1.1 — DETERMINISM REPORT schema, kelas sumber (agent5), 3 mode eksekusi + degradasi jujur, gates G1-G3, §7 konsumen Hub | DITERIMA #399 + v1.1 (07:1x, sha e20cc2f3) |
| DETERMINISM-SOURCE-INVENTORY.md (agent5) = data kelas sumber; DEVIATION-CATALOG §6 K1-K10 = aturan antar-runtime | rujukan (bukan milik agent4) |
| Verifikasi upstream cron: ScheduleTrigger v2 deterministik; cron v1 generated-mode nondeterministik → N8N-NONDETERMINISTIC dipersempit (Rosetta §7 Q4) | TERVIRIFIKASI 2026-09-09 |
| AGENT4-CRON-PARSER-SPEC.md — jalur v2 parser deterministik 5-field | selesai (kronologi #435) |

## 3. Modul implementasi (post-freeze, struktural)

```
determinism/
  analyze/     # impor-time: scan statis sumber nondeterminisme (kelas E/clock/cron/RNG/...)
               #   -> DETERMINISM REPORT {verdict, sources[], external_points[]}
  mode/        # eksekusi normal (violation log) vs deterministic (seed 8B, jam beku,
               #   urutan iterasi tetap, external record-replay)
  record/      # (template_sha256, content_version, triggerTimes) + eksekusi-dua-versi
               #   live-vs-candidate (OPEN-HUB-1, agent1 #536) -> penegakan #374
```

## 4. Acceptance (falsifiable, satuan eksplisit)

- G1: 7 workflow RUNNABLE subset exec-diff — mode deterministik, 2 run seed sama → output
  kanon identik 7/7; seed beda → beda BOLEH tapi tercatat.
- G2: workflow NON_DETERMINISTIC sintetis (Date.now) → verdict benar; mode deterministik
  ditolak dgn alasan eksplisit.
- G3: Math.random liar di mode deterministik → violation tercatat, bukan divergen senyap.

## 5. Kait lintas-agent

- agent1: determinism-record §6.2.1 + record-replay = mitra kontrak (REPORT = data replay).
- agent3: QuickJS proxy violation-check (1 flag check/panggilan) — hot path 0 byte.
- agent5: inventaris sumber + gates L2/L3.
- agent2: penyimpanan report sbg metadata workflow (L0).
- Rosetta: penghapus sumber (cron random-second → detik=0 eksplisit via migrasi).

## 6. Riwayat

v0.1 2026-09-09: dibuat utk klaim swarm-task W1-DETERM-SPEC (agent4).
