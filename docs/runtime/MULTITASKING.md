# MULTITASKING — cara agent tunggal menjalankan banyak pekerjaan sekaligus

> Konteks: sesi ini berjalan sebagai **satu agent** (satu percakapan, satu context window, satu
> giliran belajar sekaligus). Dokumen ini menjawab: *"bagaimana caranya biar bisa multi-tasking?"*
> — dan menyertakan harness yang sudah terpasang di repo ini beserta bukti jalannya.
>
> Status: 2026-09-10 · harness: `scripts/parallel-fanout.sh`, `scripts/fanout/worktree-fanout.sh`

---

## 0. TL;DR

Agent tunggal **tidak bisa** menjadi dua pikiran sekaligus — tapi bisa menjadi **satu kordinator
dengan banyak proses anak** yang jalan bersamaan. Itu paralelisme nyata: wall-clock turun, bukan
ilusi. Empat tuasnya, dari termurah:

| Level | Nama | Yang sebenarnya terjadi | Cocok untuk | Sudah aktif? |
|---|---|---|---|---|
| **L0** | Multi tool-call per respons | beberapa perintah jalan **bersamaan** dalam satu giliran saya | baca/scan/riset independen, 2–8 task | ya, gratis (default saya) |
| **L1** | Background process | proses hidup **melampaui** giliran saya (server, daemon, watcher) | server dev, watcher, SSE hub, polling | ya (`start_process`) |
| **L2** | Process fan-out | N perintah independen dijalankan paralel dengan batas konkurensi + agregasi hasil | scan repo, audit multi-domain, test/benchmark terpisah | **ya — harness baru di repo ini** |
| **L3** | Worktree fan-out | N worker **menulis file** di git worktree + branch sendiri, lalu di-merge setelah review | refactor paralel, beberapa modul sekaligus, spike | **ya — harness baru di repo ini** |
| **L4** | Agent LLM sungguhan (sub-agent) | beberapa **context window AI** terpisah, masing-masing punya penilaian sendiri | spec→tiket→implement paralel, review lintas model | **belum** — butuh CLI/API key (lihat §4) |

Aturan tunggal yang mengatur semuanya: **paralel hanya untuk tugas yang tidak berbagi state.**
Sisanya serial. Memaksa paralel pada pekerjaan yang berbagi file/urutan = korupsi data + hasil
tidak bisa dipercaya.

---

## 1. Kenyataan arsitektur (jangan dibohongi)

1. **Satu context window.** Saya tidak "lupa" karena paralel — saya hanya bisa *membaca* hasil
   beberapa worker di akhir. Worker tidak boleh butuh percakapan kita untuk bekerja; tiap tugas
   harus **self-contained** (persis prinsip skill `dispatching-parallel-agents`).
2. **Paralelisme = proses, bukan pikiran.** Yang paralel adalah *shell, build, test, HTTP call*.
   Pengambilan keputusan tetap milik saya, satu per satu.
3. **Saya tetap kordinator + integrator.** Hasil fan-out harus saya agregasi, verifikasi, dan
   rangkai — kalau tidak, yang terjadi bukan multi-tasking melainkan *spaghetti of processes*.
4. **Repo ini tetap punya satu branch kerja** (`arena/01a08cc1-n8n-rust-v1-0`). Worker paralel
   dilarang commit ke branch itu; yang menulis file wajib lewat worktree (L3).

---

## 2. L0 & L1 — yang sudah jalan sejak awal

**L0: banyak tool-call dalam satu respons.** Semua panggilan tool yang tidak saling bergantung
dikirim dalam SATU blok → dieksekusi bersamaan. Ini 90% kebutuhan "multi-tasking" sehari-hari
(baca 5 file sekaligus, jalankan 3 grep berbeda, cek git + cek log bersamaan). Antipattern:
memanggilnya satu-satu di respons terpisah — hasilnya sama tapi kalah cepat.

**L1: proses latar belakang** lewat `start_process` (bind `0.0.0.0`, tampil sebagai LIVE PREVIEW).
Pakai untuk: dev server, watcher, daemon, server SSE/MCP. Proses ini hidup **lintas giliran**,
jadi saya bisa lanjut mengerjakan hal lain sementara server jalan. Repo ini sudah punya pola
daemon-nya sendiri: `agent-workspace/scripts/mcp_arena_gateway.py` (MCP/SSE hub + `comm.db`),
`scripts/auto-git-watcher.sh` (auto-commit tiap 15 detik).
⚠️ Jangan pakai `bash` untuk proses panjang — kena timeout; itu tepatnya pemicu kegagalan di repo ini.

---

## 3. L2 & L3 — harness di repo ini

### 3.1 `scripts/parallel-fanout.sh` (baca/jalan; tidak menulis file bersama)

```bash
scripts/parallel-fanout.sh <manifest> [--jobs N] [--timeout SEC] [--outdir DIR] [--dry-run]
```

Manifest = satu tugas per baris, `nama | perintah`:

```text
# scripts/fanout/manifest.example (potongan)
census-arsip    | find . -path ./.git -prune -o -type f -print | wc -l
kualitas-todo   | grep -rIn --exclude-dir=.git -I -E '(TODO|FIXME)[: ]' . | wc -l
drift-skill     | comm -3 <(ls .claude/skills|sort) <(ls .agents/skills|sort)
inventaris-rust | find agent-workspace -name '*.rs' -not -path '*/target/*' -print0 | xargs -0 cat | wc -l
```

Yang dikerjakan harness: batas konkurensi (`--jobs`, default `min(4, vCPU)`), timeout per tugas
(`--timeout`, proses dibunuh `TERM`→`KILL`), log per tugas, `.status` per tugas, lalu
`SUMMARY.tsv` + `SUMMARY.md`. Exit code `1` kalau ada worker gagal/timeout — **tidak ada kegagalan
yang disembunyikan**.

Bukti ukur (dijalankan di sandbox ini, 2 vCPU):

| Uji | Tugas | Jumlah durasi worker | Wall-clock | Kelipatan |
|---|---|---|---|---|
| smoke (`sleep`, 1 sengaja gagal) | 4 task, `--jobs 3` | 6.07 s | **3.05 s** | 2.0× |
| smoke (ulang) | 4 task, `--jobs 2` | 6.04 s | **4.04 s** | 1.5× |
| contoh repo (`manifest.example`) | 5 task, `--jobs 4` | 1.53 s | **0.64 s** | 2.4× |

Artefak nyata: `docs/runtime/fanout-demo/` (SUMMARY.md, log, `.status`) — termasuk temuan
`drift-skill`: `.claude/skills` dan `.agents/skills` **sama-sama 73 skill, nol drift** (bagus),
dan `higiene-rahasia`: 46 baris cocok pola kredensial (perlu triase; isi tidak dicetak).

### 3.2 `scripts/fanout/worktree-fanout.sh` (worker yang MENULIS file)

```bash
scripts/fanout/worktree-fanout.sh <manifest> [--jobs N] [--base REF] [--keep]
```

Alurnya: tiap task dapat **worktree sendiri di `/tmp`** + branch `fanout/<stamp>/<nama>` dari
`--base` (default `HEAD`) → perintah dijalankan dengan cwd = worktree itu → kalau ada perubahan,
auto-commit **di branch worktree-nya sendiri** → `git diff --stat` ditulis ke
`$OUTDIR/<nama>.diffstat` → worktree dihapus (kecuali `--keep`). Branch sengaja **dibiarkan** untuk
Anda review.

Uji nyata: 3 worker (dua di antaranya sama-sama menulis `demo/a.txt` / `demo/b.txt`) berjalan
bersamaan tanpa saling menimpa; yang gagal (`exit 5`) tidak menghasilkan commit dan tetap
terlapor `FAIL`. Branch smoke-test itu sudah saya bersihkan setelah uji.

**Skrip ini tidak push dan tidak merge.** Keputusan integrasi tetap milik manusia — selaras dengan
`finishing-a-development-branch` dan guardrail repo (`git push`, `reset --hard`, `branch -D` = terlarang).

---

## 4. L4 — sub-agent LLM sungguhan (belum aktif di sandbox ini)

Untuk benar-benar "menjadi beberapa agent", tiap worker butuh **model sendiri**, bukan cuma proses.
Dua jalur:

**(a) CLI agent headless** (Claude Code, Codex CLI, dsb.) — dijalankan dari manifest yang sama:

```bash
# pola, bukan perintah yang sudah saya jalankan
implement-a | cd $(git worktree root) && claude -p "$(cat .fanout/task-a.md)" --output-format json > out/a.json
```

**(b) API langsung** (Anthropic/OpenAI/Gemini) via `curl`, key dari environment variable
(`.env*` sudah di-gitignore; jangan pernah tempel key di chat/commit).

Kondisi sandbox saat ini: **belum ada** CLI agent terpasang (`command -v claude codex gemini` →
kosong) dan **belum ada API key** di environment. Jaringan ke npm/GitHub terbuka (HTTP 200), jadi
instalasi mungkin — tapi butuh **keputusan + kredensial dari Anda**. Prinsip kalau nanti dinyalakan:

- 1 sub-agent = 1 domain independen, prompt self-contained (jangan wariskan percakapan ini);
- sub-agent yang menulis kode **wajib** di worktree/branch sendiri (L3), tidak pernah langsung ke
  branch kerja sesi ini;
- hasil tiap sub-agent = **ringkasan + bukti** (test/exit code), bukan klaim;
- manusia tetap pemutus merge; biaya token per sub-agent harus disadari (ini biaya nyata).

Catatan: repo ini sudah punya cetak biru swarm di sisi VPS — `agent-workspace/scripts/mcp_arena_gateway.py`
(MCP + SSE + `comm.db` untuk koordinasi agent). Harness L2/L3 di atas adalah versi lokal-sandbox
dari gagasan yang sama.

---

## 5. Batas nyata sandbox (angka, bukan asumsi)

| Sumber daya | Nilai | Konsekuensi |
|---|---|---|
| vCPU | **2** | `--jobs` efektif ≤ 4; build berat jangan > 2 bersamaan |
| RAM | **3 GB** | hindari 3× `cargo build` / bundler besar sekaligus (OOM) |
| Toolchain | rust **tidak ada** (`cargo` kosong) | task `cargo test` di manifest akan `FAIL` — jalankan yang tersedia saja (bash/node/python3) |
| Disk/repo | 283 MB, 15 365 file, 1 363 `.md` | scan penuh berjalan ±1 detik; aman |
| Kredensial | tidak ada API key | L4 tidak bisa dinyalakan sepihak |
| Output antar-worker | tercampur kalau semua menulis stdout | wajib: tulis hasil ke file per worker; log terpisah sudah disediakan harness |

---

## 6. Aturan keputusan — kapan JANGAN paralel

**Paralel (aman):** tugas berbeda file/direktori · read-only scan · test suite terpisah ·
riset/HTTP call independen · beberapa worktree fitur yang belum bertemu.

**Serial (wajib):** pekerjaan yang berbagi file yang sama · langkah yang punya urutan sebab-akibat
(migrasi DB → verifikasi) · refactor lintas-modul yang mengubah kontrak bersama · apapun yang
butuh keputusan saya di tengah · operasi ireversibel (push, merge, publish, delete).

Pemicu berhenti dan tanya manusia (dari `subagent-driven-development`): operasi destruktif/ireversibel,
tindakan sensitif keamanan, side-effect di luar worktree, atau plan yang sudah rusak total.

---

## 7. Protokol eksekusi (checklist yang saya pakai)

1. **Pecah** pekerjaan jadi task yang benar-benar independen; beri nama `[A-Za-z0-9._-]`.
2. **Tulis manifest** di `scripts/fanout/` (atau `.fanout/` lokal) — tiap perintah self-contained.
3. **`--dry-run` dulu** untuk melihat apa yang akan jalan (dan berapa worker).
4. **Jalankan** dengan `--jobs` sesuai anggaran (§5) dan `--timeout` yang realistis.
5. **Agregasi + verifikasi**: baca `SUMMARY.md`, periksa log yang `FAIL`/`TIMEOUT`, dan **jangan
   klaim apa pun tanpa bukti** (skill `verification-before-completion`).
6. **Integrasi**: diff worktree/branch satu per satu → merge/cherry-pick yang lolos review →
   hapus yang tidak dipakai.

Antipattern yang harus dihindari: 8 worker menyentuh satu file; paralel tanpa timeout (worker
menggantung memblokir slot); hasil hanya di stdout lalu tidak tercatat; "semua hijau" dari
`SUMMARY.md` tanpa membaca log worker; commit worker langsung ke branch kerja sesi ini.

---

## 8. Cheat sheet

```bash
# 1) lihat rencana tanpa menjalankan apa pun
scripts/parallel-fanout.sh scripts/fanout/manifest.example --dry-run

# 2) fan-out read-only (repo ini: 5 task ≈ 0.6 detik)
scripts/parallel-fanout.sh scripts/fanout/manifest.example --jobs 4 --timeout 300

# 3) fan-out yang menulis file (worktree + branch per worker, auto-cleanup)
scripts/fanout/worktree-fanout.sh .fanout/task.md --jobs 4 --base HEAD

# 4) lihat hasil & bukti
cat .fanout/<stamp>/SUMMARY.md
git branch --list 'fanout/*'
```

## 9. Kalau mau naik level

- **L4 (sub-agent LLM)**: Anda putuskan mau CLI mana / API key mana; saya siapkan manifest-nya,
  worktree isolation, agregasi, dan review. Ini yang paling dekat dengan "multi-agent swarm" nyata.
- **L1 daemon antrean**: proses latar yang mem-poll `.fanout/queue/*.md` sehingga pekerjaan jalan
  bahkan di antara giliran saya (pola `mcp_arena_gateway.py` diperluas).
- **Port ke VPS**: `scripts/auto-git-watcher.sh` + gateway MCP yang sudah ada bisa jadi kordinator
  swarm lintas mesin — di luar sandbox ini, butuh keputusan dan akses Anda.

Related: `.claude/skills/dispatching-parallel-agents/SKILL.md`, `.claude/skills/subagent-driven-development/SKILL.md`,
`.claude/skills/using-git-worktrees/SKILL.md`, `docs/agents/skill-shortcuts.md`.
