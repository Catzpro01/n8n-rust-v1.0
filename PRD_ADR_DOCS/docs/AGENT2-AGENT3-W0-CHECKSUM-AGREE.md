**CODE TREE:** Patch ini menargetkan KANONIK tree (/opt/agent-workspace/kernel-asli-d3bcff0, commit a7d0357), BUKAN stub rust-engine. data-plane di kanonik = 0 .rs files (Cargo.toml only), jadi implementasi = CREATE baru, bukan MODIFY existing.

# W0-CHECKSUM-AGREE Specification (agent2 + agent3)

**Status:** DRAFT v0.1
**Owners:** agent2 (Layer 0) + agent3 (CASD)
**Wave:** 0 (prerequisite untuk W2-STORAGE-L0)
**Patuh freeze #386** — dokumen, bukan kode

## 1. Ringkasan

Agreement antara Layer 0 (agent2) dan CASD (agent3) tentang:
- Kapan pakai SHA-256
- Kapan pakai BLAKE3
- Boundary yang jelas antara keduanya
- Contract untuk Envelope derivation (agent10)

## 2. Use Cases

### 2.1 Layer 0 Spill Integrity (SHA-256)
**Purpose:** Detect corruption/tampering of spill files at rest
**Algorithm:** SHA-256 (existing data-plane implementation)
**Scope:** Per-spill file checksum
**When:** Write time (compute hash, store di metadata)
**Verify:** Read time (recompute hash, compare)

**Why SHA-256:**
- Existing implementation di data-plane (agent2)
- Standard untuk integrity verification
- Tidak butuh speed (one-time compute per spill)
- 32-byte hash (acceptable overhead)

### 2.2 CASD Content Deduplication (BLAKE3)
**Purpose:** Identify identical content across spills untuk dedup
**Algorithm:** BLAKE3 (new implementation)
**Scope:** Content-addressed hash (same content = same hash)
**When:** Write time (compute hash, check index, skip if exists)
**Verify:** Read time (hash is key, no verification needed)

**Why BLAKE3:**
- 4x faster dari SHA-256 (important untuk large spills)
- Streaming API (incremental update tanpa buffer entire content)
- 32-byte hash (same size as SHA-256)
- 1920 bytes transient RAM (acceptable, agent10 measurement)

### 2.3 Envelope Derivation (BLAKE3 derive_key)
**Purpose:** Derive cryptographic material untuk Execution Envelope (per EXEC-ENVELOPE-SPEC agent10 §3, §6.2)
**Algorithm:** BLAKE3 derive_key (agent10 spec)
**Scope:** Key derivation (bukan content hashing)
**When:** Execution start (derivation details per agent10 EXEC-ENVELOPE-SPEC §6.2: chain_key via secret-manager + chain_key_id per segmen; salt_exec per-eksekusi)
**Verify:** N/A (key derivation, not verification)

**Why BLAKE3 derive_key:**
- KDF (Key Derivation Function) built-in
- Deterministic (same input = same output)
- Fast (needed per-execution)
- 32-byte output (suitable for chain_key)

## 3. Boundary Definition

```
┌─────────────────────────────────────────────────────────┐
│                    Spill File                           │
│                                                         │
│  Content: [actual data bytes]                          │
│                                                         │
│  Metadata:                                              │
│    - layer0_checksum: SHA-256(content)  ← integrity    │
│    - casd_hash: BLAKE3(content)         ← dedup        │
│                                                         │
└─────────────────────────────────────────────────────────┘
         ↓                              ↓
    Layer 0                         CASD
    (verify corruption)             (dedup by content)
```

**Key Insight:**
- Layer 0 checksum = **integrity** (detect corruption)
- CASD hash = **identity** (dedup identical content)
- Both computed at write time, stored di metadata
- Different algorithms karena different use cases

## 4. Contract

### 4.1 SpillStore Trait (Layer 0)
```rust
pub trait SpillStore {
    fn write_spill(&self, content: &[u8]) -> Result<SpillRef>;
    fn read_spill(&self, spill_ref: &SpillRef) -> Result<Vec<u8>>;
    fn verify_integrity(&self, spill_ref: &SpillRef) -> Result<bool>;
}

pub struct SpillRef {
    pub spill_id: u64,
    pub layer0_checksum: [u8; 32],  // SHA-256
    pub casd_hash: [u8; 32],        // BLAKE3
}
```

### 4.2 CASD Integration
```rust
pub struct CASD {
    store: Arc<dyn SpillStore>,
    index: HashMap<[u8; 32], SpillRef>,  // casd_hash -> spill_ref
}

impl CASD {
    pub fn dedup_write(&mut self, content: &[u8]) -> Result<SpillRef> {
        let casd_hash = blake3::hash(content);
        
        // Check if content already exists
        if let Some(&spill_ref) = self.index.get(&casd_hash) {
            // Content exists, return existing ref (skip write)
            return Ok(spill_ref);
        }
        
        // Content new, write to store (Layer 0 computes SHA-256)
        let spill_ref = self.store.write_spill(content)?;
        
        // Verify CASD hash matches (sanity check)
        assert_eq!(spill_ref.casd_hash, casd_hash);
        
        // Add to index
        self.index.insert(casd_hash, spill_ref);
        Ok(spill_ref)
    }
}
```

### 4.3 Envelope Integration
```rust
pub struct ExecutionEnvelope {
    chain_key: [u8; 32],  // BLAKE3 derive_key output
}

impl ExecutionEnvelope {
    pub fn new(execution_id: u64, secret: &[u8]) -> Self {
        let chain_key = blake3::derive_key(
            b"execution-envelope-v1",
            &[execution_id.to_le_bytes().as_ref(), secret].concat()
        );
        Self { chain_key }
    }
}
```

## 5. T-11 Decision (BLAKE3 vs SHA-256)

**Current Status:** T-11 belum diputus pemilik proyek

**Proposal agent2+agent3:**
- Layer 0 integrity = SHA-256 (existing, standard)
- CASD dedup = BLAKE3 (fast, streaming)
- Envelope derivation = BLAKE3 derive_key (KDF)

**Rationale:**
- Different use cases butuh different algorithms
- SHA-256 untuk integrity (standard, well-understood)
- BLAKE3 untuk dedup/derivation (fast, modern)
- Tidak ada conflict karena different purposes

**Fallback (jika T-11 = SHA-256 only):**
- CASD bisa pakai SHA-256 (slower, tapi work)
- Envelope derivation pakai HKDF-SHA256 (standard KDF)
- Performance penalty: ~4x slower untuk CASD

## 6. Test Plan

### 6.1 W0-CHECKSUM-AGREE Tests
1. **Boundary test:** Verify Layer 0 checksum ≠ CASD hash (different algorithms)
2. **Integration test:** CASD dedup_write dengan Layer 0 integrity verification
3. **Performance test:** BLAKE3 vs SHA-256 speed comparison (100 spills, 1MB each)

### 6.2 W0-Spill-TEST Tests (agent3+agent5)
1. **Integrity test:** Corrupt 1 byte → SHA-256 mismatch detected
2. **Dedup test:** 50 spills (30 duplicate, 20 unique) → dedup ratio ≥40%
3. **UMask test:** umask 000 → spill files mode 0600 (agent5 SEC-07)

## 7. Dependencies

**Menunggu:**
1. Wave 0 ditambahkan ke task_queue (prio P0)
2. K-1 decision (kernel canonical)
3. T-11 decision (BLAKE3 vs SHA-256)
4. Zero-code dibuka untuk Wave 0

**Ready:**
- Spec v0.1 (dokumen ini)
- Contract definition (SpillStore trait)
- Test plan (W0-Spill-TEST fixtures)

## 8. Gates (falsifiable)

| Gate | Target | Cara Ukur |
|---|---|---|
| Boundary clarity | 100% | Layer 0 checksum ≠ CASD hash untuk semua spills |
| Integration | 100% | CASD dedup_write + Layer 0 verify = success |
| BLAKE3 speed | ≥3x SHA-256 | 100 spills 1MB each, measure time |
| Integrity | 100% | Corrupt 1 byte → SHA-256 mismatch detected |
| Dedup ratio | ≥40% | 50 spills (30 dup, 20 unique) → disk savings ≥40% |
| UMask | 100% | umask 000 → spill files mode 0600 |

## 9. Status

**Draft v0.1** — siap untuk review adversarial.

**Next steps:**
1. Review agent5 (security perspective)
2. Review agent10 (Envelope integration)
3. Tunggu Wave 0 di queue + K-1 + T-11 decisions
4. Implementasi setelah zero-code dibuka

_Ditulis oleh agent2 (Layer 0) + agent3 (CASD). Review adversarial dipersilakan._

---

## Correction v0.1.1 (2026-09-09 08:07 UTC, per agent10 #763)

### C-6 Fix: Remove Merkle Tree Reference

**Original (line 45):** "Derive chain_key untuk Merkle tree di Execution Envelope"

**Corrected:** "Derive cryptographic material untuk Execution Envelope (per EXEC-ENVELOPE-SPEC agent10 §3, §6.2)"

**Rationale:** 
- Merkle root REVOKED (#555, rekonsiliasi #628 butir 6)
- envelope_root = chain head h_{n-1}, BUKAN Merkle root
- C-03 exists karena Merkle root TIDAK MEMBERI URUTAN, sedangkan Envelope terletak pada urutan
- Konstruksi crypto Envelope = agent10 domain (P3-03, diputus matt)

**chain_key Clarification (line 48):**
- chain_key via secret-manager + chain_key_id per segmen (agent10 §6.2)
- salt_exec = per-eksekusi material
- W0-CHECKSUM-AGREE TIDAK menentapkan konstruksi Envelope, hanya menyediakan BLAKE3 derive_key primitive

### C-5 Fix: Clarify Code Tree Target

**Issue:** Dokumen ini menyebut "data-plane" tanpa specify pohon mana (stub rust-engine vs kanonik kernel-asli-d3bcff0).

**Clarification:**
- **Target:** KANONIK tree (/opt/agent-workspace/kernel-asli-d3bcff0, commit a7d0357)
- **data-plane di kanonik:** Cargo.toml ada, 0 .rs files (per agent10 §7.2 verification)
- **Implikasi:** Implementasi = CREATE crates/data-plane/src/lib.rs (baru), bukan MODIFY existing
- **testkit:** TIDAK ADA di kanonik (hanya di stub rust-engine). W0-HASH-FIX perlu CREATE crates/testkit/ di kanonik.

**Risiko yang dihindari:**
- Agent3 memperbaiki testkit di stub (akan dibuang) → gate tetap merah di kanonik
- Patch diterapkan ke pohon yang salah → wasted effort

**Referensi:**
- agent10 PRD-3 Alignment §3 (C-5): /opt/agent-workspace/docs/AGENT10-PRD3-ALIGNMENT.md
- fern dekrit #726: K-1 = kanonik a7d0357

---

**Reviewer:** agent10 (compliance), agent1 (engine), agent5 (security)
**Status:** v0.1.1 - Merkle reference removed, code tree clarified
