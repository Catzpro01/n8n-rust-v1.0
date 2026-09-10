# PREFACTOR-3: Storage Integration Guide
## Single Storage Path — Kernel SpillStore + Storage WAL

**Author:** agent2 (Storage Owner)  
**Date:** 2026-09-09  
**Status:** COMPLETE  
**Task:** matt #1664 PREFACTOR-3

---

## Executive Summary

**Objective:** Ensure TB-02 uses kernel SpillStore/SpilledList contracts for spill operations, with optional storage WAL for crash safety.

**Key Finding:** Storage `spill_intent` table is NOT duplicate — it's Write-Ahead Log (WAL) for crash safety, complementary to kernel SpillStore.

**Recommendation:** 
- Use kernel SpillStore for actual spill operations
- Keep storage spill_intent as WAL (optional, for crash safety)
- Integrate: WAL → SpillStore → WAL completion

---

## Kernel Contracts (kernel-asli-d3bcff0)

### SpillStore Trait
```rust
pub trait SpillStore: Send + Sync {
    /// Persist items and return handle
    async fn write(&self, items: &[Item]) -> Result<SpilledList, KernelError>;
    
    /// Read one item by index (O(1)-ish)
    async fn read_at(&self, handle: &SpilledList, index: usize) -> Result<Item, KernelError>;
    
    /// Read range of items
    async fn read_range(&self, handle: &SpilledList, start: usize, len: usize) -> Result<Vec<Item>, KernelError>;
    
    /// Read all items (requires Governor approval)
    async fn read_all(&self, handle: &SpilledList) -> Result<Vec<Item>, KernelError>;
    
    /// Delete backing file (GC lifecycle)
    async fn delete(&self, handle: &SpilledList) -> Result<(), KernelError>;
}
```

### SpilledList Struct
```rust
pub struct SpilledList {
    pub path: SpillPath,      // Opaque path identifier
    pub len: u32,             // Number of items
    pub total_bytes: u64,     // Total serialized size
    pub codec: SpillCodec,    // Serialization format
}
```

### FileSpillStore Implementation
- Location: `data_plane::FileSpillStore`
- Backing: File-based spill storage
- Permissions: 0600/0700 (Ruling 10)

---

## Storage WAL (spill_intent)

### Purpose
Write-Ahead Log for crash safety — tracks intent BEFORE operation, completes AFTER.

### Schema
```sql
CREATE TABLE spill_intent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER NOT NULL REFERENCES execution(id),
    intent_type TEXT NOT NULL CHECK (intent_type IN (
        'write_spill',      -- About to write spill
        'delete_spill',     -- About to delete spill
        'write_event',      -- About to write event
        'update_task'       -- About to update task
    )),
    target_path TEXT,        -- Spill path (for spill operations)
    payload_json TEXT,       -- Operation payload
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### API
```rust
// Record intent BEFORE operation
let intent_id = storage.record_intent(exec_id, "write_spill", Some("/path/to/spill"))?;

// Perform actual operation (e.g., kernel SpillStore::write)
let spilled_list = kernel_spill_store.write(items).await?;

// Complete intent AFTER operation
storage.complete_intent(intent_id)?;

// On crash recovery: find incomplete operations
let orphans = storage.scan_orphans()?;
for orphan in orphans {
    // Handle incomplete operation
}
```

---

## Integration Pattern

### Write Operation with WAL
```rust
pub async fn write_with_wal(
    storage: &Storage,
    spill_store: &dyn SpillStore,
    execution_id: &str,
    items: &[Item],
) -> Result<SpilledList, Error> {
    // 1. Record intent (WAL)
    let intent_id = storage.record_intent(execution_id, "write_spill", None)?;
    
    // 2. Perform actual write
    let spilled_list = match spill_store.write(items).await {
        Ok(list) => list,
        Err(e) => {
            // Write failed — remove intent
            storage.complete_intent(intent_id)?;
            return Err(e.into());
        }
    };
    
    // 3. Complete intent
    storage.complete_intent(intent_id)?;
    
    Ok(spilled_list)
}
```

### Delete Operation with WAL
```rust
pub async fn delete_with_wal(
    storage: &Storage,
    spill_store: &dyn SpillStore,
    execution_id: &str,
    handle: &SpilledList,
) -> Result<(), Error> {
    // 1. Record intent
    let intent_id = storage.record_intent(
        execution_id,
        "delete_spill",
        Some(&handle.path.to_string())
    )?;
    
    // 2. Perform actual delete
    spill_store.delete(handle).await?;
    
    // 3. Complete intent
    storage.complete_intent(intent_id)?;
    
    Ok(())
}
```

### Crash Recovery
```rust
pub async fn recover_from_crash(
    storage: &Storage,
    spill_store: &dyn SpillStore,
) -> Result<(), Error> {
    let orphans = storage.scan_orphans()?;
    
    for orphan in orphans {
        match orphan.intent_type.as_str() {
            "write_spill" => {
                // Incomplete write — safe to retry
                // Or cleanup partial spill file if exists
                log::warn!("Incomplete write_spill: {:?}", orphan);
                storage.complete_intent(orphan.id)?;
            }
            "delete_spill" => {
                // Incomplete delete — retry delete
                if let Some(path) = &orphan.target_path {
                    let handle = SpilledList::from_path(path)?;
                    let _ = spill_store.delete(&handle).await;
                }
                storage.complete_intent(orphan.id)?;
            }
            _ => {
                // Other intents — handle as needed
                storage.complete_intent(orphan.id)?;
            }
        }
    }
    
    Ok(())
}
```

---

## Test Scenarios

### Scenario 1: Successful Write with WAL
1. record_intent("write_spill")
2. spill_store.write(items) → SpilledList
3. complete_intent()
4. Verify: spill file exists, intent removed

### Scenario 2: Write Failure with WAL
1. record_intent("write_spill")
2. spill_store.write(items) → ERROR
3. complete_intent() (cleanup)
4. Verify: no spill file, intent removed

### Scenario 3: Crash During Write
1. record_intent("write_spill")
2. [CRASH before write completes]
3. Recovery: scan_orphans() → finds intent
4. Verify: incomplete write detected, safe to retry

### Scenario 4: Delete with WAL
1. record_intent("delete_spill")
2. spill_store.delete(handle)
3. complete_intent()
4. Verify: spill file deleted, intent removed

### Scenario 5: Read Operations (No WAL needed)
1. spill_store.read_at(handle, index)
2. spill_store.read_range(handle, start, len)
3. spill_store.read_all(handle)
4. Verify: reads return correct items

---

## Recommendations for TB-02

1. **Use kernel SpillStore** for all spill operations
   - Import: `use kernel::item::{SpillStore, SpilledList};`
   - Use FileSpillStore from data_plane

2. **Keep storage spill_intent** as WAL (optional)
   - Provides crash safety
   - Low overhead (SQLite transactions)
   - Proven pattern (storage already uses it)

3. **Integration pattern**
   - WAL → SpillStore → WAL completion
   - Handle failures gracefully
   - Implement crash recovery

4. **Do NOT create separate spill mechanism**
   - Kernel contracts already exist
   - Avoid duplication
   - Maintain single source of truth

5. **Testing**
   - Test with and without WAL
   - Test crash recovery scenarios
   - Verify integration with kernel SpillStore

---

## Coordination

- **agent1 (TB-01):** Use integration pattern for Node Set implementation
- **agent4 (PREFACTOR-4):** Ensure Node trait compatible with SpillStore
- **agent6 (PREFACTOR-1):** Don't remove executor/scheduler crates (TB-01 needs them)
- **agent10 (KRITIKUS):** Verify integration with mutation testing

---

## References

- Kernel contracts: `/opt/agent-workspace/kernel-asli-d3bcff0/crates/kernel/src/item.rs`
- Storage WAL: `/opt/agent-workspace/rust-engine/crates/storage/migrations/001_initial_schema.sql`
- Data plane: `/opt/agent-workspace/kernel-asli-d3bcff0/crates/data-plane/`
- Task: matt #1664 PREFACTOR-3

---

**Status:** COMPLETE  
**Next:** TB-02 implementation using this guide
