//! n8n-engine: executor workflow sinkron (tanpa runtime async — ringan).
//!
//! v0.6.0: trait `Node::execute_multi` (default: gabung input lalu
//! `execute`) — node Merge melihat tiap input terpisah; `run_with`
//! membawa payload webhook; `run` = `run_with(None)`.

use n8n_core::{Workflow, WorkflowNode};
use serde::{Deserialize, Serialize};
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

/// Output node per cabang: indeks = cabang `main[i]` di connections.
/// Node biasa mengembalikan persis 1 cabang.
pub type BranchOutputs = Vec<Vec<Value>>;

/// Konteks eksekusi: output node lain + payload webhook (bila ada).
pub struct ExecContext<'a> {
    pub outputs: &'a HashMap<String, BranchOutputs>,
    /// Payload request webhook (server `/hook`) — None saat run biasa.
    pub webhook: Option<&'a Value>,
}

impl ExecContext<'_> {
    pub fn output_of(&self, name: &str) -> Option<&BranchOutputs> {
        self.outputs.get(name)
    }
}

/// Satu edge masuk: dari node + cabang mana items ini datang.
/// Urutan = urutan node pendahulu di file (deterministik) — Merge
/// memakai ini sebagai input1, input2, ...
#[derive(Debug, Clone)]
pub struct MultiInput {
    pub from: String,
    pub branch: usize,
    pub items: Vec<Value>,
}

/// Kontrak satu tipe node. `Send + Sync` supaya kelak bisa paralel.
pub trait Node: Send + Sync {
    /// Nama tipe persis n8n, mis. `"n8n-nodes-base.set"`.
    fn node_type(&self) -> &'static str;
    /// `items` = gabungan output para (pendahulu, cabang) — urutan edge.
    fn execute(
        &self,
        node: &WorkflowNode,
        items: Vec<Value>,
        ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs>;
    /// Varian multi-input (dipakai engine). Default: gabungkan semua
    /// input lalu panggil `execute` — node biasa tak perlu override.
    fn execute_multi(
        &self,
        node: &WorkflowNode,
        inputs: Vec<MultiInput>,
        ctx: &ExecContext,
    ) -> EngineResult<BranchOutputs> {
        let items = inputs.into_iter().flat_map(|i| i.items).collect();
        self.execute(node, items, ctx)
    }
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

    /// Daftar tipe terdaftar, terurut — untuk CLI `nodes` dan API.
    pub fn types(&self) -> Vec<String> {
        let mut v: Vec<String> = self.nodes.keys().cloned().collect();
        v.sort();
        v
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunReport {
    /// Output per nama node, per cabang.
    pub outputs: HashMap<String, BranchOutputs>,
    /// Urutan eksekusi aktual.
    pub order: Vec<String>,
    /// Durasi per node (milidetik) — observabilitas bawaan tiap run.
    pub durations_ms: HashMap<String, u128>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
}

struct Plan {
    order: Vec<String>,
    /// Nama node → daftar (nama pendahulu, indeks cabang).
    incoming: HashMap<String, Vec<(String, usize)>>,
}

/// Urutan topo (Kahn) deterministik sesuai urutan node di file.
fn plan(workflow: &Workflow) -> EngineResult<Plan> {
    let enabled: Vec<&WorkflowNode> = workflow.nodes.iter().filter(|n| !n.disabled).collect();
    let mut incoming: HashMap<String, Vec<(String, usize)>> = HashMap::new();
    for n in &enabled {
        incoming.entry(n.name.clone()).or_default();
    }
    for n in &enabled {
        for (bi, branch) in workflow.branches(&n.name).iter().enumerate() {
            for s in branch {
                if let Some(list) = incoming.get_mut(s) {
                    list.push((n.name.clone(), bi));
                }
            }
        }
    }
    let mut done: Vec<String> = Vec::new();
    let mut done_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    loop {
        let mut progressed = false;
        for n in &enabled {
            if done_set.contains(&n.name) {
                continue;
            }
            let ready = incoming
                .get(&n.name)
                .map(|ps| ps.iter().all(|(p, _)| done_set.contains(p)))
                .unwrap_or(true);
            if !ready {
                continue;
            }
            done_set.insert(n.name.clone());
            done.push(n.name.clone());
            progressed = true;
        }
        if !progressed {
            break;
        }
    }
    if done.len() != enabled.len() {
        let stuck: Vec<&str> = enabled
            .iter()
            .map(|n| n.name.as_str())
            .filter(|k| !done_set.contains(*k))
            .collect();
        return Err(EngineError::new(format!(
            "cycle detected, stuck at: {}",
            stuck.join(", ")
        )));
    }
    Ok(Plan {
        order: done,
        incoming,
    })
}

pub struct Engine;

impl Engine {
    /// Rencana eksekusi TANPA menjalankan (dry-run; tak butuh registry).
    pub fn explain(workflow: &Workflow) -> EngineResult<Vec<String>> {
        Ok(plan(workflow)?.order)
    }

    /// Run biasa (tanpa payload webhook).
    pub fn run(workflow: &Workflow, registry: &Registry) -> EngineResult<RunReport> {
        Self::run_with(workflow, registry, None)
    }

    /// Run dengan payload webhook opsional (dibaca node webhook).
    pub fn run_with(
        workflow: &Workflow,
        registry: &Registry,
        webhook: Option<Value>,
    ) -> EngineResult<RunReport> {
        let p = plan(workflow)?;
        let by_name: HashMap<&str, &WorkflowNode> =
            workflow.nodes.iter().map(|n| (n.name.as_str(), n)).collect();
        let mut done: HashMap<String, BranchOutputs> = HashMap::new();
        let mut durations_ms: HashMap<String, u128> = HashMap::new();
        for name in &p.order {
            let node = by_name.get(name.as_str()).copied().ok_or_else(|| {
                EngineError::new(format!("plan menyebut node hilang '{name}'"))
            })?;
            let mut inputs = Vec::new();
            if let Some(ps) = p.incoming.get(name) {
                for (pr, bi) in ps {
                    let branch_items = done
                        .get(pr)
                        .and_then(|b| b.get(*bi))
                        .cloned()
                        .unwrap_or_default();
                    inputs.push(MultiInput {
                        from: pr.clone(),
                        branch: *bi,
                        items: branch_items,
                    });
                }
            }
            let node_impl = registry.get(&node.node_type).ok_or_else(|| {
                EngineError::new(format!(
                    "unknown node type '{}' (node '{name}')",
                    node.node_type
                ))
            })?;
            let ctx = ExecContext {
                outputs: &done,
                webhook: webhook.as_ref(),
            };
            let t0 = std::time::Instant::now();
            let out = node_impl
                .execute_multi(node, inputs, &ctx)
                .map_err(|e| EngineError::new(format!("node '{name}' failed: {e}")))?;
            durations_ms.insert(name.clone(), t0.elapsed().as_millis());
            done.insert(name.clone(), out);
        }
        Ok(RunReport {
            outputs: done,
            order: p.order,
            durations_ms,
        })
    }

    /// Linter: error + warning deterministik (urutan file).
    pub fn lint(workflow: &Workflow, registry: &Registry) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        let err = |message: String| Diagnostic {
            level: Level::Error,
            message,
        };
        let warn = |message: String| Diagnostic {
            level: Level::Warning,
            message,
        };
        for n in &workflow.nodes {
            if registry.get(&n.node_type).is_none() {
                out.push(err(format!(
                    "node '{}': tipe tak dikenal '{}'",
                    n.name, n.node_type
                )));
            }
        }
        for (from, to) in workflow.dangling_edges() {
            out.push(err(format!(
                "edge gantung: '{from}' -> '{to}' (target tak ada)"
            )));
        }
        {
            let names: std::collections::HashSet<&str> =
                workflow.nodes.iter().map(|n| n.name.as_str()).collect();
            let mut ghosts: Vec<&str> = workflow
                .connections
                .keys()
                .map(String::as_str)
                .filter(|k| !names.contains(k))
                .collect();
            ghosts.sort_unstable();
            for g in ghosts {
                out.push(warn(format!(
                    "connections dari '{g}' yang bukan node — diabaikan"
                )));
            }
        }
        if let Err(e) = plan(workflow) {
            out.push(err(e.to_string()));
        }
        let mut targets: std::collections::HashSet<String> = std::collections::HashSet::new();
        for n in &workflow.nodes {
            for s in workflow.successors(&n.name) {
                targets.insert(s);
            }
        }
        if workflow.nodes.len() > 1 {
            for n in &workflow.nodes {
                let has_out = !workflow.successors(&n.name).is_empty();
                if !has_out && !targets.contains(n.name.as_str()) {
                    out.push(warn(format!("node '{}' terisolasi (tanpa edge)", n.name)));
                }
            }
        }
        for n in &workflow.nodes {
            if n.disabled {
                continue;
            }
            if !targets.contains(n.name.as_str())
                && !n.node_type.to_lowercase().contains("trigger")
            {
                out.push(warn(format!(
                    "node '{}' tak punya pendahulu dan bukan trigger — berjalan dengan 0 item",
                    n.name
                )));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Emit;
    struct Pass;
    struct EchoHook;

    impl Node for Emit {
        fn node_type(&self) -> &'static str {
            "test.emit"
        }
        fn execute(
            &self,
            _node: &WorkflowNode,
            _items: Vec<Value>,
            _ctx: &ExecContext,
        ) -> EngineResult<BranchOutputs> {
            Ok(vec![vec![Value::String("e".to_string())]])
        }
    }

    impl Node for Pass {
        fn node_type(&self) -> &'static str {
            "test.pass"
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

    impl Node for EchoHook {
        fn node_type(&self) -> &'static str {
            "test.echohook"
        }
        fn execute(
            &self,
            _node: &WorkflowNode,
            _items: Vec<Value>,
            ctx: &ExecContext,
        ) -> EngineResult<BranchOutputs> {
            Ok(vec![vec![ctx.webhook.cloned().unwrap_or(Value::Null)]])
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
        r.register(Arc::new(EchoHook));
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
            report.outputs["B"][0],
            vec![Value::String("e".to_string())]
        );
        assert_eq!(report.durations_ms.len(), 2);
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

    #[test]
    fn explain_plans_without_executing() {
        let wf = workflow(
            vec![node("A", "test.missing"), node("B", "test.missing")],
            HashMap::from([(
                "A".to_string(),
                serde_json::json!({ "main": [[{ "node": "B" }]] }),
            )]),
        );
        assert_eq!(
            Engine::explain(&wf).expect("plan"),
            vec!["A".to_string(), "B".to_string()]
        );
    }

    #[test]
    fn lint_reports_unknown_type_and_dangling_edge() {
        let wf = workflow(
            vec![node("A", "test.missing")],
            HashMap::from([(
                "A".to_string(),
                serde_json::json!({ "main": [[{ "node": "Ghost" }]] }),
            )]),
        );
        let diags = Engine::lint(&wf, &registry());
        assert!(
            diags
                .iter()
                .any(|d| d.level == Level::Error && d.message.contains("tak dikenal")),
            "{diags:?}"
        );
        assert!(
            diags
                .iter()
                .any(|d| d.level == Level::Error && d.message.contains("gantung")),
            "{diags:?}"
        );
    }

    #[test]
    fn execute_multi_sees_separate_inputs() {
        struct Probe;
        impl Node for Probe {
            fn node_type(&self) -> &'static str {
                "test.probe"
            }
            fn execute(
                &self,
                _node: &WorkflowNode,
                _items: Vec<Value>,
                _ctx: &ExecContext,
            ) -> EngineResult<BranchOutputs> {
                unreachable!("probe hanya via execute_multi")
            }
            fn execute_multi(
                &self,
                _node: &WorkflowNode,
                inputs: Vec<MultiInput>,
                _ctx: &ExecContext,
            ) -> EngineResult<BranchOutputs> {
                let summary: Vec<Value> = inputs
                    .iter()
                    .map(|i| {
                        serde_json::json!({"from": i.from, "branch": i.branch, "n": i.items.len()})
                    })
                    .collect();
                Ok(vec![vec![Value::Array(summary)]])
            }
        }
        let mut reg = registry();
        reg.register(Arc::new(Probe));
        let wf = workflow(
            vec![
                node("A", "test.emit"),
                node("B", "test.emit"),
                node("P", "test.probe"),
            ],
            HashMap::from([
                (
                    "A".to_string(),
                    serde_json::json!({ "main": [[{ "node": "P" }]] }),
                ),
                (
                    "B".to_string(),
                    serde_json::json!({ "main": [[{ "node": "P" }]] }),
                ),
            ]),
        );
        let report = Engine::run(&wf, &reg).expect("run");
        assert_eq!(
            report.outputs["P"][0],
            vec![serde_json::json!([
                {"from": "A", "branch": 0, "n": 1},
                {"from": "B", "branch": 0, "n": 1}
            ])]
        );
    }

    #[test]
    fn run_with_threads_webhook_payload() {
        let wf = workflow(vec![node("A", "test.echohook")], HashMap::new());
        let report = Engine::run_with(&wf, &registry(), Some(serde_json::json!({"h": 1})))
            .expect("run");
        assert_eq!(
            report.outputs["A"][0],
            vec![serde_json::json!({"h": 1})]
        );
    }
}
