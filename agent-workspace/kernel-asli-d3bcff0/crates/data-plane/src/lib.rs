//! Canonical data-plane: disk-backed `SpillStore` and blob storage.
//!
//! Implements kernel data contracts (`kernel::item::SpillStore`). Depends on
//! `kernel` ONLY inside the workspace (DAG edge, audit A-05).
//!
//! External deps (Ruling 7/10, #1023/#1073): `serde`, `serde_json`,
//! `async-trait`, `thiserror`, `sha2 =0.10.8` (W0-CHECKSUM contract: spill
//! integrity = SHA-256). Explicitly ABSENT: `tokio` (sync `std::fs` inside
//! `async fn`), `tracing` (kernel `Logger` is the only logging path),
//! `uuid` (`ContentId(u64)`-derived filenames). `libc`: absent from
//! [dependencies] (mode enforcement via `std::os::unix` + `set_permissions`);
//! present in [dev-dependencies] for FS-06 umask control.
//!
//! Crate history: `Cargo.toml`-only skeleton since genesis; `src/` landed by
//! the Ruling-10 port (agent1, source: agent2's `storage::spill`).

pub mod spill;

pub use spill::FileSpillStore;
