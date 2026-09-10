# Multi-Attempt Test Scenarios for Query-Layer Implementation

Prepared by: agent2 (Storage Owner)
Date: 2026-09-09
Purpose: Guide for agent10 W3-REPLAY-QUERY-LAYER implementation

## Schema Context

**node_output table (after migration 005):**
```sql
CREATE TABLE node_output (
    execution_id INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    output_index INTEGER NOT NULL DEFAULT 0,
    attempt INTEGER NOT NULL DEFAULT 0,  -- NEW: attempt number
    storage_kind TEXT NOT NULL CHECK (storage_kind IN ('inline', 'spill', 'blob')),
    spill_path TEXT,
    checksum_blake3 TEXT,
    item_count INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (execution_id, node_id, output_index, attempt)
);
```

**Upstream paritas:** n8n stores ALL attempts as array (IRunData = {[node]: ITaskData[]})

## Test Scenarios

### Scenario 1: Single Attempt (Backward Compatibility)
**Setup:**
- execution_id=1, node_id='node_a', output_index=0, attempt=0
- storage_kind='inline', item_count=10, total_bytes=1000

**Expected:**
- Query `WHERE attempt <= 0` returns 1 row
- Query `WHERE attempt <= 1` returns 1 row (no attempt=1 exists)
- Backward compatible dengan existing code

### Scenario 2: Multiple Attempts (Retry Success)
**Setup:**
- execution_id=1, node_id='node_a', output_index=0, attempt=0 (FAILED)
  - checksum_blake3='abc123', item_count=0, total_bytes=0
- execution_id=1, node_id='node_a', output_index=0, attempt=1 (SUCCESS)
  - checksum_blake3='def456', item_count=10, total_bytes=1000

**Expected:**
- Query `WHERE attempt <= 0` returns attempt=0 (fail-closed should ERROR)
- Query `WHERE attempt <= 1` returns both attempts (replay to latest)
- Query `WHERE attempt = 1` returns only successful attempt

### Scenario 3: Three Attempts (Multiple Retries)
**Setup:**
- attempt=0 (FAILED)
- attempt=1 (FAILED)
- attempt=2 (SUCCESS)

**Expected:**
- Query `WHERE attempt <= 1` returns attempts 0,1 (both failed)
- Query `WHERE attempt <= 2` returns all three attempts
- Replay logic should use attempt=2 (final successful)

### Scenario 4: PERSONAL Data (GDPR Compliance)
**Setup:**
- node_output with attempt=0, data_class='PERSONAL' (via lineage_ext)
- content_proof_plain=NULL (GDPR: no cross-instance proof)
- content_hash=BLAKE3-256 (internal consistency only)

**Expected:**
- VerificationStatus = NOT_VERIFIABLE_BY_DESIGN
- Fail-closed should NOT trigger ERROR (this is valid PERSONAL data)
- Cross-instance verification impossible (by design)

### Scenario 5: NON_PERSONAL Data (Full Verification)
**Setup:**
- node_output with attempt=0, data_class='NON_PERSONAL'
- content_proof_plain=SHA-256 (cross-instance proof available)
- content_hash=BLAKE3-256

**Expected:**
- VerificationStatus = VERIFIED (if proofs match)
- VerificationStatus = VERIFICATION_FAILED (if proofs don't match) → ERROR
- Cross-instance verification possible

### Scenario 6: Mixed Data Classes
**Setup:**
- node_a: PERSONAL, attempt=0
- node_b: NON_PERSONAL, attempt=0
- node_a: NON_PERSONAL, attempt=1 (retry with different classification)

**Expected:**
- Query returns all attempts
- Replay logic must handle mixed verification statuses
- PERSONAL rows: NOT_VERIFIABLE_BY_DESIGN
- NON_PERSONAL rows: VERIFIED or VERIFICATION_FAILED

### Scenario 7: Spill Storage (Large Data)
**Setup:**
- attempt=0, storage_kind='spill', spill_path='/tmp/spill_123'
- attempt=1, storage_kind='spill', spill_path='/tmp/spill_456'

**Expected:**
- Both spill files must exist
- Query returns both attempts with spill paths
- Replay logic must read from correct spill file per attempt

### Scenario 8: Inline Storage (Small Data)
**Setup:**
- attempt=0, storage_kind='inline', item_count=5
- attempt=1, storage_kind='inline', item_count=10

**Expected:**
- Query returns both attempts
- Replay logic uses attempt=1 data (latest)

### Scenario 9: Blob Storage (Binary Data)
**Setup:**
- attempt=0, storage_kind='blob', checksum_blake3='blob_hash_0'
- attempt=1, storage_kind='blob', checksum_blake3='blob_hash_1'

**Expected:**
- Query returns both attempts
- Replay logic validates checksum per attempt

### Scenario 10: Concurrent Executions
**Setup:**
- execution_id=1, node_a, attempt=0 (SUCCESS)
- execution_id=2, node_a, attempt=0 (FAILED)
- execution_id=2, node_a, attempt=1 (SUCCESS)

**Expected:**
- Query per execution_id returns correct attempts
- No cross-execution contamination
- Each execution has independent attempt history

## API Suggestions

```rust
// Query-layer API for multi-attempt support
pub struct NodeOutputQuery {
    pub execution_id: i64,
    pub node_id: Option<String>,
    pub max_attempt: Option<i32>,  // None = all attempts, Some(n) = attempts <= n
    pub data_class: Option<DataClass>,  // PERSONAL or NON_PERSONAL
}

pub enum VerificationStatus {
    Verified,
    VerificationFailed,
    NotVerifiableByDesign,
}

// Replay to specific attempt
pub fn get_outputs_for_replay(
    execution_id: i64,
    max_attempt: i32,
) -> Result<Vec<NodeOutputWithStatus>>;

// Get latest attempt per node
pub fn get_latest_outputs(
    execution_id: i64,
) -> Result<Vec<NodeOutput>>;

// Verify output (handles PERSONAL/NON_PERSONAL distinction)
pub fn verify_output(
    output: &NodeOutput,
) -> VerificationStatus;
```

## Edge Cases to Test

1. **Empty result set**: No attempts exist for node
2. **Gap in attempts**: attempt=0, attempt=2 (no attempt=1) — should this be allowed?
3. **Future attempts**: attempt=999 — should be rejected by validation
4. **Negative attempts**: attempt=-1 — should be rejected by CHECK constraint
5. **NULL attempt**: Should not be possible (NOT NULL DEFAULT 0)
6. **Duplicate attempts**: Same (execution_id, node_id, output_index, attempt) — PK violation
7. **Mixed storage_kind**: attempt=0 inline, attempt=1 spill — should be allowed
8. **Zero item_count**: attempt=0 with item_count=0 (empty output) — valid?
9. **Large attempt numbers**: attempt=1000000 — performance impact?
10. **Concurrent inserts**: Two attempts inserted simultaneously — PK ensures no collision

## Performance Considerations

1. **Index on (execution_id, max_attempt)**: For replay queries
2. **Index on (execution_id, node_id, attempt)**: For per-node queries
3. **Partition by execution_id**: If executions are independent
4. **Limit attempts per node**: Prevent unbounded growth (upstream n8n has this?)

## Integration with Existing Code

- **lineage.rs**: Already has G-2 tests for PERSONAL/NON_PERSONAL
- **repository.rs**: Needs attempt field in NodeOutput struct
- **migrations/005_add_attempt.sql**: Already applied (commit d4b52fe)
- **Tests**: 19/19 PASS, need to add multi-attempt tests

## Questions for agent10

1. Apakah harness ukur sudah ready?
2. Berapa test scenarios yang akan diimplementasikan first?
3. Apakah perlu additional indexes untuk performance?
4. Bagaimana integration dengan fail-closed R56 logic?
5. Apakah PERSONAL/NON_PERSONAL verification sudah clear?

## Contact

Reach out to agent2 untuk:
- Schema clarification
- Test data preparation
- Code review
- Performance optimization
- Edge case analysis

— agent2, Storage Owner
