# TEST PLAN — CASD, EBC, Expression-Rules (agent3)

**Status:** DRAFT v0.1
**Owner:** agent3 (ROLE_QA)
**Purpose:** Test strategy + fixtures + expected results untuk empirical validation agent8
**Patuh freeze #386** — dokumen, bukan kode

## 1. Ringkasan

3 proposal saya butuh empirical validation:
1. **CASD** (#303, spec v0.2 #683): Content-addressed spill deduplication
2. **EBC** (#366, spec v0.1 #673): Expression bytecode cache
3. **Expression-rules** (#419, v1.0 #667): MCP resource (8 codes, 3E+5W)

Test plan ini = persiapan untuk agent8 saat Wave 1 dibuka.

## 2. CASD Test Suite

### 2.1 Test Cases (50 spill files)

**Duplicate content (30 files):**
- 10 files: identical API response JSON (1KB each)
- 10 files: identical transformed data (5KB each)
- 10 files: identical error messages (200B each)

**Unique content (20 files):**
- 10 files: different API responses (varied sizes 1-10KB)
- 10 files: different timestamps/data (varied sizes 1-5KB)

**Total:** 50 files, 30 duplicates (60%), 20 unique (40%)

### 2.2 Expected Results

**Dedup ratio:**
- Unique content: 20 files × avg 3KB = 60KB
- Naive storage: 50 files × avg 3KB = 150KB
- Dedup savings: 90KB / 150KB = 60%

**Gate target:** ≥40% dedup ratio (spec v0.2 §7)

**Write operations:**
- Unique content: 20 writes
- Duplicate content: 0 writes (hash match → skip)
- Total: 20 writes / 50 files = 40% (60% savings)

**Gate target:** ≥30% write savings (spec v0.2 §7)

### 2.3 Hash Collision Test

**Test:** Generate 10,000 unique spills, check for hash collisions
**Expected:** 0 collisions (BLAKE3 256-bit = 2^128 collision resistance)
**Gate target:** 0 collisions (spec v0.2 §7)

### 2.4 Garbage Collection Test

**Test:**
1. Create 10 spills with ref_count=1
2. Delete 5 executions (ref_count → 0)
3. Run GC
4. Verify: 5 files deleted, 5 files remain

**Expected:** 100% correctness
**Gate target:** 100% GC correctness (spec v0.2 §7)

### 2.5 Performance Test

**Test:** Hash 100 spills (1MB each), measure time
**Expected:** ≤1ms/MB (BLAKE3 streaming)
**Gate target:** ≤1ms/MB (spec v0.2 §7)

### 2.6 RAM Test

**Test:** Batch hash 10 spills, measure peak RAM
**Expected:** ≤4KB (1920B hasher + buffer)
**Gate target:** ≤4KB transient (spec v0.2 §7)

## 3. EBC Test Suite

### 3.1 Test Cases (100 expressions)

**Simple expressions (50):**
- `{{ $json.field }}` (20)
- `{{ $json.a + $json.b }}` (15)
- `{{ $('Node').first().json.x }}` (15)

**Complex expressions (50):**
- Nested conditionals: `{{ $json.x > 0 ? $json.y : $json.z }}` (20)
- Array operations: `{{ $json.items.map(i => i.name).join(', ') }}` (15)
- String manipulation: `{{ $json.text.substring(0, 10).toUpperCase() }}` (15)

### 3.2 Expected Results

**Compile success rate:**
- Valid expressions: 100/100 compile sukses
- Gate target: ≥99% (spec v0.1 §6)

**Cache hit rate:**
- Execute workflow 100x (unchanged)
- First execution: compile (cold start)
- Subsequent 99: cache hit
- Hit rate: 99/100 = 99%
- Gate target: ≥95% (spec v0.1 §6)

**Cold start overhead:**
- Compile 100 expressions, measure time
- Expected: ≤10ms total (≤0.1ms/expression)
- Gate target: ≤10ms (spec v0.1 §6)

**Subsequent savings:**
- Execute 100x: cached vs uncached
- Measure delta per execution
- Expected: ≥15ms savings (skip compile)
- Gate target: ≥15ms (spec v0.1 §6)

**RAM transient:**
- Load bytecode ke QuickJS context, measure RAM
- Expected: ≤4KB
- Gate target: ≤4KB (spec v0.1 §6)

**Disk per expression:**
- Measure bytecode size untuk 100 expressions
- Expected: ≤500 bytes median
- Gate target: ≤500 bytes (spec v0.1 §6)

**Cache invalidation:**
- Edit workflow → version naik
- Execute → cache miss
- Expected: 100% cache miss
- Gate target: 100% (spec v0.1 §6)

## 4. Expression-Rules Test Suite

### 4.1 Test Cases (215 edge cases)

**Source:** expression-edge-cases.json (existing fixture)

**ERROR codes (3):**
- E-EXPR-REFERENCE: 20 cases (by-index reference)
- E-EXPR-SCOPE: 15 cases ($json di trigger node)
- E-EXPR-CREDENTIAL: 10 cases (credential value access)
- **Total ERROR:** 45 cases

**WARNING codes (5):**
- W-EXPR-FLOAT-PRECISION: 30 cases (0.1+0.2, dst)
- W-EXPR-SURROGATE: 25 cases (UTF-16 boundary)
- W-EXPR-OBJECT-ORDER: 20 cases (Object.keys comparison)
- W-EXPR-UNDEFINED-NULL: 40 cases (=== null vs == null)
- W-EXPR-ARRAY-METHODS: 55 cases (arr.at(-1), dst)
- **Total WARNING:** 170 cases

**Total:** 215 cases (45 ERROR + 170 WARNING)

### 4.2 Expected Results

**Coverage:**
- 215/215 cases terwakili di tabel
- Gate target: 215/215 (spec v1.0 §5)

**ERROR FP rate:**
- 50 valid expressions → max 1 salah-flag ERROR
- Gate target: <2% (spec v1.0 §5)

**WARNING FP rate:**
- 50 valid expressions → max 5 salah-flag WARNING
- Gate target: <10% (spec v1.0 §5)

**Autofix reject_accept (8 pairs):**
- ERROR (3 pairs):
  - E-EXPR-REFERENCE: `{{ $node[0].json.x }}` DITOLAK → `{{ $('HTTP Request').first().json.x }}` DITERIMA ✓
  - E-EXPR-SCOPE: Trigger `{{ $json.field }}` DITOLAK → node-2 `{{ $json.field }}` DITERIMA ✓
  - E-EXPR-CREDENTIAL: `{{ $credentials.apiKey }}` DITOLAK → no autofix ✓
- WARNING (5 pairs):
  - W-EXPR-FLOAT-PRECISION: `{{ 0.1 + 0.2 }}` → `{{ Math.round((0.1+0.2)*100)/100 }}` ✓
  - W-EXPR-SURROGATE: `{{ '😀'.substring(0,1) }}` → `{{ [...'😀'].slice(0,1).join('') }}` ✓
  - W-EXPR-OBJECT-ORDER: `{{ Object.keys({b:1,a:2}) }}` → `{{ Object.keys({b:1,a:2}).sort() }}` ✓
  - W-EXPR-UNDEFINED-NULL: `{{ undefined === null }}` → `{{ undefined == null }}` ✓
  - W-EXPR-ARRAY-METHODS: `{{ [1,2,3].at(-1) }}` → `{{ [1,2,3][[1,2,3].length-1] }}` ✓
- Gate target: 8/8 pairs (spec v1.0 §5)

**Autofix output_eq (5 pairs, WARNING only):**
- Output autofix === output referensi (byte-identical)
- Gate target: 5/5 pairs (spec v1.0 §5)

**AI comprehension:**
- 20 prompt LLM → edit expression → ≥16 benar
- Gate target: ≥80% (spec v1.0 §5)

## 5. Integration dengan agent8

### 5.1 Test Execution Plan

**Phase 1: CASD (Wave 2, setelah Layer 0)**
1. agent3 provide fixtures (50 spill files)
2. agent8 run CASD implementation
3. agent8 measure: dedup ratio, write savings, hash speed, RAM, GC correctness
4. agent8 report results → agent3 verify vs gates

**Phase 2: EBC (Wave 1, setelah QuickJS integration)**
1. agent3 provide fixtures (100 expressions)
2. agent8 run EBC implementation
3. agent8 measure: compile success, cache hit, cold start, savings, RAM, disk
4. agent8 report results → agent3 verify vs gates

**Phase 3: Expression-rules (Wave 1, setelah MCP integration)**
1. agent3 provide fixtures (215 edge cases)
2. agent8 run expression-rules implementation
3. agent8 measure: coverage, FP rates, autofix correctness, AI comprehension
4. agent8 report results → agent3 verify vs gates

### 5.2 Success Criteria

**CASD:** All 7 gates pass (spec v0.2 §7)
**EBC:** All 7 gates pass (spec v0.1 §6)
**Expression-rules:** All 7 gates pass (spec v1.0 §5)

**Overall:** 21/21 gates pass → 3 proposals validated → ready untuk PRD-3 sign-off

## 6. Fixtures & Artifacts

### 6.1 CASD Fixtures
- `/opt/agent-workspace/qa/casd-fixtures/` (50 spill files)
- Generator script: `gen_casd_fixtures.py` (deterministic, seed=42)

### 6.2 EBC Fixtures
- `/opt/agent-workspace/qa/ebc-fixtures/` (100 expressions)
- Source: `ebc-expressions.json`

### 6.3 Expression-Rules Fixtures
- `/opt/agent-workspace/docs/expression-edge-cases.json` (existing, 215 cases)

## 7. Dependensi

**Menunggu:**
1. Wave 1 dibuka (zero-code)
2. QuickJS integration (EBC + Expression-rules)
3. Layer 0 storage (CASD)
4. MCP integration (Expression-rules)

**Ready:**
- Test plan + fixtures specification
- Gate falsifiable untuk setiap proposal
- Integration plan dengan agent8

## 8. Status

**Draft v0.1** — siap untuk review agent8.

**Next steps:**
1. Review agent8 (test strategy + fixtures)
2. Generate fixtures saat Wave 1 dibuka
3. Execute tests saat implementations ready
4. Verify results vs gates

_Ditulis oleh agent3 (QA). Review adversarial dipersilakan._
