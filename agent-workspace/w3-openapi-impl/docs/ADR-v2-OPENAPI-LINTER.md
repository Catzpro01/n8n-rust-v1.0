# ADR-v2 — OPENAPI-DESIGN-LINTER: Validasi Design-Time untuk Node OpenAPI Ter-generate

**Pengaju:** agent9 (ROLE_INTEGRATION, pemilik eksklusif `crates/nodes-openapi` + `openapi-codegen`) · **Tanggal:** 2026-09-09 · **Sayembara:** ADR v2 "Zero-Sacrifice Performance & High-Value Innovation" (fern #1490 §3) · **PILAR 2: fitur leverage tinggi** · **Status: PENGAJUAN** (evaluasi @matt + tim verifier)

## 1. Masalah

Workflow yang memakai node OpenAPI ter-generate baru divalidasi **saat eksekusi** — parameter salah baru ketahuan SETELAH request dikirim (atau setelah node gagal). Biaya nyata operator:

1. **Round-trip API produksi terbuang** (quota/rate-limit/race pada side-effect `NonIdempotent` — request yang salah tetap terkirim sebelum gagal).
2. **Debugging post-mortem**: error CG-E/runtime muncul di tengah run, bukan di design-time.
3. **Data untuk mencegah ini SUDAH ADA tapi belum terpakai**: 1.211 `NodeDescriptor` (ParameterSchema: kind/required/options per field), manifest verifikasi (`ops_rejected` 19 entri github berkode CG-E + alasan), korpus rejection 22 kasus dengan kode eksak, tabel SUPPORTED §1.2.

Ini persis kelas masalah "Smart Workflow Linter" (contoh fern #1490 §B) — saya ambil **niche domain saya** (node OpenAPI ter-generate), komplementer dengan workflow-linter umum (bukan duplikat lane).

## 2. Solusi

Lapis `oapi-lint` **read-only di atas registry `nodes-openapi`** (nol perubahan pipeline codegen):

```
input : sub-tree parameter node OpenAPI dalam workflow JSON (+ nama node + operasi)
output: Vec<Diagnostic> { node, param, kode, pesan, fix-hint }   // tanpa side-effect, tanpa I/O
```

**Rules v1 (semua berbasis data eksisting):**

| Rule | Deteksi | Sumber data |
|---|---|---|
| L-101 | parameter tak dikenal node/operasi itu | ParameterSchema fields |
| L-102 | field `required` hilang | ParameterSchema required |
| L-103 | nilai di luar `options` (enum) | ParameterOption |
| L-104 | tipe nilai ≠ kind schema (string/number/bool) | ParameterKind |
| L-105 | operasi masuk `ops_rejected` manifest → tampilkan alasan CG-E + saran terdekat | manifest.json |
| L-106 | field N1 array-of-string terisi → warning Json-opaque (batas diketahui) | tabel N1 spec |

**Integrasi**: (a) command CLI `lint` (deterministik, `--json` 100% bebas field presentasi/waktu — sesuai standar CLI fern #1490 §4.1); (b) pasca-merge EXEC: pre-run hook (gagal-collections → `Permanent` SEBELUM request dikirim — menghemat round-trip item §1.1).

**Batas v0.1 (jujur)**: tidak mengeksekusi expression (lane expr/agent3); lint statis murni.

## 3. Bukti "Zero-Regression"

**Zero-regression by construction — murni ADDITIVE** *(persempit 58b / koreksi presisi agent10 #1581: klaim di bawah berlaku HANYA untuk varian (i) command `lint` mandiri; varian (ii) pre-run hook PASCA-MERGE menyentuh jalur eksekusi dan karenanya TIDAK diliput klaim ini — ia dievaluasi terpisah saat implementasi, dengan gate INTEG-07..11 yang sudah terdefinisi):*

1. [VARIAN (i) SAJA] Nol baris `ingest/validate/translate/emit` berubah → golden INTEG-01 **byte-identical tetap** (test drift otomatis membuktikan). *Baseline kini TERUKUR (agent10 #1581: 56/56 + 5 golden bernama hijau pasca-kutover R41) — jadi klaim ini dapat DIUJI, bukan sekadar diasumsikan.*
2. `oapi-lint` = **konsumen read-only** registry (fungsi `registry()` yang sudah teruji); nol mutasi state.
3. Test eksisting 59/59 + clippy 0 + `#![forbid(unsafe_code)]` tak tersentuh; tanpa dep baru (serde_json sudah ter-pin).
4. Tanpa HTTP client, tanpa eksekusi apa pun (konsisten #1116 b4; hard-cap RAM tak tersentuh — in-memory query).
5. Implementor≠reviewer tetap; merge hanya via matt (SOP #864).

## 4. Proyeksi Metrik Terukur

| Metrik | Baseline | Target terukur |
|---|---|---|
| Recall kelas terpetakan (R L-101..106) | 0% (tak ada linter) | **22/22 korpus rejection terdeteksi design-time** (target 100%) |
| Precision (false-positive pada operasi sehat) | — | **0 FP pada 1.206 operasi github OK + 5 frankfurter** (golden corpus sebagai kontrol negatif) |
| Latensi lint penuh (1.211 descriptor) | — | **< 100 ms** in-memory setelah registry build (≈µs/rule/node) |
| Dampak pipeline codegen | 59/59, 84 MB peak | **identik** (golden + `/usr/bin/time -v` re-run sebagai bukti no-op) |

## 5. Rencana Verifikasi Mutan

Setiap mutan WAJIB bukti terpasang `sha_sebelum ≠ sha_sesudah` (Ruling 50e) dan test-nya MATI:

| Mutan | Test yang harus mati |
|---|---|
| M1: balik comparator required (L-102) | test deteksi required-hilang |
| M2: sabotase enum-lookup jadi always-true (L-103) | test nilai-di-luar-options |
| M3: loader manifest `ops_rejected` dikosongkan (L-105) | test saran-alasan CG-E |
| M4: sisipkan `sleep(250ms)` di path lint | test latensi-budget <100 ms |
| M5 (no-op guard): linter menulis/mutasi registry | test immutability (registry hash sebelum==sesudah lint) |

## 6. Rencana eksekusi (bila disetujui)

Scope ±1 crate kecil + test; estimasi 1 sesi implementasi + 1 sesi review. **TIDAK mengganggu gerbang aktif saya** (W3-OPENAPI-IMPL menunggu agent4+agent10; EXEC menunggu merge). Prasyarat: pengesahan matt; review independen 2 agen; gate tambahan INTEG-13 (lint corpus) masuk spec v0.5.4.

— agent9
