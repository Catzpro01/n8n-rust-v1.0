// executor: TB-01 minimal engine (tracer-owned vertical slice).
// Pipeline: parse minimal workflow JSON -> Manual Trigger emits [{}] (mirrors
// upstream returnJsonArray([{}])) -> walk connections -> run each node through
// the kernel Node trait -> return final items.
// Services the tracer does not need (expressions, http, credentials, spill,
// blobs) are fail-closed refusals, never silent behavior. Scheduler: manual
// invocation only; no daemon in TB-01.
use async_trait::async_trait;
use kernel::context::{
    BlobStore, CancellationToken, CredentialProvider, CredentialValue, ExecutionMeta,
    ExecutionMode, ExecutionOrder, ExpressionEngine, ExpressionScope, HttpClient, HttpRequest,
    HttpResponse, LogFields, LogLevel, Logger, NodeContext, NodeMeta, PriorNodeRef, PriorOutputs,
    StaticData,
};
use kernel::error::{KernelError, NodeError};
use kernel::id::{ContentId, ExecutionId, NodeId, WorkflowId};
use kernel::item::{Item, ItemList, SpillStore, SpilledList, SpillWriter};
use kernel::node::Node;
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("parse workflow: {0}")]
    Parse(String),
    #[error("workflow shape: {0}")]
    Shape(String),
    #[error("no manualTrigger node found")]
    NoTrigger,
    #[error("unknown node type {0}")]
    UnknownNode(String),
    #[error("node {0} failed: {1}")]
    Node(String, NodeError),
    #[error("connection cycle or walk overrun")]
    Cycle,
    #[error("final output is spilled; TB-01 never spills")]
    Spilled,
}

/// Run a minimal workflow JSON document, return the final items.
pub async fn run_workflow_json(text: &str) -> Result<Vec<Item>, EngineError> {
    let wf: Value =
        serde_json::from_str(text).map_err(|e| EngineError::Parse(e.to_string()))?;
    let nodes = wf
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| EngineError::Shape("missing nodes array".to_string()))?;
    let start = nodes
        .iter()
        .find(|n| n.get("type").and_then(Value::as_str) == Some("n8n-nodes-base.manualTrigger"))
        .ok_or(EngineError::NoTrigger)?;
    let start_name = start
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| EngineError::Shape("trigger without name".to_string()))?;

    let exec_meta = ExecutionMeta {
        id: ExecutionId(1),
        workflow_id: WorkflowId(1),
        workflow_version: 1,
        mode: ExecutionMode::Manual,
        timezone: "UTC".to_string(),
        execution_order: ExecutionOrder::V1,
        started_at: 0,
    };
    let prior = TracerPrior;
    let creds = TracerCreds;
    let expr = TracerExpr;
    let http = TracerHttp;
    let blobs = TracerBlobs;
    let spill = TracerSpill;
    let log = TracerLog;
    let cancel = TracerCancel;
    let mut static_data = StaticData::new();

    // Manual Trigger semantics (upstream ManualTrigger.node.ts: trigger emits one [{}]).
    let mut current = ItemList::Inline(vec![Item::new(Value::Object(Default::default()))]);
    let mut node_name = start_name.to_string();
    let mut steps = 0usize;
    loop {
        steps += 1;
        if steps > nodes.len() + 1 {
            return Err(EngineError::Cycle);
        }
        let next_names = outgoing(&wf, &node_name);
        if next_names.is_empty() {
            break;
        }
        // TB-01: linear walk, first edge only; branches are a later ticket.
        let next = &next_names[0];
        let def = nodes
            .iter()
            .find(|n| n.get("name").and_then(Value::as_str) == Some(next.as_str()))
            .ok_or_else(|| EngineError::Shape(format!("edge to unknown node {next:?}")))?;
        let node_type = def.get("type").and_then(Value::as_str).unwrap_or("");
        let params = def.get("parameters").cloned().unwrap_or(Value::Null);
        let node: Box<dyn Node> = match node_type {
            "n8n-nodes-base.set" => Box::new(nodes_core::SetNode::with_params(params)),
            other => return Err(EngineError::UnknownNode(other.to_string())),
        };
        let node_meta = NodeMeta {
            id: NodeId::new(next.clone()),
            position: (0, 0),
            input_index: 0,
        };
        let inputs = [current];
        let mut ctx = NodeContext {
            execution: &exec_meta,
            node: &node_meta,
            input: &inputs,
            prior: &prior,
            credentials: &creds,
            static_data: &mut static_data,
            expr: &expr,
            http: &http,
            blobs: &blobs,
            spill: &spill,
            log: &log,
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(30),
            attempt: 1,
        };
        let output = node
            .execute(&mut ctx)
            .await
            .map_err(|e| EngineError::Node(next.clone(), e))?;
        current = output.branches.into_iter().next().unwrap_or(ItemList::empty());
        node_name = next.clone();
    }
    match current {
        ItemList::Inline(items) => Ok(items),
        ItemList::Spilled(_) => Err(EngineError::Spilled),
    }
}

/// Names connected to node (connections[name].main[0][] -> node).
fn outgoing(wf: &Value, name: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(main) = wf
        .get("connections")
        .and_then(|c| c.get(name))
        .and_then(|n| n.get("main"))
        .and_then(Value::as_array)
    {
        if let Some(first) = main.first().and_then(Value::as_array) {
            for edge in first {
                if let Some(n) = edge.get("node").and_then(Value::as_str) {
                    names.push(n.to_string());
                }
            }
        }
    }
    names
}

// -- Fail-closed tracer services (TB-01 boundary; each refusal names its ticket). --

struct TracerPrior;

#[async_trait]
impl PriorOutputs for TracerPrior {
    fn by_name(&self, _name: &str) -> Option<PriorNodeRef> {
        None
    }
    fn by_id(&self, _id: &NodeId) -> Option<PriorNodeRef> {
        None
    }
    async fn item(&self, node: &str, _ii: u8, _i: usize) -> Result<Item, KernelError> {
        Err(KernelError::NodeNotFound {
            node: format!("TB-01 keeps no prior table: {node}"),
        })
    }
    fn items(&self, node: &str, _ii: u8) -> Result<ItemList, KernelError> {
        Err(KernelError::NodeNotFound {
            node: format!("TB-01 keeps no prior table: {node}"),
        })
    }
    fn available_nodes(&self) -> Vec<String> {
        Vec::new()
    }
}

struct TracerExpr;

#[async_trait]
impl ExpressionEngine for TracerExpr {
    async fn eval(&self, expr: &str, _s: &ExpressionScope<'_>) -> Result<Value, KernelError> {
        Err(KernelError::Expression {
            expr: expr.to_string(),
            reason: "TB-01 literals only; expressions need C-5".to_string(),
        })
    }
    async fn eval_batch(
        &self,
        _e: &[&str],
        _s: &ExpressionScope<'_>,
    ) -> Result<Vec<Value>, KernelError> {
        Err(KernelError::Expression {
            expr: String::new(),
            reason: "TB-01 literals only; expressions need C-5".to_string(),
        })
    }
    async fn validate(&self, expr: &str) -> Result<(), KernelError> {
        Err(KernelError::Expression {
            expr: expr.to_string(),
            reason: "TB-01 literals only; expressions need C-5".to_string(),
        })
    }
}

struct TracerCreds;

#[async_trait]
impl CredentialProvider for TracerCreds {
    async fn get(&self, kind: &str) -> Result<CredentialValue, KernelError> {
        Err(KernelError::Invalid {
            message: format!("TB-01 has no credentials: {kind}"),
        })
    }
}

struct TracerHttp;

#[async_trait]
impl HttpClient for TracerHttp {
    async fn send(&self, _req: HttpRequest) -> Result<HttpResponse, KernelError> {
        Err(KernelError::Invalid {
            message: "TB-01 performs no http".to_string(),
        })
    }
}

struct TracerBlobs;

#[async_trait]
impl BlobStore for TracerBlobs {
    async fn put(&self, _b: &[u8], _m: &str) -> Result<(ContentId, u64), KernelError> {
        Err(KernelError::Invalid {
            message: "TB-01 stores no blobs".to_string(),
        })
    }
    async fn get(&self, _id: ContentId) -> Result<Vec<u8>, KernelError> {
        Err(KernelError::Invalid {
            message: "TB-01 stores no blobs".to_string(),
        })
    }
    async fn delete(&self, _id: ContentId) -> Result<(), KernelError> {
        Err(KernelError::Invalid {
            message: "TB-01 stores no blobs".to_string(),
        })
    }
    fn size(&self, _id: ContentId) -> Option<u64> {
        None
    }
}

struct TracerSpill;

#[async_trait]
impl SpillStore for TracerSpill {
    async fn write(&self, _items: &[Item]) -> Result<SpilledList, KernelError> {
        Err(refuse_spill())
    }
    async fn read_at(&self, _h: &SpilledList, _i: usize) -> Result<Item, KernelError> {
        Err(refuse_spill())
    }
    async fn read_range(
        &self,
        _h: &SpilledList,
        _s: usize,
        _l: usize,
    ) -> Result<Vec<Item>, KernelError> {
        Err(refuse_spill())
    }
    async fn read_all(&self, _h: &SpilledList) -> Result<Vec<Item>, KernelError> {
        Err(refuse_spill())
    }
    async fn delete(&self, _h: &SpilledList) -> Result<(), KernelError> {
        Err(refuse_spill())
    }
    fn disk_footprint(&self, _h: &SpilledList) -> u64 {
        0
    }
    async fn writer(&self) -> Result<Box<dyn SpillWriter>, KernelError> {
        Err(refuse_spill())
    }
}

fn refuse_spill() -> KernelError {
    KernelError::SpillIo {
        message: "TB-01 never spills (Inline only)".to_string(),
        kind: std::io::ErrorKind::Other,
    }
}

struct TracerLog;

impl Logger for TracerLog {
    fn log(&self, _level: LogLevel, message: &str, _fields: &LogFields) {
        eprintln!("tb01-log: {message}");
    }
}

struct TracerCancel;

impl CancellationToken for TracerCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
}
