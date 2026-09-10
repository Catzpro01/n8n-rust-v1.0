# Rosetta Normalizer Integration Test Plan

**Author**: agent3 (Co-Maintainer rosetta)
**Date**: 2026-09-09
**Status**: DRAFT v0.1
**Purpose**: Integration test plan for N-08/09/22 dengan Rosetta output

## 1. BACKGROUND

### Division of Responsibility (from #1083/#1106)
- **Rosetta crate (agent4)**: N-01..07/11 + N-20 workflow-level
- **Execution harness (agent1/agent3)**: N-06/08/09/10/12/21/22

### Test Objectives
1. Verify determinisme lintas eksekusi (2x run produces identical digest)
2. Verify cache hit/miss patterns untuk EBC
3. Verify error handling edge cases
4. Verify integration dengan Rosetta output format

## 2. TEST SCENARIOS

### 2.1 N-08 (Expression Normalization)

**Test Cases:**
```
N08-01: Simple expression ($json.name)
N08-02: Complex nested expression ($json.data.items[0].value)
N08-03: Expression with operators ($json.a + $json.b)
N08-04: Expression with function calls ($now.toISO())
N08-05: Multi-line expression
N08-06: Expression with escaped characters
N08-07: Empty expression (edge case)
N08-08: Expression with special characters (unicode)
```

**Expected Output:**
- All expressions normalized to canonical form
- Digest stable across multiple runs
- Cache key consistent untuk identical expressions

### 2.2 N-09 (Code Block Normalization)

**Test Cases:**
```
N09-01: Simple JavaScript code block
N09-02: Code with comments (should be stripped)
N09-03: Code with varying whitespace (should normalize)
N09-04: Code with different line endings (CRLF vs LF)
N09-05: Code with import statements
N09-06: Code with async/await
N09-07: Empty code block (edge case)
N09-08: Code with template literals
N09-09: Minified code (already compact)
N09-10: Code with dead code (unreachable branches)
```

**Expected Output:**
- Comments stripped
- Whitespace normalized
- Line endings normalized to LF
- Digest stable across runs
- Semantically equivalent code produces same digest

### 2.3 N-22 (Parameter Schema Normalization)

**Test Cases:**
```
N22-01: Simple parameter schema (string type)
N22-02: Complex nested schema (object with arrays)
N22-03: Schema with union types (multi-key)
N22-04: Schema with optional fields
N22-05: Schema with default values
N22-06: Schema with constraints (min/max, pattern)
N22-07: Empty schema (edge case)
N22-08: Schema with references
N22-09: Schema with additionalProperties
N22-10: Schema with conditional logic (if/then/else)
```

**Expected Output:**
- Keys sorted (BTreeMap)
- Optional fields handled consistently
- Default values normalized
- Digest stable across runs
- Equivalent schemas produce same digest

## 3. INTEGRATION TEST MATRIX

### 3.1 Rosetta Output Format
```json
{
  "version": 1,
  "node_type": "n8n-nodes-base.code",
  "param_schemas": {},
  "expressions": [],
  "code_blocks": []
}
```

### 3.2 Test Execution Flow
```
1. Input: Rosetta output (JSON)
2. Apply N-08 (expression normalization)
3. Apply N-09 (code block normalization)
4. Apply N-22 (parameter schema normalization)
5. Compute digest (sha256 with u32-BE version framing)
6. Verify: digest stable across 2 runs
7. Verify: cache hit/miss patterns
```

### 3.3 Determinism Verification
```rust
// Test: 2x run produces identical digest
let run1 = normalize_and_digest(rosetta_output);
let run2 = normalize_and_digest(rosetta_output);
assert_eq!(run1, run2, "Digest harus deterministik");
```

### 3.4 Cache Hit/Miss Patterns
```rust
// Test: EBC cache behavior
let cache = ExpressionBytecodeCache::new();

// First access: cache miss
let result1 = cache.get_or_compute(expression, || normalize(expression));
assert!(cache.miss_count() == 1);

// Second access: cache hit
let result2 = cache.get_or_compute(expression, || normalize(expression));
assert!(cache.hit_count() == 1);
assert_eq!(result1, result2);
```

## 4. EDGE CASES

### 4.1 Error Handling
```
EDGE-01: Malformed JSON input
EDGE-02: Missing required fields
EDGE-03: Invalid expression syntax
EDGE-04: Invalid code syntax
EDGE-05: Circular references in schema
EDGE-06: Extremely large input (performance)
EDGE-07: Unicode normalization edge cases
EDGE-08: Timezone handling (ISO-8601 to EPOCH:ms)
```

### 4.2 Performance Considerations
```
PERF-01: Large workflow (more than 1000 nodes)
PERF-02: Deep nesting (more than 10 levels)
PERF-03: Many expressions (more than 100 per node)
PERF-04: Large code blocks (more than 10KB)
PERF-05: Cache size limits
```

## 5. TEST DATA

### 5.1 Sample Rosetta Outputs
```
testdata/
  simple_workflow.json
  complex_workflow.json
  code_heavy_workflow.json
  expression_heavy_workflow.json
  edge_cases/
    empty_workflow.json
    malformed.json
    unicode_heavy.json
  performance/
    large_workflow.json
    deep_nesting.json
```

### 5.2 Expected Digests
```json
{
  "simple_workflow.json": "sha256:abc123...",
  "complex_workflow.json": "sha256:def456...",
  "code_heavy_workflow.json": "sha256:ghi789..."
}
```

## 6. IMPLEMENTATION PLAN

### Phase 1: Test Infrastructure (1 day)
1. Create test harness
2. Set up test data directory
3. Implement digest comparison utilities
4. Implement cache simulation

### Phase 2: Unit Tests (2 days)
1. N-08 tests (8 test cases)
2. N-09 tests (10 test cases)
3. N-22 tests (10 test cases)
4. Edge case tests (8 test cases)

### Phase 3: Integration Tests (1 day)
1. End-to-end flow tests
2. Determinism verification
3. Cache hit/miss verification
4. Performance benchmarks

### Phase 4: Documentation (0.5 day)
1. Test report
2. Coverage metrics
3. Performance results
4. Known issues

## 7. SUCCESS CRITERIA

- All 36 unit tests PASS
- All 4 integration tests PASS
- Determinism verified (2x run = identical digest)
- Cache behavior verified (hit/miss patterns correct)
- Edge cases handled gracefully (no panic, clear error messages)
- Performance acceptable (less than 1s for 100-node workflow)

## 8. COORDINATION

### Dependencies
- agent4: Rosetta output format stable
- agent1: Execution harness integration
- agent3: Test implementation + execution

### Timeline
- Phase 1: 1 day (infrastructure)
- Phase 2: 2 days (unit tests)
- Phase 3: 1 day (integration tests)
- Phase 4: 0.5 day (documentation)
- **Total**: 4.5 days

### Deliverables
1. Test harness code
2. Test data suite (sample workflows)
3. Test report (pass/fail, coverage, performance)
4. Integration guide (how to use tests)

## 9. NEXT STEPS

1. Review: agent4 + agent1 review test plan
2. Approve: matt approve test plan
3. Execute: agent3 implement + run tests
4. Report: Share results dengan team
5. Iterate: Fix issues found during testing

---

**Questions for Review:**
1. Are there additional test scenarios we should cover?
2. What is the expected timeline for Rosetta output format stability?
3. Should we include regression tests for known bugs?
4. What is the acceptable performance threshold?

**Contact**: agent3 (Co-Maintainer rosetta)
