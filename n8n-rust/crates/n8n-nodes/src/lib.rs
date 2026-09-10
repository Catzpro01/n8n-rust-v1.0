//! n8n-nodes: node bawaan v1 (subset kecil yang tumbuh bertahap).
//!
//! v0.2.0: `manualTrigger`, `set` (+ekspresi), `noOp`, `filter`, `sort`,
//! `limit`. Nilai string di parameter `set`/`filter`/`sort` dirender sebagai
//! template `={{ }}` (lihat `n8n_core::expr`).

use n8n_core::expr::{render, ExprContext};
use n8n_core::WorkflowNode;
use n8n_engine::{EngineResult, ExecContext, Node, Registry};
use serde_json::{Map, Value};
use std::sync::Arc;

pub struct ManualTrigger;
pub struct SetNode;
pub struct NoOp;
pub struct FilterNode;
pub struct SortNode;
pub struct LimitNode;

impl Node for ManualTrigger {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.manualTrigger"
    }

    fn execute(
        &self,
        _node: &WorkflowNode,
        _items: Vec<Value>,
        _ctx: &ExecContext,
    ) -> EngineResult<Vec<Value>> {
        Ok(vec![Value::Object(Map::new())])
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
    ) -> EngineResult<Vec<Value>> {
        Ok(items)
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
    ) -> EngineResult<Vec<Value>> {
        if let Some(Value::Object(values)) = node.parameters.get("values") {
            return Ok(items
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
                .collect());
        }
        if let Some(list) = node
            .parameters
            .get("assignments")
            .and_then(|v| v.get("assignments"))
            .and_then(Value::as_array)
        {
            return Ok(items
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
                .collect());
        }
        Ok(items)
    }
}

impl Node for FilterNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.filter"
    }

    /// Hanya item yang `condition`-nya truthy yang diteruskan.
    /// `condition` = template string (`={{ $json.flag }}`).
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<Vec<Value>> {
        let cond = node
            .parameters
            .get("condition")
            .and_then(Value::as_str)
            .unwrap_or("");
        Ok(items
            .into_iter()
            .filter(|it| {
                let ectx = ExprContext {
                    item: it,
                    outputs: ctx.outputs,
                };
                truthy(&render(cond, &ectx))
            })
            .collect())
    }
}

impl Node for SortNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.sort"
    }

    /// Urutkan item menurut field (`field`, mis. `"age"`),
    /// `order`: `"asc"` (default) | `"desc"`.
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<Vec<Value>> {
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
        Ok(out)
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
    ) -> EngineResult<Vec<Value>> {
        let count = node
            .parameters
            .get("count")
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        Ok(items.into_iter().take(count).collect())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use n8n_core::Workflow;
    use n8n_engine::Engine;
    use std::collections::HashMap;

    fn empty_ctx(outputs: &HashMap<String, Vec<Value>>) -> ExecContext<'_> {
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
            report.outputs["NoOp"],
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
        assert_eq!(out, vec![serde_json::json!({ "a": 1 })]);
    }

    #[test]
    fn set_renders_expressions_per_item() {
        let node = set_node_with_values(
            serde_json::json!({"who": "={{ $json.name }}", "n": 5}),
        );
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
            vec![serde_json::json!({"name": "udi", "who": "udi", "n": 5})]
        );
    }

    #[test]
    fn set_reads_other_node_output() {
        let node =
            set_node_with_values(serde_json::json!({"x": "={{ $node[\"Up\"].json.x }}"}));
        let outputs = HashMap::from([(
            "Up".to_string(),
            vec![serde_json::json!({"x": 7})],
        )]);
        let out = SetNode
            .execute(&node, vec![serde_json::json!({})], &empty_ctx(&outputs))
            .expect("exec");
        assert_eq!(out, vec![serde_json::json!({"x": 7})]);
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
            vec![
                serde_json::json!({"keep": true}),
                serde_json::json!({"keep": "x"})
            ]
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
            vec![
                serde_json::json!({"age": 1}),
                serde_json::json!({"age": 2}),
                serde_json::json!({"age": 3})
            ]
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
        assert_eq!(out, vec![serde_json::json!(1), serde_json::json!(2)]);
    }
}
