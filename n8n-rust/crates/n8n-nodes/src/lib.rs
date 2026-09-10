//! n8n-nodes: node bawaan v1 (subset kecil yang tumbuh bertahap).
//!
//! v0.3.0: `manualTrigger`, `set` (+ekspresi), `noOp`, `filter`, `sort`,
//! `limit`, `if` (2 cabang: [true, false]), `httpRequest` (subset).
//! Nilai string di parameter dirender sebagai template `={{ }}`
//! (lihat `n8n_core::expr`).

use n8n_core::expr::{render, render_value, ExprContext};
use n8n_core::WorkflowNode;
use n8n_engine::{BranchOutputs, EngineResult, ExecContext, Node, Registry};
use serde_json::{json, Map, Value};
use std::sync::Arc;

pub struct ManualTrigger;
pub struct SetNode;
pub struct NoOp;
pub struct FilterNode;
pub struct SortNode;
pub struct LimitNode;
pub struct IfNode;
pub struct HttpNode;

impl Node for ManualTrigger {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.manualTrigger"
    }

    fn execute(
        &self,
        _node: &WorkflowNode,
        _items: Vec<Value>,
        _ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        Ok(vec![vec![Value::Object(Map::new())]])
    }
}

impl Node for NoOp {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.noOp"
    }

    fn execute(
        &self,
        _node: &WorkflowNode,
        items: Vec<Value>,
        _ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        Ok(vec![items])
    }
}

impl Node for SetNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.set"
    }

    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        if let Some(Value::Object(values)) = node.parameters.get("values") {
            let out: Vec<Value> = items
                .into_iter()
                .map(|it| {
                    let ectx = ExprContext {
                        item: &it,
                        outputs: ctx.outputs,
                    };
                    let rendered: Map<String, Value> = values
                        .iter()
                        .map(|(k, v)| {
                            let nv = match v {
                                Value::String(s) => render(s, &ectx),
                                other => other.clone(),
                            };
                            (k.clone(), nv)
                        })
                        .collect();
                    merge_value(it, &rendered)
                })
                .collect();
            return Ok(vec![out]);
        }
        if let Some(list) = node
            .parameters
            .get("assignments")
            .and_then(|v| v.get("assignments"))
            .and_then(Value::as_array)
        {
            let out: Vec<Value> = items
                .into_iter()
                .map(|it| {
                    let ectx = ExprContext {
                        item: &it,
                        outputs: ctx.outputs,
                    };
                    let mut values = Map::new();
                    for a in list.iter().filter_map(Value::as_object) {
                        if let (Some(Value::String(name)), Some(value)) =
                            (a.get("name"), a.get("value"))
                        {
                            let nv = match value {
                                Value::String(s) => render(s, &ectx),
                                other => other.clone(),
                            };
                            values.insert(name.clone(), nv);
                        }
                    }
                    merge_value(it, &values)
                })
                .collect();
            return Ok(vec![out]);
        }
        Ok(vec![items])
    }
}

impl Node for FilterNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.filter"
    }

    /// Hanya item yang `condition`-nya truthy yang diteruskan.
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        let cond = node
            .parameters
            .get("condition")
            .and_then(Value::as_str)
            .unwrap_or("");
        let out: Vec<Value> = items
            .into_iter()
            .filter(|it| {
                let ectx = ExprContext {
                    item: it,
                    outputs: ctx.outputs,
                };
                truthy(&render(cond, &ectx))
            })
            .collect();
        Ok(vec![out])
    }
}

impl Node for SortNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.sort"
    }

    /// Urutkan item menurut field (`field`), `order`: `"asc"` | `"desc"`.
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        let field = node
            .parameters
            .get("field")
            .and_then(Value::as_str)
            .unwrap_or("");
        let desc = node
            .parameters
            .get("order")
            .and_then(Value::as_str)
            .map(|s| s.eq_ignore_ascii_case("desc"))
            .unwrap_or(false);
        let template = format!("={{{{ $json.{field} }}}}");
        let mut out = items;
        out.sort_by(|a, b| {
            let ka = render(
                &template,
                &ExprContext {
                    item: a,
                    outputs: ctx.outputs,
                },
            );
            let kb = render(
                &template,
                &ExprContext {
                    item: b,
                    outputs: ctx.outputs,
                },
            );
            let ord = compare_values(&ka, &kb);
            if desc {
                ord.reverse()
            } else {
                ord
            }
        });
        Ok(vec![out])
    }
}

impl Node for LimitNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.limit"
    }

    /// Teruskan maksimal `count` item pertama (default 1).
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        _ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        let count = node
            .parameters
            .get("count")
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        Ok(vec![items.into_iter().take(count).collect()])
    }
}

impl Node for IfNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.if"
    }

    /// Belah item ke 2 cabang `[true, false]` menurut `condition`
    /// (template `={{ }}`, truthy). Cabang = `main[0]` / `main[1]`.
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        let cond = node
            .parameters
            .get("condition")
            .and_then(Value::as_str)
            .unwrap_or("");
        let mut yes = Vec::new();
        let mut no = Vec::new();
        for it in items {
            let ectx = ExprContext {
                item: &it,
                outputs: ctx.outputs,
            };
            if truthy(&render(cond, &ectx)) {
                yes.push(it);
            } else {
                no.push(it);
            }
        }
        Ok(vec![yes, no])
    }
}

impl Node for HttpNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.httpRequest"
    }

    /// Subset v1: SATU request per eksekusi (input items hanya jadi konteks
    /// ekspresi via item pertama — fan-out per item = tiket lanjutan).
    /// Parameter: `url` (wajib, template), `method` (default GET),
    /// `headers` (object, nilai di-render), `body` (JSON, di-render).
    /// Output: `[{status, headers, body, url}]`; body JSON di-parse bila bisa.
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        let item0 = items.first().unwrap_or(&Value::Null);
        let ectx = ExprContext {
            item: item0,
            outputs: ctx.outputs,
        };
        let url_v = node
            .parameters
            .get("url")
            .map(|v| render_value(v, &ectx))
            .unwrap_or(Value::Null);
        let url = url_v
            .as_str()
            .ok_or_else(|| EngineError::new("httpRequest: 'url' wajib string"))?
            .to_string();
        let method = node
            .parameters
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("GET")
            .to_uppercase();
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(Value::Object(h)) = node.parameters.get("headers") {
            for (k, v) in h {
                let rendered = render_value(v, &ectx);
                let s = match &rendered {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => {
                        return Err(EngineError::new(format!(
                            "httpRequest: nilai header '{k}' harus skalar"
                        )))
                    }
                };
                let name = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                    .map_err(|_| {
                        EngineError::new(format!("httpRequest: nama header tak valid '{k}'"))
                    })?;
                let value = reqwest::header::HeaderValue::from_str(&s).map_err(|_| {
                    EngineError::new(format!("httpRequest: nilai header '{k}' tak valid"))
                })?;
                headers.insert(name, value);
            }
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| EngineError::new(format!("httpRequest: client: {e}")))?;
        let http_method: reqwest::Method = method
            .parse()
            .map_err(|e| EngineError::new(format!("httpRequest: method: {e}")))?;
        let mut req = client.request(http_method, url.clone());
        req = req.headers(headers);
        if let Some(body) = node.parameters.get("body") {
            let rendered = render_value(body, &ectx);
            req = req.json(&rendered);
        }
        let resp = req
            .send()
            .map_err(|e| EngineError::new(format!("httpRequest: {e}")))?;
        let status = resp.status().as_u16();
        let mut rh = Map::new();
        for (k, v) in resp.headers().iter() {
            rh.insert(
                k.to_string(),
                Value::String(v.to_str().unwrap_or("").to_string()),
            );
        }
        let final_url = resp.url().to_string();
        let text = resp
            .text()
            .map_err(|e| EngineError::new(format!("httpRequest: baca body: {e}")))?;
        let body_v: Value = serde_json::from_str(&text).unwrap_or(Value::String(text));
        let mut out = Map::new();
        out.insert("status".to_string(), json!(status));
        out.insert("headers".to_string(), Value::Object(rh));
        out.insert("body".to_string(), body_v);
        out.insert("url".to_string(), Value::String(final_url));
        Ok(vec![vec![Value::Object(out)]])
    }
}

fn merge_value(item: Value, values: &Map<String, Value>) -> Value {
    match item {
        Value::Object(mut o) => {
            o.extend(values.clone());
            Value::Object(o)
        }
        other => {
            let mut o = Map::new();
            o.insert("value".to_string(), other);
            o.extend(values.clone());
            Value::Object(o)
        }
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => {
            let t = s.trim().to_lowercase();
            !(t.is_empty() || t == "false" || t == "0")
        }
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let rank = |v: &Value| -> u8 {
        match v {
            Value::Null => 0,
            Value::Bool(_) => 1,
            Value::Number(_) => 2,
            Value::String(_) => 3,
            Value::Array(_) => 4,
            Value::Object(_) => 5,
        }
    };
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let xf = x.as_f64().unwrap_or(f64::NAN);
            let yf = y.as_f64().unwrap_or(f64::NAN);
            xf.partial_cmp(&yf).unwrap_or(Ordering::Equal)
        }
        (Value::String(x), Value::String(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => rank(a).cmp(&rank(b)),
    }
}

pub fn register_all(registry: &mut Registry) {
    registry.register(Arc::new(ManualTrigger));
    registry.register(Arc::new(SetNode));
    registry.register(Arc::new(NoOp));
    registry.register(Arc::new(FilterNode));
    registry.register(Arc::new(SortNode));
    registry.register(Arc::new(LimitNode));
    registry.register(Arc::new(IfNode));
    registry.register(Arc::new(HttpNode));
}

#[cfg(test)]
mod tests {
    use super::*;
    use n8n_core::Workflow;
    use n8n_engine::Engine;
    use std::collections::HashMap;

    fn empty_ctx(outputs: &HashMap<String, BranchOutputs>) -> ExecContext<'_> {
        ExecContext { outputs }
    }

    fn set_node_with_values(values: serde_json::Value) -> WorkflowNode {
        WorkflowNode {
            id: "s".to_string(),
            name: "Set".to_string(),
            node_type: "n8n-nodes-base.set".to_string(),
            type_version: 3.4,
            position: [0.0, 0.0],
            parameters: HashMap::from([("values".to_string(), values)]),
            disabled: false,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn fixture_runs_end_to_end() {
        let raw = include_str!("../../../fixtures/manual-to-set.json");
        let wf = Workflow::from_json(raw).expect("parse fixture");
        let mut registry = Registry::default();
        register_all(&mut registry);
        let report = Engine::run(&wf, &registry).expect("run");
        assert_eq!(
            report.order,
            vec![
                "Manual Trigger".to_string(),
                "Set".to_string(),
                "NoOp".to_string()
            ]
        );
        assert_eq!(
            report.outputs["NoOp"][0],
            vec![serde_json::json!({ "greeting": "halo", "n": 1 })]
        );
        assert_eq!(report.durations_ms.len(), 3);
    }

    #[test]
    fn set_supports_n8n_assignments_shape() {
        let node = WorkflowNode {
            id: "s".to_string(),
            name: "Set".to_string(),
            node_type: "n8n-nodes-base.set".to_string(),
            type_version: 3.4,
            position: [0.0, 0.0],
            parameters: HashMap::from([(
                "assignments".to_string(),
                serde_json::json!({ "assignments": [{ "name": "a", "value": 1 }] }),
            )]),
            disabled: false,
            extra: HashMap::new(),
        };
        let outputs = HashMap::new();
        let out = SetNode
            .execute(&node, vec![serde_json::json!({})], &empty_ctx(&outputs))
            .expect("exec");
        assert_eq!(out, vec![vec![serde_json::json!({ "a": 1 })]]);
    }

    #[test]
    fn set_renders_expressions_per_item() {
        let node =
            set_node_with_values(serde_json::json!({"who": "={{ $json.name }}", "n": 5}));
        let outputs = HashMap::new();
        let out = SetNode
            .execute(
                &node,
                vec![serde_json::json!({"name": "udi"})],
                &empty_ctx(&outputs),
            )
            .expect("exec");
        assert_eq!(
            out,
            vec![vec![serde_json::json!({"name": "udi", "who": "udi", "n": 5})]]
        );
    }

    #[test]
    fn set_reads_other_node_output() {
        let node =
            set_node_with_values(serde_json::json!({"x": "={{ $node[\"Up\"].json.x }}"}));
        let outputs = HashMap::from([("Up".to_string(), vec![vec![json!({"x": 7})]])]);
        let out = SetNode
            .execute(&node, vec![serde_json::json!({})], &empty_ctx(&outputs))
            .expect("exec");
        assert_eq!(out, vec![vec![serde_json::json!({"x": 7})]]);
    }

    #[test]
    fn filter_keeps_only_truthy_items() {
        let node = WorkflowNode {
            id: "f".to_string(),
            name: "Filter".to_string(),
            node_type: "n8n-nodes-base.filter".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: HashMap::from([(
                "condition".to_string(),
                serde_json::json!("={{ $json.keep }}"),
            )]),
            disabled: false,
            extra: HashMap::new(),
        };
        let outputs = HashMap::new();
        let out = FilterNode
            .execute(
                &node,
                vec![
                    serde_json::json!({"keep": true}),
                    serde_json::json!({"keep": false}),
                    serde_json::json!({"keep": 0}),
                    serde_json::json!({"keep": "x"}),
                ],
                &empty_ctx(&outputs),
            )
            .expect("exec");
        assert_eq!(
            out,
            vec![vec![
                serde_json::json!({"keep": true}),
                serde_json::json!({"keep": "x"})
            ]]
        );
    }

    #[test]
    fn sort_orders_by_field() {
        let node = WorkflowNode {
            id: "s".to_string(),
            name: "Sort".to_string(),
            node_type: "n8n-nodes-base.sort".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: HashMap::from([
                ("field".to_string(), serde_json::json!("age")),
                ("order".to_string(), serde_json::json!("asc")),
            ]),
            disabled: false,
            extra: HashMap::new(),
        };
        let outputs = HashMap::new();
        let out = SortNode
            .execute(
                &node,
                vec![
                    serde_json::json!({"age": 3}),
                    serde_json::json!({"age": 1}),
                    serde_json::json!({"age": 2}),
                ],
                &empty_ctx(&outputs),
            )
            .expect("exec");
        assert_eq!(
            out,
            vec![vec![
                serde_json::json!({"age": 1}),
                serde_json::json!({"age": 2}),
                serde_json::json!({"age": 3})
            ]]
        );
    }

    #[test]
    fn limit_takes_first_n() {
        let node = WorkflowNode {
            id: "l".to_string(),
            name: "Limit".to_string(),
            node_type: "n8n-nodes-base.limit".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: HashMap::from([("count".to_string(), serde_json::json!(2))]),
            disabled: false,
            extra: HashMap::new(),
        };
        let outputs = HashMap::new();
        let out = LimitNode
            .execute(
                &node,
                vec![
                    serde_json::json!(1),
                    serde_json::json!(2),
                    serde_json::json!(3),
                ],
                &empty_ctx(&outputs),
            )
            .expect("exec");
        assert_eq!(
            out,
            vec![vec![serde_json::json!(1), serde_json::json!(2)]]
        );
    }

    #[test]
    fn if_splits_into_two_branches() {
        let node = WorkflowNode {
            id: "i".to_string(),
            name: "If".to_string(),
            node_type: "n8n-nodes-base.if".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: HashMap::from([(
                "condition".to_string(),
                serde_json::json!("={{ $json.age > 18 }}"),
            )]),
            disabled: false,
            extra: HashMap::new(),
        };
        let outputs = HashMap::new();
        let out = IfNode
            .execute(
                &node,
                vec![
                    serde_json::json!({"age": 20}),
                    serde_json::json!({"age": 10}),
                ],
                &empty_ctx(&outputs),
            )
            .expect("exec");
        assert_eq!(
            out,
            vec![
                vec![serde_json::json!({"age": 20})],
                vec![serde_json::json!({"age": 10})]
            ]
        );
    }

    #[test]
    fn http_get_returns_canned_response() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let port = listener.local_addr().expect("addr").port();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = r#"{"ok":true}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).expect("write");
        });

        let node = WorkflowNode {
            id: "h".to_string(),
            name: "HTTP".to_string(),
            node_type: "n8n-nodes-base.httpRequest".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: HashMap::from([
                (
                    "url".to_string(),
                    serde_json::json!(format!("http://127.0.0.1:{port}/echo")),
                ),
                ("method".to_string(), serde_json::json!("GET")),
            ]),
            disabled: false,
            extra: HashMap::new(),
        };
        let outputs = HashMap::new();
        let out = HttpNode
            .execute(&node, vec![serde_json::json!({})], &empty_ctx(&outputs))
            .expect("exec");
        handle.join().expect("server thread");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].len(), 1);
        assert_eq!(out[0][0]["status"], json!(200));
        assert_eq!(out[0][0]["body"], json!({"ok": true}));
    }
}
