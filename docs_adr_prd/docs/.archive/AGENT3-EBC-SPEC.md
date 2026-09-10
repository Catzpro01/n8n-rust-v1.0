# EXPRESSION BYTECODE CACHE (EBC) — Specification (agent3)

**Status:** DRAFT v0.1
**Owner:** agent3 (ROLE_QA)
**Proposal:** #366
**Patuh freeze #386** — dokumen, bukan kode

## 1. Ringkasan

**Masalah:** QuickJS evaluate expression dari source string setiap kali. Untuk workflow yang dijalankan berulang (cron, webhook), parse + compile overhead terakumulasi.

**Solusi:** Compile expression ke bytecode SEKALI saat workflow disimpan, cache di SQLite. Runtime load bytecode langsung, skip parse+compile.

**Manfaat:**
- Cold start: +5ms (compile pertama kali)
- Subsequent runs: -15-30ms (skip compile)
- RAM: 0 persistent (bytecode di SQLite), ~4KB transient (load ke QuickJS context)

## 2. Arsitektur

```
Workflow save → Expression Extractor → QuickJS Compiler → Bytecode
                                                              ↓
SQLite: expression_cache(workflow_id, version, node_id, param_name, bytecode, compiled_at)
                                                              ↓
Runtime → Load bytecode → QuickJS Context (~4KB transient) → Execute → Discard
```

## 3. Strategi Kompilasi

### 3.1 Compile-once
- Trigger: workflow save / update
- Extract semua expression dari node parameters (regex `\{\{.*?\}\}`)
- Compile setiap expression ke QuickJS bytecode
- Store di SQLite dengan key `(workflow_id, version, node_id, param_name)`

### 3.2 Cache Invalidation
- Key include `version` → workflow edit = version naik = cache stale otomatis
- Tidak perlu explicit invalidation logic
- Cold start untuk workflow baru / edited: compile on first execution

### 3.3 Bytecode Format
- QuickJS native bytecode (binary format)
- Size: ~2-5x source string (masih kecil, <1KB per expression typical)
- Portable: bytecode bisa di-load di QuickJS context mana pun (version-compatible)

## 4. Integration dengan MCP

### 4.1 Expression-Rules Resource
- `n8n://expression-rules` (AGENT3-EXPRESSION-RULES-MCP.md v1.0)
- AI agent tahu jebakan (3E + 5W codes) SEBELUM compile
- Prevent: AI write expression yang bakal fail compile

### 4.2 MCP Tool Enhancement
- `inspect_workflow` bisa expose: "N expressions cached, M expressions pending compile"
- `preflight` bisa estimate: "cold start +Xms (compile Y expressions)"

### 4.3 Error Handling
- Compile error → E-EXPR-SYNTAX (new code, ERROR class)
- AI agent dapat diagnostic: "expression invalid, fix before save"
- Cache tidak disimpan untuk invalid expressions

## 5. Biaya yang Jujur (ERR-029 compliance)

| Resource | Persistent | Transient | Satuan |
|---|---|---|---|
| RAM | 0 byte | ~4KB per execution | (bytecode di SQLite, load ke QuickJS context lalu discard) |
| Disk | ~500 bytes/expression | N/A | (bytecode size ~2-5x source, typical 200-500 bytes) |
| CPU (cold start) | +5ms | N/A | (compile first time, one-time cost) |
| CPU (subsequent) | -15-30ms | N/A | (skip compile, savings per execution) |
| SQLite | 1 table | N/A | (expression_cache: workflow_id, version, node_id, param_name, bytecode, compiled_at) |

**Akuntansi jujur:**
- "0 RAM persistent" = bytecode di SQLite (disk), bukan di heap engine
- "~4KB transient" = bytecode load ke QuickJS context per execution, discard setelah execute
- "+5ms cold start" = compile overhead untuk workflow baru / edited
- "-15-30ms subsequent" = savings dari skip compile (estimate, perlu empirical validation agent8)

## 6. Gate Falsifiable

| Gate | Target | Cara Ukur |
|---|---|---|
| Compile success rate | ≥99% | 1000 valid expressions → ≥990 compile sukses |
| Cache hit rate | ≥95% | 100 executions workflow unchanged → ≥95 cache hits |
| Cold start overhead | ≤10ms | 100 expressions compile → median ≤10ms |
| Subsequent savings | ≥15ms | 100 executions cached vs uncached → delta ≥15ms |
| RAM transient | ≤4KB | QuickJS context size dengan bytecode loaded |
| Disk per expression | ≤500 bytes | 100 expressions → median bytecode size ≤500 bytes |
| Cache invalidation | 100% | Workflow edit → version naik → cache miss 100% |

## 7. Dependensi

**Menunggu:**
1. QuickJS integration di engine (PRD-2 §4.2)
2. SQLite schema untuk expression_cache (Layer 0 agent2)
3. Empirical validation agent8 (cold start + savings measurements)

**Ready:**
- Arsitektur compile-once + cache invalidation
- Integration plan dengan MCP expression-rules
- Biaya honest (ERR-029 compliance)
- Gate falsifiable (7 metrics)

**Tidak butuh:**
- Kernel K-1 decision (EBC = engine-level, bukan kernel-level)
- CASD integration (bytecode cache independent dari spill dedup)

## 8. Risiko & Mitigasi

| Risiko | Mitigasi |
|---|---|
| QuickJS version upgrade → bytecode incompatible | Version field di cache schema; invalidate semua cache saat upgrade |
| Expression sangat kompleks → bytecode besar | Limit: max 10KB bytecode per expression; fallback ke source evaluation |
| Cache corruption (SQLite error) | Fallback ke source evaluation; log warning; recompile on next save |
| Cold start penalty untuk workflow besar | Parallel compile (multi-thread); progress indicator di MCP |

## 9. Koneksi ke Proposal Lain

- **CASD (#303):** Independent — CASD dedup spill content, EBC cache bytecode. Tidak ada overlap.
- **WCB (#375):** Complementary — WASM plugin mungkin butuh expression evaluation; EBC provide cached bytecode.
- **MCP (agent7):** EBC enhance `inspect_workflow` + `preflight` dengan cache status.
- **Expression-rules (agent3):** AI agent tahu jebakan sebelum compile → prevent invalid expressions.

## 10. Status

**Draft v0.1** — siap untuk review adversarial.

**Next steps:**
1. Review agent2 (SQLite schema integration)
2. Review agent8 (empirical validation plan)
3. Review agent7 (MCP tool enhancement)
4. Tunggu QuickJS integration di engine (PRD-2 §4.2)

_Ditulis oleh agent3 (QA). Review adversarial dipersilakan._
