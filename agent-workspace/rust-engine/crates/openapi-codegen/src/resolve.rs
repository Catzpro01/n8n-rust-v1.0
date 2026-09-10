//! Resolver `$ref` in-file dengan deteksi siklik.
//! CG-E-202: `$ref` siklik ATAU lintas-file (v1: registry provenansi kosong —
//! hanya "#/..." yang legal; eksternal ditolak fail-loud).

use crate::error::{codes, CodegenError};
use serde_json::Value;

pub struct Resolver<'a> {
    root: &'a Value,
    spec: String,
}

impl<'a> Resolver<'a> {
    pub fn new(root: &'a Value, spec: impl Into<String>) -> Self {
        Self { root, spec: spec.into() }
    }

    /// Ikuti rantai `$ref` sampai nilai akhir (objek tanpa `$ref`).
    pub fn resolve(&self, node: &Value) -> Result<Value, CodegenError> {
        let mut current = node.clone();
        let mut seen: Vec<String> = Vec::new();
        loop {
            let Some(r) = current.get("$ref").and_then(Value::as_str) else {
                return Ok(current);
            };
            if !r.starts_with("#/") {
                return Err(CodegenError::new(
                    codes::REF_CYCLIC_OR_EXTERNAL,
                    &self.spec,
                    format!("$ref lintas-file '{r}' tidak terdaftar di registry provenansi (v1: in-file saja)"),
                ));
            }
            if seen.iter().any(|s| s.as_str() == r) {
                return Err(CodegenError::new(
                    codes::REF_CYCLIC_OR_EXTERNAL,
                    &self.spec,
                    format!("$ref siklik: {} -> {r}", seen.join(" -> ")),
                ));
            }
            seen.push(r.to_string());
            current = self.pointer(&r[1..])?;
        }
    }

    /// Nama spec untuk pesan error lintas modul.
    pub fn spec_name(&self) -> &str {
        &self.spec
    }

    /// Navigasi JSON pointer (RFC 6901) dengan unescape ~0/~1.
    fn pointer(&self, ptr: &str) -> Result<Value, CodegenError> {
        let mut cur = self.root;
        let segs = ptr.split('/').skip(1); // buang kosong di depan (setelah '#')
        for raw in segs {
            let seg = raw.replace("~1", "/").replace("~0", "~");
            let next = cur.get(&seg).ok_or_else(|| {
                CodegenError::new(
                    codes::REF_CYCLIC_OR_EXTERNAL,
                    &self.spec,
                    format!("target $ref '/{seg}' tidak ditemukan"),
                )
            })?;
            cur = next;
        }
        Ok(cur.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolve_infile_chain() {
        let root = json!({"components": {
            "parameters": {"P": {"name": "q", "in": "query",
                "schema": {"$ref": "#/components/schemas/S"}}},
            "schemas": {"S": {"type": "string"}}}});
        let r = Resolver::new(&root, "t.json");
        let v = r.resolve(&json!({"$ref": "#/components/parameters/P"})).unwrap();
        assert_eq!(v["name"], "q");
        let s = r.resolve(&json!({"$ref": "#/components/schemas/S"})).unwrap();
        assert_eq!(s["type"], "string");
    }

    #[test]
    fn reject_cyclic() {
        let root = json!({"a": {"$ref": "#/b"}, "b": {"$ref": "#/a"}});
        let r = Resolver::new(&root, "t.json");
        let err = r.resolve(&json!({"$ref": "#/a"})).unwrap_err();
        assert_eq!(err.code, "CG-E-202");
        assert!(err.reason.contains("siklik"));
    }

    #[test]
    fn reject_external() {
        let root = json!({"a": 1});
        let r = Resolver::new(&root, "t.json");
        let err = r.resolve(&json!({"$ref": "lain.json#/x"})).unwrap_err();
        assert_eq!(err.code, "CG-E-202");
        assert!(err.reason.contains("lintas-file"));
    }

    #[test]
    fn reject_missing_target() {
        let root = json!({"a": 1});
        let r = Resolver::new(&root, "t.json");
        let err = r.resolve(&json!({"$ref": "#/tidak/ada"})).unwrap_err();
        assert_eq!(err.code, "CG-E-202");
    }
}
