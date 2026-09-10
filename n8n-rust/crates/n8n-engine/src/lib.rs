//! n8n-engine: executor workflow sinkron (tanpa runtime async — ringan).
//!
//! v1: topological order (Kahn) deterministik sesuai urutan node di file,
//! node `disabled` dikeluarkan dari graf, tipe tak dikenal = error eksplisit
//! (gagal cepat lebih jujur daripada diam-diam melewatkan node).

use n8n_core::{Workflow, WorkflowNode};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError {
    message: String,
}

impl EngineError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;

/// Kontrak satu tipe node. `Send + Sync` supaya kelak bisa paralel.
pub trait Node: Send + Sync {
    /// Nama tipe persis n8n, mis. `"n8n-nodes-base.set"`.
    fn node_type(&self) -> &'static str;
    /// `items` = gabungan output para pendahulu (urutan deterministik).
    fn execute(&self, node: &WorkflowNode, items: Vec<Value>) -> EngineResult<Vec<Value>>;
}

#[derive(Default)]
pub struct Registry {
    nodes: HashMap<String, Arc<dyn Node>>,
}

impl Registry {
    pub fn register(&mut self, node: Arc<dyn Node>) {
        self.nodes.insert(node.node_type().to_string(), node);
    }

    pub fn get(&self, node_type: &str) -> Option<&Arc<dyn Node>> {
        self.nodes.get(node_type)
    }

    /// Daftar tipe terdaftar, terurut — untuk CLI `nodes`.
    pub fn types(&self) -> Vec<String> {
        let mut v: Vec<String> = self.nodes.keys().cloned().collect();
        v.sort();
        v
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunReport {
    /// Output per nama node.
    pub outputs: HashMap<String, Vec<Value>>,
    /// Urutan eksekusi aktual.
    pub order: Vec<String>,
}

pub struct Engine;

impl Engine {
    pub fn run(workflow: &Workflow, registry: &Registry) -> EngineResult<RunReport> {
        let enabled: Vec<&WorkflowNode> =
            workflow.nodes.iter().filter(|n| !n.disabled).collect();

        // Peta nama -> daftar nama pendahulu (hanya antar node aktif).
        let mut preds: HashMap<&str, Vec<&str>> = HashMap::new();
        for n in &enabled {
            preds.entry(n.name.as_str()).or_default();
        }
        for n in &enabled {
            for s in workflow.successors(&n.name) {
                if let Some(list) = preds.get_mut(s.as_str()) {
                    list.push(n.name.as_str());
                }
            }
        }

        // Kahn: berulang kali jalankan node yang semua pendahulunya sudah
        // selesai, sesuai urutan file — deterministik penuh.
        let mut done: HashMap<&str, Vec<Value>> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        loop {
            let mut progressed = false;
            for n in &enabled {
                let key = n.name.as_str();
                if done.contains_key(key) {
                    continue;
                }
                let ready = preds
                    .get(key)
                    .map(|ps| ps.iter().all(|p| done.contains_key(*p)))
                    .unwrap_or(true);
                if !ready {
                    continue;
                }
                let mut items = Vec::new();
                if let Some(ps) = preds.get(key) {
                    for p in ps {
                        items.extend(done.get(*p).cloned().unwrap_or_default());
                    }
                }
                let node_impl = registry.get(&n.node_type).ok_or_else(|| {
                    EngineError::new(format!(
                        "unknown node type '{}' (node '{}')",
                        n.node_type, n.name
                    ))
                })?;
                let out = node_impl.execute(n, items).map_err(|e| {
                    EngineError::new(format!("node '{}' failed: {}", n.name, e))
                })?;
                done.insert(key, out);
                order.push(n.name.clone());
                progressed = true;
            }
            if !progressed {
                break;
            }
        }

        if order.len() != enabled.len() {
            let stuck: Vec<&str> = enabled
                .iter()
                .map(|n| n.name.as_str())
                .filter(|k| !done.contains_key(*k))
                .collect();
            return Err(EngineError::new(format!(
                "cycle detected, stuck at: {}",
                stuck.join(", ")
            )));
        }

        let outputs: HashMap<String, Vec<Value>> = done
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        Ok(RunReport { outputs, order })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Emit;
    struct Pass;

    impl Node for Emit {
        fn node_type(&self) -> &'static str {
            "test.emit"
        }
        fn execute(&self, _node: &WorkflowNode, _items: Vec<Value>) -> EngineResult<Vec<Value>> {
            Ok(vec![Value::String("e".to_string())])
        }
    }

    impl Node for Pass {
        fn node_type(&self) -> &'static str {
            "test.pass"
        }
        fn execute(&self, _node: &WorkflowNode, items: Vec<Value>) -> EngineResult<Vec<Value>> {
            Ok(items)
        }
    }

    fn node(name: &str, t: &str) -> WorkflowNode {
        WorkflowNode {
            id: format!("id-{name}"),
            name: name.to_string(),
            node_type: t.to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: HashMap::new(),
            disabled: false,
            extra: HashMap::new(),
        }
    }

    fn registry() -> Registry {
        let mut r = Registry::default();
        r.register(Arc::new(Emit));
        r.register(Arc::new(Pass));
        r
    }

    fn workflow(nodes: Vec<WorkflowNode>, connections: HashMap<String, Value>) -> Workflow {
        Workflow {
            name: "t".to_string(),
            nodes,
            connections,
            active: false,
            settings: HashMap::new(),
            extra: HashMap::new(),
        }
    }

    #[test]
    fn linear_chain_passes_items_through() {
        let wf = workflow(
            vec![node("A", "test.emit"), node("B", "test.pass")],
            HashMap::from([(
                "A".to_string(),
                serde_json::json!({ "main": [[{ "node": "B", "type": "main", "index": 0 }]] }),
            )]),
        );
        let report = Engine::run(&wf, &registry()).expect("run");
        assert_eq!(report.order, vec!["A".to_string(), "B".to_string()]);
        assert_eq!(
            report.outputs["B"],
            vec![Value::String("e".to_string())]
        );
    }

    #[test]
    fn unknown_type_is_an_explicit_error() {
        let wf = workflow(vec![node("A", "test.missing")], HashMap::new());
        let err = Engine::run(&wf, &registry()).expect_err("must fail");
        assert!(err.to_string().contains("unknown node type"), "{err}");
    }

    #[test]
    fn cycle_is_detected() {
        let wf = workflow(
            vec![node("A", "test.pass"), node("B", "test.pass")],
            HashMap::from([
                (
                    "A".to_string(),
                    serde_json::json!({ "main": [[{ "node": "B" }]] }),
                ),
                (
                    "B".to_string(),
                    serde_json::json!({ "main": [[{ "node": "A" }]] }),
                ),
            ]),
        );
        let err = Engine::run(&wf, &registry()).expect_err("must fail");
        assert!(err.to_string().contains("cycle"), "{err}");
    }
}
