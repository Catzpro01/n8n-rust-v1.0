#!/usr/bin/env bash
# worktree-fanout.sh — Fan-out tugas yang MENULIS FILE, masing-masing di git worktree terpisah.
#
# Ini lapisan di atas parallel-fanout.sh: tiap worker dapat worktree + branch sendiri
# (fanout/<stamp>/<nama>), jadi 5 worker bisa mengedit kode bersamaan tanpa saling menimpa.
# Setiap worktree yang berubah di-auto-commit di branch-nya; hasilnya tinggal direview/di-merge.
#
# Pemakaian:
#   scripts/fanout/worktree-fanout.sh <manifest> [--jobs N] [--timeout SEC] [--base REF]
#                                     [--outdir DIR] [--keep] [--dry-run]
#
# Manifest: sama seperti parallel-fanout.sh → `nama | perintah` (satu baris per tugas).
#   Perintah dijalankan DENGAN cwd = worktree miliknya. Jangan pakai '}' di dalam perintah.
#
# Hasil:
#   $OUTDIR/<nama>.log        stdout+stderr worker (via parallel-fanout.sh)
#   $OUTDIR/<nama>.diffstat   ringkasan perubahan worktree tsb
#   $OUTDIR/SUMMARY.md        tabel status
#   branch fanout/<stamp>/<nama>   (tetap ada, menunggu review manusia)
#
# Worktree dibuat di /tmp (di luar repo) dan dihapus lagi di akhir — kecuali --keep.
# Skrip ini TIDAK push dan TIDAK merge ke branch kerja Anda. Keputusan tetap milik manusia.
#
# Exit code: 0 semua sukses · 1 ada worker gagal/timeout · 2 salah pemakaian.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FANOUT="$HERE/../parallel-fanout.sh"

MANIFEST=""; JOBS=""; TIMEOUT="${FANOUT_DEFAULT_TIMEOUT:-600}"; BASE="HEAD"; OUTDIR=""; KEEP=0; DRY=0

die() { printf 'ERROR: %s\n' "$*" >&2; exit 2; }
usage() { sed -n '2,26p' "$0" | sed 's/^# \{0,1\}//'; exit "${1:-0}"; }

while [ $# -gt 0 ]; do
  case "$1" in
    -j|--jobs)    JOBS="${2:?}"; shift 2 ;;
    -t|--timeout) TIMEOUT="${2:?}"; shift 2 ;;
    -b|--base)    BASE="${2:?}"; shift 2 ;;
    -o|--outdir)  OUTDIR="${2:?}"; shift 2 ;;
    -k|--keep)    KEEP=1; shift ;;
    -n|--dry-run) DRY=1; shift ;;
    -h|--help)    usage 0 ;;
    *) [ -z "$MANIFEST" ] || die "manifest hanya boleh satu"; MANIFEST="$1"; shift ;;
  esac
done

[ -n "$MANIFEST" ] || usage 2
[ -f "$MANIFEST" ] || die "manifest tidak ditemukan: $MANIFEST"
[ -x "$FANOUT" ] || [ -f "$FANOUT" ] || die "tidak menemukan $FANOUT"
git rev-parse --show-toplevel >/dev/null 2>&1 || die "bukan repo git"

REPO="$(git rev-parse --show-toplevel)"
STAMP="$(date '+%Y%m%d-%H%M%S')"
BASESHA="$(git -C "$REPO" rev-parse "$BASE")" || die "base ref tidak valid: $BASE"
WTROOT="${FANOUT_WTROOT:-/tmp/fanout-wt/$STAMP-$RANDOM}"
[ -n "$OUTDIR" ] || OUTDIR="$REPO/.fanout-worktrees/$STAMP"
mkdir -p "$OUTDIR" || die "tidak bisa membuat outdir: $OUTDIR"
OUTDIR="$(cd "$OUTDIR" && pwd)"

# ---- parse manifest (aturan sama dengan parallel-fanout.sh) -----------------
NAMES=(); CMDS=(); lineno=0
while IFS= read -r line || [ -n "$line" ]; do
  lineno=$((lineno + 1))
  case "$line" in ''|'#'*) continue ;; esac
  if [[ "$line" == *"|"* ]];    then name="${line%%|*}"; cmd="${line#*|}"
  elif [[ "$line" == *$'\t'* ]]; then name="${line%%$'\t'*}"; cmd="${line#*$'\t'}"
  else die "baris $lineno tidak valid: $line"; fi
  name="${name#"${name%%[![:space:]]*}"}"; name="${name%"${name##*[![:space:]]}"}"
  cmd="${cmd#"${cmd%%[![:space:]]*}"}";    cmd="${cmd%"${cmd##*[![:space:]]}"}"
  [ -n "$name" ] && [ -n "$cmd" ] || die "baris $lineno: nama/perintah kosong"
  [[ "$name" =~ ^[A-Za-z0-9._-]+$ ]] || die "baris $lineno: nama '$name' hanya boleh [A-Za-z0-9._-]"
  case "$cmd" in *'}'*) die "baris $lineno: karakter '}' tidak boleh dipakai di perintah";; esac
  NAMES+=("$name"); CMDS+=("$cmd")
done < "$MANIFEST"
[ "${#NAMES[@]}" -gt 0 ] || die "manifest kosong"

printf '== worktree-fanout ==\nrepo   : %s\nbase   : %s (%s)\nworktree: %s\ntugas  : %d\noutdir : %s\n\n' \
  "$REPO" "$BASE" "${BASESHA:0:12}" "$WTROOT" "${#NAMES[@]}" "$OUTDIR"

# ---- siapkan worktree + manifest turunan ------------------------------------
GEN="$(mktemp "${TMPDIR:-/tmp}/fanout-gen.XXXXXX")"
CREATED=()
cleanup_worktrees() {
  local i=0
  while [ "$i" -lt "${#CREATED[@]}" ]; do
    if [ "$KEEP" = 1 ]; then
      printf '  (dibiarkan) worktree %s\n' "${CREATED[$i]}"
    else
      git -C "$REPO" worktree remove --force "${CREATED[$i]}" 2>/dev/null || true
    fi
    i=$((i + 1))
  done
}
trap 'cleanup_worktrees' EXIT

i=0
while [ "$i" -lt "${#NAMES[@]}" ]; do
  name="${NAMES[$i]}"; cmd="${CMDS[$i]}"; wt="$WTROOT/$name"
  br="fanout/$STAMP/$name"
  printf '  + worktree %-24s → %s (%s)\n' "$name" "$wt" "$br"
  if [ "$DRY" = 0 ]; then
    git -C "$REPO" worktree add -q -b "$br" "$(realpath -m "$wt")" "$BASESHA" \
      || die "gagal membuat worktree untuk $name"
  fi
  CREATED+=("$(realpath -m "$wt")")
  # perintah: cd ke worktree → jalankan → commit kalau ada perubahan → tulis diffstat
  printf "%s | cd %q && { %s ; } ; rc=\$? ; git add -A ; if ! git diff --cached --quiet ; then git -c user.name='fanout' -c user.email='fanout@local' commit -q -m 'fanout(%s): auto-commit dari worktree paralel' ; fi ; git diff --stat %s..HEAD > %q 2>&1 ; exit \$rc\n" \
    "$name" "$wt" "$cmd" "$name" "$BASESHA" "$OUTDIR/$name.diffstat" >> "$GEN"
  i=$((i + 1))
done
printf '\n'

if [ "$DRY" = 1 ]; then
  echo "--- manifest turunan (yang akan dieksekusi):"; cat "$GEN"
  rm -f "$GEN"; exit 0
fi

# ---- jalankan paralel -------------------------------------------------------
ARGS=(--timeout "$TIMEOUT" --outdir "$OUTDIR")
[ -n "$JOBS" ] && ARGS+=(--jobs "$JOBS")
bash "$FANOUT" "$GEN" "${ARGS[@]}"
RC=$?
rm -f "$GEN"

# ---- laporan ----------------------------------------------------------------
printf '\n== branch hasil (worktree dihapus%s) ==\n' "$([ "$KEEP" = 1 ] && echo ' — KECUALI --keep')"
i=0
while [ "$i" -lt "${#NAMES[@]}" ]; do
  name="${NAMES[$i]}"; br="fanout/$STAMP/$name"
  printf '\n### %s\n' "$br"
  git -C "$REPO" log --oneline -1 "$br" 2>/dev/null || echo "  (tidak ada commit)"
  if [ -s "$OUTDIR/$name.diffstat" ]; then sed 's/^/  /' "$OUTDIR/$name.diffstat"; else echo "  (tidak ada perubahan file)"; fi
  printf '  review: git diff %s..%s\n' "${BASESHA:0:8}" "$br"
  i=$((i + 1))
done
printf '\nLangkah berikutnya (keputusan Anda, bukan otomatis):\n'
printf '  - lihat diff satu per satu, baru cherry-pick/merge yang lolos review\n'
printf '  - hapus branch yang tidak dipakai: git branch -d fanout/%s/<nama>\n' "$STAMP"
printf '  - hapus semua: git branch | grep "fanout/%s/" | xargs -r git branch -d\n' "$STAMP"
printf '\nlog & status: %s\n' "$OUTDIR"

exit "$RC"
