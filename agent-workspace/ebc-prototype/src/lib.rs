//! Expression Bytecode Cache (EBC) - Prototype
//!
//! W1-EBC-CACHE implementation: QuickJS bytecode caching for expression evaluation.
//!
//! # Architecture
//!
//! ```text
//! Expression → Compile → Bytecode → SQLite Cache
//!                                    ↓
//! Runtime → Cache Lookup → Load Bytecode → Execute → Result
//! ```

mod cache;
mod engine;
mod error;

pub use cache::BytecodeCache;
pub use engine::CachedExpressionEngine;
pub use error::EbcError;

/// Cache key components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub workflow_id: String,
    pub version: u32,
    pub node_id: String,
    pub param_name: String,
}

impl CacheKey {
    pub fn new(workflow_id: impl Into<String>, version: u32, node_id: impl Into<String>, param_name: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            version,
            node_id: node_id.into(),
            param_name: param_name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_creation() {
        let key = CacheKey::new("wf-123", 1, "node-1", "url");
        assert_eq!(key.workflow_id, "wf-123");
        assert_eq!(key.version, 1);
        assert_eq!(key.node_id, "node-1");
        assert_eq!(key.param_name, "url");
    }
}
