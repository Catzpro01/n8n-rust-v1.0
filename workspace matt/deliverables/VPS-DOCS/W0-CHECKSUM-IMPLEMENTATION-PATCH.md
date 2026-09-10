# W0-CHECKSUM-AGREE Implementation Patch

**Author:** agent3 (QA/Expression)
**Date:** 2026-09-09
**Status:** READY FOR REVIEW
**Requires:** agent2 (Storage) or matt (kernel owner) to apply

## Summary

This patch implements the dual-hash system agreed in W0-CHECKSUM-AGREE spec:
- **layer0_checksum** (SHA-256): Layer 0 integrity verification
- **casd_hash** (BLAKE3): CASD content deduplication

## Changes Required

### 1. kernel/crates/kernel/src/item.rs

Add two optional fields to `SpilledList` struct (after line 218):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpilledList {
    pub path: SpillPath,
    pub len: u32,
    pub total_bytes: u64,
    pub codec: SpillCodec,
    /// Layer 0 integrity checksum (SHA-256). Computed by data-plane on write.
    /// None for legacy spills or when integrity checking is disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer0_checksum: Option<[u8; 32]>,
    /// CASD content hash (BLAKE3). Used for content-addressed deduplication.
    /// None for legacy spills or when CASD is disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub casd_hash: Option<[u8; 32]>,
}
```

**Rationale:**
- `Option<[u8; 32]>` allows backward compatibility with existing spills
- `skip_serializing_if` keeps JSON compact for legacy spills
- Kernel doesn't compute hashes (no sha2/blake3 deps per D115)
- Data-plane implements hash computation on write

### 2. kernel/crates/data-plane/Cargo.toml

Add dependencies:

```toml
[dependencies]
sha2 = "0.10"
blake3 = "1.5"
```

### 3. kernel/crates/data-plane/src/spill.rs (FileSpillStore implementation)

Modify `write()` method to compute both hashes:

```rust
use sha2::{Sha256, Digest};
use blake3;

async fn write(&self, items: &[Item]) -> Result<SpilledList, KernelError> {
    // ... existing serialization code ...
    let serialized = serialize_items(items);
    
    // Compute Layer 0 integrity hash (SHA-256)
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(&serialized);
    let layer0_checksum: [u8; 32] = sha256_hasher.finalize().into();
    
    // Compute CASD content hash (BLAKE3)
    let mut blake3_hasher = blake3::Hasher::new();
    blake3_hasher.update(&serialized);
    let casd_hash: [u8; 32] = *blake3_hasher.finalize().as_bytes();
    
    // Write to disk
    let path = self.allocate_path();
    write_to_disk(&path, &serialized)?;
    
    Ok(SpilledList {
        path,
        len: items.len() as u32,
        total_bytes: serialized.len() as u64,
        codec: SpillCodec::Postcard,
        layer0_checksum: Some(layer0_checksum),
        casd_hash: Some(casd_hash),
    })
}
```

### 4. Add verify_integrity method to SpillStore trait

In `kernel/crates/kernel/src/item.rs` (SpillStore trait):

```rust
/// Verify spill integrity by recomputing checksum.
/// Returns Ok(true) if valid, Ok(false) if corrupted, Err if unsupported.
async fn verify_integrity(&self, handle: &SpilledList) -> Result<bool, KernelError> {
    // Default implementation for backward compatibility
    if handle.layer0_checksum.is_none() {
        return Err(KernelError::Unsupported("legacy spill"));
    }
    Ok(true) // Override in data-plane
}
```

Data-plane implementation:

```rust
async fn verify_integrity(&self, handle: &SpilledList) -> Result<bool, KernelError> {
    let expected = handle.layer0_checksum
        .ok_or(KernelError::Unsupported("legacy spill"))?;
    
    let data = read_from_disk(&handle.path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let actual: [u8; 32] = hasher.finalize().into();
    
    Ok(actual == expected)
}
```

### 5. Tests (kernel/crates/kernel/tests/checksum_test.rs)

```rust
#[tokio::test]
async fn test_dual_hash_computation() {
    let store = FileSpillStore::new(temp_dir());
    let items = vec![test_item()];
    
    let handle = store.write(&items).await.unwrap();
    
    // Both hashes should be computed
    assert!(handle.layer0_checksum.is_some());
    assert!(handle.casd_hash.is_some());
    
    // Hashes should be different (different algorithms)
    assert_ne!(
        handle.layer0_checksum.unwrap(),
        handle.casd_hash.unwrap()
    );
}

#[tokio::test]
async fn test_integrity_verification() {
    let store = FileSpillStore::new(temp_dir());
    let items = vec![test_item()];
    
    let handle = store.write(&items).await.unwrap();
    
    // Should verify successfully
    assert!(store.verify_integrity(&handle).await.unwrap());
    
    // Corrupt the file
    corrupt_file(&handle.path);
    
    // Should detect corruption
    assert!(!store.verify_integrity(&handle).await.unwrap());
}

#[tokio::test]
async fn test_casd_dedup() {
    let store = FileSpillStore::new(temp_dir());
    let items = vec![test_item()];
    
    // Write same content twice
    let handle1 = store.write(&items).await.unwrap();
    let handle2 = store.write(&items).await.unwrap();
    
    // CASD hashes should match (same content)
    assert_eq!(handle1.casd_hash, handle2.casd_hash);
    
    // But paths should be different (not yet deduplicated)
    // Dedup happens at CASD layer, not SpillStore
    assert_ne!(handle1.path, handle2.path);
}
```

## Backward Compatibility

- Existing spills without hashes: `layer0_checksum = None`, `casd_hash = None`
- `verify_integrity()` returns `Err(Unsupported)` for legacy spills
- CASD layer handles `None` gracefully (treats as unique content)

## Migration Path

1. Apply this patch to kernel canonical (a7d0357)
2. New spills automatically get both hashes
3. Existing spills continue to work (hashes are None)
4. Optional: background job to compute hashes for legacy spills

## Testing

- Unit tests: dual hash computation, integrity verification
- Integration tests: CASD dedup with BLAKE3 hashes
- Corruption tests: 1-byte corruption detected by SHA-256

## Acceptance Criteria

- [ ] SpilledList has layer0_checksum and casd_hash fields
- [ ] FileSpillStore.write() computes both hashes
- [ ] verify_integrity() method exists and works
- [ ] Tests pass (dual hash, integrity, dedup)
- [ ] Backward compatible with existing spills
- [ ] No new kernel dependencies (sha2/blake3 in data-plane only)

## Next Steps

1. **agent2 or matt**: Apply this patch to kernel canonical
2. **agent3**: Write integration tests with CASD layer
3. **agent5**: Security review (hash collision resistance)
4. **agent8**: Performance benchmark (hash computation overhead)

---

**Note:** This patch requires write access to kernel files. Please coordinate with matt (kernel owner) or request elevated permissions via swarm-admin.

---

## Update v1.1 (2026-09-09 07:58 UTC)

### Klausul Tinjau T-11 (per agent1 #735)

**Status:** INTERIM W0, bukan permanent commitment

**Review Trigger:** Setelah agent8 measurement T-11 (#660)

**Decision Matrix:**
| Scenario | Action | Migration |
|----------|--------|-----------|
| One algorithm >2x faster, same security | Migrate to single-hash | Deprecate slower hash field in W2 |
| Performance comparable (<2x difference) | Keep dual-hash | No change (different purposes) |
| Security concern with one algorithm | Replace vulnerable hash | Emergency migration path |

**Migration Path:**
1. Add new single-hash field: `content_hash: Option<[u8; 32]>`
2. Populate new field on write (use winning algorithm)
3. Read path: prefer new field, fallback to legacy dual-hash
4. After 30 days: deprecate old fields in schema
5. After 90 days: remove old fields (breaking change, requires ADR)

### Sequential Hash Implementation (per agent1 #735, R1/R2 #578)

**Requirement:** Compute hashes SEQUENTIALLY, not in parallel

**Rationale:** 
- Parallel: 2 hashers alive = ~2.5KB + buffer = exceeds 4KB budget
- Sequential: max(600B, 1920B) + buffer = ~6KB peak (acceptable for write-time)

**Implementation:**
```rust
// SEQUENTIAL - one hasher at a time
let serialized = serialize_items(items);

// Step 1: SHA-256 (compute, finalize, drop)
let mut sha256 = Sha256::new();
sha256.update(&serialized);
let layer0_checksum: [u8; 32] = sha256.finalize().into();
drop(sha256); // explicit drop for clarity

// Step 2: BLAKE3 (reuse buffer, compute, finalize, drop)
let mut blake3 = blake3::Hasher::new();
blake3.update(&serialized);
let casd_hash: [u8; 32] = *blake3.finalize().as_bytes();
drop(blake3);

// Peak RAM: max(sha256_size, blake3_size) + buffer
// = max(600B, 1920B) + 4KB = ~6KB
```

**Error Codes:**
- `E_SPILL_CORRUPT` → Layer 0 integrity failure (SHA-256 mismatch)
- `E_CASD_DEDUP_ERROR` → CASD logic error (BLAKE3 mismatch, not corruption)

### Performance Measurement Request (agent8)

**Baseline needed:**
- 1000 spills, sizes: 1KB, 10KB, 100KB, 1MB
- Metrics: hash time (ms), peak RAM (KB), throughput (MB/s)
- Compare: SHA-256 only vs BLAKE3 only vs dual-hash (sequential)

**Acceptance Criteria:**
- Dual-hash overhead < 20% vs single-hash
- Peak RAM < 8KB (measured, not estimated)
- Throughput impact < 10% for typical spill (10KB)

---

**Reviewers:** agent1 (engine), agent5 (security), agent8 (performance)
**Implementer:** agent2 (storage) or agent3 (if granted write access)
**Approver:** matt (kernel owner)

---

## Update v1.2 (2026-09-09 07:59 UTC)

### Streaming Hash Implementation (per agent1 #737)

**Problem:** 
- Sequential hash with full buffer = 6KB peak (50% over 4KB budget)
- Budget 4KB is LOCKED (fern #431), not flexible

**Solution:**
- Stream hash updates during serialization, no full buffer
- Peak RAM = max(hasher_sizes) + chunk_buffer = 2.2KB < 4KB ✓
- Consistent with host-streaming philosophy (#470)

**Corrected Measurements (agent10):**
- SHA-256 hasher: **112 bytes** (not ~600B assumption)
- BLAKE3 hasher: 1920 bytes (measured)
- Chunk buffer: 256 bytes (fixed)
- **Total peak: 2288 bytes ≈ 2.2KB < 4KB ✓**

**Implementation:**
```rust
// STREAMING HASH - compute during serialization, no full buffer
pub async fn write(&self, items: &[Item]) -> Result<SpilledList, KernelError> {
    let path = self.allocate_path();
    let mut file = File::create(&path).await?;
    
    // Initialize both hashers
    let mut sha256 = Sha256::new();
    let mut blake3 = blake3::Hasher::new();
    
    // Stream serialization with hash updates
    let mut writer = ChunkedHashWriter::new(&mut sha256, &mut blake3, &mut file);
    let total_bytes = serialize_items_streaming(items, &mut writer, CHUNK_SIZE)?;
    
    // Finalize hashes
    let layer0_checksum: [u8; 32] = sha256.finalize().into();
    let casd_hash: [u8; 32] = *blake3.finalize().as_bytes();
    
    Ok(SpilledList {
        path,
        len: items.len() as u32,
        total_bytes,
        codec: SpillCodec::Postcard,
        layer0_checksum: Some(layer0_checksum),
        casd_hash: Some(casd_hash),
    })
}

struct ChunkedHashWriter<'"'"'a> {
    sha256: &'"'"'a mut Sha256,
    blake3: &'"'"'a mut blake3::Hasher,
    file: &'"'"'a mut File,
}

impl<'"'"'a> ChunkedHashWriter<'"'"'a> {
    fn new(sha256: &'"'"'a mut Sha256, blake3: &'"'"'a mut blake3::Hasher, file: &'"'"'a mut File) -> Self {
        Self { sha256, blake3, file }
    }
    
    fn write_chunk(&mut self, chunk: &[u8]) -> io::Result<()> {
        // Update both hashers with this chunk (no extra buffer)
        self.sha256.update(chunk);
        self.blake3.update(chunk);
        
        // Write to disk immediately
        self.file.write_all(chunk)?;
        Ok(())
    }
}

const CHUNK_SIZE: usize = 256;  // small chunk, keeps RAM low
```

**Peak RAM Breakdown:**
```
SHA-256 hasher state:     112 bytes  (measured by agent10)
BLAKE3 hasher state:     1920 bytes  (measured by agent10)
Chunk buffer:             256 bytes  (fixed CHUNK_SIZE)
File handle:              ~100 bytes  (OS overhead)
─────────────────────────────────────
Total peak:              2388 bytes ≈ 2.3KB < 4KB ✓
```

**Key Insight:**
- Peak RAM is INDEPENDENT of spill size
- 1KB spill and 1MB spill both use 2.3KB peak
- Streaming = O(1) memory, not O(n)

**Performance Trade-off:**
- Streaming: slightly slower (more syscalls) but RAM-compliant
- Buffered: faster but violates budget
- Decision: RAM compliance > performance (budget is locked)

**Agent8 Measurement Request (updated):**
- Compare: streaming (256B chunks) vs buffered (full serialization)
- Metrics: throughput (MB/s), peak RAM, syscall count
- Expected: streaming 10-20% slower but 2.3KB vs 6KB peak

---

**Reviewers:** agent1 (engine, budget compliance), agent5 (security), agent8 (performance)
**Implementer:** agent2 (storage) or agent3 (if granted write access)
**Approver:** matt (kernel owner)
