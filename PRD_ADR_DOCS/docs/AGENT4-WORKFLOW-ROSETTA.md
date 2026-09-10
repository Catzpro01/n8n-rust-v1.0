# WORKFLOW ROSETTA — Self-Healing Import + Migration Receipt (draf agent4)

Status: usulan sayembara #298 (pesan #341); deliverable W1-ROSETTA-PARSE (queue swarm-task,
2026-09-09). Komponen: rule registry + migration receipt + diff-test gate + stickyNote
note-binding (mandat #547). Riwayat: v1 → v1.1 (tambah §4.4 stickyNote/binding, §8 kaitan Hub/i18n).
Dasar: korpus 171 template nyata, verifikasi upstream V-1/V-2 (docs/AGENT4-NODE-ALIAS-DEPRECATION.md),
schema compiler v0.2 (partial/defer DEV-A5), DEVIATION-CATALOG (agent5), normalizer N-01..N-22.

## 1. Prinsip

1. Kernel TIDAK PERNAH mengimplementasikan node lawas/deprecated. Rosetta = pengganti emulasi:
   konversi terjadi di PINTU MASUK (impor), bukan saat runtime.
2. Tidak ada migrasi senyap: setiap perubahan tercatat di Migration Receipt per workflow.
3. Aturan migrasi = klaim yang harus lolos diff-test: workflow asli (n8n-JS) vs hasil migrasi
   (engine), output dibandingkan lewat kanon normalizer. Tanpa lulus, aturan TIDAK masuk registri.
4. Zero RAM runtime: migrasi impor-only; receipt kecil di metadata workflow.

## 2. Pipeline impor

```
workflow mentah (era mana pun)
   -> 1. PARSE semua node (tipe tak dikenal = node opaque + peringatan, bukan gagal)
   -> 2. MIGRATION PIPELINE: untuk tiap node, cari aturan berversi di registry:
          - typeVersion lama -> baru (intra-tipe, mekanisme n8n yang kita generalisasi)
          - tipe deprecated -> tipe modern (function->code, cron->scheduleTrigger, ...)
          - tipe tak dikenal tanpa aturan -> opaque + DEV-CATALOG, lanjut (tidak dinding)
   -> 3. MIGRATION RECEIPT: {rules_applied[], assumptions[], deviations[], opaque[]}
   -> 4. workflow bersih + receipt disimpan; diff-test gate utk aturan baru
```

## 3. Skema Receipt (draf, JSON kecil di metadata)

```json
{
  "schema": 1,
  "engine_version": "0.x",
  "rules": [
    {"node": "cron-1", "rule": "cron-v1-to-schedule-v2", "rule_version": "1.0",
     "changed": {"type": "n8n-nodes-base.scheduleTrigger", "typeVersion": 2},
     "mapping": "triggerTimes.item[] -> rule.interval[]; mode=custom -> {field:cronExpression}",
     "assumptions": ["item tanpa mode ({\"hour\":7}) -> everyDay hour=7 (default tercatat, tpl-1771)"],
     "deviation_id": "DEV-A4"}
  ],
  "opaque_nodes": [],
  "warnings": ["ekspresi custom '* */6 * * *' dipindah apa adanya (5-field)"]
}
```
Per agent5 (#379): `deviation_id` WAJIB = ID DEVIATION-CATALOG (DEV-A2, DEV-A4, dst); `null` bila migrasi
persis-setara. Receipt = INSTANCE dari katalog; tidak ada dua sumber kebenaran.
Data DEV terkait: require() npm hanya di 1/171 file (tpl-1381, bahan agent1 #310).

## 4. Rule v1 yang siap diverifikasi (berbasis data korpus + upstream)

### 4.1 cron v1 -> ScheduleTrigger v2 (13 file korpus: tpl-159,175,199,1471,1771,1841,...)
Verifikasi upstream (#264): ScheduleTrigger.rule = fixedCollection item; mode unit -> "<unit>Interval";
custom -> item {field: "cronExpression", expression: "5-field"}; detik acak dari toCronExpression DIBUANG.
Tabel mode nyata dari korpus (13/13 file = {"triggerTimes":{"item":[...]}}):
- {"mode":"everyMinute"}                      -> rule [{field:"minutesInterval", minutesInterval:1}]  (cek default)
- {"mode":"everyHour"}                        -> hourInterval 1 (default menit 0; K-2 perlu keputusan)
- {"mode":"everyX","unit":"minutes|hours",...} -> minutesInterval|hoursInterval = value
- {"mode":"custom","cronExpression":"5-field"}-> cronExpression passthrough (tokenizer aman, TANPA eval)
- {"mode":"everyDay"/dst + hour/minute}       -> daysInterval + jam/menit
- TANPA mode ({"hour":7})                     -> asumsi default TERCATAT di receipt (lihat tpl-1771)
- multi-item array                           -> multi interval (K-3)
Parser: tokenizer subset aman (mode/unit/value); ekspresi custom diverifikasi 5-field lalu pass-through
ke crate cron Rust. Dilarang eval/exec.

### 4.2 function / functionItem -> Code v2 (12+9 file korpus)
Verifikasi upstream V-1/V-2 (#264): Code v2 param jsCode; mode runOnceForAllItems (default) /
runOnceForEachItem; pola resmi all-items: `const items = $input.all();`.
- function (per-run, $input.first())  -> Code runOnceForAllItems + prelude $input.all() (V-3)
- functionItem (per-item, $item)      -> Code runOnceForEachItem + prelude $input.item (V-3b butuh uji live)
- deteksi require() paket npm         -> GAGAL EKSPLISIT + receipt (bukan diam-diam beda perilaku)
- helperFunctions/FS dsb.             -> evaluasi per kasus; di luar = opaque + deviasi

### 4.3 tipe deprecated lain (dari korpus)
rssFeedRead, writeBinaryFile/readBinaryFile, editImage, xml, executeCommand, spreadsheetFile,
splitInBatches dsb. -> BUKAN aturan v1; kandidat aturan berikutnya via prosedur masuk (ADR + diff-test).

## 5. Prosedur masuk aturan (gate)

1. ADR singkat (masalah + solusi + alternatif).
2. Fixture: kumpulkan workflow nyata pemakai node tsb dari korpus (atau buat minimal set).
3. Diff-test: jalankan asli vs migrasi di harness (mode deterministik) -> output kanon identik?
4. Catat asumsi/default apa pun yang dipakai (wajib) -> masuk skema receipt.
5. Setelah lolos: aturan + versi didaftarkan; selftest menyertainya.
Aturan TIDAK pernah masuk lewat keputusan diam-diam; tiap aturan punya rule_version untuk audit.

## 6. Posisi di unified stack (usulan agent2 #344 / agent3 #342)

L-1 ROSETTA (agent4) -> L0 STORAGE (agent2) -> L1 CASD (agent3) -> L2 TIMELINE (agent1) -> L3 ENVELOPE (agent5).
Catatan penilaian (#343): determinism substrate = prasyarat lintas entri (Fase 1a agent5 #346);
Rosetta jalan sejak hari pertama (impor + receipt) dan diff-test-nya memaksa determinisme.

## 4.4 StickyNote: parse + note-binding (mandat #547, masuk W1-ROSETTA-PARSE)

stickyNote = node kanvas dekoratif n8n (tipe `n8n-nodes-base.stickyNote`, muncul di 46%
template — temuan matt). TIDAK dieksekusi; isi = teks markdown + posisi/size absolut.

1. PARSE (sudah cakupan Rosetta): konten, width/height, position (x,y) diekstrak ke kanon.
   stickyNote lama TANPA konten deskriptif tetap di-parse — konten tidak dinilai/dimutasi.
2. NOTE-BINDING = index deterministik: node kerja yg bounding-box-nya TER-COVER kotak
   stickyNote → binding ke stickyNote tsb. Aturan murni geometri + urutan deterministik
   (tanpa wall-clock/random — konsisten determinism contract); overlap ambigu →
   ditandai unbound + siap override manual.
3. OVERRIDE: binding manual opsional lewat manifest `documentation.notes_binding`
   (AGENT4-WORKFLOW-HUB-MANIFEST v0.4) utk kasus ambigu — override MENANG atas geometri.
4. OUTPUT (konsumsi #547): (a) UI self-doc: catatan per node utk kanvas; (b) i18n: teks
   stickyNote = bahan sumber bahasa (source_lang) utk tabel workflow_i18n agent2 — Rosetta
   TIDAK menerjemahkan, hanya menyediakan sumber + binding; (c) Hub: pilar documentation{}
   manifest merujuk binding ini. Exposure via MCP = n8n://templates/{id}?lang (agent7).
5. Falsifiable (HUB-7 di manifest): 20 template → binding dihitung ulang 2× dari file sama
   = identik 20/20; coverage node per template tercatat sbg metrik.

## 4.5 Kaitan Rosetta ↔ Workflow Hub (mandat #524)

Impor template Hub memakai pipeline yang sama: PARSE (node era apa pun) → migrasi → receipt
→ kanon; template Hub yg memuat node deprecated TIDAK ditolak — masuk receipt sbg
deviation_id (HUB-3), janji eksekusi per-node (konsisten T-14 opsi B). Manifest sidecar
(AGENT4-WORKFLOW-HUB-MANIFEST.md) = metadata; Rosetta = gerbang isi workflow.

## 7. Pertanyaan terbuka (utk PRD-3 / pemilik)

- Q1: default mode "everyHour" menit = 0? (K-2) - keputusan utk rule cron.
- Q2: apakah workflow yang sudah dimigrasi disimpan SEBAGAI migrasi (tersimpan) atau migrasi
      hanya di memori saat eksekusi (workflow asli tetap di disk)? Rekomendasi: simpan asli +
      cache migrasi + receipt; hash asli tetap jadi identitas (prinsip corpus kami).
- Q3: aturan versi lama tetap dipertahankan utk workflow lama (rollback deterministik aturan)?
- Q4 (agent5 #379, KINI TERVIRIFIKASI 2026-09-09): target diff-test cron v1 mode
      ter-generasi memang nondeterministik (randomInt(60) per run, workflow/src/cron.ts);
      TAPI ScheduleTrigger v2 terbukti deterministik (cronExpression as-is + stableInt
      nodeKey, Schedule/GenericFunctions.ts:170). Usulan kategori N8N-NONDETERMINISTIC
      dipersempit: hanya cron v1 mode ter-generasi; detik=0 eksplisit hanya utk konversi
      mode tsb, custom 5-field & jalur v2 pass-through apa adanya.
      **STATUS 2026-09-09: TERTANGANI oleh PRD-3 v3.2 (matt) §5 L3 matriks metode** —
      kelas non-deterministik: engine↔engine = PIN (detik/RNG dipatok di kedua sisi);
      n8n asli↔engine = NORMALISASI saja (pin mustahil di sisi n8n). Selaras persis usulan
      ini; Rosetta §4.1 (detik=0 utk mode ter-generasi) = sisi normalisasi.
- Q5: definisi hitungan "cron" utk inventaris determinisme: 13 file node cron v1 vs 14 file
      bila cronExpression di scheduleTrigger (tpl-2371) ikut dihitung - dua jalur beda kode;
      klasifikasi harus per definisi eksplisit (pelajaran ERR-029).
