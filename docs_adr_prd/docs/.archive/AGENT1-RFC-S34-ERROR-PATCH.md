# RFC PATCH §3.4 — `KernelError` enrichment + `From<KernelError> for NodeError`

- Author: agent1 (sole `error.rs` writer per #1116: "Satu-satunya yang boleh menyentuh error.rs adalah agent1 lewat RFC §3.4").
- Mandate: #1080 ambiguity-1 (§3.4 direction-(c)) + Ruling 12 + **Ruling 17 (#1102)** — "Ini mengubah bentuk RFC §3.4, jadi masukkan sebelum agent1 menulisnya."
- Status: PATCH ARTIFACT for review. NO kernel edit until @fern sentence + 2 formal #922C reviews run in parallel + quorum (Ruling 12/ambiguity-1).
- Version: v1.7 (supersedes v1.6 `392328466354`). Changelog: N-8 (both sessions agree) - A-10 restatement moves to `///` ON the fn; old impl-level doc comment REPLACED (stated explicitly; summary line kept); in-body `//` copy removed (single rustdoc-visible home). Rehearsal mirrors exactly (doc-comment reps added). v1.6 changelog: review sesi-A #1225 fully answered - B-4 (M-B2a exact + M-B2b as a PAIR covering negative and positive asserts, FAIL lines pasted); B-5(1) A-10 preservation statement + verbatim quote; N-6 B-3 pin cfg(unix)-gated; N-7 section-7 to section-10 renumber. v1.5 changelog: Ruling 33 (B-3 ACCEPTED with leak-forcing rationale + fd ASYMMETRY pre-check-vs-post-refusal written explicitly); the v1.3 CONVERGENCE RULE is STRUCK (the asymmetry is INTENTIONAL, not temporary — EMFILE stays Permanent even after stabilization); stale D7 fixed (it still said Transient). PROSE-ONLY delta: code blocks identical to rehearsed v1.4. v1.4 changelog: §3c FULLY EXHAUSTIVE per agent10 #1205 (a)(b) (five explicit Resource arms, no wildcard at either level — a sixth variant anywhere is a COMPILE error forcing a decision); A-10 note restated (d); §6.3 M-B2a/M-B2b mutants with rehearsal FAIL lines (c). v1.3 changelog: B-2 RESOLVED per Ruling 32 (five-branch `is_retryable()` + cause-not-variant principle + retry assertions); B-3/EMFILE convergence rule vs FileHandles-retryable; v1.2 changelog: B-3 (EMFILE→Permanent fallback — `TooManyOpenFiles`/`Uncategorized` still feature-gated on rustc 1.98 stable, caught by merge rehearsal); row-4 shrinks to stable names; §6.1 gains B-3 pin test. v1.1 changelog: B-1 fixed (KNOWN LIMITATION + log-before-return obligation + `UNKNOWN_BYTES`); N-2 Debug-logging sentence; N-3 §6 test plan (exact 9-row test + mutant protocol); N-4 row-5 metric note; ETIMEDOUT atomicity dependency; B-2 ESCALATED to @matt (row 2 vs `is_retryable()`); N-1 noted (stay Transient).
- Scope note (#1125 Fakta 3): `error.rs` is byte-identical (`a431ce3112a0`) in both trees → this patch applies cleanly wherever W0-KERNEL-UNIFY lands.

## 1. Binding decisions implemented

Ruling 17 (#1102), adopted verbatim except where flagged EXT:

| # | KernelError source | NodeError target | Basis |
|---|---|---|---|
| 1 | `Expression{expr,reason}` | `Expression{expr,reason}` 1:1 | R17 approved |
| 2 | `SpillIo{kind: StorageFull\|QuotaExceeded}` | `ResourceExhausted{resource: Disk, requested: UNKNOWN_BYTES, available: UNKNOWN_BYTES}` | R17 ENOSPC row. KNOWN LIMITATION (B-1): `message` + kind-subclass DROPPED on this arm (`NodeError::ResourceExhausted` has no detail field, D5) — compensated by log-before-return. Non-retryable via Ruling 32 (§8: Disk to false) |
| 3 | `SpillIo{kind: PermissionDenied\|ReadOnlyFilesystem}` | `Permanent{message, code}` | R17 EACCES row (+EROFS, flagged EXT — same non-retryable class) |
| 4 | `SpillIo{kind: Interrupted\|WouldBlock\|TimedOut}` | `Transient{message, retry_after: None}` | R17 EINTR/EAGAIN rows (+ETIMEDOUT EXT); `None` = kernel invents no backoff (D14 lives at node layer). EMFILE/ENFILE → row 5 by necessity (B-3, §8) |
| 5 | `SpillIo{..}` (all other kinds) | `Permanent{message, code}` | Explicit default: only KNOWN-transient kinds retry; unknown kinds fail toward non-retry (the safe direction per R17's ENOSPC argument) |
| 6 | `IndexOutOfBounds\|NodeNotFound\|Codec\|Invalid` | `Internal{message: <source Display>}` | R17 approved ("bug → wajib jadi test"; Display preserves human detail) |
| 7 | `Transient{message,retry_after}` | `Transient` 1:1 (mirror-exact) | R17 approved |
| 8 | `Timeout{elapsed}` | `Timeout` 1:1 (mirror-exact) | R17 approved |
| 9 | `ResourceExhausted{resource,requested,available}` | `ResourceExhausted` 1:1 (mirror-exact) | R17 approved |

`code` in rows 3/5 = `ErrorCode(format!("{kind:?}"))` — no invented taxonomy, greppable (D63).

### Row-2 obligation (B-1 compensation, lands with the #1143-butir-4 logging task)

`From` stays pure and total, so the dropped detail must live in exactly one
other place: the spill layer MUST log the full `message` via the kernel
`Logger` (single logging path, D93 redaction) BEFORE returning a
`KernelError::SpillIo` with `kind` in `{StorageFull, QuotaExceeded}`. This
obligation is part of the `Option<Arc<dyn Logger>>` logging task that follows
this RFC's merge — it is recorded here so the table's KNOWN LIMITATION is
closed by construction, not by discipline.

```rust
/// Sentinel for unknown byte quantities (B-1): grep-able fabrication.
/// Governor (D72) consumers MUST treat this as "unknown", never as "zero".
pub const UNKNOWN_BYTES: u64 = 0;
```

## 2. Design rules (frozen in this RFC)

- **D1 — SpillIo Display FROZEN.** `#[error("spill store i/o failed: {message}")]` UNCHANGED. `kind` never enters Display (CT-safety; kind available via field + Debug). N-2 CONSEQUENCE (audit): anything logged with `{}` loses the retry classification — **logging a `KernelError` MUST use `{:?}` (Debug) so `kind` is visible** (D63 greppability); Display stays frozen for CT.
- **D2 — `kind: std::io::ErrorKind`** (Copy, Debug, PartialEq). No new dep, no Serialize needed (`KernelError` isn't Serialize).
- **D3 — EMFILE evidence** (probed on target rustc 1.98.1, not recalled): `EMFILE→TooManyOpenFiles`, `ENFILE→TooManyOpenFiles`, `EINTR→Interrupted`, `EAGAIN→WouldBlock`, `ENOSPC→StorageFull`, `EACCES→PermissionDenied`, `EROFS→ReadOnlyFilesystem`, `EDQUOT→QuotaExceeded`. TOOLCHAIN LIMITATION (B-3, caught by rehearsal): `TooManyOpenFiles` and `Uncategorized` are still behind feature gates (`io_error_too_many_open_files`, `io_error_uncategorized`) on stable — constructible via `from_raw_os_error` but NOT nameable in patterns. No errno-normalization hack is used (rejected: errno tables are platform-specific; reimplementing std's mapping is exactly what std refuses to expose).
- **D4 — New variants currently unconstructed** (`Transient`/`Timeout`/`ResourceExhausted` gain constructors with their users: spill retry layer, Governor D72, D58 deadlines). Declared now per R17 so `From` is total and mechanical.
- **D5 — `NodeError` UNCHANGED.** All change is additive on the `KernelError` side + one new `From` impl.
- **D6 — ETIMEDOUT→Transient is sound ONLY because spill writes are atomic** (tmp+rename: `data-plane/src/spill.rs` payload + refcount paths, verified by agent10 at :129,135,141 + :294,314). If a future writer adds a NON-atomic path, this mapping MUST be re-reviewed — retry after ETIMEDOUT must never read a half-written file as valid.
- **D7 — N-1 resolution (R33):** EMFILE/ENFILE go `Permanent` — R17's fail-safe direction made DELIBERATE (leak-forcing rationale, §9). The `ResourceExhausted{FileHandles}` alternative is more precise but inherits B-1; not pursued.
- **D8 — N-4 row-5 observability:** `From` is pure, so row-5 hits are counted CONSUMER-side by `Permanent.code` (the kind string). Consumers SHOULD export a counter per code so the table grows from observed errnos, not guesses.

## 3. Exact patch — `crates/kernel/src/error.rs`

### 3a. Enum: `SpillIo` gains `kind`, 3 new variants (mirror-exact Display)

```rust
    #[error("spill store i/o failed: {message}")]
    SpillIo {
        message: String,
        kind: std::io::ErrorKind,   // NEW (R17): retry classification needs it
    },
```

appended after `Invalid` (order: keep existing 6 first, additions last):

```rust
    /// Retryable kernel-level failure (mirror of `NodeError::Transient`).
    #[error("transient failure (retry_after={retry_after:?}): {message}")]
    Transient {
        message: String,
        retry_after: Option<std::time::Duration>,
    },

    /// A kernel-level deadline elapsed (mirror of `NodeError::Timeout`, D58).
    #[error("timeout after {elapsed:?}")]
    Timeout { elapsed: std::time::Duration },

    /// Governor/refusal at kernel level (mirror of `NodeError::ResourceExhausted`, D72).
    #[error("resource exhausted: {resource} (requested {requested} bytes, {available} available)")]
    ResourceExhausted {
        resource: Resource,
        requested: u64,
        available: u64,
    },
```

### 3b. New `From<KernelError> for NodeError` (appended after the `ErrorCode` impl block)

```rust
impl From<KernelError> for NodeError {
    /// Ruling 17: retry classification. Only KNOWN-transient I/O kinds map to
    /// `Transient`; everything else fails toward `Permanent` (never retry an
    /// identical failure) or `ResourceExhausted` (disk full/quota).
    fn from(e: KernelError) -> Self {
        match e {
            KernelError::Expression { expr, reason } => NodeError::Expression { expr, reason },
            KernelError::Transient { message, retry_after } => {
                NodeError::Transient { message, retry_after }
            }
            KernelError::Timeout { elapsed } => NodeError::Timeout { elapsed },
            KernelError::ResourceExhausted { resource, requested, available } => {
                NodeError::ResourceExhausted { resource, requested, available }
            }
            KernelError::SpillIo { message, kind } => match kind {
                std::io::ErrorKind::Interrupted
                | std::io::ErrorKind::WouldBlock
                | std::io::ErrorKind::TimedOut => NodeError::Transient {
                    // NOTE (B-3): EMFILE/ENFILE produce `TooManyOpenFiles`,
                    // which stable cannot name -> they fall to the default arm
                    // (Permanent). Pinned by the B-3 test in §6.1; R33: stays
                    // Permanent even if std stabilizes the name (post-refusal,
                    // not pre-check — see §9).
                    message,
                    retry_after: None,
                },
                std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded => {
                    NodeError::ResourceExhausted {
                        resource: Resource::Disk,
                        requested: UNKNOWN_BYTES,
                        available: UNKNOWN_BYTES,
                    }
                }
                other => NodeError::Permanent {
                    message,
                    code: ErrorCode(format!("{other:?}")),
                },
            },
            // Bug-class variants (R17): every occurrence must become a test.
            other => NodeError::Internal { message: other.to_string() },
        }
    }
}
```

## 4. Call-site updates (mechanical, same patch)

Every `SpillIo { message }` literal gains `kind` (exhaustive list, grepped):

| Site | Current | New `kind` | Why |
|---|---|---|---|
| `crates/data-plane/src/spill.rs` `io_to_kernel` | `message: e.to_string()` | `kind: e.kind()` | production path: real kinds flow (this is the R17 payoff) |
| `crates/kernel/tests/contract.rs:64` | `message: "no such spill file…"` | `kind: std::io::ErrorKind::NotFound` | in-memory stub: file absent = NotFound (honest) |
| `crates/kernel/examples/spill_bench.rs:90` (`io_err`) | `message: e.to_string()` | `kind: e.kind()` | same as production path |
| `spill_bench.rs:134,168` | `message: "no such spill file…"` | `kind: std::io::ErrorKind::NotFound` | same as contract.rs |
| `crates/data-plane/tests/fs_gates.rs:475` | `Err(KernelError::SpillIo { .. })` | NO CHANGE | `..` pattern is forward-compatible — pinned deliberately |

## 5. CT impact — asserts stay Err

`contract.rs` error assertions are `is_err()`-only (lines 276, 277, 320, 788, 792); zero assertions on error Display strings or variant shapes. D1 (frozen Display) + §4 keep every one green with identical expectations. No CT text changes.

## 6. Verification plan (post-quorum, executed by agent1)

1. Apply §3 + §4 to canonical `kernel-asli-d3bcff0` (+ both trees until unify step 3 closes, #1143).
2. `cargo test --workspace --locked` → must stay 35/35 (19 kernel + 13 FS + 3 unit).
3. `cargo clippy --workspace --all-targets --locked` → 0 warnings.
4. Run the §6.1 From-table test (added by this patch to `error.rs` tests) → green.
5. Run the §6.2 mutant (swap Transient/Permanent arms) → the §6.1 test MUST FAIL → restore → green.
6. Report tree state + file sha256 (evidence rule #991§2).

### 6.1 Exact From-table test (N-3a; ships WITH the patch)

```rust
#[cfg(test)]
mod from_table_tests {
    use super::*;
    use std::io::ErrorKind as K;

    fn spill(kind: K) -> KernelError {
        KernelError::SpillIo { message: "m".into(), kind }
    }

    #[test]
    fn all_nine_rows() {
        // Row 1: Expression 1:1.
        assert!(matches!(
            NodeError::from(KernelError::Expression { expr: "e".into(), reason: "r".into() }),
            NodeError::Expression { .. }
        ));
        // Row 2: disk exhaustion (B-1: message dropped by shape; B-2 pending).
        for k in [K::StorageFull, K::QuotaExceeded] {
            assert!(matches!(
                NodeError::from(spill(k)),
                NodeError::ResourceExhausted {
                    resource: Resource::Disk,
                    requested: UNKNOWN_BYTES,
                    available: UNKNOWN_BYTES,
                }
            ));
        }
        // Rows 3+5: Permanent with message + kind-code preserved.
        for k in [K::PermissionDenied, K::ReadOnlyFilesystem, K::NotFound, K::InvalidData, K::UnexpectedEof, K::Other] {
            match NodeError::from(spill(k)) {
                NodeError::Permanent { message, code } => {
                    assert_eq!(message, "m");
                    assert_eq!(code, ErrorCode(format!("{k:?}")));
                }
                other => panic!("{k:?} must map to Permanent, got {other:?}"),
            }
        }
        // Row 4: known-transient set, STABLE names only (D6: sound because
        // spill writes are atomic; B-3: TooManyOpenFiles is ungateable).
        for k in [K::Interrupted, K::WouldBlock, K::TimedOut] {
            assert!(matches!(
                NodeError::from(spill(k)),
                NodeError::Transient { retry_after: None, .. }
            ));
        }
        // Row 6: bug-class -> Internal carrying the source Display.
        for e in [
            KernelError::IndexOutOfBounds { index: 1, len: 0 },
            KernelError::NodeNotFound { node: "n".into() },
            KernelError::Codec { codec: "json", reason: "r".into() },
            KernelError::Invalid { message: "i".into() },
        ] {
            match NodeError::from(e) {
                NodeError::Internal { message } => assert!(!message.is_empty()),
                other => panic!("bug-class must map to Internal, got {other:?}"),
            }
        }
        // B-3 pin: EMFILE/ENFILE (constructible but unnameable on stable)
        // MUST fall to Permanent, loudly if std ever changes the mapping.
        // N-6: Unix-gated (errno numbers are a Unix assumption - the same
        // reason section-9 rejects errno probing inside From).
        #[cfg(unix)]
        {
            for errno in [24, 23] {
                let kind = std::io::Error::from_raw_os_error(errno).kind();
                match NodeError::from(spill(kind)) {
                    NodeError::Permanent { message, code } => {
                        assert_eq!(message, "m");
                        assert!(!code.as_str().is_empty());
                    }
                    other => panic!("errno {errno} must map to Permanent, got {other:?}"),
                }
            }
        }
        // B-2/Ruling 32: retryability follows the cause, not the variant.
        for k in [K::StorageFull, K::QuotaExceeded] {
            assert!(!NodeError::from(spill(k)).is_retryable(), "{k:?} must not retry");
        }
        for k in [K::Interrupted, K::WouldBlock, K::TimedOut] {
            assert!(NodeError::from(spill(k)).is_retryable(), "{k:?} must retry");
        }
        for k in [K::PermissionDenied, K::NotFound] {
            assert!(!NodeError::from(spill(k)).is_retryable(), "{k:?} must not retry");
        }
        for r in [Resource::Disk, Resource::Memory, Resource::Cpu] {
            assert!(!NodeError::ResourceExhausted { resource: r, requested: 1, available: 0 }.is_retryable());
        }
        for r in [Resource::FileHandles, Resource::Network] {
            assert!(NodeError::ResourceExhausted { resource: r, requested: 1, available: 0 }.is_retryable());
        }
        // Rows 7-9: mirrors 1:1.
        assert!(matches!(
            NodeError::from(KernelError::Transient { message: "t".into(), retry_after: None }),
            NodeError::Transient { retry_after: None, .. }
        ));
        assert!(matches!(
            NodeError::from(KernelError::Timeout { elapsed: std::time::Duration::from_secs(1) }),
            NodeError::Timeout { .. }
        ));
        assert!(matches!(
            NodeError::from(KernelError::ResourceExhausted {
                resource: Resource::Memory, requested: 1, available: 0,
            }),
            NodeError::ResourceExhausted { resource: Resource::Memory, .. }
        ));
    }
}
```

### 6.2 Mutant protocol (N-3b; executed at merge verification, shadow-style)

Mutant: swap the row-4 and row-5 targets (Transient arm yields `Permanent`,
default arm yields `Transient`). Expected: `all_nine_rows` FAILS (at least on
`Interrupted` and on `NotFound`). Then restore → green. The From-table is
prose until this mutant dies (Ruling 27 standard).

### 6.3 B-2 mutants (N-3c / #1205-c; rehearsed, FAIL lines pasted)

M-B2a: revert the `ResourceExhausted` arm to the variant-level bug
(`{ .. } => true`, i.e. the pre-Ruling-32 shape). Expected: the Disk
no-retry assertion FAILS (the test is not decoration):

```
panicked at crates/kernel/src/error.rs:295:13:
StorageFull must not retry  (M-B2a KILLED)
```

M-B2b (PAIR - adapted to the shipped exhaustive shape; the allowlist mutant
has no allowlist to flip): (i) flip `Resource::Disk => false` to `true` -
the NEGATIVE asserts must FAIL; (ii) flip `Resource::FileHandles => true` to
`false` - the POSITIVE asserts must FAIL:

```
panicked at crates/kernel/src/error.rs:313:13:
StorageFull must not retry  (M-B2b-i KILLED)
panicked at crates/kernel/src/error.rs:325:13:
assertion failed: NodeError::ResourceExhausted {...}.is_retryable()  (M-B2b-ii KILLED)
```

Then restore pristine → green. All three mutants die in rehearsal before
merge; M-B2a is the highest-value mutant in this RFC (it reproduces the
regression section-3c exists to fix).

### 3c. `is_retryable()` becomes resource-aware (Ruling 32, B-2 resolution)

DOC COMMENT (N-8 - the old impl-level `///` at error.rs:132-137 is REPLACED,
stated explicitly so it is never a silent drop): its content moves ONTO the fn
- summary first line kept, A-10 note restated sharper. The merged file reads:

```rust
    /// True when a [`NodeError`] should be retried under D14/D57.
    ///
    /// NOTE (audit A-10, restated - deliberately side-effect-blind, by design):
    /// a `Transient` error from a `SideEffect::NonIdempotent` node must be routed
    /// to quarantine by the *scheduler*, not retried here. 'Retryable' is NOT
    /// 'will be retried': `FileHandles`/`Network => true` below is permission to
    /// retry, and the scheduler MUST still refuse it for non-idempotent side
    /// effects (else: double external effects - double email, double charge).
    pub fn is_retryable(&self) -> bool {
        match self {
        NodeError::Transient { .. } => true,
        // Ruling 32: retryability is a property of the CAUSE, not the variant
        // name. Disk does not self-heal in a retry window (and retry can make
        // it worse via temp files); Memory/Cpu are governor DECISIONS
        // (re-queue path, not node retry). FileHandles/Network self-heal.
        // NO WILDCARD at either level (#1205 a/b): a sixth Resource or a new
        // NodeError variant is a COMPILE error forcing an explicit decision -
        // never a silent inheritance.
        NodeError::ResourceExhausted { resource, .. } => match resource {
            Resource::Disk => false,
            Resource::Memory => false,
            Resource::Cpu => false,
            Resource::FileHandles => true,
            Resource::Network => true,
        },
        NodeError::Permanent { .. }
        | NodeError::Timeout { .. }
        | NodeError::Cancelled
        | NodeError::Expression { .. }
        | NodeError::Internal { .. } => false,
    }
}
```

`is_terminal()` UNCHANGED. Cheapest of the day's three same-shape incidents: the kernel ALREADY carried `resource` — `is_retryable()` just never read it. No new field; use the field that was always there. (R17, B-2, TypeVersion: the same melted-sub-classification class.)

Deliberate asymmetry with §9 (R33 — read before reviewing): `ResourceExhausted{FileHandles} → retryable` (governor PRE-CHECK: the budget says stop opening, other ops will close fds, backoff helps) vs `EMFILE → Permanent` (OS POST-REFUSAL: the system is already out, nothing changes without intervention). Same resource, opposite answers, BY DESIGN.

## 8. B-2 RESOLVED by Ruling 32 (was: escalation — kept for the record)

`NodeError::is_retryable()` (`error.rs`, D14/D57) returns true for
`Transient` AND `ResourceExhausted`. Row 2 (ENOSPC to `ResourceExhausted`, R17)
therefore routes disk-full INTO the retryable class — the retry loop R17
forbids ("memukul disk penuh berulang kali"). R17's row assumes
`ResourceExhausted` is non-retry; the code disagrees. Three resolutions:

- (a) RECOMMENDED: make `is_retryable()` resource-aware — `ResourceExhausted{resource: Disk}` yields false, others unchanged (Memory/Cpu/FileHandles/Network CAN free up; disk effectively never does). Keeps R17's row AND its intent; one method, no variant change; fits this RFC's scope (additive `error.rs` change by the sole writer).
- (b) Row 2 to `Permanent{message, code}` instead: preserves message+kind (also fully closes B-1) and is non-retryable — but rewrites R17's explicit row and loses the Governor signal for disk events.
- (c) Keep row 2, accept backoff-bounded retry for disk-full (R17 forbids retry LANGSUNG; D14 backoff is not immediate). Relies on D14 bounds I cannot verify from here.

RESOLVED: option (a) ACCEPTED with the FIVE-branch mapping (§3c, not two-branch): Disk/Memory/Cpu to false; FileHandles/Network to true+backoff. Original options kept: (a) accepted-extended; (b) row-2-to-Permanent rejected (rewrites R17 + loses Governor signal); (c) backoff-retry rejected (Disk never self-heals, retry can worsen via temp files).

## 9. B-3 RESOLUTION (EMFILE stable-nameability — decided by author, needs @agent10 re-confirm)

`ErrorKind::TooManyOpenFiles` is feature-gated on stable rustc 1.98.1
(`io_error_too_many_open_files`; caught by merge rehearsal, E0658), so R17's
EMFILE→Transient row CANNOT be written as a named pattern. Resolution:
EMFILE/ENFILE fall through to the row-5 default (`Permanent{message, code}`).
This is R17's own fail-safe direction (unknown/unnameable → non-retry);
message + kind-code are preserved (unlike B-1's arm), and D8 consumer-side
metrics observe the hits. Rejected alternative: errno probing
(`from_raw_os_error(24)`) inside `From` — errno tables are platform-specific
(Windows code 24 is NOT EMFILE); it would reimplement the mapping std refuses
to expose. If std ever stabilizes the variant, row 4 does NOT gain it back
(R33 below: EMFILE→Permanent is deliberate, not a naming accident); the §6.1
B-3 pin test is then updated to assert the STABILIZED name still lands
Permanent. @agent10: your row-4 approval needs a nod on this shrink
(Transient set minus EMFILE); the retryable-set is unchanged for all stable
names.

FD ASYMMETRY (Ruling 33 — INTENTIONAL, do not 'fix' into uniformity):
`ResourceExhausted{resource: FileHandles} → RETRYABLE` (Ruling 32) while
`io::Error EMFILE → PERMANENT` (Ruling 33). Both are about fds, the answers
differ, and that is BY DESIGN: FileHandles-exhaustion is the governor's
PRE-CHECK ('the budget says stop opening' — other operations will close fds,
so backoff genuinely helps); an EMFILE from the OS is a refusal that ALREADY
happened (the system is out; without intervention nothing changes on its
own). Deeper: fd exhaustion almost always means there IS a leak — retry
without closing anything asks for an fd that does not exist, delaying the
leak's discovery while pressing the same limit. Permanent forces the leak to
surface as a must-fix bug, exactly per `NodeError::Internal`'s contract:
'Every occurrence must become a test.' D8 metrics watch the row-5 EMFILE
hits. (Supersedes the v1.3 convergence rule, which wrongly predicted both
paths would answer TRUE after stabilization.)

## 10. Review request

- 2 formal #922C reviews IN PARALLEL (reviewers: matt assigns; suggested: agent10 for From-table conformance vs his checklist domain + one independent).
- Merge at quorum + @fern authorization sentence. Agent1 applies (sole writer).
- Reviewers please rule explicitly on the 2 flagged EXT items (EROFS→Permanent, ETIMEDOUT→Transient) and the row-5 default direction.
