#!/usr/bin/env bash
# parallel-fanout.sh — Jalankan banyak worker INDEPENDEN secara paralel dari sebuah manifest.
#
# Dipakai bila satu agent (single context window) ingin "multi-tasking":
# fan-out N tugas yang tidak berbagi state, lalu agregasi hasilnya di akhir.
#
# Pemakaian:
#   scripts/parallel-fanout.sh <manifest> [--jobs N] [--timeout SEC] [--outdir DIR] [--dry-run]
#
# Format manifest (satu tugas per baris):
#   nama | perintah shell
#   # komentar (diawali #) dan baris kosong diabaikan
#   nama<TAB>perintah   (TAB juga diterima sebagai pemisah)
#
# Aturan wajib:
#   - Tiap worker HARUS independen (tidak menulis file yang sama).
#   - Worker yang mengubah file: pakai git worktree terpisah (lihat SKILL.md).
#   - Tulis hasil ke file di $OUTDIR, jangan hanya ke stdout (stdout tetap di-log).
#
# Output:
#   $OUTDIR/<nama>.log      stdout+stderr worker
#   $OUTDIR/<nama>.status   "<exit_code>\t<durasi_detik>\t<STATUS>"
#   $OUTDIR/SUMMARY.tsv     ringkasan mesin (task, status, exit, durasi, log)
#   $OUTDIR/SUMMARY.md      ringkasan manusia + wall-clock
#
# Exit code: 0 semua sukses · 1 ada worker gagal/timeout · 2 salah pemakaian.
set -uo pipefail

MANIFEST=""
JOBS=""
TIMEOUT="${FANOUT_DEFAULT_TIMEOUT:-600}"
OUTDIR=""
DRY_RUN=0

die() { printf 'ERROR: %s\n' "$*" >&2; exit 2; }

usage() {
  sed -n '2,32p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

while [ $# -gt 0 ]; do
  case "$1" in
    -j|--jobs)    JOBS="${2:?}"; shift 2 ;;
    -t|--timeout) TIMEOUT="${2:?}"; shift 2 ;;
    -o|--outdir)  OUTDIR="${2:?}"; shift 2 ;;
    -n|--dry-run) DRY_RUN=1; shift ;;
    -h|--help)    usage 0 ;;
    --) shift; break ;;
    -*) die "opsi tidak dikenal: $1 (pakai --help)" ;;
    *)  [ -z "$MANIFEST" ] || die "manifest hanya boleh satu"; MANIFEST="$1"; shift ;;
  esac
done

[ -n "$MANIFEST" ] || usage 2
[ -f "$MANIFEST" ] || die "manifest tidak ditemukan: $MANIFEST"
command -v timeout >/dev/null || die "'timeout' tidak tersedia"

# Anggaran paralel default: min(4, vCPU) — sandbox ini 2 vCPU / 3 GB RAM.
if [ -z "$JOBS" ]; then
  CPU="$( (command -v nproc >/dev/null && nproc) || echo 2 )"
  JOBS="$CPU"; [ "$JOBS" -gt 4 ] && JOBS=4; [ "$JOBS" -lt 1 ] && JOBS=1
fi
case "$JOBS"    in ''|*[!0-9]*) die "--jobs harus angka" ;; esac
case "$TIMEOUT" in ''|*[!0-9]*) die "--timeout harus angka (detik)" ;; esac

STAMP="$(date '+%Y%m%d-%H%M%S')"
[ -n "$OUTDIR" ] || OUTDIR=".fanout/$STAMP"
mkdir -p "$OUTDIR" || die "tidak bisa membuat outdir: $OUTDIR"
OUTDIR="$(cd "$OUTDIR" && pwd)"

# ---- parse manifest ---------------------------------------------------------
NAMES=(); CMDS=(); declared=0; lineno=0
while IFS= read -r line || [ -n "$line" ]; do
  lineno=$((lineno + 1))
  case "$line" in ''|'#'*) continue ;; esac
  if [[ "$line" == *"|"* ]]; then           # pemisah: '|' PERTAMA (spasi di kiri/kanan bebas)
    name="${line%%|*}"; cmd="${line#*|}"
  elif [[ "$line" == *$'\t'* ]]; then       # alternatif: TAB
    name="${line%%$'\t'*}"; cmd="${line#*$'\t'}"
  else
    die "baris $lineno tidak valid (butuh 'nama | perintah'): $line"
  fi
  name="${name#"${name%%[![:space:]]*}"}"; name="${name%"${name##*[![:space:]]}"}"   # trim
  cmd="${cmd#"${cmd%%[![:space:]]*}"}";    cmd="${cmd%"${cmd##*[![:space:]]}"}"
  [ -n "$name" ] && [ -n "$cmd" ] || die "baris $lineno: nama/perintah kosong"
  [[ "$name" =~ ^[A-Za-z0-9._-]+$ ]] || die "baris $lineno: nama '$name' hanya boleh [A-Za-z0-9._-]"
  if [ "${#NAMES[@]}" -gt 0 ]; then
    for n in "${NAMES[@]}"; do [ "$n" = "$name" ] && die "nama duplikat: $name"; done
  fi
  NAMES+=("$name"); CMDS+=("$cmd"); declared=$((declared + 1))
done < "$MANIFEST"

[ "$declared" -gt 0 ] || die "manifest kosong (tidak ada tugas)"

printf '== parallel-fanout ==\nmanifest : %s\noutdir   : %s\ntugas    : %d\njobs     : %d\ntimeout  : %ss/tugas\n\n' \
  "$MANIFEST" "$OUTDIR" "$declared" "$JOBS" "$TIMEOUT"

if [ "$DRY_RUN" = 1 ]; then
  i=0
  while [ "$i" -lt "$declared" ]; do printf '  [%s] %s\n' "${NAMES[$i]}" "${CMDS[$i]}"; i=$((i + 1)); done
  exit 0
fi

# ---- worker -----------------------------------------------------------------
run_one() {  # $1=nama  $2=perintah
  local name="$1" cmd="$2"
  local log="$OUTDIR/$name.log" sf="$OUTDIR/$name.status"
  local t0 t1 rc dur st
  t0="$(date +%s.%N)"
  printf '# task=%s\n# cmd=%s\n# start=%s\n# ----\n' "$name" "$cmd" "$(date '+%F %T')" > "$log"
  timeout --signal=TERM --kill-after=10s "${TIMEOUT}s" bash -lc "$cmd" >>"$log" 2>&1
  rc=$?
  t1="$(date +%s.%N)"
  dur="$(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.2f", b-a}')"
  if   [ "$rc" -eq 0 ]; then st=OK
  elif [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then st=TIMEOUT
  else st=FAIL; fi
  printf '%s\t%s\t%s\n' "$rc" "$dur" "$st" > "$sf"
}

# ---- scheduler (cap concurrency, tunggu slot bebas) -------------------------
declare -A PIDNAME=()
pids=()

prune_pids() {  # buang pid yang sudah mati (dan laporkan)
  local -a keep=(); local p
  for p in "${pids[@]}"; do
    [ -n "$p" ] || continue
    if kill -0 "$p" 2>/dev/null; then
      keep+=("$p")
    else
      printf '  ✓ selesai  %-24s\n' "${PIDNAME[$p]:-?}"
      unset "PIDNAME[$p]"
    fi
  done
  pids=("${keep[@]}")
}

FANOUT_T0="$(date +%s.%N)"
i=0
while [ "$i" -lt "$declared" ]; do
  while [ "${#pids[@]}" -ge "$JOBS" ]; do
    wait -n 2>/dev/null || true
    prune_pids
    [ "${#pids[@]}" -ge "$JOBS" ] && sleep 0.2
  done
  name="${NAMES[$i]}"; cmd="${CMDS[$i]}"
  printf '  → mulai    %-24s %s\n' "$name" "$cmd"
  run_one "$name" "$cmd" &
  pids+=("$!"); PIDNAME[$!]="$name"
  i=$((i + 1))
done

for p in "${pids[@]}"; do [ -n "$p" ] && wait "$p" 2>/dev/null; done
prune_pids

WALL_TOTAL="$(awk -v a="$FANOUT_T0" -v b="$(date +%s.%N)" 'BEGIN{printf "%.2f", b-a}')"
printf '\n'

# ---- ringkasan --------------------------------------------------------------
summary_tsv="$OUTDIR/SUMMARY.tsv"
summary_md="$OUTDIR/SUMMARY.md"
printf 'task\tstatus\texit\tdurasi_detik\tlog\n' > "$summary_tsv"

total=0; ok=0; fail=0; sum_dur=0
i=0
while [ "$i" -lt "$declared" ]; do
  name="${NAMES[$i]}"; sf="$OUTDIR/$name.status"
  if [ -f "$sf" ]; then read -r rc dur st < "$sf"; else rc="-"; dur="-"; st=MISSING; fi
  printf '%s\t%s\t%s\t%s\t%s.log\n' "$name" "$st" "$rc" "$dur" "$name" >> "$summary_tsv"
  total=$((total + 1))
  case "$st" in OK) ok=$((ok + 1)) ;; *) fail=$((fail + 1)) ;; esac
  [ "$dur" != "-" ] && sum_dur="$(awk -v a="$sum_dur" -v b="$dur" 'BEGIN{printf "%.2f", a+b}')"
  i=$((i + 1))
done

{
  printf '# parallel-fanout — %s\n\n' "$(date '+%F %T')"
  printf -- '- manifest: `%s`\n- outdir: `%s`\n- jobs: %s · timeout: %ss/tugas\n\n' "$MANIFEST" "$OUTDIR" "$JOBS" "$TIMEOUT"
  printf '| task | status | exit | durasi (s) | log |\n|---|---|---|---|---|\n'
  i=0
  while [ "$i" -lt "$declared" ]; do
    name="${NAMES[$i]}"; sf="$OUTDIR/$name.status"
    if [ -f "$sf" ]; then read -r rc dur st < "$sf"; else rc="-"; dur="-"; st=MISSING; fi
    printf '| %s | %s | %s | %s | [%s.log](%s.log) |\n' "$name" "$st" "$rc" "$dur" "$name" "$name"
    i=$((i + 1))
  done
  printf '\n**%s tugas · %s OK · %s gagal/timeout · jumlah durasi worker %ss · wall-clock %ss (jobs=%s)**\n' \
    "$total" "$ok" "$fail" "$sum_dur" "$WALL_TOTAL" "$JOBS"
} > "$summary_md"

cat "$summary_md"
printf '\nlog & status: %s\n' "$OUTDIR"

[ "$fail" -eq 0 ] || exit 1
exit 0
