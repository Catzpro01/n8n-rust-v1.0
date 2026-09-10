#!/usr/bin/env bash
# Gate 50h (R58 S1a, dipasang agent1 2026-09-09).
# Menolak commit bila ada berkas .rs UNTRACKED di crates/ rust-engine.
# Latar: 23.819 baris sumber tak-terlacak lolos tanpa terdeteksi (#1581, ANGKA 1
# agent10 #1603/#1605: 23.819 -> 0 dalam sejam karena kebetulan commit bersamaan,
# bukan karena gate). Keadaan-baik-karena-kebetulan hilang-karena-kebetulan.
# Dipasang di: scripts/check-freeze.sh (kanonik, versioned) + .git/hooks/pre-commit
# (salinan aktif yang SAMA PERSIS; sinkronkan keduanya bila berubah).
# Override: FREEZE_ALLOW_UNTRACKED="alasan eksplisit" (wajib non-kosong) +
# alasan WAJIB dicatat di pesan commit. Default: menolak.
set -u
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || { echo "50h: bukan repo git"; exit 1; }
cd "$REPO_ROOT" || exit 1
UNTRACKED="$(git ls-files --others --exclude-standard -- 'crates/*.rs')"
if [ -z "$UNTRACKED" ]; then
  echo "50h: OK (0 untracked .rs di crates/)"
  exit 0
fi
COUNT="$(printf '%s\n' "$UNTRACKED" | wc -l)"
if [ -n "${FREEZE_ALLOW_UNTRACKED:-}" ]; then
  echo "50h: OVERRIDE aktif - CATAT alasan ini di pesan commit: $FREEZE_ALLOW_UNTRACKED"
  echo "50h: melewatkan $COUNT berkas untracked:"
  printf '%s\n' "$UNTRACKED"
  exit 0
fi
echo "50h: GAGAL - $COUNT berkas .rs untracked di crates/. Commit DITOLAK:"
printf '%s\n' "$UNTRACKED"
echo "50h: git-add dulu, atau set FREEZE_ALLOW_UNTRACKED=\"alasan\" + tulis alasan di pesan commit."
exit 1
