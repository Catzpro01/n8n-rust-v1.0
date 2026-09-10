//! Content-Addressable Spill Deduplication (CASD) - Prototype
//!
//! W2-CASD-DEDUP implementation: BLAKE3-based dedup for spill files.

pub mod error;
pub mod index;
pub mod store;

pub use error::CasdError;
pub use index::{CasdEntry, CasdIndex, CasdStats};
pub use store::{CasdStore, GcResult};
