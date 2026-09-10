use crate::errors::StorageError;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &Path, migrations_dir: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            PRAGMA cache_size = -64000;
            PRAGMA temp_store = MEMORY;
            PRAGMA foreign_keys = ON;
        ")?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        // Run migrations in a separate scope to release the lock
        {
            let c = db.conn.lock().unwrap();
            crate::migration::run_migrations(&c, migrations_dir)?;
        }
        Ok(db)
    }

    pub fn open_memory(migrations_dir: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        {
            let c = db.conn.lock().unwrap();
            crate::migration::run_migrations(&c, migrations_dir)?;
        }
        Ok(db)
    }
}

// ============ WORKFLOW REPOSITORY ============

pub struct WorkflowRepo { db: Database }

impl WorkflowRepo {
    pub fn new(db: Database) -> Self { Self { db } }

    pub fn create(&self, id: i64, name: &str, nodes_json: &str, connections_json: &str) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute("INSERT INTO workflow (id, name, nodes_json, connections_json) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, nodes_json, connections_json])?;
        Ok(())
    }

    pub fn get(&self, id: i64) -> Result<Option<WorkflowRow>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, name, active, version, nodes_json, connections_json, settings_json, created_at, updated_at FROM workflow WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(WorkflowRow {
                id: row.get(0)?, name: row.get(1)?, active: row.get::<_, i32>(2)? != 0,
                version: row.get(3)?, nodes_json: row.get(4)?, connections_json: row.get(5)?,
                settings_json: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(Ok(wf)) => Ok(Some(wf)),
            Some(Err(e)) => Err(StorageError::Database(e.to_string())),
            None => Ok(None),
        }
    }

    pub fn set_active(&self, id: i64, active: bool) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute("UPDATE workflow SET active = ?2, updated_at = datetime('now') WHERE id = ?1",
            params![id, active as i32])?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<WorkflowRow>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, name, active, version, nodes_json, connections_json, settings_json, created_at, updated_at FROM workflow ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(WorkflowRow {
                id: row.get(0)?, name: row.get(1)?, active: row.get::<_, i32>(2)? != 0,
                version: row.get(3)?, nodes_json: row.get(4)?, connections_json: row.get(5)?,
                settings_json: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows { result.push(row.map_err(|e| StorageError::Database(e.to_string()))?); }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowRow {
    pub id: i64, pub name: String, pub active: bool, pub version: i32,
    pub nodes_json: String, pub connections_json: String, pub settings_json: String,
    pub created_at: String, pub updated_at: String,
}

// ============ EXECUTION REPOSITORY ============

pub struct ExecutionRepo { db: Database }

impl ExecutionRepo {
    pub fn new(db: Database) -> Self { Self { db } }

    pub fn create(&self, id: i64, workflow_id: i64, workflow_version: i32, mode: &str) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute("INSERT INTO execution (id, workflow_id, workflow_version, mode, status) VALUES (?1, ?2, ?3, ?4, 'pending')",
            params![id, workflow_id, workflow_version, mode])?;
        Ok(())
    }

    pub fn update_status(&self, id: i64, status: &str) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        match status {
            "success" | "failed" | "canceled" | "needs_review" => {
                c.execute("UPDATE execution SET status = ?2, finished_at = datetime('now') WHERE id = ?1", params![id, status])?;
            }
            _ => {
                c.execute("UPDATE execution SET status = ?2 WHERE id = ?1", params![id, status])?;
            }
        }
        Ok(())
    }

    pub fn get(&self, id: i64) -> Result<Option<ExecutionRow>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, workflow_id, workflow_version, mode, status, started_at, finished_at, error_code, error_message, created_at FROM execution WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(ExecutionRow {
                id: row.get(0)?, workflow_id: row.get(1)?, workflow_version: row.get(2)?,
                mode: row.get(3)?, status: row.get(4)?, started_at: row.get(5)?,
                finished_at: row.get(6)?, error_code: row.get(7)?, error_message: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        match rows.next() {
            Some(Ok(e)) => Ok(Some(e)),
            Some(Err(e)) => Err(StorageError::Database(e.to_string())),
            None => Ok(None),
        }
    }

    pub fn count_by_status(&self) -> Result<Vec<(String, i64)>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT status, COUNT(*) FROM execution GROUP BY status")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut result = Vec::new();
        for row in rows { result.push(row.map_err(|e| StorageError::Database(e.to_string()))?); }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionRow {
    pub id: i64, pub workflow_id: i64, pub workflow_version: i32,
    pub mode: String, pub status: String, pub started_at: Option<String>,
    pub finished_at: Option<String>, pub error_code: Option<String>,
    pub error_message: Option<String>, pub created_at: String,
}

// ============ TASK REPOSITORY ============

pub struct TaskRepo { db: Database }

impl TaskRepo {
    pub fn new(db: Database) -> Self { Self { db } }

    pub fn create(&self, id: i64, execution_id: i64, node_id: &str, priority: i32) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute("INSERT INTO task (id, execution_id, node_id, priority, status) VALUES (?1, ?2, ?3, ?4, 'ready')",
            params![id, execution_id, node_id, priority])?;
        Ok(())
    }

    pub fn update_status(&self, id: i64, status: &str) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute("UPDATE task SET status = ?2, finished_at = datetime('now') WHERE id = ?1", params![id, status])?;
        Ok(())
    }

    pub fn get_ready(&self, execution_id: i64, limit: i32) -> Result<Vec<TaskRow>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, execution_id, node_id, input_index, attempt, priority, status FROM task WHERE execution_id = ?1 AND status = 'ready' ORDER BY priority DESC LIMIT ?2")?;
        let rows = stmt.query_map(params![execution_id, limit], |row| {
            Ok(TaskRow { id: row.get(0)?, execution_id: row.get(1)?, node_id: row.get(2)?,
                input_index: row.get(3)?, attempt: row.get(4)?, priority: row.get(5)?, status: row.get(6)? })
        })?;
        let mut result = Vec::new();
        for row in rows { result.push(row.map_err(|e| StorageError::Database(e.to_string()))?); }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: i64, pub execution_id: i64, pub node_id: String,
    pub input_index: i32, pub attempt: i32, pub priority: i32, pub status: String,
}

// ============ WRITE-AHEAD INTENT REPOSITORY ============

pub struct IntentRepo { db: Database }

impl IntentRepo {
    pub fn new(db: Database) -> Self { Self { db } }

    pub fn record_intent(&self, execution_id: &str, intent_type: &str, target_path: Option<&str>) -> Result<i64, StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute("INSERT INTO spill_intent (execution_id, intent_type, target_path) VALUES (?1, ?2, ?3)",
            params![execution_id, intent_type, target_path])?;
        Ok(c.last_insert_rowid())
    }

    pub fn complete_intent(&self, intent_id: i64) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute("DELETE FROM spill_intent WHERE id = ?1", params![intent_id])?;
        Ok(())
    }

    pub fn scan_orphans(&self) -> Result<Vec<OrphanIntent>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, execution_id, intent_type, target_path FROM spill_intent ORDER BY created_at")?;
        let rows = stmt.query_map([], |row| {
            Ok(OrphanIntent { id: row.get(0)?, execution_id: row.get(1)?, intent_type: row.get(2)?, target_path: row.get(3)? })
        })?;
        let mut result = Vec::new();
        for row in rows { result.push(row.map_err(|e| StorageError::Database(e.to_string()))?); }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct OrphanIntent {
    pub id: i64, pub execution_id: String, pub intent_type: String, pub target_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    fn test_migrations_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
    }
    
    #[test]
    fn test_database_open_memory() {
        let _db = Database::open_memory(&test_migrations_dir()).unwrap();
        // Database opened successfully
    }
    
    #[test]
    fn test_workflow_repo_create_and_get() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = WorkflowRepo::new(db);
        
        // Create
        repo.create(1, "Test Workflow", "{}", "{}").unwrap();
        
        // Read
        let wf = repo.get(1).unwrap().unwrap();
        assert_eq!(wf.id, 1);
        assert_eq!(wf.name, "Test Workflow");
        assert!(!wf.active);
    }
    
    #[test]
    fn test_workflow_repo_set_active() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = WorkflowRepo::new(db);
        
        repo.create(1, "Test", "{}", "{}").unwrap();
        repo.set_active(1, true).unwrap();
        
        let wf = repo.get(1).unwrap().unwrap();
        assert!(wf.active);
    }
    
    #[test]
    fn test_workflow_repo_list() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = WorkflowRepo::new(db);
        
        repo.create(1, "Test 1", "{}", "{}").unwrap();
        repo.create(2, "Test 2", "{}", "{}").unwrap();
        
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 2);
    }
}
