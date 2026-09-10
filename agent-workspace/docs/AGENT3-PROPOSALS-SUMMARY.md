# AGENT3 PROPOSALS SUMMARY — CASD, EBC, Expression-Rules, WCB

**Status:** FINAL v1.0
**Owner:** agent3 (ROLE_QA)
**Purpose:** Comprehensive summary untuk PRD-3 assembly (matt)
**Patuh freeze #386** — dokumen, bukan kode

## 1. Ringkasan Eksekutif

4 proposals saya untuk sayembara #298/#362:

| Proposal | ID | Status | Spec | Test Plan | Wave |
|---|---|---|---|---|---|
| Content-Addressable Spill Deduplication | #303 | Spec v0.2 | #683 | #703 | Wave 2 |
| Expression Bytecode Cache | #366 | Spec v0.1 | #673 | #703 | Wave 1 |
| Expression-Rules MCP Resource | #419 | v1.0 | #667 | #703 | Wave 1 |
| WASM Community Node Bridge | #375 | Handed off | agent6 #678 | agent6 | Wave 1 |

**Total gates falsifiable:** 21 gates (CASD 7 + EBC 7 + Expression-rules 7)

## 2. CASD — Content-Addressable Spill Deduplication (#303)

### 2.1 Konsep
Hash spill content dengan BLAKE3, store sekali, reference by hash. Multiple executions point ke same physical file.

### 2.2 Manfaat
- Disk savings: 40-60% (estimate)
- Write efficiency: skip write jika hash exists
- Read efficiency: OS cache benefit untuk frequently-accessed content

### 2.3 Biaya Jujur (ERR-029)
- RAM persistent: 0 byte (index di SQLite)
- RAM transient: 1920 bytes (single BLAKE3 hasher, reset antara spills)
- Disk index: ~150 bytes/entry
- CPU hash: ~1ms/MB (BLAKE3 streaming)
- CPU lookup: ~0.1ms (SQLite index query)

### 2.4 Integration
- **Layer 0 (agent2):** Storage foundation, spill directory, SQLite schema
- **WCB (agent6):** Plugin output spills → CASD dedup
- **EBC:** Independent (bytecode di SQLite, bukan spill files)
- **Envelope (agent5):** Execution receipt reference spill by content_hash

### 2.5 Gates (7)
1. Dedup ratio ≥40%
2. Hash collision = 0
3. Write savings ≥30%
4. RAM transient ≤4KB
5. Hash speed ≤1ms/MB
6. Index lookup ≤0.5ms
7. GC correctness = 100%

### 2.6 Dependensi
- Layer 0 storage foundation (agent2)
- BLAKE3 vs SHA-256 decision (T-11)
- Empirical validation agent8
- Layer 0 storage foundation (agent2)
- BLAKE3 vs SHA-256 decision (T-11)
- Empirical validation agent8

### 2.7 Status
- Spec v0.2: DONE (#683)
- Test plan: DONE (#703)
- Waiting: Layer 0, empirical validation

## 3. EBC — Expression Bytecode Cache (#366)

### 3.1 Konsep
Compile expression ke QuickJS bytecode SEKALI saat workflow disimpan, cache di SQLite. Runtime load bytecode langsung, skip parse+compile.

### 3.2 Manfaat
- Cold start: +5ms (compile pertama kali)
- Subsequent runs: -15-30ms (skip compile)
- Predictable latency

### 3.3 Biaya Jujur (ERR-029)
- RAM persistent: 0 byte (bytecode di SQLite)
- RAM transient: ~4KB per execution (load ke QuickJS context, discard setelah execute)
- Disk: ~500 bytes/expression (bytecode size ~2-5x source)
- CPU cold start: +5ms (compile first time)
- CPU subsequent: -15-30ms (skip compile)

### 3.4 Integration
- **QuickJS (PRD-2 §4.2):** Expression engine
- **SQLite (Layer 0):** Cache storage
- **MCP (agent7):** Enhance `inspect_workflow` + `preflight` dengan cache status
- **Expression-rules:** AI agent tahu jebakan sebelum compile

### 3.5 Gates (7)
1. Compile success rate ≥99%
2. Cache hit rate ≥95%
3. Cold start overhead ≤10ms
4. Subsequent savings ≥15ms
5. RAM transient ≤4KB
6. Disk per expression ≤500 bytes
7. Cache invalidation = 100%

### 3.6 Dependensi
- QuickJS integration (PRD-2 §4.2)
- SQLite schema (Layer 0 agent2)
- **Engine interface contract (agent1 spec)** — evaluator ekspresi untuk EBC
- Empirical validation agent8
- QuickJS integration (PRD-2 §4.2)
- SQLite schema (Layer 0 agent2)
- Empirical validation agent8

### 3.7 Status
- Spec v0.1: DONE (#673)
- Test plan: DONE (#703)
- Waiting: QuickJS integration, empirical validation

## 4. Expression-Rules MCP Resource (#419)

### 4.1 Konsep
MCP resource `n8n://expression-rules` yang provide 8 codes (3 ERROR + 5 WARNING) untuk AI agent yang edit workflow via MCP.

### 4.2 Manfaat
- Prevent: AI write expression yang bakal fail
- Guide: AI dapat autofix hint untuk common jebakan
- Educate: AI tahu nuansa lintas-runtime (V8 vs QuickJS)

### 4.3 Biaya Jujur (ERR-029)
- RAM: 0 byte (resource di SQLite, pull-only JIT)
- Disk: ~2KB (resource content)
- Token budget: ≤500 token (estimate ~350 token, tunggu tokenizer matt #419)

### 4.4 Integration
- **MCP (agent7):** Registered di registry v0.5 (#701), 13 URI
- **EBC:** AI tahu jebakan sebelum compile
- **DEVIATION-CATALOG (agent5):** K2/K3/K6 cross-reference
- **Limits (agent2):** $env allowlist untuk prevent covert channel

### 4.5 Gates (7)
1. Coverage = 215/215 cases
2. Token budget ≤500 token
3. ERROR FP rate <2%
4. WARNING FP rate <10%
5. AI comprehension ≥80%
6. Autofix reject_accept = 8/8 pairs
7. Autofix output_eq = 5/5 pairs

### 4.6 Dependensi
- MCP integration (agent7)
- Tokenizer rujukan matt (#419)
- Empirical validation agent8
- MCP integration (agent7)
- Tokenizer rujukan matt (#419)
- Empirical validation agent8

### 4.7 Status
- v1.0: DONE (#667)
- Test plan: DONE (#703)
- Registry: Confirmed (#701)
- Waiting: Tokenizer, empirical validation

## 5. WCB — WASM Community Node Bridge (#375)

### 5.1 Konsep
Community nodes berjalan di WASM instance terpisah (wasmtime/wasmer). Memory limit configurable per-node (default 64MB). No filesystem/network/process. Deterministic by design.

### 5.2 Manfaat
- Extensibility tanpa risiko (WASM isolation)
- Multi-language (Rust/Go/C/AssemblyScript)
- Deterministic by design (same input = same output)
- Marketplace potensial (.wasm files tanpa dependency hell)

### 5.3 Status
- Original proposal: #375
- Handed off ke agent6 (ROLE_WASM) sebagai implementation owner (#575)
- agent6 spec v0.4: DONE (#678)
- agent6 PRD-3 input: READY

### 5.4 Kontribusi agent3
- Original concept (#375)
- Comprehensive review agent6 v0.3 (#663)
- Integration plan dengan CASD (dedup plugin output spills)
- Integration plan dengan EBC (expression cache untuk WASM node parameters)

## 6. Koneksi Antar-Proposal

### 6.1 Dependency Graph
```
Layer 0 (agent2) ──┬──> CASD (agent3, Wave 2)
                   └──> EBC (agent3, Wave 1)

QuickJS (PRD-2) ──> EBC (agent3, Wave 1)

MCP (agent7) ────> Expression-rules (agent3, Wave 1)

CASD (agent3) ──> WCB (agent6, Wave 1) [dedup plugin spills]
EBC (agent3) ───> WCB (agent6, Wave 1) [cache WASM node expressions]
```

### 6.2 Independence
- **CASD:** Independent dari K-1 (Layer 1, di atas storage)
- **EBC:** Independent dari K-1 (engine-level, bukan kernel-level)
- **Expression-rules:** Independent dari K-1 (MCP resource)
- **WCB:** Independent dari K-1 (crates/wcb/, terpisah dari kernel)

### 6.3 Complementary
- CASD + WCB: Plugin output spills → dedup
- EBC + WCB: WASM node expressions → cache
- Expression-rules + EBC: AI tahu jebakan sebelum compile
- CASD + Envelope: Execution receipt reference spill by content_hash

## 7. Wave Readiness

### 7.1 Wave 1 (segera, tanpa K-1)
- **EBC:** Spec ready, tunggu QuickJS integration
- **Expression-rules:** v1.0 ready, registry confirmed, tunggu tokenizer
- **WCB:** Spec v0.4 ready (agent6), tunggu wasmtime integration

### 7.2 Wave 2 (setelah K-1)
- **CASD:** Spec v0.2 ready, tunggu Layer 0 storage

### 7.3 Status Overall
- **3 specs lengkap:** CASD, EBC, Expression-rules
- **1 handoff:** WCB ke agent6
- **1 test plan:** 21 gates falsifiable
- **IDLE:** Siap klaim task saat Wave 1 dibuka

## 8. Kontribusi Diskusi

Selain 4 proposals, saya contribute:
- Peta dependensi proposal (#562, revised #568)
- Blake3 1920B implications (#572)
- Checklist anti-tuduhan-palsu → QA spec §9.1 (#533, #552)
- Ed25519 deviation_ids_hash (#605)
- Multiple technical reviews (Hub, MCP, WCB)
- Review adversarial culture support

## 9. Lessons Learned

### 9.1 Review Adversarial Works
- agent5 koreksi ContentId = u64 bukan hash (#346)
- agent4 koreksi $json → $input.first() bukan perbaikan (#441)
- agent4 koreksi .join() tanpa argumen = COMMA (#448)
- **Pattern:** "Terima kesimpulan pertama tanpa verifikasi independen"

### 9.2 Checklist Anti-Tuduhan-Palsu
1. Alat saya benar?
2. Bukti lengkap?
3. Konteks temporal?
4. Penjelasan alternatif?
5. Konflik kepentingan?

**Adopted oleh agent5 ke QA spec §9.1 (#551)**

### 9.3 ERR-029 Compliance
- "0 RAM" tidak cukup, harus "0 persistent, transien <4KB"
- "Hemat 40-60%" harus "dedup ratio ≥40% (gate falsifiable)"
- Setiap klaim butuh satuan + cara ukur

## 10. Status Final

**4 proposals:**
- CASD (#303): Spec v0.2, test plan, waiting Layer 0
- EBC (#366): Spec v0.1, test plan, waiting QuickJS
- Expression-rules (#419): v1.0, test plan, registry confirmed, waiting tokenizer
- WCB (#375): Handed off ke agent6, spec v0.4 done

**Readiness:**
- 3 specs lengkap
- 1 test plan (21 gates)
- IDLE untuk implementasi Wave 1

**Dependencies minimal:**
- QuickJS integration (EBC + Expression-rules)
- SQLite schema (CASD + EBC)
- Layer 0 storage (CASD)
- MCP integration (Expression-rules)

**Tidak butuh K-1 decision** (semua spec independent dari kernel canonical).

_Ditulis oleh agent3 (QA). Untuk PRD-3 assembly (matt)._

### 5.5 Dependensi (agent6 implementation)
- Wasmtime integration
- **Engine interface contract (agent1 spec)** — host-streaming §5 untuk WCB
- Empirical validation agent8

## 11. Complete Dependency Matrix (updated #707)

| Proposal | Layer 0 | QuickJS | MCP | Engine Contract | Tokenizer | Validation |
|---|---|---|---|---|---|---|
| CASD | ✓ | - | - | - | - | ✓ |
| EBC | ✓ | ✓ | - | ✓ | - | ✓ |
| Expression-rules | - | - | ✓ | - | ✓ | ✓ |
| WCB | - | - | - | ✓ | - | ✓ |

**Legend:**
- ✓ = dependency required
- - = not required

**Notes:**
- Engine interface contract (agent1 spec): evaluator ekspresi untuk EBC, host-streaming §5 untuk WCB
- Semua spec independent dari K-1 (kernel canonical decision)
