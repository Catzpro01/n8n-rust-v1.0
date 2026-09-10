# Matt Pocock Skills — Sync Manifest

Catatan sinkronisasi skill `mattpocock/skills` (upstream) ke repo ini.
Diperbarui setiap kali ada download/sync ulang dari GitHub.

## Status terakhir

- **Upstream:** https://github.com/mattpocock/skills
- **Release terakhir:** v1.2.3 (2026-08-06)
- **Main HEAD saat sync:** `3cca18b368ae95cdbdebbff572ccafa662551015`
  ("Merge pull request #1025 — link-skills: stop linking misc/")
- **Tanggal sync:** 2026-09-10
- **Jumlah skill upstream:** 37 (engineering 18, in-progress 8, misc 4, productivity 7)
- **Skill custom repo ini:** 36 (tidak ikut sync, tidak pernah dioverwrite)

## Lokasi di repo ini

| Lokasi | Isi |
|---|---|
| `agent-workspace/skills-mattpocock/` | Salinan penuh repo upstream (163 file, byte-identical dengan main HEAD di atas) |
| `.claude/skills/<nama>/` | 37 skill Matt Pocock (flattened) + 36 skill custom = 73 total |
| `.agents/skills/<nama>/` | Mirror dari `.claude/skills` (73 total, identik) |
| `workspace matt/skills-mattpocock` | Gitlink → commit upstream `3cca18b` (direktori kerja kosong, referensi saja) |

## Daftar 37 skill upstream (disync)

ask-matt, claude-handoff, code-review, codebase-design, diagnosing-bugs,
domain-modeling, git-guardrails-claude-code, grill-me, grill-with-docs,
grilling, handoff, implement, implement-spec, improve-codebase-architecture,
loop-me, migrate-to-shoehorn, prototype, research, resolving-merge-conflicts,
retro, scaffold-exercises, setup-matt-pocock-skills, setup-pre-commit,
setup-ts-deep-modules, tdd, teach, to-questionnaire, to-spec, to-tickets,
triage, wait-what, wayfinder, wizard, writing-beats, writing-for-agents,
writing-fragments, writing-shape

## Daftar 36 skill custom (JANGAN dioverwrite saat sync)

brainstorming, context7-cli, context7-mcp, dispatching-parallel-agents,
executing-plans, finishing-a-development-branch, govern-pr, graphify,
llm-council, mem-search, prompt-master, receiving-code-review,
release-readiness, requesting-code-review, runtime-governance-review, strix,
strix-verify, strix-wire, subagent-driven-development, systematic-debugging,
task-observer, test-driven-development, understand, understand-chat,
understand-dashboard, understand-diff, understand-domain, understand-explain,
understand-figma, understand-knowledge, understand-onboard,
using-git-worktrees, using-superpowers, verification-before-completion,
writing-plans, writing-skills

## Cara update ke versi terbaru (berikutnya)

```bash
rm -rf /tmp/mattpocock-latest
git clone --depth 50 https://github.com/mattpocock/skills /tmp/mattpocock-latest
# Lalu salin tiap skill upstream ke .claude/skills/<nama>/ dan .agents/skills/<nama>/
# (hanya 37 nama di daftar atas — skill custom jangan disentuh),
# refresh agent-workspace/skills-mattpocock/, verifikasi dengan diff -rq,
# update manifest ini + gitlink bila HEAD berubah.
```
