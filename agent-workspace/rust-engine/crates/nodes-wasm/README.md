# nodes-wasm — WCB community-node WASM manifest + bridge runtime

Canonical crate (rust-engine) scope per **RULING 55 (A)** (`#1548`, merged at
`3e571b1`):

- Library: `src/manifest.rs` (manifest schema + parse/depth gates),
  `src/gates.rs` (MV-1…MV-7 manifest/runtime-shape gates), `src/hostconfig.rs`
  (host-config math), `src/lib.rs`.
- `tests/spill_mv6.rs` (2): MV-6 large-body spill via the ported kernel
  `data-plane::FileSpillStore`.
- In-tree tests: **21** (19 unit + 2 spill).

## Host-layer runtime-sandbox enforcement tests live in the agent6 shadow tree

The 10 tests that actually *enforce the sandbox claims* (MV-4 runtime egress-deny,
fuel metering, `grow > 32 MiB` rejection, deny non-`env|record` import namespaces,
tamper rejection) are **not** in canonical. That is deliberate — **RULING 55 (A)**
landed the crate without the wasmtime host layer because adding wasmtime as a
dev-dependency forces a workspace-wide dependency refresh (serde ≥ 1.0.228 /
async-trait ≥ 0.1.89) which is gated on **dep approval @matt (`#1125`, pending)**
and lane consent (rosetta / mcp).

Location of the full shadow suite (do **not** clean until migrated):

- Shadow tree (public, group-readable): `/mnt/extra-storage/agent6-work/W1-WCB-IMPL`
  (`crates/nodes-wasm/tests/`)
- `host_gates.rs` (6 tests) — sha256 `0c4029cda8ba75957ad3feb372c73ecfd10924f6cad527f25fb999428743b6ac`
- `host_e2e.rs` (4 tests) — sha256 `f75ccac1f805e9f0bb421da958e0d14d0d69ffc5e4788ff88b6aead1e7b10af2`

Full verified shadow suite = **31** tests (21 in-tree + 10 host) against kernel
`5bc8ea8`, clippy 0 (gate `#1395`).

## Migration path

Task **W2-WORKSPACE-DEP-REFRESH** (agent6, `#1567`, RULING 55 §2) migrates the host
layer into canonical after Wave 1 closes: refresh workspace `serde` → `=1.0.229`,
`async-trait` → `=0.1.89`, move rosetta/mcp direct pins to `{ workspace = true }`
(consent required), then full workspace test + clippy + golden byte-identical
verification. Until then, the evidence for runtime-sandbox enforcement lives only
in the shadow tree referenced above.
