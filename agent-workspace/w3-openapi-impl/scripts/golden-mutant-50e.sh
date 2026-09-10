#!/usr/bin/env bash
# golden-mutant-50e.sh — fasilitasi butir (b) Ruling 58 §7 untuk REVIEWER EMIT (agent3).
# Membuktikan golden test BISA GAGAL (menangkap drift) via mutan 50e.
# PENGGUNA: reviewer (agent3) — agent9 tidak menjalankan ini atas reviewnya sendiri.
# PRASYARAT: jalankan di rust-engine (HEAD ≥ 077b4f3), CARGO_TARGET_DIR=/mnt/extra-storage/re-cargo-target
set -euo pipefail
RE=/opt/agent-workspace/rust-engine
F=$RE/crates/nodes-openapi/src/generated/frankfurter.rs
cd "$RE"
export CARGO_TARGET_DIR=/mnt/extra-storage/re-cargo-target

echo "== 50h pra-build =="; K=/opt/agent-workspace/kernel-asli-d3bcff0; [ -z "$(git -C $K status --porcelain)" ] && echo "porcelain kernel=0 OK" || { echo "STOP: kernel kotor"; exit 1; }

echo "== baseline (sebelum mutan) =="
sha_before=$(sha256sum "$F" | cut -c1-16); echo "sha_before=$sha_before"
cargo test -p nodes-openapi --test golden 2>&1 | grep -E "^test result" | sed "s/^/  /"

echo "== PASANG MUTAN (50e: wajib sha berubah) =="
sed -i "0,/AUTO-GENERATED/s//AUTO-GENERATXD/" "$F"
sha_after=$(sha256sum "$F" | cut -c1-16); echo "sha_after=$sha_after"
[ "$sha_before" != "$sha_after" ] && echo "MUTAN TERPASANG (sha berubah) — hasil BOLEH dibaca" || { echo "NO-OP — hasil TIDAK berarti, batalkan"; exit 1; }

echo "== jalankan golden dgn mutan (HARUS FAILED + pesan drift) =="
if cargo test -p nodes-openapi --test golden 2>&1 | grep -qE "golden drift"; then
  echo ">>> MUTAN MATI dengan pesan drift = golden test TERBUKTI menangkap drift (butir-b LULUS)"
else
  echo ">>> TEMUAN: golden TIDAK menangkap mutan (butir-b GAGAL — laporkan)"
fi

echo "== pulihkan + verifikasi =="
git -C "$RE" checkout -- "$F" 2>/dev/null || cp "$F.bak" "$F"
sha_restored=$(sha256sum "$F" | cut -c1-16); echo "sha_restored=$sha_restored (harus = $sha_before)"
cargo test -p nodes-openapi --test golden 2>&1 | grep -E "^test result" | sed "s/^/  /"
