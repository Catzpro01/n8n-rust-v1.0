# anchor — W0-ANCHOR-IMPL (RFC 3161 TSA anchor client)

Implements `AGENT10-W0-ANCHOR-SPEC.md` §4 (record), §5 (protocol + V1–V8),
§6 (cadence/window), §7 (quorum, MVP = M-of-M explicit), §9 (DDL verbatim).

## Trust model (read first)

- **Zero new crypto code.** ASN.1 / signing / verification are done by the
  audited `openssl ts` CLI; transport by `curl`. This crate only orchestrates,
  parses `openssl -text` output, and enforces policy.
- **ANC-5 (no implicit trust store)** is enforced as policy: `verify` REFUSES
  to run without an explicit `--pin` (exit 2 + `POLICY REFUSAL`). A hex pin
  must have its PEM at `<tsr-dir>/pins/<fp>.pem`; a PEM path may be passed
  directly (its fingerprint is still compared to the row pin — V4).
- **Verify-before-record:** a TSR that fails any of V1–V7 is never recorded
  as ANCHORED (a FAILED row + alarm is recorded instead).
- **`local_ts` == `created_utc`** (single clock read). The payload needs
  `local_ts` (§4) but the DDL (§9) has no such column; this identity keeps the
  payload reconstructible from `anchor_log` alone, which is what offline
  audit re-verification (§5.3) requires.
- **D-A1 default** (owner undecided): fail-OPEN ops / fail-CLOSED claims.
  TSA outage → FAILED row + `ALARM` on stderr, exit status **0** (ANC-7:
  anchoring must never block execution).
- **MVP quorum = M-of-M** (all listed TSAs must succeed for ANCHORED).
  Stricter than N-of-M, i.e. it can only under-claim, never over-claim.
  Relax via `--quorum K` explicitly.
- Forbidden claim-words are never printed; states are exactly
  `ANCHORED / PARTIAL / FAILED / UNANCHORED / ANCHOR-MISMATCH`.

## Build

```bash
export CARGO_TARGET_DIR=/mnt/extra-storage/cargo-target-agent1
cargo build --release   # -> $CARGO_TARGET_DIR/release/anchor
cargo test --release
```

Deps: `rusqlite` (bundled SQLite) + `sha2`. `Cargo.lock` is committed.
Requires `openssl` 3.x CLI, `curl`, `sqlite3` on PATH.

## Use

```bash
# One window per invocation (cadence scheduling = caller/cron, per D-A2:
# 1000 envelope-entries OR 60 s, whichever-first; N counts ENTRIES).
anchor create --chain-id CHAIN --chain-head <64hex> \
  --entry-from 0 --entry-to 999 \
  --tsa https://freetsa.org/tsr --pin /path/to/freetsa-cacert.pem \
  [--tsa URL2 --pin PEM2 ...] [--quorum K] \
  [--db /mnt/extra-storage/anchor/anchor_log.sqlite] \
  [--tsr-dir /mnt/extra-storage/anchor/tsr] [--tsa-timeout 25]

# Offline audit re-verification (§5.3; needs DB + TSR files + pin only).
anchor verify --db ... --tsr-dir ... --pin ... [--chain-id CHAIN]
```

First anchor of a chain uses `prev_anchor_head = zeros`; later windows must
continue `entry_from = prev entry_to + 1` (V8, fail-closed, exit 2).

## Acceptance gates

```bash
bash tests/anc_gates.sh $CARGO_TARGET_DIR/release/anchor
```

Runs ANC-1..ANC-7 (+V8, +tamper variants) against live FreeTSA. Last run:
**14 passed, 0 failed** (2026-09-09). Workdirs kept under `/tmp/anc-test-*`
for audit.

## Integration (for matt — C-5)

Standalone crate at `/home/agent1/w0-anchor-impl/` (not yet in the canonical
tree — awaiting explicit target path per C-5). Suggested: copy to
`kernel-asli-d3bcff0/crates/anchor/` + wire `anchor create` into the
writer cadence loop and `anchor verify` into the audit path.
