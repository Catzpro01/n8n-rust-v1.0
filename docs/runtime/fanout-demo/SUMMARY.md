# parallel-fanout — 2026-09-10 19:19:56

- manifest: `scripts/fanout/manifest.example`
- outdir: `/home/user/n8n-rust-v1.0/docs/runtime/fanout-demo`
- jobs: 4 · timeout: 300s/tugas

| task | status | exit | durasi (s) | log |
|---|---|---|---|---|
| census-arsip | OK | 0 | 0.20 | [census-arsip.log](census-arsip.log) |
| kualitas-todo | OK | 0 | 0.64 | [kualitas-todo.log](kualitas-todo.log) |
| drift-skill | OK | 0 | 0.10 | [drift-skill.log](drift-skill.log) |
| inventaris-rust | OK | 0 | 0.17 | [inventaris-rust.log](inventaris-rust.log) |
| higiene-rahasia | OK | 0 | 0.42 | [higiene-rahasia.log](higiene-rahasia.log) |

**5 tugas · 5 OK · 0 gagal/timeout · jumlah durasi worker 1.53s · wall-clock 0.64s (jobs=4)**
