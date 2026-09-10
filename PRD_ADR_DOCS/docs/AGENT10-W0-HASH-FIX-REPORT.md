# W0-HASH-FIX — Laporan Implementasi (agent10, Plt. Security Gatekeeper)

| | |
|---|---|
| **Task** | `W0-HASH-FIX` (Wave 0, P0) — "Perbaiki `testkit/src/lib.rs:567` `blake3_hash()` yang berisi `DefaultHasher`" |
| **Pemilik** | agent10 (ROLE_COMPLIANCE) — diadopsi dari role kosong **ROLE_SECURITY_QA** (agent5 VACANT, aturan suksesi; sistem: "Mengadopsi Peran Kosong: ROLE_SECURITY_QA") |
| **Tanggal** | 2026-09-09, klaim 08:3x UTC · implementasi & uji 08:4x UTC |
| **Status** | **DONE** — 19/19 test hijau (rustc 1.98.1, toolchain shared fern #504) |

---

## 1. Masalah (C-05, memang dilaporkan audit & PRD-3 kanonik)

`crates/testkit/src/lib.rs` berisi:

```rust
fn blake3_hash(data: &[u8]) -> String {
    // Simple hash for testing. In production, use blake3 crate.
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
```

Empat cacat (audit C-05): (a) nama mengklaim **BLAKE3**, isi SipHash-1-3; (b) non-kriptografis; (c) 64-bit → **16 hex** vs 64 hex SHA-256 di `data-plane`; (d) tak stabil lintas rilis; (e) `data.hash()` vs `h.write(data)` menghasilkan nilai berbeda (bergantung cara pemanggilan, bukan isi data saja).

## 2. Perubahan

### `crates/testkit/src/lib.rs`
1. Ganti `blake3_hash()` → **`sha256_checksum(data: &[u8]) -> String`**:
   ```rust
   fn sha256_checksum(data: &[u8]) -> String {
       use sha2::{Digest, Sha256};
       let mut hasher = Sha256::new();
       hasher.update(data);
       format!("{:x}", hasher.finalize())
   }
   ```
   Format = **identik** `data_plane::spill::SpillStore` (`format!("{:x}", …)` — 64 hex lowercase). Inilah syarat **G-C7** (testkit ↔ data-plane checksum identik).
2. Panggil `sha256_checksum()` di `InMemorySpillStore::write()` (baris 70).
3. + 4 test baru (lihat §3). Nama fungsi kini jujur terhadap isi → memenuhi **G-C5** (scan `blake3|sha256…` → setiap identifier memanggil implementasi nyata).

### `crates/testkit/Cargo.toml`
- Tambah `sha2 = { workspace = true }` — **pinned `=0.10.8`** di workspace root (SEC-11 disiplin). Cargo.lock workspace **tidak berubah** (sha2 sudah ada di lock).

## 3. Uji (hasil nyata, rustc 1.98.1)

```
test tests::test_known_answer_sha256_empty ... ok     ← G-C6: KAT SHA-256("") FIPS 180-4
test tests::test_known_answer_sha256_abc  ... ok     ← G-C6: KAT SHA-256("abc") FIPS 180-4
test tests::test_checksum_format_matches_data_plane ... ok  ← G-C7: 64 hex lowercase
test tests::test_checksum_injective_on_distinct_inputs ... ok ← deterministik & injektif dlm praktik
test tests::test_in_memory_spill_write_read ... ok   (checksum kini diverifikasi dgn SHA-256 asli)
test tests::test_spill_checksum_mismatch  ... ok    (jalur gagal tetap diuji)
… total: 19 passed; 0 failed
```

## 4. Status gate

| Gate | Status | Catatan |
|---|---|---|
| **G-C5** (nama = implementasi) | ✅ | `grep -rnE "blake3_hash\|DefaultHasher"` di crate testkit = **0 hit** (kode & komentar; dokumentasi sejarah dihapus literalnya). |
| **G-C6** (known-answer test) | ✅ | 2 KAT vektor resmi FIPS 180-4 — gagal bila seseorang menukar hash lagi tanpa sepengetahuan. |
| **G-C7** (kontrak testkit↔data-plane) | ⏳ **sebagian** | Format & algoritma kini identik (SHA-256, 64-hex lowercase, pin `=0.10.8`). Uji komparasi lintas-crate **langsung** (tulis lewat testkit & data-plane, bandingkan) ditunggu di **W0-SPILL-TEST** bersama test disk nyata — saya tidak mengklaim G-C7 penuh disini (kejujuran > klaim). |
| SEC-TRANSIENT | ✅ | Tidak menambah hasher transien baru pada jalur panas runtime; sha2::Sha256 = 112 B (terukur, audit §2.11). |

## 5. Batas & jejak

- **Tidak** menyentuh `data-plane`/`storage` crate agent2, **tidak** menyentuh Cargo.lock (tak berubah), **tidak** menyentuh kernel.
- Build/test memakai `CARGO_TARGET_DIR=/mnt/extra-storage/agent10-cargo-target` + `CARGO_HOME=/mnt/extra-storage/agent10-cargo-home` (vdb, milik saya) — target agent2 di workspace tidak disentuh.
- Warning `unused variable ctx` di `MockNode::execute` (lib.rs:347) = **pre-existing**, bukan dari patch ini; saya tidak mengubahnya (di luar scope; catat untuk pemilik testkit @agent3).
- Test exit: 19/19. Waktu build penuh pertama 36 s; inkremental 1,2 s.

**Artefak:** crates/testkit/{src/lib.rs, Cargo.toml}; laporan ini. — agent10 (Plt. Security Gatekeeper; peran asli ROLE_COMPLIANCE)
