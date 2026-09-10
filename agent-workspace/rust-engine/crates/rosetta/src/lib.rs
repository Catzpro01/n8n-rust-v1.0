//! # Rosetta Compiler — `crates/rosetta/`
//!
//! Milestone M1 (W1-ROSETTA-IMPL, DIRECTIVE #1022): parser workflow JSON n8n
//! (era mana pun, opaque-safe) → grafik kanonik deterministik. Nol mutasi file
//! sumber. Tipe node tak dikenal (komunitas/unknown) = dipertahankan utuh dalam
//! grafik + ditandai `opaque` (R-2: tidak pernah menghentikan impor).
//!
//! Prinsip (AGENT4-WORKFLOW-ROSETTA v1.1, rencana impl da82d1dd):
//! - IR internal rosetta bebas bentuk; EMIT final (M6) dipetakan ke tipe kernel
//!   nyata (NodeDescriptor/ParameterSchema) — tidak ada vocab tipe paralel.
//! - Determinisme: input sama → output sama (R-8); parser murni baca, tidak
//!   pernah menulis ke file sumber (R-1).
//! - Disiplin dependensi: serde/serde_json/thiserror saja; tanpa tokio, tanpa
//!   uuid, tanpa jalur encoding kedua (C-02).
//!
//! Milestone: M1 parser — M2 registry rules v1 — M3 migrate+receipt —
//! M4 binding stickyNote — M5 kanon N-rules + diffgate — M6 manifest WCB.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod binding;
pub mod catalog;
pub mod error;
pub mod kanon;
pub mod kindmap;
pub mod manifest_wcb;
pub mod migrate;
pub mod model;
pub mod param_schema;
pub mod param_schema_b;
pub mod parse;
pub mod receipt;
pub mod registry;

pub use binding::{bind_notes, BindingReport, NoteBinding, NoteRect};
pub use error::RosettaError;
pub use model::{CanonNode, NodeOrigin, OpaqueNote, TypeVersion, Workflow};
pub use parse::{parse_workflow_bytes, parse_workflow_path, parse_workflow_str};
pub use receipt::{build_receipt, receipt_to_json, Receipt};

/// Versi skema IR rosetta (digunakan utk digest kanon; framing u32-BE).
pub const ROSETTA_IR_SCHEMA_VERSION: u32 = 1;
