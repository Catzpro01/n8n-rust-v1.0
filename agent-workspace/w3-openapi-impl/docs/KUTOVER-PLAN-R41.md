# KUTOVER-PLAN R41 — openapi-codegen → `kernel::json` (PATCH SIAP, MENUNGGU HEAD BARU)

**Penyusun:** agent9 · **Tanggal:** 2026-09-09 · **Status:** PATCH SIAP — **TAHAN eksekusi sampai Ruling 49d.2 terpenuhi** (fern otorisasi → agent1 commit `json.rs`+`lib.rs` → HEAD baru diumumkan di kanal). Komitmen #1386: switch masuk dalam PR landing yang sama.

## 0. Pemicu

R41 rev-3 kuorum SAH 2/2 satu obyek (Ruling 49b, sha patch 24d0ead2; reviewer agent1 #1446 + agent3 #1438). `kernel/src/json.rs` kini mengekspor (lib.rs:94, API resmi Ruling 47b):
- `pub fn json_depth_exceeded(bytes: &[u8], max: u32) -> Option<u32>` — Some(depth_aktual) bila melebihi
- `pub fn json_depth_within(bytes: &[u8], max: u32) -> bool`

## 1. Patch eksak (3 file)

### 1a. `openapi-codegen/Cargo.toml`
```toml
[dependencies]
kernel = { path = "../../../../opt/agent-workspace/kernel-asli-d3bcff0/crates/kernel" }  # READ-ONLY, pola nodes-openapi
serde_json = "=1.0.151"
sha2 = "=0.10.8"
```
*Pertanyaan pra-review Q-K1: konfirmasi openapi-codegen boleh path-dep kernel (pola identik nodes-openapi; kernel tetap read-only; TANPA edit kanonik).*

### 1b. `openapi-codegen/src/ingest.rs`
- **HAPUS** `fn check_depth()` (scanner transisional, ~40 baris) — duplikat dari `kernel::json::json_depth_exceeded` (iteratif, string-aware, O(1)-stack — terverifikasi agent10 #1402).
- **PERTAHANKAN** `const MAX_JSON_DEPTH: u32 = 64` — batas tetap milik codegen sebagai KEBIJAKAN KONSUMEN (doc `json.rs:13`: "openapi-codegen ingest: `MAX_JSON_DEPTH = 64` (or per-consumer reason)").
- **GANTI** situs panggil (ingest.rs:26):
```rust
if let Some(depth) = kernel::json::json_depth_exceeded(&bytes, MAX_JSON_DEPTH) {
    return Err(CodegenError {
        code: "CG-E-104",
        reason: format!(
            "kedalaman JSON {depth} melebihi batas {MAX_JSON_DEPTH} (dokumen: {path}); \
             dokumen ditolak sebelum parse — fail-closed (Ruling 39b/41)"
        ),
        // field lain sesuai struktur eksak CodegenError
    });
}
```
- Pesan TETAP memuat: depth aktual + batas + path → assertion test `reason.contains("65")` tetap hijau TANPA diubah.

### 1c. Test — TANPA perubahan semantik
`depth_64_accepted_65_rejected` (path unik per-proses, pasca-FINDING-1): 63 nest = depth-total 64 → `ok`; 64 nest = 65 → `CG-E-104` + reason mengandung "65" (depth aktual kini dari RETURN VALUE fn kernel — lebih kaya daripada hitungan lokal).

## 2. Bukti yang harus dihasilkan saat eksekusi (R42 + Ruling 49c: baca diff, jalankan SEMUA)

1. `git diff`/sha ingest.rs + Cargo.toml (before/after).
2. `cargo test --workspace` = **59/59** (target fresh `/mnt/extra-storage/cargo-target`) — 3 baris test-result terpaste.
3. `cargo clippy --workspace --all-targets` = 0.
4. **Golden INTEG-01 tetap byte-identical** (EMIT tak tersentuh — proof oleh test golden itu sendiri).
5. SHA256SUMS 38a refresh + tree final diumumkan SEKALI di akhir sesi (aturan #1418).

## 3. Yang TIDAK berubah

- `nodes-openapi` (sudah path-dep kernel; tidak tersentuh).
- EMIT/translate/validate/resolver → golden identik.
- Path-dep final bentuk workspace-members (catatan agent1 #1386) = tetap agenda post-merge; kutover ini TIDAK mengubah rencana itu.
- Kode error CG-E-104 + semantik fail-closed (guard SEBELUM parse — audit agent10 #1402 tetap berlaku: satu-satunya situs parse ingest.rs:30 tetap di belakang guard).

## 4. Urutan eksekusi (saat HEAD baru diumumkan agent1)

1. `git -C /opt/agent-workspace/kernel-asli-d3bcff0 log -1` → catat HEAD baru + pastikan `status --porcelain` KOSONG (Ruling 49d.4).
2. Terapkan patch §1 (staging lokal → server).
3. Jalankan bukti §2.
4. rsync 38a + SHA256SUMS + umumkan tree.
5. `msg` ACK ke kanal: kutover agent9 SELESAI (reviewer agent1 dapat memverifikasi; implementor≠reviewer tetap terjaga — saya penulis patch, verifikasi tetap agen lain).
