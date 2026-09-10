//! W3-EXEC-ENVELOPE — Execution Envelope & rolling hash-chain.
//!
//! Implementasi dari `AGENT10-EXEC-ENVELOPE-SPEC.md` v1.2 (sha 6286c1ef…) §1–§5, §11,
//! ditambah `AGENT10-AUDIT-ADDENDUM-A1-CORRECTION.md` (urutan lapisan: anchor MENJAMIN,
//! keyed chain mengurangi biaya) dan `AGENT10-W0-ANCHOR-SPEC.md` v1.1.
//!
//! ATRIBUSI (integritas, lihat spec §13 + #628 butir 5):
//! - spec/konstruksi crypto : agent10 "sesi kedua" (penulis EXEC-ENVELOPE-SPEC v1.2)
//! - implementasi + uji gate: agent10 "sesi A" (berkas ini)
//! - lane belum diratifikasi fern; implementasi ini MENGIKUTI spec apa adanya dan tidak
//!   menyuntingnya. Bila kelak lane diputus berbeda, berkas ini yang dipindahkan.
//!
//! Batas yang dipegang (spec §1 prinsip 1, dan addendum A1.4): verifier TIDAK PERNAH
//! mengembalikan "Verified" polos. Hanya `VerifiedAnchored | VerifiedUnanchored | Broken`.

#![forbid(unsafe_code)]

pub mod entry;
pub mod chain;
pub mod verify;
pub mod kat;

pub use entry::{encode_canonical, EnvelopeEntry, Status, CLASS_HAS_UNVERIFIED_EXTERNAL_IO};
pub use chain::{Chain, Checkpoint, CHECKPOINT_INTERVAL, CTX};
pub use verify::{verify, AnchorRecord, Verdict, VerifyReport};

/// Anggaran memori transien (spec §4, SEC-TRANSIENT ≤ 4 KB, C-09).
pub const TRANSIENT_BUDGET_BYTES: usize = 4096;

/// Ukuran `blake3::Hasher` terukur (spec §4: 1.920 B pada blake3 1.8.7).
pub fn hasher_size_bytes() -> usize {
    std::mem::size_of::<blake3::Hasher>()
}

/// Ukuran `sha2::Sha256` terukur (spec §4: 112 B).
pub fn sha256_size_bytes() -> usize {
    std::mem::size_of::<sha2::Sha256>()
}
