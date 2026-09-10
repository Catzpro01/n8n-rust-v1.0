//! testkit — In-memory implementations of kernel & data-plane traits for testing.
//!
//! Provides:
//! - `InMemorySpillStore`: spill store backed by `HashMap` (no disk I/O)
//! - `InMemoryBlobStore`: blob store backed by `HashMap` (no disk I/O)
//! - `MockNode`: configurable mock implementing `kernel::Node`
//! - `MockNodeContext`: builder for constructing test execution contexts
//! - `TestFixture`: convenience builder for common test scenarios
//!
//! # Example
//! ```rust,no_run
//! use testkit::*;
//!
//! #[tokio::test]
//! async fn test_mock_node() {
//!     let node = MockNode::new("test.Set")
//!         .with_output(vec![serde_json::json!({"key": "value"})]);
//!     let ctx = MockNodeContext::new()
//!         .with_input(vec![serde_json::json!({"x": 1})]);
//!     let result = node.execute(&mut ctx).await;
//!     assert!(result.is_ok());
//! }
//! ```
//!
//! Author: agent3
//! Date: 2026-09-09

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;

use kernel::{
    Node, NodeContext, NodeDescriptor, NodeError, NodeOutput, ResourceHint, SideEffect,
};

// ============================================================================
// InMemorySpillStore — drop-in replacement for data_plane::SpillStore
// ============================================================================

/// In-memory spill store for testing. Uses `HashMap` instead of disk.
/// Thread-safe via `Arc<Mutex<...>>`.
#[derive(Clone)]
pub struct InMemorySpillStore {
    /// execution_id -> { filename -> (data, checksum) }
    data: Arc<Mutex<HashMap<String, HashMap<String, (Vec<u8>, String)>>>>,
    write_count: Arc<Mutex<u64>>,
    read_count: Arc<Mutex<u64>>,
    gc_count: Arc<Mutex<u64>>,
}

impl InMemorySpillStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            write_count: Arc::new(Mutex::new(0)),
            read_count: Arc::new(Mutex::new(0)),
            gc_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Write spill data. Returns (virtual_path, checksum, bytes_written).
    pub async fn write(
        &self,
        execution_id: &str,
        data: &[u8],
    ) -> Result<(PathBuf, String, u64), kernel::KernelError> {
        let checksum = blake3_hash(data);
        let filename = format!("spill-{}.bin", *self.write_count.lock().unwrap());
        let virtual_path = PathBuf::from(format!("/in-memory/{execution_id}/{filename}"));

        let mut store = self.data.lock().unwrap();
        store
            .entry(execution_id.to_string())
            .or_default()
            .insert(filename, (data.to_vec(), checksum.clone()));

        *self.write_count.lock().unwrap() += 1;
        Ok((virtual_path, checksum, data.len() as u64))
    }

    /// Read spill data with checksum verification.
    pub async fn read(
        &self,
        path: &Path,
        expected_checksum: &str,
    ) -> Result<Vec<u8>, kernel::KernelError> {
        *self.read_count.lock().unwrap() += 1;

        // Parse execution_id and filename from virtual path
        let parts: Vec<&str> = path.to_str().unwrap_or("").split('/').collect();
        if parts.len() < 4 {
            return Err(kernel::KernelError::Storage(format!(
                "invalid spill path: {}",
                path.display()
            )));
        }
        let execution_id = parts[parts.len() - 2];
        let filename = parts[parts.len() - 1];

        let store = self.data.lock().unwrap();
        let exec_data = store.get(execution_id).ok_or_else(|| {
            kernel::KernelError::Storage(format!("execution not found: {execution_id}"))
        })?;
        let (data, actual_checksum) = exec_data.get(filename).ok_or_else(|| {
            kernel::KernelError::Storage(format!("spill not found: {}", path.display()))
        })?;

        if actual_checksum != expected_checksum {
            return Err(kernel::KernelError::Storage(format!(
                "checksum mismatch: expected {expected_checksum}, got {actual_checksum}"
            )));
        }
        Ok(data.clone())
    }

    /// GC all spill files for an execution. Returns number of files removed.
    pub async fn gc_execution(&self, execution_id: &str) -> Result<u64, kernel::KernelError> {
        let mut store = self.data.lock().unwrap();
        let count = store
            .remove(execution_id)
            .map(|m| m.len() as u64)
            .unwrap_or(0);
        *self.gc_count.lock().unwrap() += 1;
        Ok(count)
    }

    // -- Test introspection helpers --

    /// Total number of spill writes performed.
    pub fn total_writes(&self) -> u64 {
        *self.write_count.lock().unwrap()
    }

    /// Total number of spill reads performed.
    pub fn total_reads(&self) -> u64 {
        *self.read_count.lock().unwrap()
    }

    /// Total number of GC operations.
    pub fn total_gc(&self) -> u64 {
        *self.gc_count.lock().unwrap()
    }

    /// Number of execution IDs currently stored.
    pub fn active_executions(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    /// Total number of spill files across all executions.
    pub fn total_spill_files(&self) -> usize {
        self.data
            .lock()
            .unwrap()
            .values()
            .map(|m| m.len())
            .sum()
    }

    /// Assert no spill files remain (for leak detection in tests).
    pub fn assert_no_leaks(&self) {
        let total = self.total_spill_files();
        assert_eq!(
            total, 0,
            "Spill leak detected: {total} spill files still in memory"
        );
    }
}

impl Default for InMemorySpillStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// InMemoryBlobStore — drop-in replacement for data_plane::BlobStore
// ============================================================================

/// In-memory blob store for testing.
#[derive(Clone)]
pub struct InMemoryBlobStore {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemoryBlobStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn put(&self, key: &str, data: &[u8]) -> Result<PathBuf, kernel::KernelError> {
        self.data
            .lock()
            .unwrap()
            .insert(key.to_string(), data.to_vec());
        Ok(PathBuf::from(format!("/in-memory-blobs/{key}")))
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, kernel::KernelError> {
        self.data
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| kernel::KernelError::Storage(format!("blob not found: {key}")))
    }

    pub async fn delete(&self, key: &str) -> Result<(), kernel::KernelError> {
        self.data.lock().unwrap().remove(key);
        Ok(())
    }

    /// Check if a blob exists.
    pub fn exists(&self, key: &str) -> bool {
        self.data.lock().unwrap().contains_key(key)
    }

    /// Number of blobs stored.
    pub fn count(&self) -> usize {
        self.data.lock().unwrap().len()
    }
}

impl Default for InMemoryBlobStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MockNode — configurable mock implementing kernel::Node
// ============================================================================

/// Behavior mode for MockNode
#[derive(Clone)]
pub enum MockBehavior {
    /// Return the configured output items
    Success(Vec<Value>),
    /// Return an error
    Fail(String),
    /// Return output but mark as InDoubt (for non-idempotent side effects)
    InDoubt(Vec<Value>),
    /// Simulate timeout
    Timeout,
    /// Custom function (for advanced testing)
    Custom(Arc<dyn Fn(&[Value]) -> Result<Vec<Value>, NodeError> + Send + Sync>),
}

/// Mock node for testing. Implements `kernel::Node`.
pub struct MockNode {
    descriptor: NodeDescriptor,
    behavior: MockBehavior,
    call_count: Arc<Mutex<u64>>,
    last_input: Arc<Mutex<Option<Vec<Value>>>>,
}

impl MockNode {
    /// Create a new mock node with the given type kind.
    pub fn new(kind: &str) -> Self {
        Self {
            descriptor: NodeDescriptor {
                kind: kind.to_string(),
                version: 1,
                display_name: format!("Mock {kind}"),
                group: vec!["test".to_string()],
                hints: ResourceHint {
                    side_effect: SideEffect::None,
                    weight: None,
                    max_concurrency: None,
                },
                inputs: vec!["main".to_string()],
                outputs: vec!["main".to_string()],
                execute_once: false,
            },
            behavior: MockBehavior::Success(vec![]),
            call_count: Arc::new(Mutex::new(0)),
            last_input: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the output items this node will return.
    pub fn with_output(mut self, items: Vec<Value>) -> Self {
        self.behavior = MockBehavior::Success(items);
        self
    }

    /// Make this node fail with the given error.
    pub fn with_error(mut self, error_msg: &str) -> Self {
        self.behavior = MockBehavior::Fail(error_msg.to_string());
        self
    }

    /// Make this node return InDoubt (non-idempotent side effect uncertain).
    pub fn with_indoubt(mut self, items: Vec<Value>) -> Self {
        self.behavior = MockBehavior::InDoubt(items);
        self
    }

    /// Make this node timeout.
    pub fn with_timeout(mut self) -> Self {
        self.behavior = MockBehavior::Timeout;
        self
    }

    /// Set custom behavior via closure.
    pub fn with_custom<F>(mut self, f: F) -> Self
    where
        F: Fn(&[Value]) -> Result<Vec<Value>, NodeError> + Send + Sync + 'static,
    {
        self.behavior = MockBehavior::Custom(Arc::new(f));
        self
    }

    /// Set the side effect type for this node.
    pub fn with_side_effect(mut self, se: SideEffect) -> Self {
        self.descriptor.hints.side_effect = se;
        self
    }

    /// Set execute_once flag.
    pub fn with_execute_once(mut self) -> Self {
        self.descriptor.execute_once = true;
        self
    }

    /// Number of times execute() has been called.
    pub fn call_count(&self) -> u64 {
        *self.call_count.lock().unwrap()
    }

    /// Get the last input items passed to execute().
    pub fn last_input(&self) -> Option<Vec<Value>> {
        self.last_input.lock().unwrap().clone()
    }
}

#[async_trait]
impl Node for MockNode {
    fn descriptor(&self) -> &NodeDescriptor {
        &self.descriptor
    }

    async fn execute(&self, ctx: &mut NodeContext<'_>) -> Result<NodeOutput, NodeError> {
        *self.call_count.lock().unwrap() += 1;

        // Store input for later inspection
        // Note: NodeContext is a placeholder, so we store empty input for now
        *self.last_input.lock().unwrap() = Some(vec![]);

        match &self.behavior {
            MockBehavior::Success(items) => Ok(NodeOutput {
                items: items.clone(),
            }),
            MockBehavior::Fail(msg) => Err(NodeError::Execution(msg.clone())),
            MockBehavior::InDoubt(items) => {
                // In a real executor, this would set the task status to InDoubt.
                // For testing, we return success but the caller should check
                // descriptor.hints.side_effect == NonIdempotent.
                Ok(NodeOutput {
                    items: items.clone(),
                })
            }
            MockBehavior::Timeout => Err(NodeError::Timeout),
            MockBehavior::Custom(f) => {
                let items = f(&[])?;
                Ok(NodeOutput { items })
            }
        }
    }
}

// ============================================================================
// MockNodeContext — builder for test execution contexts
// ============================================================================

/// Builder for constructing `NodeContext` in tests.
///
/// Since the kernel `NodeContext` is currently a placeholder,
/// this provides a richer testing API that will evolve with the kernel.
pub struct MockNodeContext {
    pub input_items: Vec<Value>,
    pub parameters: HashMap<String, Value>,
    pub node_name: String,
    pub execution_id: String,
    pub item_index: usize,
    pub env_vars: HashMap<String, String>,
    pub spill_store: InMemorySpillStore,
    pub blob_store: InMemoryBlobStore,
    /// Prior node outputs: node_name -> Vec<Value>
    pub prior_outputs: HashMap<String, Vec<Value>>,
}

impl MockNodeContext {
    pub fn new() -> Self {
        Self {
            input_items: vec![],
            parameters: HashMap::new(),
            node_name: "TestNode".to_string(),
            execution_id: format!("exec-{}", uuid_simple()),
            item_index: 0,
            env_vars: HashMap::new(),
            spill_store: InMemorySpillStore::new(),
            blob_store: InMemoryBlobStore::new(),
            prior_outputs: HashMap::new(),
        }
    }

    /// Set input items for this node.
    pub fn with_input(mut self, items: Vec<Value>) -> Self {
        self.input_items = items;
        self
    }

    /// Set a parameter value.
    pub fn with_param(mut self, key: &str, value: Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }

    /// Set the node name.
    pub fn with_node_name(mut self, name: &str) -> Self {
        self.node_name = name.to_string();
        self
    }

    /// Set the execution ID.
    pub fn with_execution_id(mut self, id: &str) -> Self {
        self.execution_id = id.to_string();
        self
    }

    /// Add an environment variable (for $env testing).
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Set a prior node output (for $('NodeName') expression testing).
    pub fn with_prior_output(mut self, node_name: &str, items: Vec<Value>) -> Self {
        self.prior_outputs
            .insert(node_name.to_string(), items);
        self
    }

    /// Share a spill store with other contexts (for multi-node tests).
    pub fn with_shared_spill(mut self, store: InMemorySpillStore) -> Self {
        self.spill_store = store;
        self
    }

    /// Share a blob store with other contexts.
    pub fn with_shared_blob(mut self, store: InMemoryBlobStore) -> Self {
        self.blob_store = store;
        self
    }

    /// Get a parameter value.
    pub fn get_param(&self, key: &str) -> Option<&Value> {
        self.parameters.get(key)
    }

    /// Get a prior node's output.
    pub fn get_prior_output(&self, node_name: &str) -> Option<&Vec<Value>> {
        self.prior_outputs.get(node_name)
    }

    /// Build the kernel NodeContext (placeholder for now).
    pub fn build(&self) -> NodeContext<'static> {
        NodeContext {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for MockNodeContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TestFixture — convenience for multi-node workflow testing
// ============================================================================

/// A test fixture that sets up a mini workflow environment.
pub struct TestFixture {
    pub nodes: Vec<MockNode>,
    pub spill_store: InMemorySpillStore,
    pub blob_store: InMemoryBlobStore,
    pub initial_items: Vec<Value>,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            spill_store: InMemorySpillStore::new(),
            blob_store: InMemoryBlobStore::new(),
            initial_items: vec![],
        }
    }

    /// Add a node to the fixture.
    pub fn add_node(mut self, node: MockNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Set the initial input items (e.g., from a trigger).
    pub fn with_initial_items(mut self, items: Vec<Value>) -> Self {
        self.initial_items = items;
        self
    }

    /// Execute all nodes in sequence, piping output of each to input of the next.
    /// Returns the final output items.
    pub async fn run_sequential(&mut self) -> Result<Vec<Value>, NodeError> {
        let mut current_items = self.initial_items.clone();

        for node in &self.nodes {
            let ctx = MockNodeContext::new()
                .with_input(current_items.clone())
                .with_shared_spill(self.spill_store.clone())
                .with_shared_blob(self.blob_store.clone());
            let mut kernel_ctx = ctx.build();
            let output = node.execute(&mut kernel_ctx).await?;
            current_items = output.items;
        }

        Ok(current_items)
    }

    /// Assert no spill file leaks after execution.
    pub fn assert_no_spill_leaks(&self) {
        self.spill_store.assert_no_leaks();
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Codec test helpers
// ============================================================================

/// Encode items to JSON bytes (for spill simulation).
pub fn encode_items(items: &[Value]) -> Vec<u8> {
    serde_json::to_vec(items).expect("encode_items should not fail")
}

/// Decode JSON bytes to items.
pub fn decode_items(data: &[u8]) -> Vec<Value> {
    serde_json::from_slice(data).expect("decode_items should not fail")
}

// ============================================================================
// Internal helpers
// ============================================================================

fn blake3_hash(data: &[u8]) -> String {
    // Simple hash for testing. In production, use blake3 crate.
    // We implement a basic hash here to avoid adding blake3 dependency to testkit.
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{ts:x}")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_spill_write_read() {
        let store = InMemorySpillStore::new();
        let data = b"hello spill";

        let (path, checksum, bytes) = store.write("exec-1", data).await.unwrap();
        assert_eq!(bytes, data.len() as u64);
        assert_eq!(store.total_writes(), 1);

        let read_back = store.read(&path, &checksum).await.unwrap();
        assert_eq!(read_back, data);
        assert_eq!(store.total_reads(), 1);
    }

    #[tokio::test]
    async fn test_spill_checksum_mismatch() {
        let store = InMemorySpillStore::new();
        let (path, _, _) = store.write("exec-1", b"data").await.unwrap();

        let result = store.read(&path, "wrong-checksum").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_spill_gc() {
        let store = InMemorySpillStore::new();
        store.write("exec-1", b"a").await.unwrap();
        store.write("exec-1", b"b").await.unwrap();
        store.write("exec-2", b"c").await.unwrap();

        assert_eq!(store.active_executions(), 2);
        assert_eq!(store.total_spill_files(), 3);

        let removed = store.gc_execution("exec-1").await.unwrap();
        assert_eq!(removed, 2);
        assert_eq!(store.active_executions(), 1);
        assert_eq!(store.total_spill_files(), 1);

        // GC non-existent is a no-op
        let removed = store.gc_execution("exec-999").await.unwrap();
        assert_eq!(removed, 0);
    }

    #[tokio::test]
    async fn test_spill_no_leaks() {
        let store = InMemorySpillStore::new();
        store.write("exec-1", b"data").await.unwrap();
        store.gc_execution("exec-1").await.unwrap();
        store.assert_no_leaks();
    }

    #[tokio::test]
    async fn test_in_memory_blob_store() {
        let store = InMemoryBlobStore::new();

        store.put("file.bin", b"binary data").await.unwrap();
        assert!(store.exists("file.bin"));
        assert_eq!(store.count(), 1);

        let data = store.get("file.bin").await.unwrap();
        assert_eq!(data, b"binary data");

        store.delete("file.bin").await.unwrap();
        assert!(!store.exists("file.bin"));
    }

    #[tokio::test]
    async fn test_mock_node_success() {
        let node = MockNode::new("test.Set")
            .with_output(vec![serde_json::json!({"result": 42})]);

        let mut ctx = MockNodeContext::new().build();
        let result = node.execute(&mut ctx).await.unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0]["result"], 42);
        assert_eq!(node.call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_node_fail() {
        let node = MockNode::new("test.Fail")
            .with_error("boom");

        let mut ctx = MockNodeContext::new().build();
        let result = node.execute(&mut ctx).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_node_timeout() {
        let node = MockNode::new("test.Slow").with_timeout();

        let mut ctx = MockNodeContext::new().build();
        let result = node.execute(&mut ctx).await;

        assert!(matches!(result, Err(NodeError::Timeout)));
    }

    #[tokio::test]
    async fn test_mock_node_custom() {
        let node = MockNode::new("test.Transform").with_custom(|_input| {
            Ok(vec![serde_json::json!({"transformed": true})])
        });

        let mut ctx = MockNodeContext::new().build();
        let result = node.execute(&mut ctx).await.unwrap();
        assert_eq!(result.items[0]["transformed"], true);
    }

    #[tokio::test]
    async fn test_mock_node_descriptor() {
        let node = MockNode::new("n8n-nodes-base.httpRequest")
            .with_side_effect(SideEffect::NonIdempotent)
            .with_execute_once();

        let desc = node.descriptor();
        assert_eq!(desc.kind, "n8n-nodes-base.httpRequest");
        assert_eq!(desc.hints.side_effect, SideEffect::NonIdempotent);
        assert!(desc.execute_once);
    }

    #[tokio::test]
    async fn test_fixture_sequential() {
        let node1 = MockNode::new("test.Set")
            .with_output(vec![serde_json::json!({"step": 1})]);
        let node2 = MockNode::new("test.Set")
            .with_output(vec![serde_json::json!({"step": 2})]);

        let mut fixture = TestFixture::new()
            .with_initial_items(vec![serde_json::json!({"trigger": true})])
            .add_node(node1)
            .add_node(node2);

        let result = fixture.run_sequential().await.unwrap();
        assert_eq!(result[0]["step"], 2);
    }

    #[test]
    fn test_encode_decode_items() {
        let items = vec![
            serde_json::json!({"a": 1}),
            serde_json::json!({"b": "hello"}),
        ];
        let encoded = encode_items(&items);
        let decoded = decode_items(&encoded);
        assert_eq!(items, decoded);
    }

    #[test]
    fn test_mock_context_builder() {
        let ctx = MockNodeContext::new()
            .with_input(vec![serde_json::json!({"x": 1})])
            .with_param("url", serde_json::json!("https://example.com"))
            .with_node_name("HTTP Request")
            .with_execution_id("exec-abc")
            .with_env("API_KEY", "secret")
            .with_prior_output("Trigger", vec![serde_json::json!({"data": "hello"})]);

        assert_eq!(ctx.input_items.len(), 1);
        assert_eq!(ctx.get_param("url").unwrap(), "https://example.com");
        assert_eq!(ctx.node_name, "HTTP Request");
        assert_eq!(ctx.execution_id, "exec-abc");
        assert_eq!(ctx.env_vars.get("API_KEY").unwrap(), "secret");
        assert!(ctx.get_prior_output("Trigger").is_some());
    }

    #[test]
    fn test_kernel_types_serialization() {
        use kernel::ExecutionStatus;

        let status = ExecutionStatus::Success;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"success\"");

        let parsed: ExecutionStatus = serde_json::from_str("\"failed\"").unwrap();
        assert_eq!(parsed, ExecutionStatus::Failed);
    }

    #[test]
    fn test_side_effect_enum() {
        use kernel::SideEffect;

        let se = SideEffect::NonIdempotent;
        let json = serde_json::to_string(&se).unwrap();
        assert_eq!(json, "\"non_idempotent\"");

        let parsed: SideEffect = serde_json::from_str("\"idempotent\"").unwrap();
        assert_eq!(parsed, SideEffect::Idempotent);
    }
}
