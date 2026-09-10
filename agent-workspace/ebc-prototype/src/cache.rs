//! SQLite-based bytecode cache

use crate::{CacheKey, EbcError};
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

/// SQLite-backed bytecode cache
pub struct BytecodeCache {
    conn: Mutex<Connection>,
}

impl BytecodeCache {
    /// Create a new cache at the given path (or in-memory if ":memory:")
    pub fn new(path: impl AsRef<Path>) -> Result<Self, EbcError> {
        let conn = Connection::open(path)
            .map_err(|e| EbcError::Storage(e.to_string()))?;
        
        let cache = Self { conn: Mutex::new(conn) };
        cache.init_schema()?;
        Ok(cache)
    }
    
    /// Create in-memory cache (for testing)
    pub fn in_memory() -> Result<Self, EbcError> {
        Self::new(":memory:")
    }
    
    fn init_schema(&self) -> Result<(), EbcError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS expression_cache (
                workflow_id TEXT NOT NULL,
                version INTEGER NOT NULL,
                node_id TEXT NOT NULL,
                param_name TEXT NOT NULL,
                bytecode BLOB NOT NULL,
                source_hash TEXT NOT NULL,
                compiled_at INTEGER NOT NULL,
                PRIMARY KEY (workflow_id, version, node_id, param_name)
            );
            CREATE INDEX IF NOT EXISTS idx_cache_lookup 
                ON expression_cache(workflow_id, version, node_id, param_name);"
        ).map_err(|e| EbcError::Storage(e.to_string()))?;
        Ok(())
    }
    
    /// Store bytecode in cache
    pub fn put(&self, key: &CacheKey, bytecode: &[u8], source_hash: &str) -> Result<(), EbcError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        conn.execute(
            "INSERT OR REPLACE INTO expression_cache 
             (workflow_id, version, node_id, param_name, bytecode, source_hash, compiled_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![key.workflow_id, key.version, key.node_id, key.param_name, bytecode, source_hash, now]
        ).map_err(|e| EbcError::Storage(e.to_string()))?;
        Ok(())
    }
    
    /// Retrieve bytecode from cache
    pub fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, EbcError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT bytecode FROM expression_cache 
             WHERE workflow_id = ?1 AND version = ?2 AND node_id = ?3 AND param_name = ?4"
        ).map_err(|e| EbcError::Storage(e.to_string()))?;
        
        let mut rows = stmt.query(params![key.workflow_id, key.version, key.node_id, key.param_name])
            .map_err(|e| EbcError::Storage(e.to_string()))?;
        
        match rows.next().map_err(|e| EbcError::Storage(e.to_string()))? {
            Some(row) => {
                let bytecode: Vec<u8> = row.get(0).map_err(|e| EbcError::Storage(e.to_string()))?;
                Ok(Some(bytecode))
            }
            None => Ok(None),
        }
    }
    
    /// Invalidate all cache entries for a workflow (version change)
    pub fn invalidate_workflow(&self, workflow_id: &str) -> Result<usize, EbcError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "DELETE FROM expression_cache WHERE workflow_id = ?1",
            params![workflow_id]
        ).map_err(|e| EbcError::Storage(e.to_string()))?;
        Ok(count)
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats, EbcError> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM expression_cache", [], |row| row.get(0)
        ).map_err(|e| EbcError::Storage(e.to_string()))?;
        
        let total_bytes: i64 = conn.query_row(
            "SELECT COALESCE(SUM(LENGTH(bytecode)), 0) FROM expression_cache", [], |row| row.get(0)
        ).map_err(|e| EbcError::Storage(e.to_string()))?;
        
        Ok(CacheStats {
            entry_count: count as usize,
            total_bytes: total_bytes as usize,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_put_get() {
        let cache = BytecodeCache::in_memory().unwrap();
        let key = CacheKey::new("wf-1", 1, "node-1", "url");
        let bytecode = vec![1, 2, 3, 4, 5];
        
        // Put
        cache.put(&key, &bytecode, "hash123").unwrap();
        
        // Get
        let result = cache.get(&key).unwrap();
        assert_eq!(result, Some(bytecode));
    }
    
    #[test]
    fn test_cache_miss() {
        let cache = BytecodeCache::in_memory().unwrap();
        let key = CacheKey::new("wf-1", 1, "node-1", "url");
        
        let result = cache.get(&key).unwrap();
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_cache_invalidation() {
        let cache = BytecodeCache::in_memory().unwrap();
        let key1 = CacheKey::new("wf-1", 1, "node-1", "url");
        let key2 = CacheKey::new("wf-1", 1, "node-2", "body");
        let key3 = CacheKey::new("wf-2", 1, "node-1", "url");
        
        cache.put(&key1, &[1, 2, 3], "h1").unwrap();
        cache.put(&key2, &[4, 5, 6], "h2").unwrap();
        cache.put(&key3, &[7, 8, 9], "h3").unwrap();
        
        // Invalidate wf-1
        let count = cache.invalidate_workflow("wf-1").unwrap();
        assert_eq!(count, 2);
        
        // wf-1 entries gone
        assert_eq!(cache.get(&key1).unwrap(), None);
        assert_eq!(cache.get(&key2).unwrap(), None);
        
        // wf-2 still there
        assert!(cache.get(&key3).unwrap().is_some());
    }
    
    #[test]
    fn test_version_isolation() {
        let cache = BytecodeCache::in_memory().unwrap();
        let key_v1 = CacheKey::new("wf-1", 1, "node-1", "url");
        let key_v2 = CacheKey::new("wf-1", 2, "node-1", "url");
        
        cache.put(&key_v1, &[1, 2, 3], "h1").unwrap();
        cache.put(&key_v2, &[4, 5, 6], "h2").unwrap();
        
        // Different versions = different cache entries
        assert_eq!(cache.get(&key_v1).unwrap(), Some(vec![1, 2, 3]));
        assert_eq!(cache.get(&key_v2).unwrap(), Some(vec![4, 5, 6]));
    }
}
