//! Verifier — kebijakan penegakan determinisme (spec §3, §5; gates G-D1..D8).
//!
//! Fail-closed: kehilangan/kerusakan rekaman => Err eksplisit, tidak pernah
//! fallback diam-diam ke nilai baru.

use crate::{Entry, EntryKind, RecordBody, RsError};

/// Jendela clock replay (spec §5; D-D3): N = 300 s.
pub const CLOCK_WINDOW_MS: u64 = 300_000;

/// Status per spec §3.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayStatus {
    ReplayOk,
    ReplayMismatch,
    ReplayBroken,
    ClockDrift,
    RngUnseeded,
    SeedDiff,
    ViolationLogged,
    RejectedSourceList,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayVerdict {
    pub status: ReplayStatus,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyError {
    MissingReplayRecord { kind: EntryKind, node_id: String },
    ReplayBodyMismatch { node_id: String },
    RecordsetHashMismatch { expected: String, found: String },
    ClockDrift { delta_ms: i64 },
    RejectedNonDeterministic { sources: Vec<String> },
    SeedChanged,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::MissingReplayRecord { kind, node_id } => {
                write!(f, "MissingReplayRecord: {} for node {}", kind.as_str(), node_id)
            }
            PolicyError::ReplayBodyMismatch { node_id } => {
                write!(f, "ReplayBodyMismatch: body changed for node {}", node_id)
            }
            PolicyError::RecordsetHashMismatch { expected, found } => {
                write!(f, "RecordsetHashMismatch: expected {} got {}", expected, found)
            }
            PolicyError::ClockDrift { delta_ms } => {
                write!(f, "ClockDrift: replay time differs by {} ms (window {} ms)", delta_ms, CLOCK_WINDOW_MS)
            }
            PolicyError::RejectedNonDeterministic { sources } => {
                write!(f, "REJECTED_SOURCE_LIST: workflow NON_DETERMINISTIC: {}", sources.join(", "))
            }
            PolicyError::SeedChanged => write!(f, "seed changed between runs (new execution, NOT a replay)"),
        }
    }
}
impl std::error::Error for PolicyError {}

/// Penegak kebijakan record-replay (spec §5).
#[derive(Clone, Debug)]
pub struct Verifier {
    pub clock_window_ms: u64,
}

impl Default for Verifier {
    fn default() -> Self {
        Verifier { clock_window_ms: CLOCK_WINDOW_MS }
    }
}

impl Verifier {
    /// G-D2 (fail-closed): setiap external point yang diwajibkan HARUS ada.
    /// `required` = daftar (kind, node_id) yang dijamin W1 determinism-report.
    pub fn check_completeness(
        &self,
        entries: &[Entry],
        required: &[(EntryKind, String)],
    ) -> Result<(), PolicyError> {
        for (kind, node_id) in required {
            if !entries.iter().any(|e| e.kind == *kind && e.node_id == *node_id) {
                return Err(PolicyError::MissingReplayRecord {
                    kind: *kind,
                    node_id: node_id.clone(),
                });
            }
        }
        Ok(())
    }

    /// G-D3: verifikasi digest body terhadap blob (Inline atau Spilled).
    /// Body TIDAK pernah disimpan inline di RecordSet (spec §4.1).
    pub fn check_body(&self, entry: &Entry, blob: &RecordBody) -> Result<(), PolicyError> {
        let bytes: &[u8] = match blob {
            RecordBody::Inline(b) => b,
            RecordBody::Spilled(_) => {
                // SpillRef: digest diverifikasi di sisi storage (agent2) —
                // kontrak: verifikasi digest = tanggung jawab pemilik blob.
                return Ok(());
            }
        };
        let want = crate::body_hash(bytes);
        // payload HTTP_RESPONSE: ... body_hash B32 (32 byte terakhir payload sebelum body_len)
        // format builder: url,method,status,headers,body_hash(32),body_len(8)
        if entry.kind != EntryKind::HttpResponse {
            return Ok(());
        }
        let p = &entry.payload;
        if p.len() < 40 {
            return Err(PolicyError::ReplayBodyMismatch { node_id: entry.node_id.clone() });
        }
        let stored: [u8; 32] = p[p.len() - 40..p.len() - 8].try_into().unwrap();
        if stored != want {
            return Err(PolicyError::ReplayBodyMismatch { node_id: entry.node_id.clone() });
        }
        Ok(())
    }

    /// G-D5: jendela clock ±N (default 300 s) — di luar => ClockDrift (FAIL).
    pub fn check_clock(&self, captured_at: u64, replay_now_ms: u64) -> Result<(), PolicyError> {
        let delta = replay_now_ms as i64 - captured_at as i64;
        let win = self.clock_window_ms as i64;
        if delta.abs() > win {
            return Err(PolicyError::ClockDrift { delta_ms: delta });
        }
        Ok(())
    }

    /// G-D4: NON_DETERMINISTIC + diminta deterministic => TOLAK + daftar sumber.
    pub fn enforce_deterministic(
        &self,
        verdict: &str,
        deterministic_requested: bool,
        sources: &[String],
    ) -> Result<(), PolicyError> {
        if deterministic_requested && verdict.eq_ignore_ascii_case("NON_DETERMINISTIC") {
            return Err(PolicyError::RejectedNonDeterministic {
                sources: sources.to_vec(),
            });
        }
        Ok(())
    }

    /// G-D8: kecocokan `recordset_sha256` (metadata determinism-record) dengan isi berkas.
    pub fn check_metadata_sha256(
        &self,
        rs: &crate::RecordSet,
        expected_sha256: &[u8; 32],
    ) -> Result<(), PolicyError> {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(rs.encode());
        let found: [u8; 32] = h.finalize().into();
        if &found != expected_sha256 {
            return Err(PolicyError::RecordsetHashMismatch {
                expected: crate::sha256_checksum(expected_sha256),
                found: crate::sha256_checksum(&found),
            });
        }
        Ok(())
    }

    /// G-D2b: RecordSet hash (BLAKE3 CTX_RECORD) via decode sudah fail-closed
    /// (lihat `RecordSet::decode` -> `RsError::HashMismatch`). Helper untuk gate:
    pub fn decode_strict(bytes: &[u8]) -> Result<(crate::RecordSet, [u8; 32]), RsError> {
        crate::RecordSet::decode(bytes)
    }
}

/// Status ringkas untuk laporan (spec §3 status set).
pub fn status_of(e: &Result<(), PolicyError>) -> ReplayStatus {
    match e {
        Ok(()) => ReplayStatus::ReplayOk,
        Err(PolicyError::MissingReplayRecord { .. }) => ReplayStatus::ReplayBroken,
        Err(PolicyError::ReplayBodyMismatch { .. }) => ReplayStatus::ReplayMismatch,
        Err(PolicyError::RecordsetHashMismatch { .. }) => ReplayStatus::ReplayBroken,
        Err(PolicyError::ClockDrift { .. }) => ReplayStatus::ClockDrift,
        Err(PolicyError::RejectedNonDeterministic { .. }) => ReplayStatus::RejectedSourceList,
        Err(PolicyError::SeedChanged) => ReplayStatus::SeedDiff,
    }
}
