//! VALIDATE: servers (CG-E-102) + operasi & operationId (CG-E-103).
//! Operasi dikembalikan TERURUT KANONIK (OPEN-INTEG-4: order acak meracuni
//! identitas replay — agent1 #1038).

use crate::error::{codes, CodegenError};
use crate::ingest::SpecDoc;
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Operation {
    pub method: String,
    pub path: String,
    pub operation_id: String,
}

pub fn validate(doc: &SpecDoc) -> Result<Vec<Operation>, CodegenError> {
    check_servers(doc)?;
    collect_operations(doc)
}

/// CG-E-102: server URL dengan template variable `{var}` wajib punya
/// `variables.{var}.default` atau `.enum`.
fn check_servers(doc: &SpecDoc) -> Result<(), CodegenError> {
    let empty = Vec::new();
    let servers = doc.root.get("servers").and_then(Value::as_array).unwrap_or(&empty);
    for s in servers {
        let url = s.get("url").and_then(Value::as_str).unwrap_or("");
        if url.contains('{') {
            let ok = s.get("variables").and_then(Value::as_object).is_some_and(|vars| {
                vars.values().all(|v| v.get("default").is_some() || v.get("enum").is_some())
            });
            if !ok {
                return Err(CodegenError::new(
                    codes::SERVER_VAR_NO_DEFAULT,
                    &doc.path,
                    format!("server '{url}' memakai template variable tanpa default/enum"),
                ));
            }
        }
    }
    Ok(())
}

const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

fn collect_operations(doc: &SpecDoc) -> Result<Vec<Operation>, CodegenError> {
    let paths = doc
        .root
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::new(codes::NOT_OPENAPI, &doc.path, "paths hilang"))?;
    let mut ops: Vec<Operation> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (path, item) in paths {
        let Some(item) = item.as_object() else { continue };
        for m in METHODS {
            let Some(op) = item.get(m) else { continue };
            let operation_id = derive_operation_id(op, m, path);
            if !seen.insert(format!("{}:{}", m, operation_id)) {
                return Err(CodegenError::new(
                    codes::OPERATION_ID_UNRESOLVABLE,
                    &doc.path,
                    format!("operationId '{operation_id}' duplikat dalam scope node"),
                )
                .op(&operation_id));
            }
            ops.push(Operation { method: m.to_string(), path: path.clone(), operation_id });
        }
    }
    if ops.is_empty() {
        return Err(CodegenError::new(
            codes::NOT_OPENAPI,
            &doc.path,
            "0 operasi ditemukan di paths",
        ));
    }
    ops.sort_by(|a, b| a.operation_id.cmp(&b.operation_id));
    ops.dedup_by(|a, b| a.operation_id == b.operation_id);
    Ok(ops)
}

/// CG-E-103: operationId hilang boleh diderivasi deterministik dari method+path
/// (camelCase dari segmen path, variable `{x}` dibaca sebagai nama segmen).
fn derive_operation_id(op: &Value, method: &str, path: &str) -> String {
    if let Some(id) = op.get("operationId").and_then(Value::as_str) {
        return id.to_string();
    }
    let canonical: String = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            let seg = seg.trim_start_matches('{').trim_end_matches('}');
            let mut c = seg.chars();
            match c.next() {
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect();
    format!("{}{}", capitalize(method), canonical)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::ingest;

    fn fixture(name: &str) -> String {
        format!("{}/../tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
    }

    #[test]
    fn reject_server_var_no_default() {
        let doc = ingest(&fixture("server-var-no-default.json")).unwrap();
        let err = validate(&doc).unwrap_err();
        assert_eq!(err.code, "CG-E-102");
    }

    #[test]
    fn reject_duplicate_operation_id() {
        let doc = ingest(&fixture("dup-operation-id.json")).unwrap();
        let err = validate(&doc).unwrap_err();
        assert_eq!(err.code, "CG-E-103");
        assert!(err.operation.is_some());
    }

    #[test]
    fn reject_no_paths() {
        let doc = ingest(&fixture("no-paths.json")).unwrap();
        let err = validate(&doc).unwrap_err();
        assert_eq!(err.code, "CG-E-101");
    }

    #[test]
    fn derive_id_deterministic_and_sorted() {
        let doc = ingest(&fixture("missing-operation-id.json")).unwrap();
        let ops = validate(&doc).unwrap();
        assert_eq!(ops.len(), 2);
        // derivasi: get /rates/{base} -> GetRatesBase ; post /convert -> PostConvert
        assert_eq!(ops[0].operation_id, "GetRatesBase");
        assert_eq!(ops[1].operation_id, "PostConvert");
        // terurut kanonik
        assert!(ops.windows(2).all(|w| w[0].operation_id <= w[1].operation_id));
    }

    #[test]
    fn frankfurter_golden_valid_sorted() {
        let path = format!("{}/../specs/frankfurter.openapi.json", env!("CARGO_MANIFEST_DIR"));
        let doc = ingest(&path).unwrap();
        let ops = validate(&doc).unwrap();
        assert!(!ops.is_empty());
        assert!(ops.windows(2).all(|w| w[0].operation_id <= w[1].operation_id));
    }
}
