//! JSON-RPC 2.0 wire types (subset untuk MCP).
//! Framing: newline-delimited di stdio (frame.rs); pesan tunggal tanpa newline embedded.

use serde_json::{json, Value};

pub const JSONRPC: &str = "2.0";

// JSON-RPC standard error codes
pub const E_PARSE: i64 = -32700;
pub const E_INVALID_REQUEST: i64 = -32600;
pub const E_METHOD_NOT_FOUND: i64 = -32601;
pub const E_INVALID_PARAMS: i64 = -32602;
pub const E_INTERNAL: i64 = -32603;

#[derive(Debug, Clone)]
pub struct RpcRequest {
    pub id: Option<Value>,       // null|number|string; None => notification
    pub method: String,
    pub params: Value,           // struct|array|null
    pub meta: Option<Value>,     // _meta (2026-07-28: protocolVersion per-request)
}

#[derive(Debug, Clone)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum RpcOut {
    Response { id: Value, result: Value },
    Error { id: Value, error: RpcError },
    /// notification diterima; tidak ada balasan
    None_,
}

pub fn parse_request(line: &str) -> Result<RpcRequest, RpcError> {
    let v: Value = serde_json::from_str(line)
        .map_err(|e| RpcError { code: E_PARSE, message: format!("Parse error: {e}"), data: None })?;
    let obj = v.as_object()
        .ok_or_else(|| RpcError { code: E_INVALID_REQUEST, message: "bukan objek".into(), data: None })?;
    if obj.get("jsonrpc").and_then(|x| x.as_str()) != Some(JSONRPC) {
        return Err(RpcError { code: E_INVALID_REQUEST, message: "jsonrpc != 2.0".into(), data: None });
    }
    let method = obj.get("method").and_then(|m| m.as_str())
        .ok_or_else(|| RpcError { code: E_INVALID_REQUEST, message: "method wajib string".into(), data: None })?
        .to_string();
    let params = obj.get("params").cloned().unwrap_or(Value::Null);
    let id = match obj.get("id") {
        None | Some(Value::Null) => None,
        Some(other) => Some(other.clone()),
    };
    let meta = obj.get("_meta").cloned();
    Ok(RpcRequest { id, method, params, meta })
}

impl RpcRequest {
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

pub fn error_response(id: Option<Value>, code: i64, message: &str, data: Option<Value>) -> Value {
    let idv = id.unwrap_or(Value::Null);
    json!({"jsonrpc": JSONRPC, "id": idv, "error": {"code": code, "message": message,
           "data": data.unwrap_or(Value::Null)}})
}

pub fn result_response(id: Option<Value>, result: Value) -> Value {
    let idv = id.unwrap_or(Value::Null);
    json!({"jsonrpc": JSONRPC, "id": idv, "result": result})
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-01..06: framing/log sanity di level pesan tunggal
    #[test]
    fn tc_parse_ok() {
        let r = parse_request(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).unwrap();
        assert_eq!(r.method, "ping");
        assert!(r.id.is_some());
    }
    #[test]
    fn tc_parse_notification() {
        let r = parse_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).unwrap();
        assert!(r.is_notification());
    }
    #[test]
    fn tc_parse_invalid() {
        let e = parse_request(r#"{"id":1}"#).unwrap_err();
        assert_eq!(e.code, E_INVALID_REQUEST);
        let e2 = parse_request("bukan json").unwrap_err();
        assert_eq!(e2.code, E_PARSE);
    }
    #[test]
    fn tc_parse_meta_2026() {
        let r = parse_request(r#"{"jsonrpc":"2.0","id":5,"method":"tools/list","_meta":{"protocolVersion":"2026-07-28"}}"#).unwrap();
        assert_eq!(r.meta.unwrap()["protocolVersion"], "2026-07-28");
    }
}
