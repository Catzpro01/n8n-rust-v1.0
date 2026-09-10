# AGENT9-SCRAPE-L2-DESIGN — Desain Lapis 2 Scraping: `n8n-nodes-browser-use` (Interaksi Otonom)

**Penulis:** agent9 (ROLE_INTEGRATION) · **Tanggal:** 2026-09-09 · **Versi:** v0.1 (DESIGN — freeze #386 dipatuhi, 0 kode)
**Mandat:** ARSITEKTUR-SCRAPING-BERLAPIS.md §2 Lapis 2 (interaksi kompleks: login, form, infinite scroll, pagination) · queue W2-SCRAPE-L2 (P1, ROLE_INTEGRATION) · turunan kerangka AGENT9-SCRAPE-L1-DESIGN.md.

---

## 1. Masalah determinisme yang KHAS L2 (temuan desain utama)

Browser-use = agen AI visual (grounding DOM + koordinat) → **inherently non-deterministik** (keputusan model tiap langkah). Ini bentrok dengan kontrak determinisme tim (record-replay, seed, replay byte-identik). **Solusi dua-mode (inti desain ini):**

| Mode | Perilaku | Kelas determinisme |
|---|---|---|
| `agent` (rekam) | AI menjalankan goal; **setiap aksi direkam** menjadi daftar aksi deterministik `{action, selector, value, wait}` | non-deterministik (kelas N8N-NONDETERMINISTIC ala inventaris agent5) — hanya utk interaktif/editor |
| `replay` (jalankan) | menjalankan daftar aksi yang sudah direkam — TANPA panggilan model | **deterministik penuh** — bisa record-replay + seed → inilah mode produksi/Hub |

Template Hub v1 **hanya boleh memakai mode `replay`** (aksi ter-audit); mode `agent` = alat authoring. Paritas n8n: browser-use asli juga punya konsep session/reusable — `[VERIFIKASI-UPSTREAM: n8n-nodes-browser-use param persis]`.

## 2. Model parameter (→ `ParameterSchema`, arah-A agent4)

| # | Field | Tipe IR | Wajib | Default | Catatan |
|---|---|---|---|---|---|
| 1 | `mode` | `OptionsField` | ✅ | `replay` | `agent` \| `replay` (§1) |
| 2 | `startUrl` | `StringField` | ✅ | — | SSRF-guard §5 |
| 3 | `taskGoal` (agent) | `StringField` (multiline) | ✅ (agent) | — | instruksi natural-language — TIDAK boleh berisi kredensial (scan pra-run, lihat §5) |
| 4 | `actions` (replay) | `JsonField` OPAQUE | ✅ (replay) | — | daftar aksi tercatat `{action: navigate|click|type|scroll|wait|extract, selector, value, waitMs}` |
| 5 | `maxSteps` (agent) | `NumberField` | ⬜ | `15` | 1–50 — batas keras biaya |
| 6 | `sessionMode` | `OptionsField` | ⬜ | `fresh` | `fresh` \| `reuse:{sessionId}` — multi-tab/session (ARSITEKTUR L2) |
| 7 | `outputSchema` | `JsonField` OPAQUE | ⬜ | — | bentuk JSON output yang diminta (diekstraksi di akhir run) |
| 8 | `options.recordActions` | `BooleanField` | ⬜ | `true` (agent) | hasil rekam dikembalikan di output utk dipromosikan jadi replay |
| 9 | `options.ignoreRobots` | `BooleanField` | ⬜ | `false` | modul-bersama L1 |

## 3. ResourceHint & memory

- **SideEffect: `NonIdempotent` DEFAULT utk mode agent & replay yang mengandung aksi `click`/`type`** (interaksi bisa submit form = POST-like); replay murni `navigate|scroll|extract` = `Idempotent` — diklasifikasi otomatis dari isi `actions` (statis, bisa ditentukan saat import — konsisten admission agent1 §5.2 tanpa menjalankan node).
- `weight`: berat (browser penuh + panggilan model di mode agent); worker ephemeral cgroup 150MB (pola camofox §2); `max_concurrency: 1`.
- Mode `agent` menambah biaya LLM — panggilan model = via kontrak agent7 (MCP/AI-capsule), TIDAK inline di node; output model = input-eksternal → taint.

## 4. Output kanon

```
{ status, start_url, steps: [{i, action, selector, ok, took_ms}],   // jejak audit aksi
  data_ref,            // hasil outputSchema via data-plane
  recorded_actions,    // (mode agent) daftar aksi utk dipromosikan ke replay
  pii_redacted: true, took_ms }
```
`tainted-external`; seluruh isi halaman yang tersentuh = sampel sanitasi PII.

## 5. Guardrails (modul-bersama L1 + tambahan khusus L2)

1. Semua guardrail L1 berlaku (SSRF fail-closed, polite delay, robots, PII, atribusi).
2. **Kredensial TIDAK PERNAH di taskGoal/actions**: pra-run scan pola (password/token/email+pass) → tolak `E-CRED-IN-TASK`; login form diisi via referensi credential (`NodeDescriptor.credentials`, pola F-C5) yang di-resolve host-side saat aksi `type` pada field bertanda `credentialRef`.
3. `maxSteps` hard-cap; langkah ke-n+1 = berhenti `FailedTerminal E-L2-STEP-BUDGET` (bukan loop abadi).
4. Situs login/paywall: hanya dengan izin eksplisit akun milik sendiri (ToS akun) — SCRAP-E1 berlaku; blocklist etika per-sumber dari katalog.

## 6. Gate (post-freeze)

| Gate | Kriteria PASS | Reproduksi |
|---|---|---|
| SCRP-L2-1 | determinisme replay: 2 run `actions` sama, seed sama → jejak & output byte-identik | differential harness |
| SCRP-L2-2 | promosi: run `agent` → `recorded_actions` → run `replay` menghasilkan `data_ref` setara (struktur sama) ≥8/10 tugas uji | fixture alur login-mock/pagination-mock |
| SCRP-L2-3 | E-CRED-IN-TASK: 10 taskGoal berisi kredensial tanaman → 10/10 ditolak | uji unit |
| SCRP-L2-4 | step-budget: task 60-langkah dgn maxSteps=15 → berhenti di 15 + kode eksak | mock |
| SCRP-L2-5 | PII & log: 0 kredensial/PII di jejak aksi & log (scan) | check-secrets |

## 7. Terbuka

- **OPEN-SCRAPE-2**: kontrak panggilan model grounding (agent7) — antarmuka & pembatasan token per langkah.
- **SCRAP-E1** berlaku (lebih tajam lagi: L2 menyentuh login → dilarang kecuali akun sendiri + izin).
- `[VERIFIKASI-UPSTREAM: n8n-nodes-browser-use]` parameter asli (belum dibaca — node community, bukan nodes-base).
