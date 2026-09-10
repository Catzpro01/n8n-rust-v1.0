use crate::error::CasdError;
use crate::index::{CasdEntry, CasdIndex};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Content-Addressable Spill Deduplication Store
pub struct CasdStore {
    index: CasdIndex,
    spill_dir: PathBuf,
}

impl CasdStore {
    /// Create a new CASD store
    pub fn new(spill_dir: impl AsRef<Path>, index: CasdIndex) -> Result<Self, CasdError> {
        let spill_dir = spill_dir.as_ref().to_path_buf();
        fs::create_dir_all(&spill_dir)?;
        Ok(Self { index, spill_dir })
    }

    /// Write content with deduplication.
    /// Returns (hash, was_new) - hash of content, whether it was newly written.
    pub fn write(&self, execution_id: &str, spill_name: &str, content: &[u8]) -> Result<(String, bool), CasdError> {
        // 1. Compute BLAKE3 hash
        let hash = Self::compute_hash(content);
        
        // 2. Check index for existing content
        if let Some(existing) = self.index.lookup(&hash)? {
            // Content exists - verify it's actually the same (collision check)
            let existing_path = Path::new(&existing.file_path);
            if existing_path.exists() {
                // Quick check: compare sizes
                let existing_size = fs::metadata(existing_path)?.len();
                if existing_size != content.len() as u64 {
                    // Size mismatch = collision! (different content, same hash)
                    return Err(CasdError::Collision {
                        hash: hash.clone(),
                        existing: existing_size,
                        new: content.len() as u64,
                    });
                }
                
                // Quick byte-compare first 1KB
                let existing_bytes = fs::read(existing_path)?;
                let compare_len = std::cmp::min(1024, content.len());
                if existing_bytes[..compare_len] == content[..compare_len] {
                    // Same content - dedup!
                    self.index.add_ref(&hash, 0)?;
                    self.index.record_ref(execution_id, spill_name, &hash, 0)?;
                    return Ok((hash, false));
                } else {
                    // Collision! Different content
                    return Err(CasdError::Collision {
                        hash: hash.clone(),
                        existing: existing_size,
                        new: content.len() as u64,
                    });
                }
            }
            // File missing but index has entry - orphan, re-write
        }
        
        // 3. Write new content
        let file_path = self.hash_to_path(&hash);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content)?;
        file.sync_all()?;
        
        // 4. Update index
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let entry = CasdEntry {
            content_hash: hash.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            first_seen: now,
            ref_count: 1,
            total_bytes: content.len() as i64,
            collision_id: 0,
        };
        self.index.insert(&entry)?;
        self.index.record_ref(execution_id, spill_name, &hash, 0)?;
        
        Ok((hash, true))
    }

    /// Read content by hash
    pub fn read(&self, hash: &str) -> Result<Vec<u8>, CasdError> {
        let entry = self.index.lookup(hash)?
            .ok_or_else(|| CasdError::NotFound(hash.to_string()))?;
        fs::read(&entry.file_path).map_err(CasdError::Io)
    }

    /// Release a reference (for GC when execution completes/deletes)
    pub fn release(&self, _execution_id: &str, _spill_name: &str) -> Result<bool, CasdError> {
        // Look up the reference
        // For prototype, we track via add_ref/release_ref directly
        // In production, casd_refs table would track execution → hash mapping
        Ok(false)
    }

    /// Garbage collect: remove content with ref_count <= 0
    pub fn gc(&self) -> Result<GcResult, CasdError> {
        // GC: find entries with ref_count <= 0, delete file + index row
        // For prototype: stats-based (no actual deletion in this stub)
        let stats = self.index.stats()?;
        Ok(GcResult {
            files_removed: 0,  // Prototype: no actual GC
            bytes_freed: 0,
        })
    }

    /// Get store statistics
    pub fn stats(&self) -> Result<crate::index::CasdStats, CasdError> {
        self.index.stats()
    }

    /// Compute BLAKE3 hash of content
    fn compute_hash(content: &[u8]) -> String {
        let hash = blake3::hash(content);
        hash.to_hex().to_string()
    }

    /// Map hash to file path (2-char prefix sharding)
    fn hash_to_path(&self, hash: &str) -> PathBuf {
        let prefix = &hash[..2];
        self.spill_dir.join(prefix).join(format!("{}.spill", hash))
    }
}

#[derive(Debug)]
pub struct GcResult {
    pub files_removed: usize,
    pub bytes_freed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_store() -> (CasdStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        let index = CasdIndex::in_memory().unwrap();
        let store = CasdStore::new(tmp.path().join("spills"), index).unwrap();
        (store, tmp)
    }

    #[test]
    fn test_write_new() {
        let (store, _tmp) = setup_store();
        let content = b"hello world";
        let (hash, was_new) = store.write("exec-1", "output", content).unwrap();
        assert!(was_new);
        assert!(!hash.is_empty());
        
        // Verify file exists
        let data = store.read(&hash).unwrap();
        assert_eq!(data, content);
    }

    #[test]
    fn test_write_dedup() {
        let (store, _tmp) = setup_store();
        let content = b"identical content";
        
        // First write - new
        let (hash1, was_new1) = store.write("exec-1", "output", content).unwrap();
        assert!(was_new1);
        
        // Second write - dedup!
        let (hash2, was_new2) = store.write("exec-2", "output", content).unwrap();
        assert!(!was_new2);
        assert_eq!(hash1, hash2);
        
        // Stats: 1 unique, 2 refs
        let stats = store.stats().unwrap();
        assert_eq!(stats.unique_entries, 1);
        assert_eq!(stats.total_references, 2);
    }

    #[test]
    fn test_dedup_ratio() {
        let (store, _tmp) = setup_store();
        let content = b"repeated data across executions";
        
        // Write same content 10 times
        for i in 0..10 {
            store.write(&format!("exec-{}", i), "output", content).unwrap();
        }
        
        let stats = store.stats().unwrap();
        assert_eq!(stats.unique_entries, 1);
        assert_eq!(stats.total_references, 10);
        // Dedup ratio: 1 - (physical/logical) = 1 - (30/300) = 90%
        assert!(stats.savings_pct() > 89.0);
    }

    #[test]
    fn test_different_content() {
        let (store, _tmp) = setup_store();
        
        let (hash1, _) = store.write("exec-1", "a", b"content A").unwrap();
        let (hash2, _) = store.write("exec-1", "b", b"content B").unwrap();
        
        assert_ne!(hash1, hash2);
        
        let data1 = store.read(&hash1).unwrap();
        let data2 = store.read(&hash2).unwrap();
        assert_eq!(data1, b"content A");
        assert_eq!(data2, b"content B");
    }

    #[test]
    fn test_hash_determinism() {
        // Same content always produces same hash
        let h1 = CasdStore::compute_hash(b"test data");
        let h2 = CasdStore::compute_hash(b"test data");
        assert_eq!(h1, h2);
        
        // Different content produces different hash
        let h3 = CasdStore::compute_hash(b"different data");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_large_content() {
        let (store, _tmp) = setup_store();
        let content = vec![42u8; 1_000_000]; // 1MB
        
        let (hash, was_new) = store.write("exec-1", "large", &content).unwrap();
        assert!(was_new);
        
        let data = store.read(&hash).unwrap();
        assert_eq!(data.len(), 1_000_000);
        assert_eq!(data[0], 42);
    }
}
