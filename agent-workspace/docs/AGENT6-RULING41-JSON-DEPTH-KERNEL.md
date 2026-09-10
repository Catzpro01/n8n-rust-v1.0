# AGENT6 — RULING 41 / 47b: Pemindahan Pemeriksa Kedalaman JSON ke Kernel (RFC patch)

Tanggal: 2026-09-09 15:05 UTC · Oleh: agent6 (W1-WCB-IMPL)
Mandat: matt RULING 41 (#1345) + revisi RULING 47b (#1420), review agent1 #1361/#1399.
Referensi: RULING 39b (#1317), RULING 40 (#1332), agent3 #1334, agent10 #1402.

## 1. Ringkasan perubahan (patch rev-3 ke kernel-asli-d3bcff0)
- **Baru**: `crates/kernel/src/json.rs` — modul murni.
  - `pub fn json_depth_exceeded(bytes: &[u8], max: u32) -> Option<u32>` — SATU scanner loop:
    `None` = dalam batas; `Some(found)` = melebihi dan `found` = kedalaman yang dicapai.
    (API resmi Ruling 47b; menghilangkan kebutuhan pemindaian ganda untuk pesan error —
    cacat `mcp/frame.rs:19-20`.)
  - `pub fn json_depth_within(bytes: &[u8], max: u32) -> bool` — view bool TIPIS
    (`.is_none()` di atas scanner yang sama; BUKAN implementasi kedua).
  - Iteratif (tanpa rekursi); string/escape dihormati; `depth.saturating_add(1)`
    (micro-fix agent1 #1361: wrap u32 >4G = fail-closed, bukan fail-open release).
  - Murni: tanpa IO/alokasi/kebijakan; konstanta tetap di konsumen.
- **lib.rs** (+2 baris): `pub mod json;` + re-export `pub use json::{json_depth_exceeded, json_depth_within};`
  (additive; TIDAK masuk daftar kontrak tipe "breaking + ADR").

## 2. Konsumen (satu scanner; EMPAT pemanggil — Ruling 47b)
1. agent6 nodes-wasm `gates.rs` (artefak luar WCB): konstanta `MAX_JSON_DEPTH=64`;
   scanner lokal + test r39b dihapus saat landing, parse_manifest di atas kernel fn.
2. agent2 storage ingest lineage_ext: memanggil dari kernel (fail-closed sebelum tulis).
3. agent9 openapi-codegen `ingest.rs`: hapus `check_depth` transisional saat landing.
4. agent7 `mcp/frame.rs`: ganti `json_depth()` lokal ke kernel fn (perilaku sama; satu baris
   perbaikan pemindaian ganda frame.rs:19-20 dimungkinkan oleh `Option`).
Catatan sequencing: patch TIDAK didorong langsung ke kernel beku. Landing setelah reviewer 2/2
(agent1 + agent3) + otorisasi fern; sesudah itu keempat pemanggil beralih di PR yang sama.

## 3. Bukti uji (staging = salinan kanonik + patch; /home/agent6/r41-stage, privat, transient)
```
cargo test -p kernel  (stage)   -> lib 11/11 (json 10 + error) ; tests/ 35/35 ; 19/19 ; 0 gagal
clippy --all-targets -D warnings = 0
```
- REV-3 (respon agent10 #1428 Catatan A): doc comment fungsi memuat invariant byte-vs-char
  (seluruh karakter struktural JSON = ASCII; UTF-8 tak pernah memuat ASCII di urutan multi-byte) —
  pengetahuan penulis asli yang wajib tinggal di kernel utk pemanggil berikutnya. + test batas 63/64/65.
- 10 unit test json: empty/flat ok; depth==64 diterima (None); 65 -> Some(65); dokumen 74 ->
  Some(65) (kembali pd kedalaman PERTAMA yang melewati batas); kurung dalam string diabaikan;
  escaped quote; string tanpa penutup; max=0 (hanya skalar; any bracket -> Some(1)).
- Konsisten dgn test nodes-wasm r39b (ekspektasi found = max+1).

## 4. Artefak (rev-3)
- Patch: `/mnt/extra-storage/agent6-work/W1-WCB-IMPL/r41-kernel-json-depth.patch`
  sha256 `24d0ead28a763b266a782b26de33dd243bc2fe959d85fd77328b23e6181fb857` (rev-3)
- SHA256SUMS.r41 di direktori yang sama; dokumen ini sha (di kuda-kuda).

## 5. Permintaan review (jalur §3.4)
- Reviewer 1 (kernel owner): @agent1 (rev-1 #1399 APPROVE; rev-2 #1427 APPROVE; rev-3 butuh verifikasi 40-detik)
- Reviewer 2 (independen): @agent3
- Otorisasi pendaratan ke kanonik: @fern
