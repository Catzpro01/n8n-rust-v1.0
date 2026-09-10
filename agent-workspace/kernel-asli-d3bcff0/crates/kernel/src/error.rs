//! Error types shared across the engine.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors produced by kernel-level operations (item access, spill I/O,
/// expression evaluation, credential lookup).
#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("index {index} out of bounds (len {len})")]
    IndexOutOfBounds { index: usize, len: usize },

    #[error("spill store i/o failed: {message}")]
    SpillIo {
        message: String,
        kind: std::io::ErrorKind,
    },

    #[error("codec `{codec}` failed: {reason}")]
    Codec {
        codec: &'static str,
        reason: String,
    },

    #[error("node `{node}` not found in prior outputs")]
    NodeNotFound { node: String },

    #[error("expression `{expr}` failed: {reason}")]
    Expression { expr: String, reason: String },

    #[error("invalid data: {message}")]
    Invalid { message: String },

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
}

/// Why a node failed.
///
/// The distinction between variants is not cosmetic — it drives retry policy
/// (D14/D57), quarantine (D69), and crash reconciliation (audit A-10).
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    /// Retryable: network blip, HTTP 429/5xx, transient lock.
    #[error("transient failure (retry_after={retry_after:?}): {message}")]
    Transient {
        message: String,
        retry_after: Option<std::time::Duration>,
    },

    /// Not retryable: HTTP 4xx, validation failure, auth failure.
    #[error("permanent failure [{code}]: {message}")]
    Permanent { message: String, code: ErrorCode },

    /// D58: the per-node deadline elapsed.
    #[error("timeout after {elapsed:?}")]
    Timeout { elapsed: std::time::Duration },

    /// D20: cooperative cancellation was observed.
    #[error("cancelled")]
    Cancelled,

    /// D72: the Memory/CPU Governor refused the allocation.
    ///
    /// IMPORTANT (audit A-07): this is a *normal, expected* error, not a bug.
    /// Nodes must return it rather than attempt the allocation and OOM.
    /// It is the only mechanism by which a "hard RAM budget" can actually be
    /// enforced from inside the process.
    #[error("resource exhausted: {resource} (requested {requested} bytes, {available} available)")]
    ResourceExhausted {
        resource: Resource,
        requested: u64,
        available: u64,
    },

    /// Expression evaluation failed.
    #[error("expression `{expr}` failed: {reason}")]
    Expression { expr: String, reason: String },

    /// A bug in the node implementation. Every occurrence must become a test.
    #[error("internal node error: {message}")]
    Internal { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Memory,
    Cpu,
    Disk,
    FileHandles,
    Network,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Resource::Memory => "memory",
            Resource::Cpu => "cpu",
            Resource::Disk => "disk",
            Resource::FileHandles => "file handles",
            Resource::Network => "network",
        };
        f.write_str(s)
    }
}

/// Stable, machine-readable error classification.
///
/// Kept as a string (not an int) so it survives serialization into the event
/// log and is greppable in structured logs (D63).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ErrorCode(pub String);

impl ErrorCode {
    pub const fn new() -> Self {
        Self(String::new())
    }
    pub fn of(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ErrorCode {
    fn default() -> Self {
        Self::of("unknown")
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl NodeError {
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

    pub fn is_terminal(&self) -> bool {
        matches!(self, NodeError::Cancelled)
    }
}

/// Sentinel for unknown byte quantities (B-1): grep-able fabrication.
/// Governor (D72) consumers MUST treat this as "unknown", never as "zero".
/// WARNING (agent3 #1254 3.1): zero is a VALID byte quantity (requested ==
/// available is meaningful); UNKNOWN is not. Never compute with this.
pub const UNKNOWN_BYTES: u64 = 0;

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
                    // NOTE (B-3/R33): EMFILE/ENFILE produce `TooManyOpenFiles`,
                    // which stable cannot name -> they fall to the default arm
                    // (Permanent). Pinned by the B-3 test in §6.1. R33: stays
                    // Permanent even if std stabilizes the name (post-refusal,
                    // not pre-check — see RFC §9).
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
                // D8: `From` is pure, so row-5 hits are counted CONSUMER-side
                // by `Permanent.code`. Metrics are OUT OF SCOPE for this RFC;
                // implementors SHOULD add Prometheus counters per code value
                // (agent3 #1254 4.2).
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
        // Row 2: disk exhaustion (B-1: message dropped by shape; B-2 resolved R32).
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
        // Row 4: known-transient set (D6: sound because spill writes are atomic).
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
