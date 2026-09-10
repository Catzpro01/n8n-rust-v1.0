# SPESIFIKASI PARSER & MIGRASI cron v1 -> ScheduleTrigger v2 (draf agent4)

Status: dokumen (freeze #386: boleh lanjut sbg spesifikasi normalizer, bukan kode).
Dasar bukti: 13 file cron nyata di korpus + verifikasi upstream V-2 (#264) + respon #351/#384.
Posisi: aturan v1 dari WORKFLOW ROSETTA (docs/AGENT4-WORKFLOW-ROSETTA.md §4.1).

## 1. Bentuk nyata di korpus (13/13 file = {"triggerTimes":{"item":[...]}})

Semua file cron v1 di korpus: parameter = triggerTimes (fixedCollection item). Mode teramati:
| # | Bentuk item nyata | File contoh |
|---|---|---|
| 1 | {"mode":"everyMinute"} | tpl-199 |
| 2 | {"mode":"everyHour"} (tanpa argumen) | tpl-159 |
| 3 | {"mode":"everyX","unit":"minutes","value":5} | tpl-1471, tpl-1841 |
| 4 | {"mode":"custom","cronExpression":"* */6 * * *"} | tpl-175 |
| 5 | {"hour":7} — TANPA mode (default tersirat) | tpl-1771 |
| 6 | item lain dgn mode everyDay/everyWeek/everyMonth + hour/minute/dayOfMonth | (cek: tpl dgn mode lain; lengkapi saat fixtures) |
| 7 | multi-item array | (belum teramati di 13; ScheduleTrigger v2 mendukung multi -> K-3) |

## 2. Aturan parser (AMAN, tanpa eval/exec)

1. Tokenizer subset: terima HANYA struktur JSON yg sudah dikenal: key mode/unit/value/hour/minute/
   weekday/dayOfMonth/cronExpression. Nilai di luar skema -> tolak dgn pesan + receipt warning.
2. mode tidak dikenal / struktur invalid -> node TIDAK dimigrasi diam-diam: workflow ditandai
   IMPOR-BERHASIL + node opaque + deviation_id (DEV-...; kasus ini tercatat, bukan gagal total).
3. cronExpression custom: WAJIB valid 5-field (split spasi, tiap field = angka/step/range/list/*;
   tanpa field detik). Valid -> pass-through ke crate cron Rust (parse ulang saat runtime).
   Invalid -> tolak dgn pesan eksplisit.
4. Ekspresi jamak 6-field (dengan detik) dari sumber lain: buang detik + catat di receipt
   (asumsi: detik acak n8n TIDAK ditiru; detik=0 eksplisit — lihat Q4/KEPUTUSAN matt).

## 3. Mapping migrasi cron v1 -> ScheduleTrigger v2

| cron v1 item | ScheduleTrigger v2 rule item |
|---|---|
| {"mode":"everyMinute"} | {field:"minutesInterval", minutesInterval:1} |
| {"mode":"everyHour"} | {field:"hoursInterval", hoursInterval:1, (menit: 0 default — Q1/K-2)} |
| {"mode":"everyX","unit":"minutes","value":v} | {field:"minutesInterval", minutesInterval:v} |
| {"mode":"everyX","unit":"hours","value":v} | {field:"hoursInterval", hoursInterval:v} |
| {"mode":"everyDay","hour":h,"minute":m} | {field:"daysInterval", daysInterval:1, hour:h, minute:m} |
| {"mode":"everyWeek","weekday":d,...} | {field:"weeksInterval", weeksInterval:1, weekday:d, ...} |
| {"mode":"everyMonth","dayOfMonth":x,...} | {field:"monthsInterval", monthsInterval:1, dayOfMonth:x, ...} |
| {"mode":"custom","cronExpression":"5-field"} | {field:"cronExpression", expression:"5-field"} |
| {"hour":7} TANPA mode | default tersirat (kemungkinan everyDay hour=7) — WAJIB asumsi tercatat di receipt + keputusan explicit (lihat §4 D1) |

## 4. Keputusan default yang DIBUTUHKAN (tidak bisa diputuskan parser sendiri)

- D1: item tanpa mode (tpl-1771: {"hour":7}) -> asumsi "everyDay hour=7"? Verifikasi historis:
  cron v1 UI lama punya pilihan Basic/Advanced; bentuk {"hour":7} kemungkinan artefak versi
  sangat lama. Keputusan: asumsi + receipt, atau tolak. Usul: asumsi + receipt (konsisten
  prinsip Rosetta: tidak senyap).
- D2: menit untuk everyHour / jam untuk everyDay bila field tidak ada (default 0?) — K-2,
  menunggu PRD-3.
- D3: kategori deviasi N8N-NONDETERMINISTIC utk detik acak toCronExpression (usul agent5 #379,
  Q4 dokumen Rosetta) — menunggu matt.
- D4: aturan weekday numerik cron (0-6: 0=Minggu) vs ScheduleTrigger weekday — verifikasi
  kesetaraan saat fixtures diuji; jangan asumsi.

## 5. Fixtures & gate diff-test

- Fixture: 13 file tpl (tpl-159,175,199,1471,1771,1841,... daftar penuh dari korpus) + tpl-2371
  (scheduleTrigger cronExpression, jalur berbeda).
- Gate: migrasi dijalankan; hasil = ScheduleTrigger dgn rule item setara; verifikasi ekivalensi
  jadwal (detik=0 vs n8n detik acak: bandingkan HARI/JAM/MENIT pemicu, bukan detik) — karena
  target n8n nondeterministik di detik, gate memakai jadwal minute-granular, bukan byte output.
- Semua kasus dijalankan via normalizer harness (agent1 diff_harness v0.3, siap #357) saat mode
  deterministik tersedia (Fase 1a).

## 6. Batas (jujur) — diperbarui 2026-09-09 (verifikasi upstream SELESAI)

- VERIFIKASI TUNTAS (Schedule/GenericFunctions.ts, di-cache /home/agent4/upstream/):
  baris 170 `if (interval.field === 'cronExpression') return interval.expression;` ->
  ScheduleTrigger v2 cronExpression DIKEMBALIKAN APA ADANYA, TANPA suntikan detik acak.
  Interval ter-generasi (minutes/hours/days/dst) memakai `stableInt(nodeKey,...)` =
  jitter STABIL per (workflowId:nodeId) — DETERMINISTIK lintas run, bukan random.
  Kontras: Cron v1 (workflow/src/cron.ts) memakai `randomInt(60)` SETIAP RUN utk mode
  ter-generasi; custom cron v1 juga as-is (deterministik).
  => Kelas N8N-NONDETERMINISTIC hanya berlaku utk Cron v1 mode ter-generasi; jalur
  ScheduleTrigger v2 (termasuk tpl-2371) DETERMINISTIK — tidak perlu detik=0 paksa.
- Custom cron v1 5-field (mis. tpl-175 "* */6 * * *") juga as-is -> deterministik.
- Item cron v1 TANPA mode (tpl-1771 {"hour":7}): di cron.ts jatuh ke return
  item.cronExpression.trim() -> kemungkinan error runtime n8n asli; konfirmasi saat n8n
  live; keputusan D1 (asumsi + receipt) tetap berlaku.
- 13 file = batas bawah variasi; mode everyWeek/everyMonth/multi-item belum teramati di korpus
  tapi didukung upstream (K-3) — masukkan ke fixtures sintetis terverifikasi manual.
