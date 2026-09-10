//! openapi-codegen — generator build-time OpenAPI 3.0/3.1 -> node kernel
//! (W3-OPENAPI-IMPL, agent9 ROLE_INTEGRATION).
//!
//! Spec acuan: docs/AGENT9-INTEGRATION-SPEC.md v0.5.3
//! (konsensus 2/2: agent4 #1017 SCHEMA/AST + agent1 #1061 kernel/replay).
//!
//! Pipeline (spec paragraf 1): INGEST -> VALIDATE -> TRIAGE -> TRANSLATE -> EMIT.
//! M1 = INGEST+VALIDATE. M2 = TRIAGE+TRANSLATE (modul ini + resolve.rs).
//! M3 = EMIT (nodes-openapi memakai tipe kernel NYATA — R-A agent4),
//! M4 = golden + rejection corpus + translation pairs, M5 = ukur RAM + clippy.
//!
//! v0.1 TANPA HTTP client (Ruling matt #1116): golden spec dibaca dari disk;
//! eksekusi request kelak via trait kernel `HttpClient` (context.rs:433).

#![forbid(unsafe_code)]

pub mod emit;
pub mod error;
pub mod ingest;
pub mod resolve;
pub mod translate;
pub mod validate;

pub use error::CodegenError;
pub use ingest::SpecDoc;
pub use emit::{emit, EmitArtifacts};
pub use translate::{translate, translate_collect, IrField, IrKind, TranslateReport, TranslatedOp, TriSideEffect};
pub use validate::Operation;
