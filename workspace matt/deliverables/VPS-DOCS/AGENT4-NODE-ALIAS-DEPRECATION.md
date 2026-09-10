# AGENT4 — REGISTRY ALIAS NODE DEPRECATED (function/cron) v0.1

**Penulis:** agent4 (Schema Compiler & Node Registry) — 2026-09-09
**Mandat/konteks:** Keputusan fern #212: node deprecated `function` & `cron` disetujui masuk katalog alias ke `Code` & `Schedule Trigger`. Dokumen ini menyediakan tabel pemetaan + semantik yang dibutuhkan implementasi (engine: spec agent1 S8.1; parser/registry: spec agent4 AGENT4_SCHEMA_COMPILER_SPEC.md F-C3).
**Prinsip:** klaim mapping ditandai `[VERIFIKASI-UPSTREAM]` bila belum dicek ke kode sumber n8n — tidak ada tebakan diam-diam (ZERO ASSUMPTION #93).

## 1. Bukti kebutuhan dari korpus nyata (96 template publik + wave-3)
| Node deprecated | File kena | Contoh nyata (file) |
|---|---|---|
| `n8n-nodes-base.function` (v1) | 12 | tpl-100 (2 node, `functionCode` isi `items[0].json = ...`) |
| `n8n-nodes-base.functionItem` (v1) | 9 | tpl-1381 (`functionCode` isi `item = JSON.parse(...)`), tpl-156 |
| `n8n-nodes-base.cron` (v1) | 13 | tpl-1471 `triggerTimes: {'item':[{'mode':'everyX','unit':'minutes','value':5}]}`, tpl-159 `{'item':[{'mode':'everyHour'}]}` |
| `n8n-nodes-base.splitInBatches` (v1-3) | 11 | tpl-11572 (v3, param `options`) |

Reproduksi: `python3 /home/agent4/examples_deprecated.py` (memindai corpus/).

## 2. Registry alias (format usulan, bagian dari schema registry per F-C3)
```json
{
  "schema_version": 1,
  "aliases": [
    {
      "from": {"type": "n8n-nodes-base.function", "typeVersion": 1},
      "to":   {"type": "n8n-nodes-base.code", "typeVersion": 2},
      "param_map": [
        {"from": "functionCode", "to": "jsCode", "transform": "identity"}
      ],
      "semantics": {
        "execution_mode": "run_once_for_all_items",
        "note": "function n8n lama menerima seluruh batch sbg var items[]; Code modern default per-item. Map ke mode Run Once for All Items supaya items[] tetap bermakna. [VERIFIKASI-UPSTREAM: nama param jsCode & mode default Code v2]"
      },
      "verified": false
    },
    {
      "from": {"type": "n8n-nodes-base.functionItem", "typeVersion": 1},
      "to":   {"type": "n8n-nodes-base.code", "typeVersion": 2},
      "param_map": [
        {"from": "functionCode", "to": "jsCode", "transform": "item_to_json_scope",
         "note": "functionItem: kode menyentuh var `item` per-item. Di Code modern padanan per-item = $json; transform wajib mengganti referensi `item` -> `$json` ATAU membungkus dgn prolog. [VERIFIKASI-UPSTREAM]"}
      ],
      "semantics": {"execution_mode": "per_item"},
      "verified": false
    },
    {
      "from": {"type": "n8n-nodes-base.cron", "typeVersion": 1},
      "to":   {"type": "n8n-nodes-base.scheduleTrigger", "typeVersion": 1.3},
      "param_map": [
        {"from": "triggerTimes", "to": "rule", "transform": "cron_triggerTimes_to_rule",
         "note": "AMEND 2026-09-09 (bukti implementasi W1-ROSETTA-IMPL): triggerTimes di 13 file korpus = OBJEK JSON (bukan string Python); target typeVersion dikoreksi 2 -> 1.3 (22 file korpus memakai scheduleTrigger 1.1-1.3; bentuk rule identik; v1.3 tertinggi terbukti). Hasil: rule.interval[{field:'cronExpression', expression: ekspresi 5-field}] — dibuktikan nyata di korpus (tpl cronExpression v1.1). Referensi kode: crates/rosetta/src/registry.rs + migrate.rs."}
      ],
      "semantics": {"note": "satu rule per item dlm array"},
      "verified": false
    }
  ]
}
```

## 3. Aturan aplikasi alias (di parser/resolver workflow — L1)
1. **Lokasi**: setelah parse (L1, format OK) & sebelum validasi parameter — alias adalah tahap resolve, bukan rewrite file. File korpus TIDAK diubah (provenansi tetap).
2. **Satu arah, tercatat**: tiap aplikasi alias menghasilkan entri di laporan impor + katalog deviasi (agent5): `ALIAS function#Data1 -> code#Data1 (jsCode)` — tidak pernah diam-diam.
3. **Gagal eksplisit**: alias yang tidak dikenal (mis. `n8n-nodes-base.function` typeVersion 2, atau param `functionCode` berisi require() modul eksternal — lihat tpl-1381 `require('torrent-search-api')`!) → impor GAGAL dengan pesan menyebut node & alasannya (PRD-2 §10.1), TIDAK menurunkan ke codegen yang salah.
4. **require() eksternal**: tpl-1381 membuktikan function lama bisa require npm. Kebijakan (selaras PRD-2 §2.1 no-community-npm): node dgn require eksternal masuk katalog deviasi sbg DEV-xxx (butuh runtime npm → tidak didukung), bukan coba-coba dijalankan.
5. **Peta versi** (F-C3): registry menyimpan per (kind,typeVersion); cron v1 ≠ splitInBatches v3 dst. Migrasi antar-typeVersion TIDAK otomatis.
6. **splitInBatches** (Loop Over Items) TIDAK deprecated — catatan: 11 file memakainya, bukan bagian MVP-26 (temuan F-C1); keputusan T-4 apakah naik kelas — di luar registry ini.

## 4. [SELESAI 2026-09-09] Verifikasi upstream (V-1, V-2) — n8n master (commit 3afdf4a0f8a46fe4087ed463b2eda2e888beabbe)
Berkas dibaca langsung dari repo n8n-io/n8n (salinan lokal /home/agent4/upstream/):
- **V-1 Code v2 — VERIFIED**: `JavascriptCodeDescription.ts` → property `name: 'jsCode'`; `Code.node.ts` → param `mode` nilai `runOnceForAllItems` (default) & `runOnceForEachItem`; eksekusi: `runCodeAllItems(code)` vs `runCodeForEachItem(code, numInputItems)` via JsTaskRunnerSandbox; bahasa: `jsCode` (JS) / `pythonCode` (Python). Pola all-items resmi: `const items = $input.all(); return items.map(...)`.
- **V-2 ScheduleTrigger v2 — VERIFIED**: param `rule` = `fixedCollection` → opsi `interval` (array). Tiap item: `field` options `seconds|minutes|hours|days|weeks|months|cronExpression`; field terpilih menentukan sub-param: `secondsInterval/minutesInterval/hoursInterval/daysInterval/weeksInterval` (number) atau utk `cronExpression` → sub-param **`expression`** (string, placeholder `0 15 * 1 sun`); ada pula `triggerAtHour/triggerAtMinute` utk waktu spesifik & `misfirePolicy`.
- **V-2b util cron (n8n-workflow/src/cron.ts) — VERIFIED**: `TriggerTime` = union `everyMinute | everyX{unit:minutes|hours, value} | everyHour{minute} | everyDay{hour,minute} | everyWeek{weekday 0-6, hour, minute} | everyMonth{dayOfMonth, hour, minute} | custom{cronExpression}`; `toCronExpression` menyisipkan **detik acak** (`randomInt(60)`) — nondeterministik sengaja di level trigger.
- **Status deprecated — VERIFIED**: `Cron.node.ts` v1 = `hidden: true` (dipertahankan utk impor workflow lama); node `Function` & `Cron` masih ada di repo sbg kompatibilitas.
- **V-3 transform function→code**: rekomendasi tetap **prolog wrapper**, bukan regex-rewrite. Bukti pola resmi all-items di atas: kode lama `items[0].json=...; return items` kompatibel dgn prelude `const items = $input.all();` (bentuk item sama {json,...}). functionItem (per-item, var `item`) → mode `runOnceForEachItem` + prolog `const item = $input.item;` — perilaku return plain-object vs {json} [VERIFIKASI-UPSTREAM: perlu konfirmasi wrapper sandbox JsTaskRunner; diuji saat n8n Node 22 aktif].
- **REKOMENDASI MAPPING cron→ScheduleTrigger (baru, berbasis V-2b)**: jangan salin field (skema beda). Konversi tiap `triggerTimes.item[]` ke ekspresi cron via semantik toCronExpression, buang kolom detik (minute-granular), lalu hasilkan item ScheduleTrigger `{field:'cronExpression', expression: <cron-5-field>}`. everyX menit → `*/v * * * *`; everyHour → `m * * * *` (default m=0 bila kosong); dst. Ini juga menghapus keacakan detik (tidak relevan utk diff output).
- **K-1**: usul tetap YA (alias = resolve ke implementasi Code/ScheduleTrigger modern, satu code path).

## 4b. Yang masih butuh keputusan (untuk PRD-3)
- **V-3b**: konfirmasi perilaku wrapper per-item Code v2 utk return plain-object (functionItem) — uji di instance n8n (Node 22) saat differential harness jalan.
- **K-2**: kebijakan cron item tanpa field default (mis. everyHour tanpa minute) — usul default 0 + catat deviasi.
- **K-3**: impor cron multi-item (triggerTimes punya multipleValues) → beberapa item interval ScheduleTrigger — didukung (interval array).

Dokumen ini input utk PRD-3-PERFECTION-CHECKLIST (matt) & integrasi spec engine agent1. Tidak ada kode produk ditulis. — agent4
