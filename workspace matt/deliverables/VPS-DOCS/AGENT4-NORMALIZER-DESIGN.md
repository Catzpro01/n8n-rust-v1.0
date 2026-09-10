# Desain JSON Normalizer untuk Differential Testing (draf agent4)

**Mandat:** fern #168/#170 — rancang normalizer yang mengabaikan timestamps/ID dinamis saat mencocokkan output node engine Rust vs n8n asli.
**Ruang lingkup:** dokumen desain (belum kode; patuh zero-code-Rust; prototipe Python boleh setelah desain disetujui).

## 1. Masalah
Differential testing membandingkan output A (n8n asli) vs output B (engine kita) item-per-item (PRD-2 S10.2). Output mentah tidak bisa dibandingkan langsung karena:
1. **Nilai dinamis**: `$execution.id`, `$runIndex`? (deterministik), node id uuid, execution timestamps, `$now`/`$today` saat trigger, pairedItem references, dll.
2. **Kebisingan metadata**: field non-n8n yang disuntik (mis. `_corpus_meta` — ERR-022), field kosong yang tidak relevan.
3. **Non-determinisme urutan**: hasil HTTP paralel, object key ordering, branch concurrency.
4. **Presisi representasi**: float formatting, JSON number vs string coercion, timezone offset representation.

## 2. Arsitektur: 3 tahap
```
raw output A/B (item stream JSON)
   → 1) SCHEMA-STRIP  (buang field non-kontrak: _corpus_meta, meta instance, dll)
   → 2) CANONICALIZE  (urutkan key, normalisasi angka/tanggal/coercion, sortir array tak-berurut)
   → 3) DIFF-ENGINE    (compare canon A vs canon B; hasil: identik | berbeda+deviasi)
```
Deviasi → DEVIATION-CATALOG.md (pemilik: agent5). Normalizer tidak pernah "menyamakan" perbedaan semantik — hanya menghapus noise yang disepakati.

## 3. Daftar aturan normalisasi (v0.1 — tiap aturan = 1 fungsi + 1 test)
| ID | Aturan | Kategori | Contoh |
|---|---|---|---|
| N-01 | Hapus field `_corpus_meta` & field root non-n8n apa pun (allowlist skema) | strip | ERR-022 |
| N-02 | Hapus `id`/uuid yang berubah antar run (node internal `id`) | ignore | — |
| N-03 | Normalisasi timestamp ISO-8601 ke UTC epoch (ms) | canon | `2026-09-09T04:00:00.000Z` |
| N-04 | `$execution.id` & resumeUrl → token `{{EXEC_ID}}` | ignore | — |
| N-05 | Urutkan key object secara rekursif | canon | {b,a}=={a,b} |
| N-06 | Sortir array item bila node deklaratif `orderless` (daftar explicit; default: urutan dipertahankan) | canon | hasil HTTP paralel |
| N-07 | Number formatting: -0 → 0; 1.0 vs 1 → setara bila keduanya number | canon | coercion |
| N-08 | Timezone: representasi offset +07 vs +0700 → setara (parse ke instant) | canon | Luxon edge |
| N-09 | pairedItem: abaikan bila menunjuk index di luar batch (crash recovery) | ignore | — |
| N-10 | binary data: bandingkan hanya metadata (fileName/mimeType/size), isi via sha256 terpisah | ignore-detail | — |
| N-11 | null vs undefined vs missing key pada `json` → setara | canon | V8 vs QuickJS |
| N-12 | error object: bandingkan `message` + `code` saja (stack trace diabaikan) | canon | — |

**Prinsip:** aturan hanya menghapus/me-normalisasi noise yang (a) disepakati tim, (b) tidak mungkin menjadi deviasi semantik. Aturan baru = ADR kecil + test, bukan keputusan diam-diam.

## 4. Input workflow juga dinormalisasi (sebelum eksekusi kedua sisi)
- N-20: strip field non-n8n dari workflow JSON (sama N-01 untuk file korpus yang tercemar ERR-022).
- N-21: set `timezone` eksplisit (default UTC) supaya $now deterministik.
- N-22: ganti trigger non-deterministik (schedule/webhook) dengan Manual/interval tetap saat mode diff — atau tandai `SKIP-EXEC` dan hanya uji L1.

## 5. Keluaran & pelaporan
- `normalize(workflow|output, ruleset) → canon JSON`
- Laporan diff: `PASS | DIFF(n) | SKIP(reason)` per file + artefak mentah kedua sisi disimpan (rekomendasi agent5 R2).
- Aturan aktif dicatat di header laporan (reproducibility).

## 6. Pembagian usulan (koordinasi #168)
- agent4: desain ini + implementasi aturan N-01..N-05,N-07,N-11 inti + test (prototipe Python).
- agent1: integrasi pipeline (dari runner harness), N-06, N-10, N-12 + laporan.
- agent3: N-08 (timezone/Luxon), N-09, N-22 (trigger) + validasi expression edge-case suite (215 kasus, #133).
- agent5: review ruleset + kepemilikan DEVIATION-CATALOG (hasil DIFF masuk ke sana).
Menunggu konfirmasi pembagian & apakah prototipe Python boleh langsung ditulis (QA tooling seperti verify_corpus.py agent5 — bukan kode produk Rust).
