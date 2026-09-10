//! Diagnostik & aturan validasi authoring (F-6 final: 3E struktural + 5W expression).
//! Prototipe: pola scanner untuk subset W; mesin penuh = tabel 215 kasus agent3
//! yang di-serve via resource n8n://expression-rules (content_version 2).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const E_REF_DANGLING: &str = "E-REF-DANGLING";
pub const E_DAG_CYCLE: &str = "E-DAG-CYCLE";
pub const E_DUP_NODE_NAME: &str = "E-DUP-NODE-NAME";
pub const E_EXPR_REFERENCE: &str = "E-EXPR-REFERENCE";
pub const E_EXPR_SCOPE: &str = "E-EXPR-SCOPE";
pub const E_EXPR_CREDENTIAL: &str = "E-EXPR-CREDENTIAL";
pub const W_EXPR_FLOAT_PRECISION: &str = "W-EXPR-FLOAT-PRECISION";
pub const W_EXPR_SURROGATE: &str = "W-EXPR-SURROGATE";
pub const W_EXPR_OBJECT_ORDER: &str = "W-EXPR-OBJECT-ORDER";
pub const W_EXPR_UNDEFINED_NULL: &str = "W-EXPR-UNDEFINED-NULL";
pub const W_EXPR_ARRAY_METHODS: &str = "W-EXPR-ARRAY-METHODS";
/// Kode RESERVED (registri agent1 §4 / E-WCB): server dilarang serve konten kode ini.
pub const RESERVED_CODES: &[&str] = &["E-WCB-TRAP", "E-WCB-FUEL", "E-WCB-TIMEOUT"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Error, Warning }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autofix: Option<Value>,
}

impl Diagnostic {
    pub fn err(code: &str, node: Option<&str>, path: Option<&str>, msg: &str, hint: Option<&str>, autofix: Option<Value>) -> Self {
        Diagnostic { severity: Severity::Error, code: code.into(), node: node.map(String::from), path: path.map(String::from), message: msg.into(), hint: hint.map(String::from), autofix }
    }
    pub fn warn(code: &str, node: Option<&str>, path: Option<&str>, msg: &str, hint: Option<&str>, autofix: Option<Value>) -> Self {
        Diagnostic { severity: Severity::Warning, code: code.into(), node: node.map(String::from), path: path.map(String::from), message: msg.into(), hint: hint.map(String::from), autofix }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub valid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Diagnostic>,
    pub is_error: bool, // SEP-1303: isError benar utk tools
}

/// Model workflow n8n minimal utk validasi prototipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniWorkflow {
    pub nodes: Vec<MiniNode>,
    pub connections: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniNode {
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub parameters: Value,
    #[serde(default)]
    pub credentials: Value, // {name:{...}} — TIDAK pernah dibaca nilainya
}

fn collect_strings(v: &Value, out: &mut Vec<(String, String)>) {
    match v {
        Value::String(s) => out.push((s.clone(), s.clone())),
        Value::Array(a) => for x in a { collect_strings(x, out); },
        Value::Object(o) => for x in o.values() { collect_strings(x, out); },
        _ => {}
    }
}

/// Scanner pola ekspresi sederhana (subset 3W; mesin penuh menyusul dari tabel agent3).
fn expression_checks(text: &str, node: &str, warnings: &mut Vec<Diagnostic>) {
    // W-EXPR-UNDEFINED-NULL: `=== null` / `!== null` — lewatkan undefined (== null covers both)
    if text.contains("=== null") || text.contains("!== null") {
        warnings.push(Diagnostic::warn(W_EXPR_UNDEFINED_NULL, Some(node), Some("expression"),
            "perbandingan ketat dengan null melewatkan undefined", 
            Some("gunakan `== null` untuk mencakup null & undefined"), 
            Some(serde_json::json!({"replace": {"from": "=== null", "to": "== null"}}))));
    }
    // W-EXPR-OBJECT-ORDER: Object.keys(...) tanpa .sort() sebelum perbandingan/order
    if text.contains("Object.keys(") && !text.contains(".sort()") {
        warnings.push(Diagnostic::warn(W_EXPR_OBJECT_ORDER, Some(node), Some("expression"),
            "Object.keys() tanpa .sort() — urutan kunci insertion-order (K3, Optional: add .sort())",
            Some("tambahkan .sort() sebelum perbandingan"), None));
    }
    // W-EXPR-ARRAY-METHODS: .at(-1) — QuickJS subset (ES2022+ belum tentu ada)
    if text.contains(".at(") {
        warnings.push(Diagnostic::warn(W_EXPR_ARRAY_METHODS, Some(node), Some("expression"),
            ".at() ES2022+ belum tentu ada di QuickJS — gunakan arr[arr.length-1]",
            Some("ganti dengan indexing aritmetik"), 
            Some(serde_json::json!({"autoconvert": true}))));
    }
}

/// Validasi struktural (3E) + expression warnings.
pub fn validate_workflow(wf: &MiniWorkflow) -> DiagnosticReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // E-DUP-NODE-NAME
    let mut seen = std::collections::HashMap::new();
    for n in &wf.nodes {
        let c = seen.entry(n.name.clone()).or_insert(0usize);
        *c += 1;
        if *c == 2 {
            errors.push(Diagnostic::err(E_DUP_NODE_NAME, Some(&n.name), None,
                &format!("nama node duplikat: {}", n.name),
                Some("rename salah satu node (nama unik per workflow)"), None));
        }
    }

    // E-DAG-CYCLE via DFS atas connections {from: [{type, nodes:[to...]}]}
    let mut adj: std::collections::HashMap<String, Vec<String>> = Default::default();
    for (from, outs) in wf.connections.as_object().unwrap_or(&serde_json::Map::new()) {
        for out in outs.as_array().unwrap_or(&vec![]) {
            if let Some(list) = out.get("nodes").and_then(|x| x.as_array()) {
                for to in list {
                    if let Some(name) = to.as_str() {
                        adj.entry(from.clone()).or_default().push(name.to_string());
                    }
                }
            }
        }
    }
    // DFS detect cycle
    {
        use std::collections::HashMap;
        let mut state: HashMap<String, u8> = HashMap::new(); // 0=white 1=gray 2=black
        fn dfs(n: &str, adj: &std::collections::HashMap<String, Vec<String>>, state: &mut HashMap<String, u8>, errs: &mut Vec<Diagnostic>, chain: &mut Vec<String>) {
            match state.get(n) {
                Some(2) => return,
                Some(1) => {
                    let mut cyc: Vec<String> = chain.iter().skip_while(|x| *x != n).cloned().collect();
                    cyc.push(n.to_string());
                    errs.push(Diagnostic::err(E_DAG_CYCLE, Some(n), None,
                        &format!("cycle terdeteksi: {}", cyc.join(" -> ")),
                        Some("hapus/arahkan ulang koneksi penyebab cycle"), None));
                    return;
                }
                _ => {}
            }
            state.insert(n.to_string(), 1);
            chain.push(n.to_string());
            if let Some(nexts) = adj.get(n) {
                for nx in nexts { dfs(nx, adj, state, errs, chain); }
            }
            chain.pop();
            state.insert(n.to_string(), 2);
        }
        let mut chain = Vec::new();
        for n in wf.nodes.iter().map(|x| x.name.clone()) {
            dfs(&n, &adj, &mut state, &mut errors, &mut chain);
        }
    }

    // Referensi: tangkap $node["X"] / $node['X'] / $node.X dalam parameter
    let names: std::collections::HashSet<String> = wf.nodes.iter().map(|n| n.name.clone()).collect();
    for n in &wf.nodes {
        let mut texts = Vec::new();
        collect_strings(&n.parameters, &mut texts);
        for (text, _) in &texts {
            // kredensial: referensi cred di expression → E-EXPR-CREDENTIAL
            let low = text.to_lowercase();
            if low.contains("$credentials") && !low.contains("$credentials[\"") {
                // tanda kurung buka menandakan refs-only pattern; heuristic prototipe:
                if !text.contains("$credentials[") {
                    errors.push(Diagnostic::err(E_EXPR_CREDENTIAL, Some(&n.name), Some("expression"),
                        "akses credentials non-refs terdeteksi — wajib refs-only (D-6/F-9)",
                        Some("gunakan referensi kredensial, jangan nilai langsung"), None));
                }
            }
            // dangling ref
            for m in text.match_indices("$node[") {
                let after = &text[m.0 + 6..];
                if let Some(end) = after.find(']') {
                    let quoted = &after[..end];
                    let name = quoted.trim_matches(|c| c == '"' || c == '\'');
                    if !name.is_empty() && !names.contains(name) {
                        errors.push(Diagnostic::err(E_REF_DANGLING, Some(&n.name), Some("expression"),
                            &format!("referensi node tidak dikenal: {name}"),
                            Some("periksa nama node (case-sensitive)"), None));
                    }
                }
            }
            expression_checks(text, &n.name, &mut warnings);
        }
    }

    let valid = errors.is_empty();
    DiagnosticReport { valid, errors, warnings, is_error: !valid }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn wf(nodes: Value, conns: Value) -> MiniWorkflow {
        serde_json::from_value(json!({"nodes": nodes, "connections": conns})).unwrap()
    }

    // A7-5: skenario sintetis kode benar/error — subset
    #[test]
    fn tc_valid_clean() {
        let w = wf(json!([{"name":"A","type":"n8n-nodes-base.set","parameters":{"v":"x"}},
                          {"name":"B","type":"n8n-nodes-base.noOp","parameters":{}}]),
                   json!({"A": [{"type":"main","nodes":["B"]}]}));
        let r = validate_workflow(&w);
        assert!(r.valid);
        assert!(r.errors.is_empty());
    }
    #[test]
    fn tc_dup_name() {
        let w = wf(json!([{"name":"A","type":"t","parameters":{}},{"name":"A","type":"t","parameters":{}}]), json!({}));
        let r = validate_workflow(&w);
        assert!(!r.valid);
        assert!(r.errors.iter().any(|e| e.code == E_DUP_NODE_NAME));
    }
    #[test]
    fn tc_dangling_ref() {
        let w = wf(json!([{"name":"A","type":"t","parameters":{"x":"={{ $node[\"Ghost\"].json.v }}"}}]), json!({}));
        let r = validate_workflow(&w);
        assert!(r.errors.iter().any(|e| e.code == E_REF_DANGLING));
    }
    #[test]
    fn tc_cycle() {
        let w = wf(json!([{"name":"A","type":"t","parameters":{}},{"name":"B","type":"t","parameters":{}}]),
                   json!({"A":[{"type":"main","nodes":["B"]}],"B":[{"type":"main","nodes":["A"]}]}));
        let r = validate_workflow(&w);
        assert!(r.errors.iter().any(|e| e.code == E_DAG_CYCLE));
    }
    #[test]
    fn tc_warnings_3w() {
        let w = wf(json!([{"name":"X","type":"t","parameters":{
            "a":"={{ Object.keys({b:1,a:2}) }}",
            "b":"={{ undefined === null }}",
            "c":"={{ [1,2,3].at(-1) }}"}}]), json!({}));
        let r = validate_workflow(&w);
        assert!(r.valid); // warnings tidak menolak
        let codes: Vec<&str> = r.warnings.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&W_EXPR_OBJECT_ORDER));
        assert!(codes.contains(&W_EXPR_UNDEFINED_NULL));
        assert!(codes.contains(&W_EXPR_ARRAY_METHODS));
    }
    #[test]
    fn tc_reserved_codes() {
        assert!(RESERVED_CODES.contains(&"E-WCB-TRAP"));
    }
}
