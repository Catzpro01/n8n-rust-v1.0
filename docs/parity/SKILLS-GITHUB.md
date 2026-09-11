# SKILL REKOMENDASI (GitHub) — untuk mengejar paritas n8n dengan hasil maksimal

> **Pertanyaan yang dijawab:** skill apa (dari GitHub) yang relevan dipakai supaya pekerjaan
> "menyamakan n8n-rust dengan n8n" hasilnya maksimal — bukan menebak, tapi punya sumber otoritatif.
> Diperiksa 2026-09-11 via GitHub API (bintang & tanggal push = saat audit).

---

## 0. Skill yang SUDAH ada di repo ini (73) dan mana yang dipakai untuk audit ini

Router repo: `docs/agents/skill-shortcuts.md`. Yang dipakai untuk tugas ini:

| Skill lokal | Dipakai untuk |
|---|---|
| `research` | ambil fakta ke **sumber primer** (repo n8n + docs.n8n.io), simpan jadi Markdown di repo → dokumen ini & `GAP-ANALYSIS.md` |
| `understand` / `understand-explain` | memetakan kode repo (crate, node, engine, UI) sebelum menilai gap |
| `verification-before-completion` | setiap klaim di audit diuji (`curl`, `grep`, hitung file) — bukan asumsi |
| `code-review` | lensa dua sumbu (standar repo + spesifikasi) untuk menilai klaim di README/ACCURACY_COMPARISON |
| `writing-plans` / `to-tickets` | mengubah hasil gap analysis jadi rencana & tiket (Fase 0–2) |
| `tdd` / `test-driven-development` | aturan saat mengeksekusi item roadmap nanti |
| `visual-regression-qa` | membandingkan tampilan editor n8n asli vs rust (bila ada screenshot/browser) |
| `ui-ux-pro-max` | merancang halaman baru (NDV, credentials, executions) agar mirip n8n |
| `grilling` / `wait-what` | menguji asumsi roadmap secara kritis sebelum investasi besar |

**Kesimpulan:** untuk *audit* sudah cukup. Yang kurang untuk *eksekusi* adalah **sumber otoritatif tentang n8n itu sendiri** — itulah 8 rekomendasi di bawah.

---

## 1. Sumber nomor satu: skill resmi di repo n8n asli ⭐ wajib

**`n8n-io/n8n` → `.agents/skills/` (22 skill)** — 203k★, dipush 2026-09-11.
n8n sendiri memakai Agent Skills lintas harness (`.agents/skills` = kanonik, `.claude/plugins/n8n/skills` = symlink, `.opencode/skills` = override).

| Skill n8n | Fungsi (dari frontmatter aslinya) | Nilai untuk repo ini |
|---|---|---|
| `public-api` | menambah/mengubah endpoint REST publik n8n (handler, mapper, spec) | **paling relevan**: cetak biru 90 endpoint yang harus disamakan |
| `spec-driven-development` | menjaga implementasi sinkron dengan spec di `.agents/specs/` | pola kerja untuk roadmap Fase 0–2 |
| `conventions` | pola kode & konvensi repo n8n (AGENTS.md) | supaya struktur node/engine kita tidak menyimpang |
| `ui-design` | pedoman membangun UI di `editor-ui` & design-system | acuan halaman baru (NDV, executions, credentials) |
| `node-add-oauth` | menambah credential OAuth2 ke node (file credential + test + konstanta CLI) | pola wajib untuk 456 credential type |
| `protect-endpoints` | memasang scope RBAC pada REST controller | desain guard endpoint kita (P0 keamanan) |
| `db-migrations` | penulisan migrasi DB n8n | pola migrasi saat pindah dari JSON file ke SQLite (141 tabel) |
| `reproduce-bug` | reproduksi bug jadi test merah | untuk celah semantik engine |
| `human-like-code-review` | review PR seperti manusia (konteks, arsitektur, bug, keamanan) | review tiap PR Fase 1 |
| `create-issue`, `create-pr`, `gh-stack` | alur tiket/PR/stack PR | housekeeping repo |
| `community-pr-readiness-check`, `create-community-node-lint-rule` | kesiapan node community | saat membuka node custom |
| `create-skill` | menulis skill baru di repo n8n | bila kita mau menulis skill internal sendiri |
| `experiments`, `telemetry`, `content-design`, `create-instance-ai-eval`, `loom-transcript`, `linear-issue`, `nathan` | khusus internal n8n (eksperimen, telemetri, eval instance AI, Linear, deploy test) | sebagian tidak relevan; `create-instance-ai-eval` berguna saat menambah evaluasi AI |

**Cara pakai:**
```bash
# salin hanya yang relevan (lisensi n8n: Sustainable Use License — cek sebelum memakai ulang kode)
gh api repos/n8n-io/n8n/contents/.agents/skills/public-api/SKILL.md -H "Accept: application/vnd.github.raw" > /tmp/skill.md
```
Simpan ke `.claude/skills/n8n-public-api/SKILL.md` + mirror di `.agents/skills/`, lalu **tambahkan baris router** di `docs/agents/skill-shortcuts.md` dan catat di `docs/agents/mattpocock-sync.md`.
Catatan lisensi: **jangan** menyalin kode n8n (Sustainable Use License), cukup **pola & checklist**-nya; skill ini internal untuk repo kita.

---

## 2. `czlonkowski/n8n-mcp` — 22.868★, push 2026-09-09 ⭐ wajib

MCP server yang menyajikan **skema & dokumentasi node n8n** (parameter, typeVersion, opsi, credential, contoh workflow) ke agent.
Ini penutup celah terbesar pada pekerjaan node: kita tidak lagi mengarang parameter — ambil dari sumber n8n.

```bash
claude mcp add n8n-mcp -- npx n8n-mcp      # atau jalankan sebagai server MCP lokal
```
Pakai saat: mengimplementasi ulang node (mis. `Gmail`, `Postgres`, `Notion`) agar nama parameter &
perilaku default sama dengan n8n, dan saat menulis **contract test** yang membandingkan node rust vs node n8n.

---

## 3. `anthropics/skills` — 175.724★, push 2026-09-10 (resmi Anthropic)

Skill yang relevan untuk repo ini (dari daftar resmi):

| Skill | Nilai |
|---|---|
| `webapp-testing` | uji UI editor (Playwright) — dasar e2e Fase 2 |
| `frontend-design` | kualitas visual halaman baru |
| `canvas-design` | khusus untuk canvas node/edge (aset & tata letak) |
| `mcp-builder` | bila kita menambah MCP server triger/klien seperti n8n 2.x |
| `skill-creator` | membuat skill internal (mis. "tambah node baru ala n8n") |
| `doc-coauthoring` | dokumentasi halaman/fitur yang rapi |
| `xlsx` / `docx` / `pdf` | bila ingin ekspor laporan audit ke format kantor |

---

## 4. `ChromeDevTools/chrome-devtools-mcp` — 51.615★, push 2026-09-11

Chrome DevTools untuk agent: snapshot DOM, console, network, screenshot.
**Kegunaan spesifik:** membandingkan **n8n asli (instance lokal/demo) vs editor rust** secara terukur —
ambil screenshot & struktur DOM yang sama, lalu catat selisih layout (sidebar 64px, header 56px, panel, DSB).
Kombinasikan dengan skill lokal `visual-regression-qa`.

## 5. `ComposioHQ/awesome-claude-skills` — 74.845★ (katalog)
+ `travisvn/awesome-claude-skills` (15.029★) + `BehiSecc/awesome-claude-skills` (10.115★).
Dipakai untuk **menemukan** skill baru; dari katalog ini yang berguna: `webapp-testing`, `mcp-builder`, `artifacts-builder`.
Alternatif katalog: `davepoon/buildwithclaude` (3.437★) — hub skill/agent/plugin.

## 6. `obra/superpowers` — 284.931★, push 2026-09-11
Sumber sebagian besar skill lokal (brainstorming, writing-plans, executing-plans, TDD, verification).
**Rekomendasi:** jadwalkan sync ulang (repo ini menyimpan manifest di `docs/agents/mattpocock-sync.md`) agar
skill yang dipakai untuk mengeksekusi roadmap tetap versi terbaru.

## 7. Sumber domain n8n (bukan skill, tapi bahan skill/docs)

| Repo | Bintang | Pakai untuk |
|---|---|---|
| `n8n-io/n8n-docs` | 1.764 | dokumentasi resmi; `.md` per halaman + `llms.txt` = bahan grounding |
| `Synaptiv-AI/awesome-n8n` | 192 | `llms.txt` lengkap hasil bersih untuk dikonsumsi agent |
| `n8n-io/n8n-nodes-starter` | 1.178 | kerangka node custom (acuan struktur node baru) |
| `nerding-io/n8n-nodes-mcp` | 3.043 | contoh node MCP di ekosistem n8n |
| `n8n-io/n8n` (repo utama) | 203.980 | sumber otoritatif perilaku: `packages/cli`, `packages/nodes-base`, `editor-ui` |

---

## 8. Cara memasang di repo ini (prosedur)

1. Taruh skill di **dua** lokasi (repo ini memelihara mirror): `.claude/skills/<nama>/SKILL.md` dan `.agents/skills/<nama>/SKILL.md`.
2. Tambahkan baris ke tabel router `docs/agents/skill-shortcuts.md` (format: `TRIGGER → skill — deliverable`).
3. Catat di `docs/agents/mattpocock-sync.md` (upstream, commit, tanggal sync) bila berasal dari repo pihak ketiga.
4. Jangan salin kode berlisensi n8n (Sustainable Use License) — ambil pola/checklist saja.
5. Uji: panggil satu kali, pastikan deliverable skill benar-benar muncul (prinsip `verification-before-completion`).

## 9. Urutan pemakaian yang disarankan

```
Audit (sudah)  : research + understand + verification-before-completion
Desain Fase 1  : n8n/.agents/skills{ui-design, public-api, protect-endpoints, conventions} + mcp-builder
Implement node : czlonkowski/n8n-mcp (skema) + n8n-nodes-starter (struktur) + node-add-oauth
Verifikasi UI  : chrome-devtools-mcp + visual-regression-qa + axe-core-a11y
Verifikasi API : n8n/.agents/skills/public-api + contract test terhadap openapi.yml n8n
Rilis          : release-readiness + strix (bila permukaan berubah)
```

**Satu kalimat:** skill lokal sudah cukup untuk *cara kerja*; yang menambah lompatan akurasi adalah
**skill & sumber dari dalam n8n sendiri** (`n8n-io/n8n` → `.agents/skills`, `n8n-mcp`, `n8n-docs`),
karena akurasi paritas hanya bisa datang dari spesifikasi pemilik fiturnya.
