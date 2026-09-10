//! Facade layanan (kontrak agent1 #503) + 5 tools authoring:
//! inspect_workflow, patch_node, validate, get_receipt, preflight.
//! Authoring-only: TIDAK ada eksekusi/fetch/kredensial (MCP-11/H-3/D-5).
//! Redaksi oracle diterapkan di batas facade (A7-6) — lihat server.rs.

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::diag::{validate_workflow, DiagnosticReport, MiniWorkflow};
use crate::redact::{redact_value, TaintLevel, taint_label};

#[derive(Clone)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
}

pub fn tool_defs() -> Vec<ToolDef> {
    vec![
        ToolDef { name: "inspect_workflow", description: "Peta topologi workflow ringkas (authoring)", schema: json!({
            "type":"object","properties":{"workflow_id":{"type":"string"}},"required":["workflow_id"]}) },
        ToolDef { name: "patch_node", description: "Terapkan ops RFC 6902 subset pada parameter node; receipt menjadi stale (wajib re-validate)", schema: json!({
            "type":"object","properties":{"workflow_id":{"type":"string"},"name":{"type":"string"},"ops":{"type":"array","items":{"type":"object"}}},"required":["workflow_id","name","ops"]}) },
        ToolDef { name: "validate", description: "Validasi workflow (E-*/W-* F-6) + DiagnosticReport terstruktur + autofix", schema: json!({
            "type":"object","properties":{"workflow_id":{"type":"string"}},"required":["workflow_id"]}) },
        ToolDef { name: "get_receipt", description: "Receipt Rosetta ringkas workflow", schema: json!({
            "type":"object","properties":{"workflow_id":{"type":"string"}},"required":["workflow_id"]}) },
        ToolDef { name: "preflight", description: "Estimasi eksternal & kredensial-butuh (refs-only, TANPA nilai) + fanout; est_token tanpa fetch (H-3b)", schema: json!({
            "type":"object","properties":{"workflow_id":{"type":"string"}},"required":["workflow_id"]}) },
    ]
}

pub fn tools_list() -> Value {
    json!({"tools": tool_defs().iter().map(|t| json!({
        "name": t.name, "description": t.description, "inputSchema": t.schema
    })).collect::<Vec<_>>()})
}

/// Store prototipe (dalam memori; engine nyata = WorkflowService).
pub struct Facade {
    workflows: HashMap<String, MiniWorkflow>,
    /// receipt: content_version saat validate terakhir + taint input
    receipts: HashMap<String, Receipt>,
}
#[derive(Clone, Debug)]
pub struct Receipt {
    pub content_version: u64,
    pub taint: TaintLevel,
    pub ts: &'static str,
}

fn sample_ok() -> MiniWorkflow {
    serde_json::from_value(json!({
        "nodes": [
            {"name":"Trigger","type":"n8n-nodes-base.manualTrigger","parameters":{}},
            {"name":"Set","type":"n8n-nodes-base.set","parameters":{"v":"={{ $json.x }}"} },
            {"name":"End","type":"n8n-nodes-base.noOp","parameters":{}}
        ],
        "connections": {"Trigger":[{"type":"main","nodes":["Set"]}],"Set":[{"type":"main","nodes":["End"]}]}
    })).unwrap()
}
fn sample_bad() -> MiniWorkflow {
    serde_json::from_value(json!({
        "nodes": [
            {"name":"A","type":"n8n-nodes-base.manualTrigger","parameters":{}},
            {"name":"A","type":"n8n-nodes-base.set","parameters":{}},
            {"name":"B","type":"n8n-nodes-base.set","parameters":{"x":"={{ $node[\"Ghost\"].json.v }}","y":"={{ undefined === null }}","z":"={{ Object.keys({b:1,a:2}) }}"}}
        ],
        "connections": {"A":[{"type":"main","nodes":["B"]}]}
    })).unwrap()
}
fn sample_orders() -> MiniWorkflow {
    serde_json::from_value(json!({
        "nodes": [
            {"name":"Fetch","type":"n8n-nodes-base.httpRequest","parameters":{"url":"https://api.example.test/orders"}},
            {"name":"Extract","type":"n8n-nodes-base.code","parameters":{"code":"={{ [1,2,3].at(-1) }}"}}
        ],
        "connections": {"Fetch":[{"type":"main","nodes":["Extract"]}]}
    })).unwrap()
}

impl Default for Facade {
    fn default() -> Self {
        Self::new()
    }
}

impl Facade {
    pub fn new() -> Self {
        let mut workflows = HashMap::new();
        workflows.insert("wf-demo-ok".into(), sample_ok());
        workflows.insert("wf-demo-bad".into(), sample_bad());
        workflows.insert("wf-orders".into(), sample_orders());
        let mut receipts = HashMap::new();
        receipts.insert("wf-demo-ok".into(), Receipt { content_version: 1, taint: TaintLevel::Internal, ts: "2026-09-09T00:00:00Z" });
        receipts.insert("wf-demo-bad".into(), Receipt { content_version: 1, taint: TaintLevel::External, ts: "2026-09-09T00:00:00Z" });
        receipts.insert("wf-orders".into(), Receipt { content_version: 1, taint: TaintLevel::External, ts: "2026-09-09T00:00:00Z" });
        Facade { workflows, receipts }
    }

    pub fn call(&mut self, tool: &str, params: &Value) -> Result<Value, Value> {
        let workflow_id = params.get("workflow_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let err = |msg: &str| Err(json!({"isError": true, "error": msg}));
        match tool {
            "inspect_workflow" => {
                let wf = self.workflows.get(&workflow_id).ok_or_else(|| json!({"isError": true, "error": "workflow tidak ditemukan"}))?;
                let mut topo = Vec::new();
                for n in &wf.nodes {
                    let mut succ: Vec<String> = Vec::new();
                    if let Some(conns) = wf.connections.get(&n.name).and_then(|o| o.as_object()) {
                        for out in conns.values() {
                            if let Some(arr) = out.as_array() {
                                for e in arr {
                                    if let Some(nodes) = e.get("nodes").and_then(|nd| nd.as_array()) {
                                        for v in nodes {
                                            if let Some(name) = v.as_str() { succ.push(name.to_string()); }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    topo.push(json!({"name": n.name, "type": n.node_type, "succ": succ}));
                }
                let t = self.receipts.get(&workflow_id).map(|r| taint_label(r.taint)).unwrap_or("INTERNAL");
                Ok(json!({"workflow_id": workflow_id, "topo": topo, "taint_flags": t}))
            }
            "validate" => {
                let wf = self.workflows.get(&workflow_id).ok_or_else(|| json!({"isError": true, "error": "workflow tidak ditemukan"}))?;
                let rep = validate_workflow(wf);
                let r = self.receipts.get_mut(&workflow_id).unwrap();
                r.content_version += 1;
                Ok(report_json(&rep))
            }
            "patch_node" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let ops = params.get("ops").cloned().unwrap_or(json!([]));
                let wf = self.workflows.get_mut(&workflow_id).ok_or_else(|| json!({"isError": true, "error": "workflow tidak ditemukan"}))?;
                let node = wf.nodes.iter_mut().find(|n| n.name == name)
                    .ok_or_else(|| json!({"isError": true, "error": format!("node {name} tidak ada")}))?;
                let ops_arr = ops.as_array().ok_or_else(|| json!({"isError": true, "error": "ops wajib array"}))?;
                if ops_arr.len() > 64 { return err("ops > 64 — batas authoring"); }
                let mut patched = Vec::new();
                for op in ops_arr {
                    let path = op.get("path").and_then(|p| p.as_str()).unwrap_or("");
                    if path.to_lowercase().contains("credential") {
                        return err("jalur credentials dilarang di-patch (refs-only, D-6/F-9)");
                    }
                    apply_op(&mut node.parameters, op)?;
                    let preview = op.get("value").map(|v| {
                        let t = v.to_string();
                        if t.chars().count() > 120 { format!("{}…", t.chars().take(120).collect::<String>()) } else { t }
                    }).unwrap_or_default();
                    patched.push(json!({"path": path, "value": preview}));
                }
                // receipt menjadi stale (H-7 workflow-level): re-validate wajib
                if let Some(r) = self.receipts.get_mut(&workflow_id) {
                    r.content_version += 1;
                }
                Ok(json!({"applied": true, "receipt": "stale — jalankan validate ulang", "patched": patched}))
            }
            "get_receipt" => {
                let r = self.receipts.get(&workflow_id)
                    .ok_or_else(|| json!({"isError": true, "error": "receipt tidak ditemukan"}))?;
                Ok(json!({"workflow_id": workflow_id, "content_version": r.content_version,
                          "rosetta": "ok", "taint": taint_label(r.taint), "ts": r.ts}))
            }
            "preflight" => {
                let wf = self.workflows.get(&workflow_id).ok_or_else(|| json!({"isError": true, "error": "workflow tidak ditemukan"}))?;
                let mut ext = Vec::new();
                let mut cred = Vec::new();
                for n in &wf.nodes {
                    let s = n.parameters.to_string();
                    if s.contains("http") || n.node_type.contains("http") {
                        ext.push(format!("{}/http-eksternal", n.name));
                    }
                    if n.node_type.contains("http") { cred.push(format!("httpRequest:{}", n.name)); }
                }
                Ok(json!({
                    "workflow_id": workflow_id,
                    "external": ext,
                    "credentials_required": cred, // refs SAJA — tanpa nilai
                    "fanout": wf.nodes.len(),
                    "est_token": crate::frame::est_tokens(&wf.connections.to_string()) * 4,
                    "note": "est_token = metadata tanpa fetch (H-3b); nilai kredensial tidak pernah disajikan"
                }))
            }
            _ => err("tool tidak dikenal (authoring-only: eksekusi/fetch tidak tersedia — MCP-11/H-3)"),
        }
    }
}

pub fn report_json(rep: &DiagnosticReport) -> Value {
    json!({
        "valid": rep.valid,
        "diagnostics": {
            "errors": rep.errors,
            "warnings": rep.warnings
        },
        "hint": if rep.valid { "bersih" } else { "lihat autofix per diagnostic" }
    })
}

/// RFC 6902 subset: replace/add/remove pada path /a/b/c (objek; array index numerik).
fn apply_op(params: &mut Value, op: &Value) -> Result<(), Value> {
    let o = op.as_object().ok_or_else(|| json!({"isError": true, "error": "op wajib objek"}))?;
    let path = o.get("path").and_then(|p| p.as_str()).unwrap_or("").to_string();
    let segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let node = segs.last().map(|s| s.to_string());
    let parent_segs = &segs[..segs.len().saturating_sub(1)];
    let mut cur = params;
    for seg in parent_segs {
        cur = cur.get_mut(*seg).ok_or_else(|| json!({"isError": true, "error": format!("path {path} tidak ada")}))?;
    }
    match o.get("op").and_then(|v| v.as_str()).unwrap_or("") {
        "replace" | "add" => {
            if let Some(k) = node {
                if let Some(map) = cur.as_object_mut() {
                    map.insert(k, o.get("value").cloned().unwrap_or(Value::Null));
                } else if let Some(arr) = cur.as_array_mut() {
                    if let Ok(idx) = k.parse::<usize>() {
                        if idx < arr.len() { arr[idx] = o.get("value").cloned().unwrap_or(Value::Null); }
                        else { arr.push(o.get("value").cloned().unwrap_or(Value::Null)); }
                    } else { return Err(json!({"isError": true, "error": "index array tidak valid"})); }
                } else { return Err(json!({"isError": true, "error": "parent bukan objek/array"})); }
            } else { return Err(json!({"isError": true, "error": "path kosong"})); }
        }
        "remove" => {
            if let Some(k) = node {
                if let Some(map) = cur.as_object_mut() { map.remove(&k); }
                else if let Some(arr) = cur.as_array_mut() {
                    if let Ok(idx) = k.parse::<usize>() { if idx < arr.len() { arr.remove(idx); } }
                    else { return Err(json!({"isError": true, "error": "index array tidak valid"})); }
                }
            }
        }
        other => return Err(json!({"isError": true, "error": format!("op {other} tidak didukung (subset: replace/add/remove)")})),
    }
    Ok(())
}

pub fn facade_reply(tool: &str, result: &Value) -> Value {
    // Redaksi oracle di batas facade (A7-6): seluruh konten string di-redact.
    let redacted = redact_value(result);
    json!({
        "content": [{"type": "text", "text": redacted.to_string()}],
        "structuredContent": redacted,
        "isError": redacted.get("isError").and_then(|v| v.as_bool()).unwrap_or(false) || tool == "__none__",
        "tool": tool
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tc_tools_list_has_5() {
        let v = tools_list();
        assert_eq!(v["tools"].as_array().unwrap().len(), 5);
        let names: Vec<&str> = v["tools"].as_array().unwrap().iter()
            .map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(names, vec!["inspect_workflow","patch_node","validate","get_receipt","preflight"]);
    }
    #[test]
    fn tc_validate_clean_and_bad() {
        let mut f = Facade::new();
        let ok = f.call("validate", &json!({"workflow_id": "wf-demo-ok"})).unwrap();
        assert_eq!(ok["valid"], true);
        let bad = f.call("validate", &json!({"workflow_id": "wf-demo-bad"})).unwrap();
        assert_eq!(bad["valid"], false);
        let codes: Vec<String> = bad["diagnostics"]["errors"].as_array().unwrap().iter()
            .map(|e| e["code"].as_str().unwrap_or("").to_string()).collect();
        assert!(codes.contains(&crate::diag::E_DUP_NODE_NAME.to_string()));
        assert!(codes.contains(&crate::diag::E_REF_DANGLING.to_string()));
    }
    #[test]
    fn tc_patch_node_and_stale_receipt() {
        let mut f = Facade::new();
        let r = f.call("patch_node", &json!({"workflow_id":"wf-demo-ok","name":"Set","ops":[
            {"op":"replace","path":"/v","value":"={{ $json.y }}"}]})).unwrap();
        assert_eq!(r["applied"], true);
        // receipt menua: validate → version bertambah
        let v1 = f.call("get_receipt", &json!({"workflow_id":"wf-demo-ok"})).unwrap()["content_version"].as_u64().unwrap();
        let _ = f.call("validate", &json!({"workflow_id":"wf-demo-ok"})).unwrap();
        let v2 = f.call("get_receipt", &json!({"workflow_id":"wf-demo-ok"})).unwrap()["content_version"].as_u64().unwrap();
        assert!(v2 > v1);
    }
    #[test]
    fn tc_patch_credentials_denied() {
        let mut f = Facade::new();
        let r = f.call("patch_node", &json!({"workflow_id":"wf-demo-ok","name":"Set","ops":[
            {"op":"replace","path":"/credentials/token","value":"sekret"}]}));
        assert!(r.is_err());
    }
    // TC-30..36: uji negatif — tidak ada tool eksekusi
    #[test]
    fn tc_no_execution_tool() {
        let mut f = Facade::new();
        let r = f.call("execute_workflow", &json!({"workflow_id":"wf-demo-ok"}));
        assert!(r.is_err());
        let msg = r.unwrap_err();
        assert!(msg["error"].as_str().unwrap_or("").contains("MCP-11"));
    }
    #[test]
    fn tc_preflight_refs_only_no_values() {
        let mut f = Facade::new();
        let p = f.call("preflight", &json!({"workflow_id":"wf-orders"})).unwrap();
        assert!(p["credentials_required"].as_array().is_some());
        assert!(p["est_token"].is_number());
        assert!(!p.to_string().to_lowercase().contains("password"));
    }
    #[test]
    fn tc_taint_preserved_inspect() {
        let mut f = Facade::new();
        let r = f.call("inspect_workflow", &json!({"workflow_id":"wf-demo-bad"})).unwrap();
        assert_eq!(r["taint_flags"], "TAINTED-EXTERNAL");
    }
}
