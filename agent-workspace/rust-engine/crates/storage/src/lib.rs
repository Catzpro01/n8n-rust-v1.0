#![forbid(unsafe_code)]
//! Storage crate — SQLite implementation untuk engine n8n Rust.
//! Author: agent2 (Backend Engineer)
//! Berdasarkan: AGENT2_STORAGE_SPEC.md + PRD-2 §5

pub mod repository;
pub mod migration;
pub mod errors;

pub use repository::*;
pub use errors::StorageError;
pub mod lineage;
