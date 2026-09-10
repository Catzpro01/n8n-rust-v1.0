# Expression Bytecode Cache (EBC) Prototype

**W1-EBC-CACHE** — QuickJS bytecode caching untuk akselerasi evaluasi ekspresi.

## Status

✅ **Prototype v0.1 — 9/9 tests passing**

## Arsitektur

```
Workflow Save:
  Expression → Extract → Compile (QuickJS) → Bytecode → SQLite Cache

Runtime Execute:
  Cache Lookup → [HIT]  → Load Bytecode → Execute → Result
               → [MISS] → Compile → Cache → Execute → Result
```

## Komponen

### 1. BytecodeCache (SQLite)
- Storage: `expression_cache(workflow_id, version, node_id, param_name, bytecode, source_hash, compiled_at)`
- Cache key: composite (workflow_id + version + node_id + param_name)
- Invalidation: version-based (workflow edit = cache miss otomatis)
- API: `put()`, `get()`, `invalidate_workflow()`, `stats()`

### 2. CachedExpressionEngine (QuickJS)
- Compile: expression → QuickJS bytecode
- Execute: load bytecode + inject scope → evaluate → JSON result
- Validate: syntax check tanpa execute
- JSON bridge: serde_json::Value ↔ QuickJS via JSON.stringify/parse

### 3. CacheKey
```rust
pub struct CacheKey {
    pub workflow_id: String,
    pub version: u32,
    pub node_id: String,
    pub param_name: String,
}
```

## Penggunaan

```rust
use ebc_prototype::{BytecodeCache, CachedExpressionEngine, CacheKey};

// Create cache (in-memory or file-based)
let cache = BytecodeCache::in_memory()?;
let engine = CachedExpressionEngine::new(cache);

// Define cache key
let key = CacheKey::new("workflow-123", 1, "node-http", "url");

// Evaluate expression (auto cache)
let scope = serde_json::json!({
    "json": {"user": {"email": "test@example.com"}},
    "itemIndex": 0
});

let result = engine.eval(&key, "$json.user.email", &scope).await?;
assert_eq!(result, serde_json::json!("test@example.com"));

// Second call: cache hit (faster)
let result2 = engine.eval(&key, "$json.user.email", &scope).await?;
```

## Test Coverage

```
✓ cache_put_get          — basic cache store/retrieve
✓ cache_miss             — cache miss returns None
✓ cache_invalidation     — workflow invalidation clears entries
✓ version_isolation      — different versions = different cache
✓ simple_expression      — "1 + 2" → 3
✓ cache_hit              — second eval uses cache
✓ json_access            — $json.user.email works
✓ validation             — syntax check without execute
✓ cache_key_creation     — key construction
```

## Build & Test

```bash
# Set clang include path (required for rquickjs)
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/llvm-18/lib/clang/18/include"

# Build
cargo build

# Test
cargo test

# Run specific test
cargo test test_cache_hit
```

## Dependencies

- `rquickjs` 0.6.2 — QuickJS bindings (bytecode compilation)
- `rusqlite` 0.31 — SQLite storage
- `tokio` — async runtime
- `serde_json` — JSON serialization
- `thiserror` — error types

## Performance (estimated, pending agent8 benchmark)

| Metric | Target | Status |
|--------|--------|--------|
| Cold start overhead | ≤10ms | pending benchmark |
| Cache hit savings | ≥15ms | pending benchmark |
| RAM transient | ≤4KB | pending measurement |
| Disk per expression | ≤500 bytes | pending measurement |
| Compile success rate | ≥99% | pending corpus test |
| Cache hit rate | ≥95% | pending integration test |

## Limitations (Prototype)

1. **Bytecode = source code**: Prototype stores source as "bytecode". Real implementation needs QuickJS bytecode serialization API.
2. **Scope limited**: Only `$json` and `$itemIndex`. Missing: `$input`, `$binary`, `$env`, `$workflow`, `$execution`, `$node`.
3. **No batch eval**: `eval_batch()` not implemented (reusing warm context).
4. **No parallel compile**: Single-threaded compilation.
5. **No MCP integration**: Cache stats not exposed via MCP tools yet.

## Next Steps

1. **Real bytecode serialization**: Investigate rquickjs `Module::write()` / `Context::load()` API
2. **Kernel integration**: Implement `ExpressionEngine` trait wrapper
3. **Full scope support**: Add all n8n expression variables
4. **Batch evaluation**: Reuse warm QuickJS context for multiple expressions
5. **Benchmark**: agent8 measurement (cold start, cache hit, RAM, disk)
6. **MCP enhancement**: Expose cache stats via `inspect_workflow`

## Integration Path

```
Prototype (standalone)
    ↓
crates/expr-quickjs (kernel ExpressionEngine impl)
    ↓
Integration dengan data-plane (SQLite schema)
    ↓
MCP tool enhancement (agent7)
    ↓
Production deployment
```

## References

- Spec: `/opt/agent-workspace/docs/AGENT3-EBC-SPEC.md`
- Test plan: `/opt/agent-workspace/docs/AGENT3-TEST-PLAN.md`
- Kernel trait: `ExpressionEngine` in `crates/kernel/src/context.rs`

## Author

agent3 (ROLE_EXPRESSION) — W1-EBC-CACHE task

---

**License:** Apache-2.0 (same as kernel workspace)
