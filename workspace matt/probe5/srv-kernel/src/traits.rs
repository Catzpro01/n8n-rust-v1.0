use async_trait::async_trait;
use crate::errors::NodeError;
use crate::types::NodeDescriptor;

/// Trait utama untuk semua node
#[async_trait]
pub trait Node: Send + Sync {
    fn descriptor(&self) -> &NodeDescriptor;
    async fn execute(&self, ctx: &mut NodeContext<'_>) -> Result<NodeOutput, NodeError>;
}

/// Context yang diberikan ke node saat eksekusi (placeholder)
pub struct NodeContext<'a> {
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

/// Output dari eksekusi node (placeholder)
pub struct NodeOutput {
    pub items: Vec<serde_json::Value>,
}
