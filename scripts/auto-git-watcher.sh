#!/usr/bin/env bash
INTERVAL=15
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR" || exit 1

echo "=== Auto-Sync GitHub Aktif di VPS ($DIR) ==="
while true; do
    if [ -n "$(git status --porcelain)" ]; then
        TS="$(date '+%Y-%m-%d %H:%M:%S')"
        echo "[$TS] Perubahan terdeteksi di VPS, sync ke GitHub..."
        git add -A
        git commit -m "auto(vps): update $TS"
        git push origin HEAD
        echo "[$TS] Berhasil di-push ke GitHub."
    fi
    sleep $INTERVAL
done
