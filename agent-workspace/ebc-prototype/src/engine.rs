//! Cached expression engine using QuickJS bytecode

use crate::{BytecodeCache, CacheKey, EbcError};
use rquickjs::{Context, Runtime};
use serde_json::Value;

/// Expression engine with bytecode caching
pub struct CachedExpressionEngine {
    cache: BytecodeCache,
}

impl CachedExpressionEngine {
    pub fn new(cache: BytecodeCache) -> Self {
        Self { cache }
    }
    
    /// Evaluate an expression, using cache if available
    pub async fn eval(
        &self,
        key: &CacheKey,
        expr: &str,
        scope: &Value,
    ) -> Result<Value, EbcError> {
        let source_hash = Self::hash_source(expr);
        
        // Try cache first
        if let Some(bytecode) = self.cache.get(key)? {
            match self.execute_bytecode(&bytecode, scope).await {
                Ok(result) => return Ok(result),
                Err(_) => {}
            }
        }
        
        // Cache miss: compile and cache
        let bytecode = self.compile_expression(expr)?;
        self.cache.put(key, &bytecode, &source_hash)?;
        
        self.execute_bytecode(&bytecode, scope).await
    }
    
    /// Compile expression (prototype: store as source)
    fn compile_expression(&self, expr: &str) -> Result<Vec<u8>, EbcError> {
        let rt = Runtime::new().map_err(|e| EbcError::Compilation(e.to_string()))?;
        let ctx = Context::full(&rt).map_err(|e| EbcError::Compilation(e.to_string()))?;
        
        ctx.with(|ctx| {
            // Validate expression compiles
            let wrapped = format!("(function($json, $itemIndex) {{ return {}; }})", expr);
            ctx.eval::<(), _>(wrapped.clone())
                .map_err(|e| EbcError::Compilation(e.to_string()))?;
            
            // Store source as "bytecode" for prototype
            Ok(wrapped.into_bytes())
        })
    }
    
    /// Execute bytecode with given scope
    async fn execute_bytecode(&self, bytecode: &[u8], scope: &Value) -> Result<Value, EbcError> {
        let rt = Runtime::new().map_err(|e| EbcError::Execution(e.to_string()))?;
        let ctx = Context::full(&rt).map_err(|e| EbcError::Execution(e.to_string()))?;
        
        ctx.with(|ctx| {
            let source = std::str::from_utf8(bytecode)
                .map_err(|e| EbcError::Deserialization(e.to_string()))?;
            
            let json = scope.get("json").unwrap_or(&Value::Null);
            let item_index = scope.get("itemIndex")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            
            // Convert to JSON strings for JS injection
            let json_str = serde_json::to_string(json)
                .map_err(|e| EbcError::Execution(e.to_string()))?;
            
            // Build complete evaluation script
            let script = format!(
                r#"
                var $json = {};
                var $itemIndex = {};
                var __fn = {};
                JSON.stringify(__fn($json, $itemIndex));
                "#,
                json_str, item_index, source
            );
            
            // Execute and get JSON string result
            let result_str: String = ctx.eval(script.clone())
                .map_err(|e| EbcError::Execution(e.to_string()))?;
            
            // Parse back to serde_json::Value
            let result: Value = serde_json::from_str(&result_str)
                .map_err(|e| EbcError::Execution(e.to_string()))?;
            
            Ok(result)
        })
    }
    
    /// Deterministic source hash using BLAKE3 (stable across Rust versions).
    /// DECISION: Not DefaultHasher (SipHash-1-3) because:
    ///   (a) Non-deterministic across Rust versions (impl can change)
    ///   (b) Same class of bug as C-05 in testkit (matt finding)
    ///   (c) Cache key must be stable for cache invalidation correctness
    /// BLAKE3 is already a dependency and provides deterministic 256-bit output.
    fn hash_source(source: &str) -> String {
        blake3::hash(source.as_bytes()).to_hex().to_string()
    }
    
    /// Validate expression without executing
    pub async fn validate(&self, expr: &str) -> Result<(), EbcError> {
        let rt = Runtime::new().map_err(|e| EbcError::Validation(e.to_string()))?;
        let ctx = Context::full(&rt).map_err(|e| EbcError::Validation(e.to_string()))?;
        
        ctx.with(|ctx| {
            let wrapped = format!("(function() {{ return {}; }})", expr);
            ctx.eval::<(), _>(wrapped.clone())
                .map_err(|e| EbcError::Validation(e.to_string()))?;
            Ok(())
        })
    }
    
    pub fn cache_stats(&self) -> Result<crate::cache::CacheStats, EbcError> {
        self.cache.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_simple_expression() {
        let cache = BytecodeCache::in_memory().unwrap();
        let engine = CachedExpressionEngine::new(cache);
        
        let key = CacheKey::new("wf-1", 1, "node-1", "value");
        let expr = "1 + 2";
        let scope = serde_json::json!({"json": {}, "itemIndex": 0});
        
        let result = engine.eval(&key, expr, &scope).await.unwrap();
        assert_eq!(result, serde_json::json!(3));
    }
    
    #[tokio::test]
    async fn test_cache_hit() {
        let cache = BytecodeCache::in_memory().unwrap();
        let engine = CachedExpressionEngine::new(cache);
        
        let key = CacheKey::new("wf-1", 1, "node-1", "value");
        let expr = "'hello ' + 'world'";
        let scope = serde_json::json!({"json": {}, "itemIndex": 0});
        
        let result1 = engine.eval(&key, expr, &scope).await.unwrap();
        assert_eq!(result1, serde_json::json!("hello world"));
        
        let stats = engine.cache_stats().unwrap();
        assert_eq!(stats.entry_count, 1);
        
        let result2 = engine.eval(&key, expr, &scope).await.unwrap();
        assert_eq!(result2, serde_json::json!("hello world"));
    }
    
    #[tokio::test]
    async fn test_json_access() {
        let cache = BytecodeCache::in_memory().unwrap();
        let engine = CachedExpressionEngine::new(cache);
        
        let key = CacheKey::new("wf-1", 1, "node-1", "email");
        let expr = "$json.user.email";
        let scope = serde_json::json!({
            "json": {"user": {"email": "test@example.com"}},
            "itemIndex": 0
        });
        
        let result = engine.eval(&key, expr, &scope).await.unwrap();
        assert_eq!(result, serde_json::json!("test@example.com"));
    }
    
    #[tokio::test]
    async fn test_validation() {
        let cache = BytecodeCache::in_memory().unwrap();
        let engine = CachedExpressionEngine::new(cache);
        
        assert!(engine.validate("1 + 2").await.is_ok());
        assert!(engine.validate("1 +").await.is_err());
    }
}
