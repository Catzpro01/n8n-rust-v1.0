# AGENT7-PRD3-MCP-INPUT — Masukan agent7 untuk Rakitan PRD-3 (Pilar Integrasi MCP)

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** INPUT siap-serap (W5)
**Tujuan:** memberi matt (pemilik rakitan PRD-3 / PRD-3-PERFECTION-CHECKLIST) paket ringkas pilar MCP: keputusan terkunci, dokumen sumber, usulan butir checklist, ketergantungan, dan pertanyaan terbuka.
Patuh freeze #386.

---

## 1. Ringkasan pilar (untuk diletakkan di PRD-3)

Server MCP engine = **antarmuka authoring & validasi** (BUKAN eksekusi) yang memungkinkan coding agent (internal dulu — usulan O-2) menginspeksi, menambal (JSON-Patch RFC 6902), memvalidasi, dan membaca receipt workflow n8n-format, dengan biaya token rendah: capsule mastery ≤500 token, resources JIT pull-only ≤500 token/URI, hasil terkompresi (topo-map ≤2% token mentah). Transport v1 = stdio (D-2). Protokol: adapter ganda (D-1). Fondasi isi: dokumen agent1 (tools/registri E-*/gate), agent4 (KLL resources/DiagnosticReport), agent3 (expression-rules) — disahkan #431.4.

## 2. Keputusan terkunci (pemilik proyek, 2026-09-09)

| ID | Keputusan | Lokasi rincian |
|---|---|---|
| D-1 | Adapter ganda protokol MCP (keluarga 2025 + 2026); pin saat Fase 1 | AGENT7-PROTOCOL-ROSTER §3 |
| D-2 | Transport v1 = stdio; Streamable HTTP = Fase 2+ (ber-OAuth) | AGENT7-PROTOCOL-ROSTER §2 |
| D-3 | Posisi server = mode binary tunggal (`engine mcp`) | AGENT7-MCP-INTEGRATION-SPEC §4.1 |
| D-4 | Scope resource v1 = subset authoring inti (schema, alias, expression-rules, errors, receipt) | AGENT7-JIT-ROUTING-REGISTRY §2 |
| D-5 | MCP authoring/validasi saja; eksekusi = API terpisah (F-8) | AGENT7-MCP-SESSION-THREATMODEL §4 |
| D-6 | Kredensial refs-only + allowlist `$env` dikunci | AGENT7-MCP-INTEGRATION-SPEC §8 |
| D-7 | Review adversarial agent1–5 dulu sebelum masuk PRD-3 (sedang berjalan, #469/#475) | — |

## 3. Dokumen sumber pilar (verifikasi sha256 di server)

- `AGENT7-MCP-INTEGRATION-SPEC.md` (induk; `0c5ad285…`)
- `AGENT7-PROTOCOL-ROSTER.md` (`df912fc8…`)
- `AGENT7-JIT-ROUTING-REGISTRY.md` (`e5adb150…`)
- `AGENT7-MCP-SESSION-THREATMODEL.md` (`210dda56…`)
- Fondasi: `AGENT1-MCP-SKILL-SPEC.md`, `AGENT4-MCP-KNOWLEDGE-SPEC.md`, `AGENT3-EXPRESSION-RULES-MCP.md`

## 4. Usulan butir checklist PRD-3 sisi MCP (untuk PRD-3-PERFECTION-CHECKLIST)

| ID usulan | Butir | Kriteria lulus | Sumber gate |
|---|---|---|---|
| MCP-01 | Permukaan v1 final (5 tools + 9 URI + 1 prompt) disetujui seluruh owner konten | tanda tangan per-owner di registri | AGENT7-JIT-ROUTING-REGISTRY §5 |
| MCP-02 | Adapter ganda: 2 keluarga protokol lulus conformance | MCP Inspector pass | A7-1 |
| MCP-03 | Seluruh angka token ≤ budget, diukur tokenizer rujukan | resource ≤500; cron case <900 | A7-3/KLL-2 (#419) |
| MCP-04 | Daftar URI 1:1 tanpa duplikasi/konflik dgn dokumen fondasi | diff 0 | A7-2/KLL-4 |
| MCP-05 | Overhead stdio terukur (dev + mesin 2 CPU) | ≤50 ms p50; RAM 0 persisten | A7-4 (kalibrasi mesin target) |
| MCP-06 | Kualitas loop perbaikan: 20 skenario sintetis | kode benar 20/20; ≤3 iterasi | A7-5/KLL-1/agent3 (ai_comprehension) |
| MCP-07 | Redaksi kredensial | 100% dari 50 sesi sintetis | A7-6 (agent5) |
| MCP-08 | Threat model T-1…T-9 disetujui agent5 | review tertulis | AGENT7-MCP-SESSION-THREATMODEL §4 |
| MCP-09 | Service facade (trait) disetujui agent1 sbg kontrak engine | review tertulis | AGENT7-MCP-INTEGRATION-SPEC §4.1 |
| MCP-10 | Contoh sesi W4 diverifikasi 1:1 dengan implementasi | walkthrough lulus | A7-5 + sesi nyata |
| MCP-11 | Enforcement F-8 (tak ada eksekusi via MCP) teruji | uji negatif: klien meminta eksekusi → tidak tersedia/ditolak | agent1 §3 |
| MCP-12 | Keputusan auth HTTP tercatat (bukan blocker v1 stdio) | ADR/nota matt | D-2 |

## 5. Ketergantungan & risiko

| Item | Jenis | Pemilik | Catatan |
|---|---|---|---|
| Tokenizer rujukan (#419) | blocker pengunci angka | matt | semua gate token (MCP-03) menunggu ini |
| K-1/K-2/K-3 + T-14 | keputusan pemilik | owner via fern/matt | menentukan scope produk & kapan pilar MCP jadi publik (O-1) |
| Schema compiler v0.2 (K-1..K-4) | isi resource | agent4 | `n8n://nodes/{name}/schema` |
| Service boundary engine (executor/registry) | kontrak | agent1 + agent7 (W7) | MCP berpotensi jadi **konsumen pertama** service facade — selaras PROVISIONAL-FREEZE (ujian API sebelum dibekukan) |
| SDK/klien aktual | keputusan D-1 checklist | agent7 (Fase 1) | 4 kriteria di ROSTER §3 |
| vdb 30GB belum ter-mount (#455) | infra | pemilik mesin | blocker build besar (CARGO_TARGET_DIR) |
| K-4/K-5/K-6 (sudoers/sshd/key) | keamanan | pemilik mesin | rekomendasi matt di KEPUTUSAN; di luar mandat agent |

## 6. Yang agent7 butuhkan dari matt

1. Jadwal & posisi PRD-3 + nama file checklist resmi (bila sudah dibuat) — supaya masukan ini bisa diserap tanpa tabrakan.
2. Keputusan/konfirmasi tokenizer rujukan (#419) — satu-satunya blocker keras untuk mengunci angka token.
3. Jawaban atas O-1…O-3 di bawah.

## 7. Pertanyaan terbuka untuk owner/matt

| ID | Pertanyaan | Opsi | Rekomendasi agent7 |
|---|---|---|---|
| O-1 | Posisi pilar MCP di roadmap | (a) paralel awal Fase 1 (spike stdio W6 sebagai konsumen pertama service facade), (b) sesudah engine inti, (c) Fase 2 | **(a)** — menguji kontrak engine lebih awal, sejalan PROVISIONAL-FREEZE |
| O-2 | Klien target pertama | (a) **agent internal (dogfooding)**, (b) langsung Claude Code/Cursor | **(a)** — biaya rendah, umpan balik cepat; eksternal setelah stabil |
| O-3 | Kanal kerja MCP | (a) cukup #general/#dev, (b) kanal baru (mis. #mcp) | **(a)** dulu; buka kanal hanya bila volume diskusi > ambang (Pasal 6.4 #aturan) |

---

*Disiapkan agent7 untuk matt — paket lengkap: 4 dokumen AGENT7 + 3 fondasi. Semua angka dan klaim punya sumber; verifikasi ulang dipersilakan.*
