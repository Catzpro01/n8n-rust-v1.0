# DETERMINISM-SOURCE-INVENTORY.md
## Inventaris sumber nondeterminisme per korpus — agent5 (QA)

**Tanggal:** 2026-09-09 06:13 UTC  
**Korpus:** 171 file `tpl-*.json` di root `/opt/agent-workspace/docs/corpus`  
**Versi korpus (sha256 gabungan):** `05b8e254475626853cf50324893a0fba7283c32f8d47587190cfcb628601468e`  
**Cara reproduksi:** `python3 /opt/agent-workspace/qa/gen_determinism_inventory.py` — deterministik, tanpa network, stdlib saja.

Ini jawaban atas seruan agent4 (#343): *"sumber nondeterminisme diinventarisir per node"*.
Angka di bawah = berapa **FILE** yang tersentuh dan berapa **INSTANCE** node, diukur dari
korpus nyata, bukan asumsi. Ia menentukan mekanisme determinisme mana yang menutup
berapa persen korpus — jadi urutan fase bisa diputuskan dari data, bukan dari selera.

> **REVISI v2 (koreksi agent4 #384, terverifikasi agent5):** baris cron dipecah dari 1 regex
> mentah menjadi 2 kelas node-type. Regex lama `cronExpression|triggerTimes|toCronExpression`
> menggabungkan dua jalur kode n8n yang berbeda. Verifikasi: 13 file cron-v1 + 1 file
> scheduleTrigger-cronExpression (`tpl-2371`), 0 tumpang tindih. Split per regex TIDAK bersih
> (`cronExpression` juga muncul di 3 file cron-v1), jadi klasifikasi harus per node-type.
> Angka kesimpulan besar (eksternal 74%) tidak berubah.

| Sumber nondeterminisme | File | Instance | Ditangani oleh |
|---|---|---|---|
| PANGGILAN EKSTERNAL (node pihak ke-3) | 128 | 638 | **record-replay** (Layer 1a) — sudah dibutuhkan utk differential testing |
| jam/tanggal (Date/$now/luxon) | 28 | 49 | **jam beku** ke timestamp eksekusi (proxy global QuickJS) |
| cron v1 node (n8n-nodes-base.cron, triggerTimes -> toCronExpression detik acak) | 13 | 13 | registri alias + **aturan detik=0 eksplisit** (DEV-A4; n8n sendiri nondeterministik) |
| Math.random/$random | 3 | 5 | **PRNG ter-seed** dari execution_id (proxy global QuickJS) |
| $execution.id (bukan $itemIndex) | 2 | 2 | **injeksi dari record** (nilai logis direkam saat run asli) |
| scheduleTrigger v2 cronExpression (n8n-nodes-base.scheduleTrigger) | 1 | 1 | registri alias + **verifikasi upstream dulu** (jalur kode beda; detik acak belum dikonfirmasi) |
| webhookUrl/$resumeWebhook | 1 | 3 | **injeksi dari record** (nilai logis direkam saat run asli) |

## Cara membaca

- **128/171 file (74%)** punya panggilan eksternal. Ini sumber
  nondeterminisme **DOMINAN**, dan ia ditangani lapisan record-replay yang **wajib kita
  bangun anyway** untuk differential testing (PRD-2 §10.2). Jadi bukan biaya baru —
  ia biaya yang sudah kita tanggung, dan determinisme eksekusi menumpang gratis di atasnya.
- Sisanya (jam, random, UUID, cron, execution.id, webhookUrl, env) butuh **proxy global
  deterministik** di crate `expr-quickjs`: jam beku + PRNG ter-seed. agent3 mengusulkan
  ini di #307/#340; tabel di atas memberi **prioritasnya berdasarkan frekuensi korpus**.
- **Cron (total 14 file unik):** 13 file cron-v1 (jalur `toCronExpression`,
  detik acak — DEV-A4, n8n nondeterministik) + 1 file scheduleTrigger-v2
  (`tpl-2371`, cronExpression). Kedua jalur adalah kode n8n yang BERBEDA; apakah scheduleTrigger
  juga menyuntik detik acak **belum diverifikasi di upstream** (agent4 #390). Jangan samakan
  prioritasnya sebelum jalur v2 dikonfirmasi.

## Catatan kejujuran — batasan inventaris ini

1. Ini memindai **string parameter**, bukan mengeksekusi expression. `$itemIndex` dan
   `$runIndex` **sengaja dikeluarkan** dari kategori `$execution.id` karena keduanya
   **deterministik** (posisi item dalam run), sedangkan `$execution.id` tidak. Regex awal
   saya mencampur keduanya; saya pisahkan. Kalau tidak, angkanya melebih-lebihkan.
2. Node `Code`/`function` bisa memanggil `Date.now()`/`Math.random()` di dalam string JS
   yang tidak selalu tertangkap pola di atas. Inventaris ini **batas bawah**, bukan lengkap.
3. Urutan iterasi object (K3 di DEVIATION-CATALOG) dan presisi float (K2) adalah sumber
   nondeterminisme **antar-runtime** (V8 vs QuickJS), bukan antar-run di runtime yang sama.
   Keduanya tidak muncul di tabel ini karena bukan properti korpus — tapi justru keduanya
   yang paling mungkin membuat gate L3 gagal. Lihat DEVIATION-CATALOG §6.
4. Cron adalah kasus khusus: **n8n sendiri nondeterministik** di jalur v1
   (`toCronExpression` menyuntik detik acak — agent4 #264, DEV-A4). Jadi 'rekam dari run
   asli' tidak cukup; konversinya harus menetapkan aturan detik yang eksplisit (usul agent5:
   kategori `N8N-NONDETERMINISTIC`, detik=0). Jalur v2 (scheduleTrigger cronExpression) belum
   dikonfirmasi perilakunya — itu keputusan/verifikasi terpisah, bukan sekadar perekaman.

_Ditulis oleh agent5 (QA & Security Auditor). Koreksi dipersilakan — inventaris ini_
_batas bawah dan saya nyatakan sendiri di mana ia tidak lengkap. Revisi v2 menyerap_
_koreksi terverifikasi agent4 (#384); itu contoh budaya verify-everything yang benar._