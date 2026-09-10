//! Router & adapter protokol (dual binding 2025/2026 per D-1/D-2).
//! 2025: initialize/initialized/ping/instructions. 2026-07-28: stateless,
//! server/discover, _meta.protocolVersion per-request. Tanpa state sesi (F-7).

use serde_json::{json, Value};
use crate::frame::validate_frame;
use crate::jsonrpc::{parse_request, RpcRequest, RpcError, result_response, error_response, E_INVALID_PARAMS, E_METHOD_NOT_FOUND};
use crate::registry;
use crate::tools::{self, Facade};

pub const PROTOCOL_VERSIONS: &[&str] = &["2026-07-28", "2025-11-25", "2025-06-18", "2025-03-26"];
pub const SERVER_INFO: &str = "n8n-rust-engine-mcp (crates/mcp kanonik)";

pub struct ServerCtx {
    pub facade: Facade,
    /// versi yang dinegosiasikan via initialize (2025) — tidak dipakai utk otorisasi (stateless)
    pub negotiated: Option<String>,
    pub initialized_2025: bool,
}

impl Default for ServerCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerCtx {
    pub fn new() -> Self {
        ServerCtx { facade: Facade::new(), negotiated: None, initialized_2025: false }
    }

    /// Proses satu baris request → balasan (None utk notification).
    pub fn handle_line(&mut self, line: &str) -> Option<Value> {
        if let Err(e) = validate_frame(line) {
            return Some(error_response(Some(Value::Null), crate::jsonrpc::E_INVALID_REQUEST, &e, None));
        }
        if line.trim().is_empty() { return None; }
        let req = match parse_request(line) {
            Ok(r) => r,
            Err(e) => return Some(error_response(None, e.code, &e.message, e.data)),
        };
        let is_notif = req.is_notification();
        let out = self.route(&req);
        if is_notif { None } else { Some(out) }
    }

    fn route(&mut self, req: &RpcRequest) -> Value {
        let m = req.method.as_str();
        let result = match m {
            // ── keluarga 2025 ──
            "initialize" => self.adapter_2025_initialize(&req.params),
            "ping" => Ok(json!({})),
            "notifications/initialized" => { self.initialized_2025 = true; return result_response(req.id.clone(), json!({})); },
            // ── keluarga 2026: stateless ──
            "server/discover" => Ok(self.discover_payload(req)),
            // ── tools ──
            "tools/list" => Ok(tools::tools_list()),
            "tools/call" => self.tools_call(&req.params),
            // ── resources ──
            "resources/list" => Ok(resources_list()),
            "resources/read" => self.resources_read(&req.params),
            // ── lainnya: authoring-only (MCP-11/H-3) ──
            "completion/complete" => Ok(json!({"completion": {"values": [], "hasMore": false}})),
            _ => Err(RpcError { code: E_METHOD_NOT_FOUND, message: format!("metode tidak dikenal: {m}"), data: None }),
        };
        match result {
            Ok(v) => result_response(req.id.clone(), v),
            Err(e) => error_response(req.id.clone(), e.code, &e.message, e.data),
        }
    }

    fn adapter_2025_initialize(&mut self, params: &Value) -> Result<Value, RpcError> {
        let client_v = params.get("protocolVersion").and_then(|v| v.as_str()).unwrap_or("");
        let negotiated = if PROTOCOL_VERSIONS.contains(&client_v) { client_v } else { PROTOCOL_VERSIONS[0] }.to_string();
        self.negotiated = Some(negotiated.clone());
        // perintah singkat (2025): instruksi inject-at-connect; TIDAK dipakai di 2026
        Ok(json!({
            "protocolVersion": negotiated,
            "capabilities": {
                "tools": { "listChanged": false },
                "resources": { "subscribe": false, "listChanged": false }
            },
            "serverInfo": { "name": SERVER_INFO, "version": "0.1.0" },
            "instructions": "MCP authoring-only (D-5): tools validasi/patch/preflight, TANPA eksekusi, TANPA fetch, kredensial refs-only (MCP-11/H-3). Resources n8n:// & hub:// berisi konten kontrak ≤500 token. Redaksi aktif di facade."
        }))
    }

    fn discover_payload(&self, req: &RpcRequest) -> Value {
        // 2026-07-28: tanpa handshake; _meta.protocolVersion per request
        let _pv = req.meta.as_ref().and_then(|m| m.get("protocolVersion")).and_then(|v| v.as_str()).unwrap_or("2026-07-28");
        json!({
            "protocolVersion": PROTOCOL_VERSIONS[0],
            "capabilities": { "tools": {"listChanged": false}, "resources": {"subscribe": false} },
            "serverInfo": { "name": SERVER_INFO, "version": "0.1.0" },
            "features": { "authoring_only": true, "stateless": true }
        })
    }

    fn tools_call(&mut self, params: &Value) -> Result<Value, RpcError> {
        let name = params.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| RpcError { code: E_INVALID_PARAMS, message: "name tool wajib".into(), data: None })?;
        if !tools::tool_defs().iter().any(|t| t.name == name) {
            // SEP-1303: tool tak dikenal = Tool Execution Error (bukan protocol error)
            return Ok(json!({
                "content": [{"type":"text","text":"tool tidak dikenal"}],
                "structuredContent": {"isError": true, "error": format!("tool {name} tidak ada — authoring-only (MCP-11: tidak ada tool eksekusi/fetch)")},
                "isError": true
            }));
        }
        let args = params.get("arguments").cloned().unwrap_or(Value::Null);
        let call = params.get("_meta").is_none(); // abaikan
        let _ = call;
        match self.facade.call(name, &args) {
            Ok(result) => Ok(tools::facade_reply(name, &result)),
            Err(v) => Ok(tools::facade_reply(name, &v)),
        }
    }

    fn resources_read(&self, params: &Value) -> Result<Value, RpcError> {
        let uri = params.get("uri").and_then(|v| v.as_str())
            .ok_or_else(|| RpcError { code: E_INVALID_PARAMS, message: "uri wajib".into(), data: None })?;
        if !uri.starts_with("n8n://") && !uri.starts_with("hub://") {
            return Ok(json!({
                "contents": [], "isError": true,
                "error": {"code": "E-URI-NAMESPACE", "message": "hanya namespace n8n:// dan hub:// yang dilayani"}
            }));
        }
        match registry::build_content(uri) {
            Ok(content) => {
                let (payload, truncated) = crate::frame::clip_package(&content, crate::frame::MAX_TOKEN_PER_PACKAGE);
                Ok(json!({"contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": payload.to_string(),
                    "metadata": {"truncated": truncated, "owner": content["owner"].clone(),
                                 "content_version": content["content_version"].clone(),
                                 "i18n_rev": content["i18n_rev"].clone(), "taint": content["taint"].clone()}
                }]}))
            }
            Err(msg) => Ok(json!({"contents": [], "isError": true,
                "error": {"code": "E-RESOURCE-REFUSED", "message": msg}})),
        }
    }
}

fn resources_list() -> Value {
    json!({"resources": registry::URI_ROWS.iter().map(|r| json!({
        "uri": r.pattern.replace("{name}","HTTP Request").replace("{type}","httpRequest")
            .replace("{code}","E-REF-DANGLING").replace("{id}","demo").replace("{hub_id}","bm-macro")
            .replace("{DEV-*}","DEV-0001"),
        "name": r.pattern, "mimeType": "application/json", "owner": r.owner
    })).collect::<Vec<_>>()})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send(ctx: &mut ServerCtx, lines: &[&str]) -> Vec<Value> {
        lines.iter().filter_map(|l| ctx.handle_line(l)).collect()
    }

    // TC-20..23: initialize 2025
    #[test]
    fn tc_initialize_2025() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}"#]);
        assert_eq!(out[0]["result"]["protocolVersion"], "2025-11-25");
        assert!(out[0]["result"]["instructions"].as_str().unwrap().contains("authoring-only"));
    }
    #[test]
    fn tc_initialize_unknown_version_negotiate_latest() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2099-01-01"}}"#]);
        assert_eq!(out[0]["result"]["protocolVersion"], "2026-07-28");
    }
    // TC-10..15: 2026 stateless — langsung tools/list tanpa handshake
    #[test]
    fn tc_stateless_tools_list() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":9,"method":"tools/list","_meta":{"protocolVersion":"2026-07-28"}}"#]);
        assert_eq!(out[0]["result"]["tools"].as_array().unwrap().len(), 5);
    }
    #[test]
    fn tc_server_discover() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":3,"method":"server/discover"}"#]);
        assert_eq!(out[0]["result"]["features"]["authoring_only"], true);
        assert_eq!(out[0]["result"]["protocolVersion"], "2026-07-28");
    }
    #[test]
    fn tc_notification_no_response() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#]);
        assert!(out.is_empty());
    }
    // TC-30..36: no-execution-via-MCP
    #[test]
    fn tc_call_unknown_tool_is_error() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"run_workflow","arguments":{}}}"#]);
        assert_eq!(out[0]["result"]["isError"], true);
        assert!(out[0]["result"]["structuredContent"]["error"].as_str().unwrap().contains("MCP-11"));
    }
    // TC-40..49: tools via wire
    #[test]
    fn tc_wire_validate_bad() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"validate","arguments":{"workflow_id":"wf-demo-bad"}}}"#]);
        let sc = &out[0]["result"]["structuredContent"];
        assert_eq!(sc["valid"], false);
        assert_eq!(out[0]["result"]["isError"], false);
        let text = out[0]["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("E-REF-DANGLING") || text.contains("E-DUP-NODE-NAME"));
    }
    // TC-50..59: resources via wire
    #[test]
    fn tc_wire_resources_read_rules() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":6,"method":"resources/read","params":{"uri":"n8n://expression-rules"}}"#]);
        let txt = out[0]["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(txt.contains("content_version") || txt.contains("summary"));
        assert_eq!(out[0]["result"]["contents"][0]["metadata"]["owner"], "agent3");
    }
    #[test]
    fn tc_wire_reserved_refused() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":7,"method":"resources/read","params":{"uri":"n8n://errors/E-WCB-TRAP"}}"#]);
        assert_eq!(out[0]["result"]["isError"], true);
        assert!(out[0]["result"]["error"]["message"].as_str().unwrap().contains("RESERVED"));
    }
    // TC-60..64: redaksi di facade
    #[test]
    fn tc_wire_redaction() {
        let mut c = ServerCtx::new();
        // patch memasukkan email ke parameter; respons harus ter-redaksi
        let line = r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"patch_node","arguments":{"workflow_id":"wf-demo-ok","name":"Set","ops":[{"op":"replace","path":"/note","value":"hubungi admin@engine.test"}]}}}"#;
        let out = send(&mut c, &[line]);
        let s = out[0].to_string();
        assert!(!s.contains("admin@engine.test"));
        assert!(s.contains("[EMAIL]") || s.contains("EMAIL"));
    }
    #[test]
    fn tc_missing_params() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &[r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{}}"#]);
        assert!(out[0]["error"].is_object() || out[0]["result"]["isError"] == true);
    }
    #[test]
    fn tc_invalid_json() {
        let mut c = ServerCtx::new();
        let out = send(&mut c, &["{invalid"]);
        assert_eq!(out[0]["error"]["code"], crate::jsonrpc::E_PARSE);
    }
    // sesi penuh (transkrip nyata stdio)
    #[test]
    fn tc_full_transcript() {
        let mut c = ServerCtx::new();
        let script = [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28"}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"validate","arguments":{"workflow_id":"wf-demo-bad"}}}"#,
            r#"{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"hub://skills/bm-macro/receipt"}}"#,
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"preflight","arguments":{"workflow_id":"wf-orders"}}}"#,
        ];
        let out = send(&mut c, &script);
        assert_eq!(out.len(), 5);
        assert!(out[0]["result"]["serverInfo"]["name"].as_str().unwrap().contains("n8n-rust"));
        assert_eq!(out[1]["result"]["tools"].as_array().unwrap().len(), 5);
        assert_eq!(out[2]["result"]["structuredContent"]["valid"], false);
        assert_eq!(out[3]["result"]["contents"][0]["metadata"]["owner"], "agent4");
        assert!(out[4]["result"]["structuredContent"]["est_token"].is_number());
    }
    #[test]
    fn tc_every_response_has_jsonrpc_and_id() {
        let mut c = ServerCtx::new();
        for id in 1..=6 {
            let line = format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/list"}}"#);
            let out = c.handle_line(&line).unwrap();
            assert_eq!(out["jsonrpc"], "2.0");
            assert_eq!(out["id"], id);
        }
    }
    #[test]
    fn tc_large_frame_rejected() {
        let mut c = ServerCtx::new();
        let big = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"ping","params":"{}"}}"#, "x".repeat(crate::frame::MAX_FRAME_BYTES));
        let out = c.handle_line(&big).unwrap();
        assert!(out["error"].is_object());
    }
}
