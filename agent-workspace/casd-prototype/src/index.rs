use crate::error::CasdError;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct CasdEntry {
    pub content_hash: String,
    pub file_path: String,
    pub first_seen: i64,
    pub ref_count: i64,
    pub total_bytes: i64,
    pub collision_id: i64,
}

pub struct CasdIndex {
    conn: Mutex<Connection>,
}

impl CasdIndex {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, CasdError> {
        let conn = Connection::open(path).map_err(|e| CasdError::Index(e.to_string()))?;
        let index = Self { conn: Mutex::new(conn) };
        index.init_schema()?;
        Ok(index)
    }

    pub fn in_memory() -> Result<Self, CasdError> {
        Self::new(":memory:")
    }

    fn init_schema(&self) -> Result<(), CasdError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS casd_index (
                content_hash TEXT NOT NULL,
                collision_id INTEGER DEFAULT 0,
                file_path TEXT NOT NULL,
                first_seen INTEGER NOT NULL,
                ref_count INTEGER DEFAULT 1,
                total_bytes INTEGER NOT NULL,
                PRIMARY KEY (content_hash, collision_id)
            );
            CREATE INDEX IF NOT EXISTS idx_casd_hash ON casd_index(content_hash);
            
            CREATE TABLE IF NOT EXISTS casd_refs (
                execution_id TEXT NOT NULL,
                spill_name TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                collision_id INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (execution_id, spill_name)
            );"
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        Ok(())
    }

    /// Look up content by hash
    pub fn lookup(&self, hash: &str) -> Result<Option<CasdEntry>, CasdError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT content_hash, file_path, first_seen, ref_count, total_bytes, collision_id 
             FROM casd_index WHERE content_hash = ?1 AND collision_id = 0"
        ).map_err(|e| CasdError::Index(e.to_string()))?;

        let result = stmt.query_row(params![hash], |row| {
            Ok(CasdEntry {
                content_hash: row.get(0)?,
                file_path: row.get(1)?,
                first_seen: row.get(2)?,
                ref_count: row.get(3)?,
                total_bytes: row.get(4)?,
                collision_id: row.get(5)?,
            })
        });

        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CasdError::Index(e.to_string())),
        }
    }

    /// Insert new content entry
    pub fn insert(&self, entry: &CasdEntry) -> Result<(), CasdError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO casd_index (content_hash, collision_id, file_path, first_seen, ref_count, total_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![entry.content_hash, entry.collision_id, entry.file_path, 
                    entry.first_seen, entry.ref_count, entry.total_bytes]
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        Ok(())
    }

    /// Increment reference count
    pub fn add_ref(&self, hash: &str, collision_id: i64) -> Result<(), CasdError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE casd_index SET ref_count = ref_count + 1 WHERE content_hash = ?1 AND collision_id = ?2",
            params![hash, collision_id]
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        Ok(())
    }

    /// Decrement reference count (for GC)
    pub fn release_ref(&self, hash: &str, collision_id: i64) -> Result<bool, CasdError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE casd_index SET ref_count = ref_count - 1 WHERE content_hash = ?1 AND collision_id = ?2",
            params![hash, collision_id]
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        
        let remaining: i64 = conn.query_row(
            "SELECT ref_count FROM casd_index WHERE content_hash = ?1 AND collision_id = ?2",
            params![hash, collision_id],
            |row| row.get(0)
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        
        Ok(remaining <= 0)
    }

    /// Remove entry (after file deletion)
    pub fn remove(&self, hash: &str, collision_id: i64) -> Result<(), CasdError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM casd_index WHERE content_hash = ?1 AND collision_id = ?2",
            params![hash, collision_id]
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        Ok(())
    }

    /// Record a reference from execution to content
    pub fn record_ref(&self, execution_id: &str, spill_name: &str, hash: &str, collision_id: i64) -> Result<(), CasdError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        conn.execute(
            "INSERT OR REPLACE INTO casd_refs (execution_id, spill_name, content_hash, collision_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![execution_id, spill_name, hash, collision_id, now]
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> Result<CasdStats, CasdError> {
        let conn = self.conn.lock().unwrap();
        let unique_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM casd_index", [], |row| row.get(0)
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        
        let total_refs: i64 = conn.query_row(
            "SELECT COALESCE(SUM(ref_count), 0) FROM casd_index", [], |row| row.get(0)
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        
        let total_bytes: i64 = conn.query_row(
            "SELECT COALESCE(SUM(total_bytes), 0) FROM casd_index", [], |row| row.get(0)
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        
        let logical_bytes: i64 = conn.query_row(
            "SELECT COALESCE(SUM(total_bytes * ref_count), 0) FROM casd_index", [], |row| row.get(0)
        ).map_err(|e| CasdError::Index(e.to_string()))?;
        
        Ok(CasdStats {
            unique_entries: unique_count as usize,
            total_references: total_refs as usize,
            physical_bytes: total_bytes as u64,
            logical_bytes: logical_bytes as u64,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CasdStats {
    pub unique_entries: usize,
    pub total_references: usize,
    pub physical_bytes: u64,
    pub logical_bytes: u64,
}

impl CasdStats {
    pub fn dedup_ratio(&self) -> f64 {
        if self.logical_bytes == 0 { return 0.0; }
        1.0 - (self.physical_bytes as f64 / self.logical_bytes as f64)
    }
    
    pub fn savings_pct(&self) -> f64 {
        self.dedup_ratio() * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_lookup() {
        let idx = CasdIndex::in_memory().unwrap();
        let entry = CasdEntry {
            content_hash: "abc123".to_string(),
            file_path: "/spills/ab/abc123.spill".to_string(),
            first_seen: 1000,
            ref_count: 1,
            total_bytes: 1024,
            collision_id: 0,
        };
        idx.insert(&entry).unwrap();
        let found = idx.lookup("abc123").unwrap().unwrap();
        assert_eq!(found.content_hash, "abc123");
        assert_eq!(found.total_bytes, 1024);
    }

    #[test]
    fn test_ref_counting() {
        let idx = CasdIndex::in_memory().unwrap();
        let entry = CasdEntry {
            content_hash: "def456".to_string(),
            file_path: "/spills/de/def456.spill".to_string(),
            first_seen: 1000,
            ref_count: 1,
            total_bytes: 2048,
            collision_id: 0,
        };
        idx.insert(&entry).unwrap();
        idx.add_ref("def456", 0).unwrap();
        let found = idx.lookup("def456").unwrap().unwrap();
        assert_eq!(found.ref_count, 2);
        
        let should_delete = idx.release_ref("def456", 0).unwrap();
        assert!(!should_delete);
        let should_delete = idx.release_ref("def456", 0).unwrap();
        assert!(should_delete);
    }

    #[test]
    fn test_dedup_stats() {
        let idx = CasdIndex::in_memory().unwrap();
        // Insert 1 unique entry referenced 5 times
        let entry = CasdEntry {
            content_hash: "dup1".to_string(),
            file_path: "/spills/dup1.spill".to_string(),
            first_seen: 1000,
            ref_count: 5,
            total_bytes: 1000,
            collision_id: 0,
        };
        idx.insert(&entry).unwrap();
        let stats = idx.stats().unwrap();
        assert_eq!(stats.unique_entries, 1);
        assert_eq!(stats.physical_bytes, 1000);
        assert_eq!(stats.logical_bytes, 5000);
        assert!((stats.savings_pct() - 80.0).abs() < 0.1);
    }
}
