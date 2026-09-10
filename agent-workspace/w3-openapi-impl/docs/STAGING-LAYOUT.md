# STAGING-LAYOUT — tata letak wajib pohon lane W3-OPENAPI (agent9)

**Penyusun:** agent9 · **Tanggal:** 2026-09-09 · **Pemicu:** umpan balik agent10 (#1520 §layout + §4 — "verifier menemukan sendiri lewat build gagal") · **Status: TERBIT** (dokumentasi lane sendiri; tanpa perubahan kode)

## 1. Peta pohon & referensi kernel (WAJIB dipahami sebelum menyalin)

| Pohon | Lokasi kanonik | Referensi kernel | Syarat lokasi |
|---|---|---|---|
| **kernel-asli** (READ-ONLY) | `/opt/agent-workspace/kernel-asli-d3bcff0/` | — (ini TUJUAN referensi) | tak dipindah; HEAD saat doc ini `231e47f` |
| **rust-engine** (workspace) | `/opt/agent-workspace/rust-engine/` | root `Cargo.toml`: `kernel = { path = "../kernel-asli-d3bcff0/crates/kernel" }` | **WAJIB sejajar** — `kernel-asli-d3bcff0` harus jadi SIBLING directory |
| **W3 staging** (dev, agent9) | `/home/agent9/W3-OPENAPI-IMPL/` (server) | per-crate: `../../../../opt/agent-workspace/kernel-asli-d3bcff0/crates/kernel` | path relatif 4-naik → clamp di `/` → bekerja di kedalaman berapa pun SELAMA kernel ada di `/opt/agent-workspace/...` |
| **38a** (salinan verifikasi) | `/opt/agent-workspace/w3-openapi-impl/` | sama seperti W3 staging | idem; + `SHA256SUMS` + tree-hash diumumkan di kanal |

**Akibat menyalin tanpa syarat:** build gagal dengan error yang menunjuk crate TAK BERSALAH (mis. `workflow`/`openapi-codegen` "crate not found") — akar masalahnya berkas Cargo.toml yang TIDAK disebut error (agent10 #1520: kelas temuan #1419/#1460).

## 2. Satu-baris perintah staging resmi (untuk verifier)

```bash
# (a) verifikasi salinan 38a — butuh /opt/agent-workspace/kernel-asli-d3bcff0 ADA di mesin yang sama
D=$(mktemp -d) && cp -a /opt/agent-workspace/w3-openapi-impl "$D/w3" && cd "$D/w3" \
  && CARGO_TARGET_DIR=/mnt/extra-storage/vfy cargo test --workspace && sha256sum -c SHA256SUMS

# (b) verifikasi rust-engine (openapi-codegen lane agent9) — WAJIB bawa sibling kernel
D=$(mktemp -d) && cp -a /opt/agent-workspace/rust-engine /opt/agent-workspace/kernel-asli-d3bcff0 "$D/" \
  && cd "$D/rust-engine" && CARGO_TARGET_DIR=/mnt/extra-storage/vfy cargo test -p openapi-codegen
```

## 3. Prosedur 50h (SEBELUM setiap build kutover — matt #1478 dinaikkan)

```bash
cd /opt/agent-workspace/kernel-asli-d3bcff0 && [ -z "$(git status --porcelain)" ] || { echo "50h: kernel kotor — STOP"; exit 1; }
git log --oneline -1   # catat HEAD; path-dep menunjuk WORKING TREE, bukan commit
```

## 4. Catatan `Cargo.lock` rust-engine (delta lane agent9 — jujur dinyatakan)

Populasi `crates/openapi-codegen` menambah `sha2` ke daftar dep crate itu → `M Cargo.lock` (+1 entri dep di blok `openapi-codegen`: `kernel, serde_json, sha2`). Delta ini **konsekuensi wajar kutover** dan seharusnya **di-commit BERSAMA file lane** (saran agent10 §4 — kelas #1419/#1460): mohon otoritas commit (matt/agent1) menyertakannya agar pohon bersih dan lock tak mengambang.

## 5. Pin serde_json (menunggu putusan matt — #1515)

- rust-engine workspace: `serde_json = "=1.0.114"` · W3/38a: `"=1.0.151"`.
- Empiris: **56/56 LULUS di 1.0.114** di rust-engine (termasuk golden byte-identical frankfurter + manifest-counts github).
- Dua opsi di meja matt: (a) lane ikut workspace 1.0.114, (b) workspace naik 1.0.151. Tidak diputuskan sendiri.

## 6. Higiene umum (pelajaran tercatat)

- `CARGO_TARGET_DIR` selalu ke `/mnt/extra-storage/...` (insiden disk 100% #1413 §3).
- Tree-hash 38a hanya diumumkan SETELAH seluruh perubahan sesi selesai (#1418).
- Verifikasi terhadap sha baru: BACA diff-nya, JALANKAN semuanya (Ruling 49c).
