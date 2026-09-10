//! EMIT (spec paragraf 1 tahap 5): hasilkan kode Rust `nodes-openapi`
//! memakai TIPE KERNEL NYATA (R-A agent4 #1017: NodeDescriptor /
//! ParameterSchema / ParameterField / ParameterOption / CredentialSpec /
//! NodeKind / SideEffect) — nol vocab tipe paralel.
//!
//! Generator = string-builder deterministik (tanpa template engine, spec 1.5).
//! Struktur keluaran: SATU FUNGSI KECIL per node (`node_NNNN`) + registry
//! yang mendorong satu-per-satu — menghindari literal vec![] raksasa yang
//! meluapkan stack thread 2MiB (temuan M3).
//!
//! Manifest verifikasi (addendum 1.6 R-B/NIT agent4 + v0.5.2):
//!   {generator, spec{sha256, hash_domain: raw-bytes}, ops_emitted,
//!    ops_rejected[{operation_id, code, reason}], nodes[]}

use crate::error::CodegenError;
use crate::ingest::SpecDoc;
use crate::translate::{IrField, IrKind, TranslateReport, TranslatedOp, TriSideEffect};
use serde_json::json;

pub struct EmitArtifacts {
    /// Kode Rust siap-compile (modul nodes-openapi).
    pub nodes_rs: String,
    /// Manifest verifikasi JSON (deterministik: kunci terurut BTreeMap).
    pub manifest_json: String,
    /// Nama file module-safe (underscore) untuk artefak ini.
    pub stem: String,
}

pub fn emit(
    doc: &SpecDoc,
    rep: &TranslateReport,
    generator_version: &str,
) -> Result<EmitArtifacts, CodegenError> {
    let vendor = vendor_of(&doc.path);
    let stem = vendor.replace('-', "_");
    // Nama file kanonik (basename): header/manifest tidak boleh tergantung
    // path absolut/cwd pemanggil — golden diff harus reproducible.
    let file_name = doc.path.rsplit('/').next().unwrap_or(&doc.path).to_string();
    let any_credentials = rep.ops.iter().any(|o| !o.credentials.is_empty());
    let params_import = if any_credentials {
        "use kernel::params::{CredentialSpec, ParameterField, ParameterKind, ParameterOption, ParameterSchema};"
    } else {
        "use kernel::params::{ParameterField, ParameterKind, ParameterOption, ParameterSchema};"
    };

    let mut nodes = String::new();
    nodes.push_str(&format!(
        "//! AUTO-GENERATED oleh openapi-codegen {generator_version} — JANGAN SUNTING MANUAL.\n\
         //! spec: {spec} sha256(raw-bytes): {sha}\n\
         //! operasi: {ok} emitted, {rej} rejected (manifest: {stem}.manifest.json)\n\
         #![allow(clippy::vec_init_then_push)] // kode ter-generate: push-per-node = anti stack-overflow (temuan M3)\n\n\
         use kernel::id::NodeKind;\n\
         use kernel::node::{{NodeDescriptor, SideEffect}};\n\
         {params_import}\n\
         use serde_json::json;\n\n\
         pub fn registry() -> Vec<NodeDescriptor> {{\n\
         \x20   let mut v = Vec::with_capacity({cap});\n",
        spec = file_name,
        sha = doc.sha256_hex,
        ok = rep.ops.len(),
        rej = rep.rejected.len(),
        stem = stem,
        params_import = params_import,
        cap = rep.ops.len(),
    ));

    for idx in 0..rep.ops.len() {
        nodes.push_str(&format!("    v.push(node_{idx:04}());\n"));
    }
    nodes.push_str("    v\n}\n");

    for (idx, op) in rep.ops.iter().enumerate() {
        nodes.push_str(&format!("\nfn node_{idx:04}() -> NodeDescriptor {{\n"));
        nodes.push_str(&format!(
            "    let mut d = NodeDescriptor::new(\n\
             \x20       NodeKind::new({kind}),\n\
             \x20       1,\n\
             \x20       {display},\n\
             \x20   );\n",
            kind = rs(&node_kind(&vendor, op)),
            display = rs(op.summary.as_deref().unwrap_or(&op.operation.operation_id)),
        ));
        nodes.push_str(&format!(
            "    d.hints.side_effect = SideEffect::{se};\n",
            se = match op.side_effect {
                TriSideEffect::Idempotent => "Idempotent",
                TriSideEffect::NonIdempotent => "NonIdempotent",
            }
        ));
        if let Some(desc) = op.description.as_deref().or(op.summary.as_deref()) {
            nodes.push_str(&format!("    d.description = Some({}.into());\n", rs(desc)));
        }
        if op.fields.is_empty() {
            nodes.push_str("    d.params = ParameterSchema::empty();\n");
        } else {
            nodes.push_str("    d.params = ParameterSchema { fields: vec![\n");
            for f in &op.fields {
                nodes.push_str(&format!("        {},\n", field_expr(f)));
            }
            nodes.push_str("    ] };\n");
        }
        if !op.credentials.is_empty() {
            nodes.push_str("    d.credentials = vec![\n");
            for c in &op.credentials {
                nodes.push_str(&format!(
                    "        CredentialSpec {{ kind: {}.into(), required: {}, purpose: None }},\n",
                    rs(&c.kind),
                    c.required
                ));
            }
            nodes.push_str("    ];\n");
        }
        nodes.push_str("    d\n}\n");
    }

    // Manifest verifikasi (deterministik).
    let manifest = json!({
        "generator": { "name": "openapi-codegen", "version": generator_version },
        "spec": {
            "file": file_name,
            "sha256": doc.sha256_hex,
            "hash_domain": "raw-bytes",
            "size_bytes": doc.size_bytes,
            "openapi": doc.root.get("openapi").cloned().unwrap_or(serde_json::Value::Null),
        },
        "ops_emitted": rep.ops.len(),
        "ops_rejected": rep
            .rejected
            .iter()
            .map(|r| json!({ "operation_id": r.operation_id, "code": r.code, "reason": r.reason }))
            .collect::<Vec<_>>(),
        "nodes": rep
            .ops
            .iter()
            .map(|o| {
                json!({
                    "kind": node_kind(&vendor, o),
                    "operation_id": o.operation.operation_id,
                    "method": o.operation.method,
                    "path": o.operation.path,
                    "side_effect": match o.side_effect {
                        TriSideEffect::Idempotent => "Idempotent",
                        TriSideEffect::NonIdempotent => "NonIdempotent",
                    },
                    "fields": o.fields.len(),
                    "credentials": o.credentials.len(),
                    "body_content_type": o.body_content_type,
                })
            })
            .collect::<Vec<_>>(),
    });
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| CodegenError::new("CG-E-101", &doc.path, format!("serialisasi manifest: {e}")))?;

    Ok(EmitArtifacts { nodes_rs: nodes, manifest_json, stem })
}

/// Ekspresi Rust satu ParameterField (builder + struct-update untuk options).
fn field_expr(f: &IrField) -> String {
    let kind = match f.kind {
        IrKind::String => "ParameterKind::String",
        IrKind::Number => "ParameterKind::Number",
        IrKind::Boolean => "ParameterKind::Boolean",
        IrKind::Options => "ParameterKind::Options",
        IrKind::Json => "ParameterKind::Json",
    };
    let mut expr = format!(
        "ParameterField::new({}, {}, {})",
        rs(&f.name),
        rs(&f.display_name),
        kind
    );
    if let Some(d) = &f.default {
        expr.push_str(&format!(".with_default({})", value_expr(d)));
    }
    if f.required {
        expr.push_str(".required()");
    }
    // Ekspresi {{ }} diizinkan pada field bebas-nilai (bukan pilihan tetap).
    if !matches!(f.kind, IrKind::Options) {
        expr.push_str(".expression()");
    }
    if let Some(opts) = &f.options {
        let items: Vec<String> = opts
            .iter()
            .map(|(l, v)| {
                format!(
                    "ParameterOption {{ name: {}.into(), value: {}, description: None }}",
                    rs(l),
                    value_expr(v)
                )
            })
            .collect();
        expr = format!(
            "ParameterField {{ options: Some(vec![{}]), ..{} }}",
            items.join(", "),
            expr
        );
    }
    expr
}

/// Value -> ekspresi `json!(...)`: serialisasi JSON valid juga literal token json!.
fn value_expr(v: &serde_json::Value) -> String {
    format!(
        "json!({})",
        serde_json::to_string(v).unwrap_or_else(|_| "null".into())
    )
}

fn node_kind(vendor: &str, op: &TranslatedOp) -> String {
    // Kanonik lowercase (konvensi node-type id n8n: `generated.stripe.charges`).
    // operationId asli tetap tercatat di manifest -> nodes[].operation_id.
    format!(
        "generated.{}.{}",
        vendor.to_lowercase(),
        sanitize(&op.operation.operation_id).to_lowercase()
    )
}

/// Nama vendor dari nama file spec: frankfurter.openapi.json -> frankfurter.
pub fn vendor_of(path: &str) -> String {
    let stem = path.rsplit('/').next().unwrap_or(path);
    let stem = stem.strip_suffix(".json").unwrap_or(stem);
    let stem = stem.strip_suffix(".openapi").unwrap_or(stem);
    sanitize(stem)
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c == '/' || c == '.' {
                '.'
            } else {
                '-'
            }
        })
        .collect()
}

/// Literal string Rust dengan escaping penuh.
fn rs(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{{{:04x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::ingest;
    use crate::translate::translate_collect;

    fn spec(name: &str) -> String {
        format!("{}/../specs/{}", env!("CARGO_MANIFEST_DIR"), name)
    }

    #[test]
    fn emit_deterministic_byte_identical() {
        let doc = ingest(&spec("frankfurter.openapi.json")).unwrap();
        let rep = translate_collect(&doc).unwrap();
        let a = emit(&doc, &rep, "0.1.0").unwrap();
        let b = emit(&doc, &rep, "0.1.0").unwrap();
        assert_eq!(a.nodes_rs, b.nodes_rs);
        assert_eq!(a.manifest_json, b.manifest_json);
    }

    #[test]
    fn emit_uses_kernel_types() {
        let doc = ingest(&spec("frankfurter.openapi.json")).unwrap();
        let rep = translate_collect(&doc).unwrap();
        let art = emit(&doc, &rep, "0.1.0").unwrap();
        // R-A agent4: tipe kernel nyata, bukan vocab paralel.
        assert!(art.nodes_rs.contains("NodeDescriptor::new("));
        assert!(art.nodes_rs.contains("NodeKind::new(\"generated.frankfurter.getcurrencies\")"));
        assert!(art.nodes_rs.contains("SideEffect::Idempotent"));
        assert!(art.nodes_rs.contains("pub fn registry() -> Vec<NodeDescriptor>"));
        // struktur fungsi-per-node (anti stack-overflow)
        assert!(art.nodes_rs.contains("v.push(node_0000());"));
        assert!(art.nodes_rs.contains("fn node_0000() -> NodeDescriptor {"));
        // nol HTTP client di hasil generate
        assert!(!art.nodes_rs.contains("reqwest"));
    }

    #[test]
    fn manifest_counts_frankfurter() {
        let doc = ingest(&spec("frankfurter.openapi.json")).unwrap();
        let rep = translate_collect(&doc).unwrap();
        let art = emit(&doc, &rep, "0.1.0").unwrap();
        assert!(art.manifest_json.contains("\"ops_emitted\": 5"));
        assert!(art.manifest_json.contains("\"hash_domain\": \"raw-bytes\""));
        assert!(art.manifest_json.contains("\"sha256\": \"e38804965823bcbd7db06471aa6a2b8b19b4dfc5f26f95d14aae8cfad1fb46a6\""));
    }

    #[test]
    fn manifest_counts_github() {
        let doc = ingest(&spec("github-rest.openapi.json")).unwrap();
        let rep = translate_collect(&doc).unwrap();
        let art = emit(&doc, &rep, "0.1.0").unwrap();
        assert!(art.manifest_json.contains("\"ops_emitted\": 1206"));
        assert!(art.manifest_json.contains("\"code\": \"CG-E-201\""));
        assert!(art.manifest_json.contains("\"code\": \"CG-E-205\""));
        // 19 rejected tercatat, bukan didiamkan
        assert_eq!(rep.rejected.len(), 19);
    }

    #[test]
    fn string_escaping() {
        assert_eq!(rs("a\"b\\c\nd"), "\"a\\\"b\\\\c\\nd\"");
    }

    #[test]
    fn vendor_derivation() {
        assert_eq!(vendor_of("specs/frankfurter.openapi.json"), "frankfurter");
        assert_eq!(vendor_of("specs/github-rest.openapi.json"), "github-rest");
    }
}
