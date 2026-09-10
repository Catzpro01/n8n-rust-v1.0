//! n8n-nodes: node bawaan v1 (subset kecil yang tumbuh bertahap).
//!
//! Cakupan v1: `manualTrigger`, `set` (subset), `noOp`.
//! Semantik `set` mendukung dua bentuk parameter:
//! - sederhana: `{"values": {"k": v}}` → gabung ke tiap item;
//! - gaya n8n: `{"assignments": {"assignments": [{"name","value",...}]}}` →
//!   pasangan nama=nilai statis (ekspresi n8n BELUM didukung — tiket lanjutan).

use n8n_core::WorkflowNode;
use n8n_engine::{EngineResult, Node, Registry};
use serde_json::{Map, Value};
use std::sync::Arc;

pub struct ManualTrigger;
pub struct SetNode;
pub struct NoOp;

impl Node for ManualTrigger {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.manualTrigger"
    }

    fn execute(&self, _node: &WorkflowNode, _items: Vec<Value>) -> EngineResult<Vec<Value>> {
        Ok(vec![Value::Object(Map::new())])
    }
}

impl Node for NoOp {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.noOp"
    }

    fn execute(&self, _node: &WorkflowNode, items: Vec<Value>) -> EngineResult<Vec<Value>> {
        Ok(items)
    }
}

impl Node for SetNode {
    fn node_type(&self) -> &'static str {
        "n8n-nodes-base.set"
    }

    fn execute(&self, node: &WorkflowNode, items: Vec<Value>) -> EngineResult<Vec<Value>> {
        if let Some(Value::Object(values)) = node.parameters.get("values") {
            return Ok(items.into_iter().map(|it| merge_value(it, values)).collect());
        }
        if let Some(list) = node
            .parameters
            .get("assignments")
            .and_then(|v| v.get("assignments"))
            .and_then(Value::as_array)
        {
            let mut values = Map::new();
            for a in list.iter().filter_map(Value::as_object) {
                if let (Some(Value::String(name)), Some(value)) = (a.get("name"), a.get("value")) {
                    values.insert(name.clone(), value.clone());
                }
            }
            return Ok(items.into_iter().map(|it| merge_value(it, &values)).collect());
        }
        Ok(items)
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

pub fn register_all(registry: &mut Registry) {
    registry.register(Arc::new(ManualTrigger));
    registry.register(Arc::new(SetNode));
    registry.register(Arc::new(NoOp));
}

#[cfg(test)]
mod tests {
    use super::*;
    use n8n_core::Workflow;
    use n8n_engine::Engine;

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
    }

    #[test]
    fn set_supports_n8n_assignments_shape() {
        let node = WorkflowNode {
            id: "s".to_string(),
            name: "Set".to_string(),
            node_type: "n8n-nodes-base.set".to_string(),
            type_version: 3.4,
            position: [0.0, 0.0],
            parameters: std::collections::HashMap::from([(
                "assignments".to_string(),
                serde_json::json!({ "assignments": [{ "name": "a", "value": 1 }] }),
            )]),
            disabled: false,
            extra: std::collections::HashMap::new(),
        };
        let out = SetNode
            .execute(&node, vec![serde_json::json!({})])
            .expect("exec");
        assert_eq!(out, vec![serde_json::json!({ "a": 1 })]);
    }
}
