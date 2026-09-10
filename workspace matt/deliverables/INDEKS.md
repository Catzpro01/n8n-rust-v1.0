# INDEKS SELURUH DOKUMEN

**Disusun: 2026-09-10** · **415 berkas · 5,0 MB**
**Lokasi:** `PRD/` 10 · `ADR/` 5 · `WORKSPACE/` 20 · `VPS-DOCS/` 380

Indeks ini menjawab dua hal: **mana yang masih berlaku**, dan **siapa menulis apa**. Tanpa ini, empat versi
PRD-3, dua salinan identik, dan 380 dokumen agen membuat mustahil tahu dokumen mana yang memerintah.

Daftar lengkap tiap berkas dengan ukurannya ada di `MANIFEST.md` — dibangkitkan otomatis dari isi direktori,
bukan dari ingatan saya.

---

## ⚠️ Temuan tata kelola yang perlu keputusan Anda

**Versi PRD-3 yang Anda sahkan adalah 3.2. Yang sekarang duduk di nama berkas kanonik adalah 3.4 — dan 3.4
secara eksplisit berstatus "DRAFT — BELUM DISETUJUI, dan belum boleh ditandatangani".**

Bukti:

| Berkas | Versi di dalamnya | Status di dalamnya |
|---|---|---|
| `PRD-3-CANONICAL-APPROVED.md` | **3.2.0-draft** | ✅ RATIFIED & APPROVED (Disahkan oleh Pemilik Proyek) |
| `PRD-3-PERFECTION-CHECKLIST.md` | **3.4.0-draft** | ⚠️ DRAFT — BELUM DISETUJUI |
| `PRD-3-PERFECTION-CHECKLIST-v3.4-matt.md` | 3.4.0-draft | ⚠️ DRAFT — BELUM DISETUJUI |

`PRD-3-PERFECTION-CHECKLIST.md` dan `PRD-3-PERFECTION-CHECKLIST-v3.4-matt.md` **identik byte-per-byte**
(sha `1a82b7af7203`). Artinya nama berkas kanonik — yang akan dibaca orang dan agen sebagai "PRD-3" —
sekarang berisi draf yang belum Anda setujui, menggantikan yang sudah.

**Selisih v3.2 → v3.4: 128 baris dari 542.** Itu bukan perubahan kosmetik.

Ini pola yang sama dengan 88 keputusan yang disetujui sendiri: draf naik versi, versi baru menempati nama
kanonik, dan tidak ada langkah "disahkan". **Tidak ada yang menyembunyikan apa pun** — status DRAFT tertulis
jujur di dalam berkasnya. Yang hilang hanya mekanisme yang mencegah draf menggantikan dokumen sah.

**Butuh keputusan Anda:** sahkan v3.4, atau kembalikan nama kanonik ke v3.2 dan simpan v3.4 sebagai draf.

---

## PRD — 10 berkas

### Yang ditulis matt (Lead Architect)

| Berkas | Versi | Status | Isi |
|---|---|---|---|
| `PRD.md` | 0.2 | DRAFT, belum disetujui | PRD paling awal. Workflow Automation Engine (Rust). 793 baris. Digantikan oleh seri PRD-1/2/3. |
| `PRD-1-N8N-ANALYSIS.md` | 1.0 | — | **Analisis lengkap n8n asli.** Versi 2.39.0, commit `3afdf4a`. 1.017 baris. Disusun dari sumber publik; batas kejujuran dinyatakan di dalamnya. |
| `PRD-2-RUST.md` | 1.4 | DRAFT, belum disetujui | **Adaptasi & peningkatan ke Rust.** 1.291 baris. Status kernel: PROVISIONAL-FREEZE (bukan beku) — revisi audit F10. Memuat tiga klaim beserta status buktinya. |
| `PRD-3-PERFECTION-CHECKLIST-v3.1-matt.md` | 3.1.0 | DRAFT | Menggantikan v3.0.0-FINAL (fern, sha `64b06ee8`). 525 baris. **Versi asli v3.0 fern sudah tidak ada — tertimpa.** |
| `PRD-3-PERFECTION-CHECKLIST-v3.2-matt.md` | 3.2.0 | DRAFT → **ini yang Anda sahkan** | 543 baris. Isi pengesahannya ada di `PRD-3-CANONICAL-APPROVED.md`. |
| `PRD-3-PERFECTION-CHECKLIST-v3.4-matt.md` | 3.4.0 | DRAFT, belum disetujui | 657 baris. **Duduk di nama kanonik — lihat temuan di atas.** |
| `PRD-3-PERFECTION-CHECKLIST.md` | 3.4.0 | DRAFT | **Duplikat identik** dari v3.4-matt. sha `1a82b7af7203`. |
| `PRD-FRONTEND-RUST.md` | 1.0 | **menunggu persetujuan Anda** | 265 baris. Disusun memakai skill `to-spec` (mattpocock/skills v1.2.3). Frontend Leptos → WASM. Berisi diagnosa irisan horizontal, keputusan framework, seam, dan 30 user story. |

### Yang bukan ditulis matt

| Berkas | Versi | Status | Penulis |
|---|---|---|---|
| `PRD-3-CANONICAL-APPROVED.md` | mengesahkan 3.2.0 | ✅ RATIFIED & APPROVED | Pemilik Proyek, 2026-09-09 07:53 UTC |
| `PRD-WORKFLOW-HUB-STANDALONE.md` | 1.0.0-CANONICAL | ✅ RESMI DISAHKAN | Ratifikasi Pemilik Proyek, 2026-09-09 |

---

## ADR — 5 berkas

**Tidak satu pun ADR ditulis oleh matt.** Kelimanya buatan agen, dan semuanya masih berstatus pengajuan —
belum ada yang saya putuskan sebagai Lead Architect. Ini sendiri temuan: ADR menumpuk tanpa adjudikasi.

| Berkas | Penulis | Status | Isi |
|---|---|---|---|
| `ADR-v2-STORAGE-SECURITY-DEEP-DIVE.md` | **agent2** (Storage Engineer) | 2026-09-09 | 636 baris, terbesar. Keamanan lapisan penyimpanan. |
| `ADR-v2-STORAGE-BATCH-WAL-OPTIMIZATION.md` | agent2 (diduga — ada field Author/Date) | ada bagian Status | 213 baris. Temuan antara lain: tidak ada connection pooling, tiap operasi buka/tutup koneksi. |
| `ADR-v2-LINEAGE-REPLAY.md` | sayembara ADR v2 (fern #1490 §3) | SUBMITTED | 112 baris. Time-travel execution replay berbasis lineage + content-digest. READ-ONLY, tidak menyentuh hot path. |
| `ADR-v2-wasm-module-cache.md` | **agent6** (ROLEWASM) | SUBMISSION, menunggu evaluasi matt | 141 baris. Content-addressed WASM module cache. Impor deny-by-default, egress deny, identitas `module_sha256` (MV-3). |
| `ADR-v2-OPENAPI-LINTER.md` | tidak tercantum | — | 75 baris. Validasi design-time untuk node OpenAPI ter-generate. Merujuk Ruling 50e (mutan wajib bukti sha). |

**Catatan:** ada salinan kedua `ADR-v2-OPENAPI-LINTER.md` di `w3-openapi-impl/docs/` di VPS. Tidak saya
tarik karena duplikat.

Dua ADR lain yang muncul di pencarian adalah **milik upstream n8n**, bukan buatan kita:
`ADR-20260828-trigger-settlement-before-execution.md` dan `ADR-20260904-store-the-workflow-with-the-execution.md`
di `upstream/n8n-2.39.0/packages/@n8n/engine/docs/adr/`.

---

## WORKSPACE/ — 20 dokumen kerja matt

Semuanya tulisan matt kecuali dinyatakan lain. Ini bukan PRD atau ADR, tapi ini yang sebenarnya memerintah
pekerjaan sehari-hari.

**Yang mengikat sekarang:**

| Berkas | Isi |
|---|---|
| `TICKETS-TRACER-BULLET.md` | 12 tiket tracer bullet + tepi blokir + prefactor. **Ini yang memerintah urutan kerja.** |
| `CLI-PARITY-SPEC.md` | 24 perintah dari sumber n8n 2.39.0, definisi paritas 5 lapis, dan **§DIVERGENSI SADAR** (D-1 exit code). |
| `REPLAN-BACKEND-MATANG.md` | 8 peran (C-1 Kritikus … C-8), definisi Stage 1, graf dependensi, aturan pelaporan. |
| `KERNEL-FORK-FINDING.md` | **5.995 baris. Log kanonik: Ruling 1–69, 13 kesalahan matt, semua pengukuran.** Dokumen terpenting untuk memahami mengapa keputusan berbentuk seperti sekarang. |

**Audit & adjudikasi:**

| Berkas | Isi |
|---|---|
| `AUDIT-D1-D100.md` | Audit 100 keputusan. |
| `ADJUDIKASI-F1-F21.md` | Adjudikasi temuan F1–F21. |
| `AUDIT-STATUS-DONE.md` | Audit klaim "DONE" — sumber temuan 9 stub crate. |
| `VERIFIKASI-DELIVERABLE-TIM.md` | Verifikasi deliverable antar-agen. |
| `KEPUTUSAN-YANG-DIBUTUHKAN.md` | Daftar keputusan yang menggantung. |

**Analisis teknis:**

| Berkas | Isi |
|---|---|
| `KERNEL-SPEC.md` | Spesifikasi kernel. |
| `ANALISIS-KORPUS-NODE.md` | Analisis korpus node. |
| `BAHASA-BERSAMA.md` | Kamus istilah. **§2 dinyatakan NON-BINDING — jangan dikutip sebagai otoritas.** |
| `SINTESIS-SPILLSTORE.md` | Sintesis SpillStore. |
| `BENCH-A03.md` | Hasil benchmark A03. |
| `rangkuman-rust-workflow-os.md` | Rangkuman awal. |

**Rencana uji W0:** `W0-Spill-TEST-PLAN.md`, `W0-CONTEXT-CONTRACT-TEST.md`, `W0-CHECKSUM-REMEDIATION.md`

**Status & ruling:** `STATUS.md`, `RULING-LEAD-ARCHITECT-20260909.md`

---

## VPS-DOCS/ — 380 berkas dari mesin agen

Salinan lengkap `/opt/agent-workspace/docs/`, ditarik sebagai satu arsip dan **diverifikasi md5 identik**
(`e8698e3cf9ab43d75c51240a8d188abd`). Dikecualikan: 5 berkas `.bak-*`, `upstream/`, `skills-mattpocock/`,
`target/`.

Ini dokumen yang ditulis **oleh para agen**, bukan oleh matt. Yang paling relevan sekarang:

| Berkas | Penulis | Kenapa penting |
|---|---|---|
| `spec/` | matt | Salinan spec yang dibaca agen — `PRD-FRONTEND-RUST.md`, `TICKETS-TRACER-BULLET.md`, `CLI-PARITY-SPEC.md`, `REPLAN-BACKEND-MATANG.md` |
| `ROSETTA-API-FOR-WORKFLOW.md` | agent4 | Riset yang saya setujui di Ruling 66. Menjawab apakah C-6 perlu parser baru. |
| `PREFACTOR-3-STORAGE-INTEGRATION.md` | agent2 | Klaim selesai PF-3 — **belum diverifikasi** |
| `PREFACTOR-4-AUDIT-TRAIT-NODE.md` | agent4 | Klaim selesai PF-4 — **belum diverifikasi**. Sumber temuan kanal parameter. |
| `CONTEXT.md` | agent3 | Kosakata domain PF-2. Dilaporkan 0 penyimpangan — **belum diverifikasi** |
| `NODE-PARAM-FORMS-TB03.md` | — | Bentuk parameter node untuk TB-03 |
| `FIXTURE-TB/` | agent9 | **Fixture yang menemukan blocker TB-02.** 2 workflow ekspor nyata + golden ekspresi |
| `N8N-HELP-SEMANTIC-REFERENCE.md` | — | Referensi semantik `--help` n8n |
| `AGENT{1,4,6,7,10}-PRD3-*.md` | para agen | Alignment dan review terhadap PRD-3 |

Sisanya: `corpus/` (korpus node dan fixture), `tools/` (skrip analisis), dan dokumen lane lama
(`R6-*`, `W0-*`, `RFC-*`, `SOP-*`, `SISTEM-*`).

---

## Ringkasan

```
deliverables/
├── INDEKS.md          dokumen ini
├── MANIFEST.md        415 berkas dengan ukurannya, dibangkitkan otomatis
├── PRD/         10
├── ADR/          5
├── WORKSPACE/   20
└── VPS-DOCS/   380
                 ───
                 415   ·  5,0 MB
```

**Cara membaca kalau waktu terbatas:** `PRD/PRD-FRONTEND-RUST.md` → `WORKSPACE/TICKETS-TRACER-BULLET.md` →
`WORKSPACE/CLI-PARITY-SPEC.md`. Tiga itu cukup untuk tahu apa yang sedang dibangun dan apa yang sudah
diputuskan. `WORKSPACE/KERNEL-FORK-FINDING.md` kalau ingin tahu mengapa.
