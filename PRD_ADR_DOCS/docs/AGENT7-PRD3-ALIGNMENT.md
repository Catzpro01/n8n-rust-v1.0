# AGENT7-PRD3-ALIGNMENT — Pernyataan Keselarasan agent7 atas PRD-3 v3.2 (matt)

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** ALIGNED dengan 2 usulan perbaikan (L5-MCP, angka inventaris)
**Dokumen diverifikasi:** `PRD-3-PERFECTION-CHECKLIST-v3.2-matt.md` (543 baris, sha `e51bcc997311…`)
**Protokol:** PRD-3 §10 — konfirmasi keselarasan per agen sebelum sign-off pemilik.

---

## 1. Pernyataan

Agent7 **MENYATAKAN SELARAS** dengan PRD-3 v3.2 untuk seluruh bagian yang menyentuh pilar MCP, dengan dua usulan perbaikan (bagian 4) yang tidak memblokir sign-off dari sisi pilar MCP. Tidak ada keberatan yang bersifat blocker.

**Dukungan urutan (§10):** setuju urutan yang diusulkan matt — (1) pemilik menjawab §8 minimal K-3, K-8, K-4; (2) Wave 0 remediasi dikerjakan; (3) revisi + sign-off eksplisit per lead; (4) Wave 1 dibuka. Khusus K-8 (apakah INOVASI masuk cakupan): tugas `[N]` milik agent7 (W1-MCP-TRANS/TOOLS) sudah DONE pada level **rencana implementasi** (freeze-compliant); eksekusi kode tetap menunggu K-8 + sign-off — tidak ada konflik dengan Zero-Code.

## 2. Verifikasi keselarasan (terukur, per bagian)

| Bagian v3.2 | Isi terkait MCP | Verifikasi agent7 | Status |
|---|---|---|---|
| §0 cara baca | INOVASI (MCP) diputuskan pemilik | Konsisten D-1…D-7 (semua keputusan pemilik sudah dikunci & terdokumentasi) | ✅ selaras |
| §2.2 | MCP = inovasi, perlakuan `[N]`, tunggu K-8 | Benar; catatan: mandat pemilik Workflow Hub (#524) & i18n (#547) menegaskan MCP sebagai bagian diminta | ✅ selaras |
| §4 Wave 1 | `W1-MCP-TRANS` & `W1-MCP-TOOLS` = `[N]`, ROLE_AI_MCP | Mapping benar; kedua tugas **DONE** (rencana: `98d0f827`, `08ecd4b3`) | ✅ |
| §5 L1–L4 | Tidak menyentuh MCP | — | ✅ |
| §5 L5 | Hub/UI/i18n `[N]` (HUB-6/7, I18N-40) | Tidak ada butir MCP → usul L5-MCP (bagian 4) | ⚠️ usul |
| §6 | swarm-task (15 tugas; MCP 2 sudah DONE) | Terverifikasi via `swarm-task list` (07:37) | ✅ |
| §7 | Kernel/cacat — bukan ranah MCP | — | ✅ |
| §8 | K-3/K-8/K-4/K-6/K-7 blocker | K-8 menyentuh MCP langsung; sisanya lintas | ✅ mendukung |
| §9 | Jejak audit (tidak ada butir MCP yang salah) | Inventaris "7 dokumen ~900 baris" perlu diperbarui (bagian 4) | ⚠️ usul |
| §10 | Status konfirmasi; agent7 belum tercatat | Dokumen ini = catatan resmi | ✅ kini tercatat |

## 3. Pernyataan khusus pilar (untuk menghindari salah-kait saat sign-off)

1. **Gate MCP tidak memblokir L1–L4** (INTI). MCP = `[N]`; kalau K-8 = di luar MVP, pilar MCP tertunda utuh tanpa menyentuh INTI — desain siap di Wave 1 kapan pun.
2. **I18N-40 (P3-05, ambang <500KB belum diukur) TIDAK mengikat sisi resource MCP**: resource MCP menyajikan SATU bahasa per request (≤500 token inti per bahasa — tokenizer #419a); bundle 40 bahasa <500KB = ranah storage agent2. Kesalahan mengaitkan P3-05 ke MCP akan memblokir hal yang tak terkait.
3. **Prinsip recommender ≠ approver**: gate A7-* / MCP-* / H-* / I-* diusulkan agent7 → agent7 TIDAK akan mengesahkannya sendirian; review agent1 (engine), agent5 (QA/keamanan), matt (rakitan) adalah syarat (D-7). Konsisten dengan sikap agent10 #628 dan matt §10.
4. **W1-MCP-TRANS/TOOLS sudah DONE level rencana** — artefak `AGENT7-W1-MCP-TRANSPORT-PLAN.md` + `AGENT7-W1-MCP-TOOLS-PLAN.md` siap dieksekusi hari pertama pasca-pembukaan; kontrak facade & routing sudah disepakati agent1 (#503, #550) dan agent3 (#610) — risiko eksekusi rendah.

## 4. Dua usulan perbaikan (non-blocking)

| # | Usulan | Detail | Pemilik |
|---|---|---|---|
| L5-MCP | Tambah butir MCP di L5 (atau catat eksplisit gate MCP diverifikasi di task Wave-1) | Usul: `L5-MCP`: 5 tools + 13 URI resolve + `?lang=` fallback lulus MCP Inspector (A7-1), payload ≤500 token/resource (A7-3), uji negatif eksekusi-via-MCP 20/20 (MCP-11/H-3), redaksi oracle 0-leak (A7-6) — dieksekusi di task W1-MCP-* (sudah DONE rencana; pengukuran saat implementasi) | matt (rakitan) + agent1/5 (review) |
| Inventaris | §2.2/§1 tabel "7 dokumen ~900 baris" utk MCP sudah bertambah | Per 07:40: **10 dokumen AGENT7** (induk v0.4 `9a30e459`, ROSTER `df912fc8`, REGISTRY v0.4 `f449e309`, THREATMODEL `210dda56`, PRD3-INPUT `65624daa`, HUB-PROPOSAL v0.2 `3ad9029f`, I18N v0.2 `0933ddc7`, 2× W1 plan `98d0f827`/`08ecd4b3`, alignment ini) + **3 fondasi** (AGENT1-MCP-SKILL-SPEC, AGENT4-MCP-KNOWLEDGE-SPEC, AGENT3-EXPRESSION-RULES-MCP). Boleh diperbarui saat revisi berikutnya | matt |

## 5. Referensi silang dokumen MCP (induk desain)

`AGENT7-MCP-INTEGRATION-SPEC.md` v0.4 (D-1…D-7 dikunci) · `AGENT7-PROTOCOL-ROSTER.md` (adapter ganda) · `AGENT7-JIT-ROUTING-REGISTRY.md` v0.4 (13 URI + owner) · `AGENT7-MCP-SESSION-THREATMODEL.md` (T-1…T-9) · `AGENT7-WORKFLOWHUB-MCP-PROPOSAL.md` v0.2 · `AGENT7-MCP-I18N-RESOURCE.md` v0.2 · `AGENT7-W1-MCP-TRANSPORT-PLAN.md` (DONE) · `AGENT7-W1-MCP-TOOLS-PLAN.md` (DONE).

*Ditulis oleh agent7 (ROLE_AI_MCP) sebagai catatan resmi keselarasan PRD-3 §10.*
