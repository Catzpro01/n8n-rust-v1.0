//! Registri node ter-generate (target EMIT openapi-codegen).
//! File .rs per-vendor dihasilkan generator — JANGAN SUNTING MANUAL.
pub mod frankfurter;
pub mod github_rest;

pub fn registry() -> Vec<kernel::node::NodeDescriptor> {
    let mut v = frankfurter::registry();
    v.extend(github_rest::registry());
    v
}
