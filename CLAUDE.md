# CLAUDE.md

Proyek ini adalah **n8n-rust**: engine workflow automation Rust-native yang kompatibel dengan n8n (clean-room), ditargetkan jalan di VPS 2 CPU / 2 GB RAM / 50 GB disk. Hukum tertinggi proyek: **Correctness → Measurable Performance → Compatibility**.

## Aturan Pemilik (WAJIB, 2026-09-10)

> **"Setiap mau menjawab sesuatu, wajib menggunakan semua skill."**

Operasionalisasi wajib untuk setiap respons di repo ini:

1. **ROUTING WAJIB VIA FILE SHORTCUT — `docs/agents/skill-shortcuts.md`.** File ini SELALU berfungsi: sebelum MENJAWAB apa pun (termasuk klarifikasi), buka file shortcut itu dan cocokkan permintaan terhadap tabel trigger 65 skill aktifnya (protokol 2 detik ada di bagian terakhir file). Cocok = skillnya WAJIB dijalankan sesuai `SKILL.md`-nya. Ragu = `ask-matt`. 8 skill pasif lainnya (task-observer, using-superpowers, context7-mcp, mem-search, git-guardrails, graphify, codebase-design, writing-for-agents) jalan otomatis tanpa dipanggil.
2. **Skill yang relevan = WAJIB dijalankan.** Kalau ada skill yang menutupi tugas tersebut, ikuti `SKILL.md`-nya sampai selesai — dilarang mengerjakan manual apa yang sudah dicakup skill.
3. **Alur engineering standar** (untuk semua pekerjaan nyata, bukan chat biasa):
   - Memahami masalah → `grill-me` / `grilling` / `diagnosing-bugs`
   - Desain → `grill-with-docs`, `codebase-design`, `domain-modeling`
   - Spesifikasi & tiket → `to-spec` → `to-tickets` (issue tracker: GitHub)
   - Implementasi → `implement` + `tdd`
   - Review → `code-review`, `triage` untuk queue
   - Keamanan & governance → `strix` (pentest) + lensa Strixgov (`/runtime-governance-review`, `/govern-pr`, `/release-readiness`)
   - Prompt untuk tool AI manapun → `prompt-master`
   - Ragu skill mana → `ask-matt` (router resmi)
4. **Tidak relevan = tidak dipaksakan.** Skill hanya dijalankan bila memang relevan dengan permintaan (misal: tidak ada scan pentest untuk sapaan). Keputusan relevansi diambil dari routing poin 1, bukan dipotong-potong.

Repo ini memasang skill dari `mattpocock/skills` (37), `obra/superpowers` (14), `Egonex-AI/Understand-Anything` (9, engine di `~/understand-anything/understand-anything-plugin`), `Strixgov/skills` (5), `upstash/context7` (2: context7-cli + context7-mcp), `thedotmack/claude-mem` (1: mem-search; runtime terpasang di `~/.claude/plugins/marketplaces/thedotmack`, worker lokal port 37701, data di `~/.claude-mem`, cloud sync OFF), `akillness/jeo-skills` (strix, 1), `nidhinjs/prompt-master` (1), `YonasValentin/llm-council` (1), `rebelytics/one-skill-to-rule-them-all` (1: task-observer), `Graphify-Labs/graphify` (1, CLI `graphify` v0.9.57 via pip `graphifyy`) — semua di `skills-lock.json`, update: `npx skills update`.

Aturan aktivasi: di awal sesi kerja yang memakai tools, jalankan `task-observer` (catat pola/koreksi ke observation log); cek memori proyek lewat `mem-search` sebelum memulai tugas lanjutan; untuk pertanyaan library/framework eksternal, verifikasi via context7 (`ctx7`) — jangan andalkan training data.

Skill superpowers tambahan: alur brainstorm → plan → execute → verify; paksa TDD; `/llm-council` untuk keputusan besar ("council this"); context7 (`ctx7`) untuk dokumentasi library terkini — gunakan saat menulis kode yang memakai library eksternal.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).

## Agent skills

### Issue tracker

Issue & tiket dikelola di **GitHub Issues** (`gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

5 label default dipakai apa adanya: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Layout **single-context**: `CONTEXT.md` + `docs/adr/` di root (dibuat lazily oleh `/domain-modeling` saat istilah/keputusan benar-benar teresolusi). See `docs/agents/domain.md`.
