# W0-SPILL-IMPL — Laporan Implementasi (agent1, adopsi ROLE_STORAGE)

**Status:** SELESAI, terverifikasi penuh di salinan pohon; menunggu review
gatekeeper + aplikasi matt (kanonik hanya-bisa-tulis-matt).
**Tanggal:** 2026-09-09. **Kontrak:** SINTESIS-SPILLSTORE §6.1 (matt) +
W0-CHECKSUM-AGREE v0.1.1 + PATCH v1.2 (agent3) + E_SPILL_* engine (agent1).

## Artefak

- Patch: `/home/agent1/w0-spill-impl/W0-SPILL-IMPL.patch` (11 berkas,
  teraplikasi-bersih `-p1` dari dalam `kernel-asli-d3bcff0`, dibuktikan §4).
- Pohon-kerja terverifikasi: `/home/agent1/spill-work/` (salinan kanonik + patch).

## Yang diimplementasi (path kanonik)

1. `crates/kernel/src/error.rs` — varian `InvalidMagic`, `ChecksumMismatch`,
   `OffsetOutOfRange` (= `E_SPILL_CORRUPT`, G-C7 "1B-corrupt→specific").
2. `crates/kernel/src/item.rs` — `SpilledList` + `layer0_checksum`/`casd_hash`
   (PATCH §1 verbatim); trait + `verify_integrity` default.
3. `CREATE crates/data-plane/src/{lib,spill}.rs` — `FileSpillStore` dipindah
   dari `examples/` (§6.1 langkah 2) + serapan §6.1 langkah 3: SHA-256 at-rest,
   `0600` di konstruktor (`OpenOptions::mode`, tanpa-jendela-chmod), GC
   per-eksekusi via `for_execution` (+ penolakan traversal `../`).
4. Format v1: header 8B (`SPL1` + ver `0x01` + reserved); magic dicek di
   SETIAP read; checksum penuh hanya di `verify_integrity` eksplisit.
5. `crates/data-plane/Cargo.toml` + `sha2 0.10`/`blake3 1` (kunci 1.8.7);
   root `members` += data-plane; dev-dep `libc` (uji umask) + dev-dep kernel
   → data-plane (rewire bench; dev-cycle legal).
6. `spill_bench.rs` — impl lokal DIHAPUS, memanggil `data_plane` (bukan salinan).
7. `CREATE crates/data-plane/tests/spill_tests.rs` — 10 uji (§6.1 langkah 4).

## Verifikasi (semua diukur, bukan klaim)

- `cargo test`: 19/19 kernel kontrak + 10/10 data-plane (roundtrip, korupsi-1B
  → `ChecksumMismatch`, magic-rusak → `InvalidMagic`, trunc → `OffsetOutOfRange`,
  umask-000 → `0600`, legacy → `Invalid`, GC, drop-tanpa-bocor, indeks-5000).
- `cargo clippy --all-targets`: 0 warning; `forbid(unsafe_code)`; fmt-bersih
  untuk semua berkas-sentuh (file `params.rs` dkk kotor = pra-ada, DIKEMBALIKAN,
  tidak ikut patch).
- Ekuivalensi pindahan: bench `spilled 20000` checksum lama = baru =
  `967530000` (byte-identik); `read_at` O(1) 0.03–0.07 ms.
- Patch diuji teraplikasi ke salinan murni terpisah + `cargo test` 29/29 di sana.

## Deviasi jujur dari PATCH v1.2

- (a) `Unsupported` → `Invalid` (varian tidak ada di `KernelError` — bug patch).
- (b) Tanpa tokio (kernel runtime-agnostik; `std::fs` sinkron dalam `async fn`
  mengikuti contoh).
- (c) Hash-update per-item di atas `BufWriter` 256KB (bukan chunk-256B tanpa
  buffer): properti O(1) sama (~2KB transien < budget 4KB), syscall minimal.
- (d) Default trait TIDAK mengklaim `Ok(true)` (PATCH §4 mengklaim valid tanpa
  memeriksa — berbahaya); default = `Err` eksplisit (legacy / tak-diimplementasi).

## Limitasi W0 yang didokumentasikan (bukan disembunyikan)

Korupsi yang masih decode sebagai JSON valid TIDAK terdeteksi oleh read —
hanya oleh `verify_integrity` eksplisit. Kebijakan verify-otomatis/periodik
engine = wave berikut (butuh keputusan Governor + biaya performa agent8).

## Tindak lanjut

1. agent10 (gatekeeper): review patch. 2. matt/fern: aplikasikan
   (`patch -p1 < W0-SPILL-IMPL.patch` + `cargo test`) lalu komit. 3. Setelah
   landing: W0-SPILL-TEST (agent10) terbuka; E_SPILL_CORRUPT/E_SPILL_IO engine
   kini punya pasangan varian sisi-kernel. 4. T-11 tetap klausul-tinjau W2.
