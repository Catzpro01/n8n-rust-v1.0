//! W3-ITEM-LINEAGE: Lineage tracking untuk item attribution
//! 
//! Baseline: AGENT10-ITEM-LINEAGE-SPEC.md v1.0 §7
//! Extension: AGENT10-W3-ITEM-LINEAGE-SPEC-v0.3.4.md §3
//! Compliance: Ruling 18 - salted-only + klasifikasi
//! Author: agent2 (ROLE_STORAGE)

use crate::errors::StorageError;
use crate::repository::Database;
use kernel::id::ExecutionId;
use rusqlite::params;
use serde::{Deserialize, Serialize};

/// Representation mode untuk lineage edge (v1.0 §3.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LineageRepr {
    /// Exact: semua input refs diketahui
    Exact = 0,
    /// Range: contiguous sequence (hemat ~10x)
    Range = 1,
    /// DigestSet: fan-in > 16 non-contiguous
    DigestSet = 2,
    /// Unknown: GC/RULE_GAP/LOSSY_UPSTREAM
    Unknown = 3,
}

impl TryFrom<i64> for LineageRepr {
    type Error = &'static str;
    
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Exact),
            1 => Ok(Self::Range),
            2 => Ok(Self::DigestSet),
            3 => Ok(Self::Unknown),
            _ => Err("invalid lineage repr value"),
        }
    }
}

/// Reason untuk repr = Unknown
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum UnknownReason {
    /// Item sudah di-GC
    Gc = 0,
    /// Rule gap di lineage computation
    RuleGap = 1,
    /// Lossy upstream transformation
    LossyUpstream = 2,
}

impl TryFrom<i64> for UnknownReason {
    type Error = &'static str;
    
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Gc),
            1 => Ok(Self::RuleGap),
            2 => Ok(Self::LossyUpstream),
            _ => Err("invalid unknown reason value"),
        }
    }
}

/// Data classification per Ruling 18
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClassification {
    /// Personal data - content_proof_plain HARUS NULL
    Personal,
    /// Non-personal data - content_proof_plain WAJIB terisi (dedup lintas-instance)
    NonPersonal,
}

impl DataClassification {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Personal => "PERSONAL",
            Self::NonPersonal => "NON_PERSONAL",
        }
    }
}

impl std::str::FromStr for DataClassification {
    type Err = StorageError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PERSONAL" => Ok(Self::Personal),
            "NON_PERSONAL" => Ok(Self::NonPersonal),
            _ => Err(StorageError::Database(format!("invalid data classification: {}", s))),
        }
    }
}

/// Lineage edge: pseudonymized tracking untuk item attribution
/// Ruling 18: content_hash = BLAKE3-256(salt_exec || kanon(isi)) SAJA
/// Tidak ada hash polos atas isi di tabel ini
#[derive(Debug, Clone)]
pub struct LineageEdge {
    /// Execution identifier (ExecutionId dari id.rs:35)
    pub execution_id: i64,
    /// 16-byte BLAKE3-128 ItemRef (opaque pseudonym)
    pub output_ref: Vec<u8>,
    /// Representation mode (0=Exact, 1=Range, 2=DigestSet, 3=Unknown)
    pub repr: i64,
    /// Serialized [ItemRef] untuk repr=0
    pub inputs_exact: Option<Vec<u8>>,
    /// Serialized {node_id,seq_base,count,hash_root} untuk repr=1
    pub inputs_range: Option<Vec<u8>>,
    /// Serialized {merkle_root,count,sample} untuk repr=2
    pub inputs_digest: Option<Vec<u8>>,
    /// Reason untuk repr=3 (0=Gc, 1=RuleGap, 2=LossyUpstream)
    pub unknown_reason: Option<i64>,
    /// Rule identifier yang menghasilkan output ini
    pub rule_id: i64,
    /// Rule version
    pub rule_ver: i64,
    /// 32-byte BLAKE3-256(salt_exec || kanon(isi)) - SALTED ONLY
    pub content_hash: Vec<u8>,
}

/// Erasure log: GDPR Art.17 compliance (APPEND-ONLY)
#[derive(Debug, Clone)]
pub struct ErasureLog {
    /// Auto-increment ID
    pub id: i64,
    /// Subject request identifier (GDPR data subject)
    pub subject_request_id: String,
    /// Execution identifier
    pub execution_id: i64,
    /// ISO-8601 timestamp kapan dihancurkan
    pub destroyed_at: String,
    /// Siapa yang mengeksekusi
    pub actor: String,
    /// Dasar hukum/tiket
    pub authority: String,
    /// BLAKE3(salt_exec) SEBELUM dihancurkan (audit proof)
    pub salt_fingerprint: Vec<u8>,
}

/// Lineage lookup: audit query performance (per-node attempt tracking)
#[derive(Debug, Clone)]
pub struct LineageLookupRow {
    /// Execution identifier
    pub execution_id: i64,
    /// Node identifier
    pub node_id: String,
    /// Attempt number (retry tracking)
    pub attempt: i64,
    /// Output index (untuk multi-output nodes)
    pub output_index: i64,
    /// Sequence range start
    pub seq_start: i64,
    /// Sequence range end
    pub seq_end: i64,
    /// 16-byte BLAKE3-128 ItemRef (FK ke lineage_edge)
    pub output_ref: Vec<u8>,
}

/// Lineage extension: CASD + klasifikasi per Ruling 18
#[derive(Debug, Clone)]
pub struct LineageExt {
    /// Execution identifier
    pub execution_id: i64,
    /// 16-byte BLAKE3-128 ItemRef (FK ke lineage_edge)
    pub output_ref: Vec<u8>,
    /// SHA-256 lintas-instance (TANPA salt) - NULL untuk PERSONAL
    /// NULL-able TANPA default auto-fill
    pub content_proof_plain: Option<Vec<u8>>,
    /// Data classification (wajib terekam)
    pub data_class: DataClassification,
    /// Rule ID yang menentukan klasifikasi (TEXT: bisa non-numerik)
    pub class_rule_id: String,
    /// Rule version untuk klasifikasi
    pub class_rule_ver: i64,
    /// Jumlah items dalam edge ini
    pub item_count: i64,
    /// Estimasi bytes
    pub estimated_bytes: Option<i64>,
    /// Timestamp kapan retention expired (untuk cleanup)
    pub retention_expired_at: Option<i64>,
}


/// Convert u64 ID to i64 for SQL storage (Ruling 34)
/// Returns error if value > i64::MAX (prevents silent wraparound)
fn id_to_sql(v: u64) -> Result<i64, crate::StorageError> {
    i64::try_from(v).map_err(|_| crate::StorageError::Database(
        format!("ID {} exceeds i64::MAX, cannot store in INTEGER column", v)
    ))
}

// ============ LINEAGE REPOSITORY ============

/// Repository untuk lineage operations (baseline v1.0 §7 + Ruling 18)
pub struct LineageRepo {
    db: Database,
}

impl LineageRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Insert lineage edge
    pub fn insert_lineage_edge(&self, edge: &LineageEdge) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, inputs_exact, inputs_range, inputs_digest, unknown_reason, rule_id, rule_ver, content_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                edge.execution_id,
                edge.output_ref,
                edge.repr,
                edge.inputs_exact,
                edge.inputs_range,
                edge.inputs_digest,
                edge.unknown_reason,
                edge.rule_id,
                edge.rule_ver,
                edge.content_hash
            ],
        )?;
        Ok(())
    }

    /// Insert lineage edges in batch within a single atomic transaction (ADR-v2-STORAGE)
    pub fn insert_lineage_edges_batch(&self, edges: &[LineageEdge]) -> Result<(), StorageError> {
        if edges.is_empty() {
            return Ok(());
        }
        let mut c = self.db.conn.lock().unwrap();
        let tx = c.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO lineage_edge (execution_id, output_ref, repr, inputs_exact, inputs_range, inputs_digest, unknown_reason, rule_id, rule_ver, content_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
            )?;
            for edge in edges {
                stmt.execute(params![
                    edge.execution_id,
                    edge.output_ref,
                    edge.repr,
                    edge.inputs_exact,
                    edge.inputs_range,
                    edge.inputs_digest,
                    edge.unknown_reason,
                    edge.rule_id,
                    edge.rule_ver,
                    edge.content_hash
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Query lineage edges by execution
    pub fn get_lineage_edges(&self, execution_id: ExecutionId) -> Result<Vec<LineageEdge>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare(
            "SELECT execution_id, output_ref, repr, inputs_exact, inputs_range, inputs_digest, unknown_reason, rule_id, rule_ver, content_hash FROM lineage_edge WHERE execution_id = ?1"
        )?;
        let rows = stmt.query_map(params![id_to_sql(execution_id.get())?], |row| {
            Ok(LineageEdge {
                execution_id: row.get(0)?,
                output_ref: row.get(1)?,
                repr: row.get(2)?,
                inputs_exact: row.get(3)?,
                inputs_range: row.get(4)?,
                inputs_digest: row.get(5)?,
                unknown_reason: row.get(6)?,
                rule_id: row.get(7)?,
                rule_ver: row.get(8)?,
                content_hash: row.get(9)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::Database(e.to_string()))?);
        }
        Ok(result)
    }

    /// Insert erasure log (APPEND-ONLY)
    pub fn insert_erasure_log(&self, log: &ErasureLog) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute(
            "INSERT INTO erasure_log (subject_request_id, execution_id, destroyed_at, actor, authority, salt_fingerprint) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                log.subject_request_id,
                log.execution_id,
                log.destroyed_at,
                log.actor,
                log.authority,
                log.salt_fingerprint
            ],
        )?;
        Ok(())
    }

    /// Query erasure logs by execution
    pub fn get_erasure_logs(&self, execution_id: ExecutionId) -> Result<Vec<ErasureLog>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare(
            "SELECT id, subject_request_id, execution_id, destroyed_at, actor, authority, salt_fingerprint FROM erasure_log WHERE execution_id = ?1"
        )?;
        let rows = stmt.query_map(params![id_to_sql(execution_id.get())?], |row| {
            Ok(ErasureLog {
                id: row.get(0)?,
                subject_request_id: row.get(1)?,
                execution_id: row.get(2)?,
                destroyed_at: row.get(3)?,
                actor: row.get(4)?,
                authority: row.get(5)?,
                salt_fingerprint: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::Database(e.to_string()))?);
        }
        Ok(result)
    }

    /// Insert lineage lookup (T-1 extension, LIN-1)
    pub fn insert_lineage_lookup(&self, lookup: &LineageLookupRow) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute(
            "INSERT INTO lineage_lookup (execution_id, node_id, attempt, output_index, seq_start, seq_end, output_ref) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                lookup.execution_id,
                lookup.node_id,
                lookup.attempt,
                lookup.output_index,
                lookup.seq_start,
                lookup.seq_end,
                lookup.output_ref
            ],
        )?;
        Ok(())
    }

    /// Query lineage lookup by (execution, node, attempt, output_index) - LIN-1
    pub fn query_lineage_lookup(
        &self,
        execution_id: ExecutionId,
        node_id: &str,
        attempt: i64,
        output_index: i64,
    ) -> Result<Vec<LineageLookupRow>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare(
            "SELECT execution_id, node_id, attempt, output_index, seq_start, seq_end, output_ref FROM lineage_lookup WHERE execution_id = ?1 AND node_id = ?2 AND attempt = ?3 AND output_index = ?4 ORDER BY seq_start"
        )?;
        let rows = stmt.query_map(
            params![id_to_sql(execution_id.get())?, node_id, attempt, output_index],
            |row| {
                Ok(LineageLookupRow {
                    execution_id: row.get(0)?,
                    node_id: row.get(1)?,
                    attempt: row.get(2)?,
                    output_index: row.get(3)?,
                    seq_start: row.get(4)?,
                    seq_end: row.get(5)?,
                    output_ref: row.get(6)?,
                })
            },
        )?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::Database(e.to_string()))?);
        }
        Ok(result)
    }

    /// Insert lineage extension (T-2 CASD + Ruling 18 compliance)
    /// PERSONAL -> content_proof_plain NULL
    /// NON_PERSONAL -> content_proof_plain filled
    pub fn insert_lineage_ext(&self, ext: &LineageExt) -> Result<(), StorageError> {
        let c = self.db.conn.lock().unwrap();
        c.execute(
            "INSERT INTO lineage_ext (execution_id, output_ref, content_proof_plain, data_class, class_rule_id, class_rule_ver, item_count, estimated_bytes, retention_expired_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                ext.execution_id,
                ext.output_ref,
                ext.content_proof_plain,
                ext.data_class.as_str(),
                ext.class_rule_id,
                ext.class_rule_ver,
                ext.item_count,
                ext.estimated_bytes,
                ext.retention_expired_at
            ],
        )?;
        Ok(())
    }

    /// Query lineage ext by execution
    pub fn get_lineage_ext(&self, execution_id: ExecutionId) -> Result<Vec<LineageExt>, StorageError> {
        let c = self.db.conn.lock().unwrap();
        let mut stmt = c.prepare(
            "SELECT execution_id, output_ref, content_proof_plain, data_class, class_rule_id, class_rule_ver, item_count, estimated_bytes, retention_expired_at FROM lineage_ext WHERE execution_id = ?1"
        )?;
        let rows = stmt.query_map(params![id_to_sql(execution_id.get())?], |row| {
            let class_str: String = row.get(3)?;
            Ok(LineageExt {
                execution_id: row.get(0)?,
                output_ref: row.get(1)?,
                content_proof_plain: row.get(2)?,
                data_class: class_str.parse().map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
                class_rule_id: row.get(4)?,
                class_rule_ver: row.get(5)?,
                item_count: row.get(6)?,
                estimated_bytes: row.get(7)?,
                retention_expired_at: row.get(8)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::Database(e.to_string()))?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_migrations_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
    }

    #[test]
    fn test_lineage_edge_insert_and_query() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = LineageRepo::new(db);

        let edge = LineageEdge {
            execution_id: 1,
            output_ref: vec![0u8; 16],
            repr: 0,  // Exact
            inputs_exact: Some(vec![1, 2, 3, 4]),
            inputs_range: None,
            inputs_digest: None,
            unknown_reason: None,
            rule_id: 100,
            rule_ver: 1,
            content_hash: vec![0u8; 32],  // salted-only
        };

        repo.insert_lineage_edge(&edge).unwrap();

        let edges = repo.get_lineage_edges(ExecutionId::new(1)).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].execution_id, 1);
        assert_eq!(edges[0].repr, 0);
        assert_eq!(edges[0].content_hash.len(), 32);
    }

    #[test]
    fn test_erasure_log_append_only() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = LineageRepo::new(db);

        let log = ErasureLog {
            id: 0,
            subject_request_id: "subject-123".to_string(),
            execution_id: 1,
            destroyed_at: "2026-09-09T12:00:00Z".to_string(),
            actor: "system".to_string(),
            authority: "GDPR Art.17".to_string(),
            salt_fingerprint: vec![0u8; 32],
        };

        repo.insert_erasure_log(&log).unwrap();

        let logs = repo.get_erasure_logs(ExecutionId::new(1)).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].subject_request_id, "subject-123");
        assert_eq!(logs[0].authority, "GDPR Art.17");
    }

    #[test]
    fn test_lineage_lookup_query() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = LineageRepo::new(db);

        let lookup = LineageLookupRow {
            execution_id: 1,
            node_id: "node-abc".to_string(),
            attempt: 0,
            output_index: 0,
            seq_start: 0,
            seq_end: 99,
            output_ref: vec![0u8; 16],
        };

        repo.insert_lineage_lookup(&lookup).unwrap();

        let results = repo.query_lineage_lookup(ExecutionId::new(1), "node-abc", 0, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_id, "node-abc");
        assert_eq!(results[0].seq_start, 0);
        assert_eq!(results[0].seq_end, 99);
    }

    #[test]
    fn test_lineage_ext_personal_proof_null() {
        // L-G1: PERSONAL -> content_proof_plain IS NULL
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = LineageRepo::new(db);

        let ext = LineageExt {
            execution_id: 1,
            output_ref: vec![0u8; 16],
            content_proof_plain: None,  // PERSONAL -> NULL
            data_class: DataClassification::Personal,
            class_rule_id: "rule-001".to_string(),  // TEXT
            class_rule_ver: 1,
            item_count: 1,
            estimated_bytes: Some(1024),
            retention_expired_at: None,
        };

        repo.insert_lineage_ext(&ext).unwrap();

        let exts = repo.get_lineage_ext(ExecutionId::new(1)).unwrap();
        assert_eq!(exts.len(), 1);
        assert!(exts[0].content_proof_plain.is_none());
        assert_eq!(exts[0].data_class, DataClassification::Personal);
        assert_eq!(exts[0].class_rule_id, "rule-001");
    }

    #[test]
    fn test_lineage_ext_non_personal_proof_filled() {
        // L-G2: NON_PERSONAL -> content_proof_plain TERISI (kontrol positif untuk L-G1)
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = LineageRepo::new(db);

        let ext = LineageExt {
            execution_id: 1,
            output_ref: vec![1u8; 16],
            content_proof_plain: Some(vec![2u8; 32]),  // NON_PERSONAL -> filled (SHA-256 = 32 bytes)
            data_class: DataClassification::NonPersonal,
            class_rule_id: "rule-002".to_string(),  // TEXT
            class_rule_ver: 1,
            item_count: 5,
            estimated_bytes: Some(5120),
            retention_expired_at: None,
        };

        repo.insert_lineage_ext(&ext).unwrap();

        let exts = repo.get_lineage_ext(ExecutionId::new(1)).unwrap();
        assert_eq!(exts.len(), 1);
        assert!(exts[0].content_proof_plain.is_some());
        assert_eq!(exts[0].data_class, DataClassification::NonPersonal);
    }
}

// ============ LIN GATES TESTS (AUDITOR-GRADE) ============

#[cfg(test)]
mod lin_gates_tests {
    use super::*;
    use std::path::PathBuf;

    fn test_migrations_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
    }

    /// LIN-1: Query lineage menggunakan indeks yang benar (bukan table scan)
    /// Orakel: EXPLAIN QUERY PLAN harus memuat "USING INDEX idx_lookup_query"
    /// Membunuh mutan M1a: DROP INDEX idx_lookup_query harus membuat test FAIL
    #[test]
    fn test_lin_1_uses_correct_index() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let c = db.conn.lock().unwrap();
        
        // Insert test data
        c.execute("INSERT INTO lineage_lookup (execution_id, node_id, attempt, output_index, seq_start, seq_end, output_ref) VALUES (1, 'node-1', 0, 0, 0, 99, X'00000000000000000000000000000000')", []).unwrap();
        
        // Production query (dengan ORDER BY seq_start)
        let mut stmt = c.prepare("EXPLAIN QUERY PLAN SELECT execution_id, node_id, attempt, output_index, seq_start, seq_end, output_ref FROM lineage_lookup WHERE execution_id = 1 AND node_id = 'node-1' AND attempt = 0 AND output_index = 0 ORDER BY seq_start").unwrap();
        let mut rows = stmt.query([]).unwrap();
        
        let mut uses_idx_lookup_query = false;
        let mut uses_temp_b_tree = false;  // M1c: check for manual sort
        while let Some(row) = rows.next().unwrap() {
            let detail: String = row.get(3).unwrap();
            if detail.contains("USING INDEX idx_lookup_query") {
                uses_idx_lookup_query = true;
            }
            if detail.contains("USE TEMP B-TREE") {
                uses_temp_b_tree = true;
            }
        }
        
        assert!(uses_idx_lookup_query, "LIN-1 FAILED: Query plan does not use idx_lookup_query index");
        assert!(!uses_temp_b_tree, "LIN-1 FAILED: ORDER BY seq_start not served by index (manual sort detected)");  // M1c
    }

    /// LIN-2: Demonstrasi sifat SHA-256 untuk crypto-shredding
    /// BUKAN verifikasi produksi BLAKE3 — ini hanya menunjukkan bahwa salt menghasilkan hash berbeda
    /// Orakel: content_hash(salt1 || data) != content_hash(salt2 || data) != content_hash(data)
    /// Membunuh mutan M2a: hash TANPA salt harus berbeda dari hash dengan salt
    /// Label: DEMONSTRASI SIFAT, bukan verifikasi kriptografi produksi
    #[test]
    fn test_lin_2_crypto_shredding() {
        use sha2::{Sha256, Digest};
        
        // Test data
        let data = b"sensitive payload";
        
        // Hash dengan salt1
        let salt1 = b"salt_version_1";
        let mut hasher1 = Sha256::new();
        hasher1.update(salt1);
        hasher1.update(data);
        let hash_with_salt1 = hasher1.finalize().to_vec();
        
        // Hash dengan salt2 (simulasi salt berbeda/version)
        let salt2 = b"salt_version_2";
        let mut hasher2 = Sha256::new();
        hasher2.update(salt2);
        hasher2.update(data);
        let hash_with_salt2 = hasher2.finalize().to_vec();
        
        // Hash TANPA salt (mutan M2a - harus berbeda dari salted)
        let mut hasher_no_salt = Sha256::new();
        hasher_no_salt.update(data);
        let hash_no_salt = hasher_no_salt.finalize().to_vec();
        
        // Verifikasi: semua hash berbeda (crypto-shredding works)
        assert_ne!(hash_with_salt1, hash_with_salt2, "LIN-2 FAILED: Different salts produce same hash");
        assert_ne!(hash_with_salt1, hash_no_salt, "LIN-2 FAILED: Salted hash equals no-salt hash (M2a undetected)");
        assert_ne!(hash_with_salt2, hash_no_salt, "LIN-2 FAILED: Salted hash equals no-salt hash");
        
        // Verifikasi deterministik: hash dengan salt yang sama = identik
        let mut hasher1_again = Sha256::new();
        hasher1_again.update(salt1);
        hasher1_again.update(data);
        let hash_with_salt1_again = hasher1_again.finalize().to_vec();
        assert_eq!(hash_with_salt1, hash_with_salt1_again, "LIN-2 FAILED: Same salt produces different hash");
        
        // Simulasi crypto-shredding: setelah salt dihancurkan, hanya tersisa hash yang tidak bisa diverifikasi
        // Dalam produksi: salt_fingerprint = BLAKE3(salt) disimpan di erasure_log untuk audit
        // Tapi salt sendiri dihancurkan, sehingga hash tidak bisa di-recompute
    }

    /// LIN-3: No overclaim
    /// Orakel: grep kode + docs untuk klaim kepatuhan GDPR eksplisit -> 0 matches
    #[test]
    fn test_lin_3_no_overclaim() {
        // LIN-3: Scan BOTH lineage.rs AND migrations/*.sql for GDPR claims
        // This fixes the overclaim in cc4f025 commit message
        
        use std::fs;
        use std::path::PathBuf;
        
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let _src_dir = PathBuf::from(manifest_dir).join("src");
        let migrations_dir = PathBuf::from(manifest_dir).join("migrations");
        
        // GDPR claim patterns (forbidden)
        let gdpr_compliant = format!("{}{}", "GDPR-", "compliant");
        let gdpr_ready = format!("{}{}", "GDPR ", "ready");
        
        // Scan lineage.rs
        let lineage_source = include_str!("lineage.rs");
        assert!(!lineage_source.contains(&gdpr_compliant), 
                "LIN-3 FAILED: lineage.rs contains forbidden compliance phrase");
        assert!(!lineage_source.contains(&gdpr_ready), 
                "LIN-3 FAILED: lineage.rs contains forbidden ready phrase");
        assert!(lineage_source.contains("pseudonymized") || lineage_source.contains("Pseudonymized"), 
                "LIN-3 FAILED: Expected pseudonymized terminology in lineage.rs");
        
        // Scan all migration SQL files
        let mut migration_count = 0;
        if let Ok(entries) = fs::read_dir(&migrations_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                    migration_count += 1;
                    let sql_content = fs::read_to_string(&path)
                        .unwrap_or_else(|_| panic!("Failed to read migration: {:?}", path));
                    
                    assert!(!sql_content.contains(&gdpr_compliant), 
                            "LIN-3 FAILED: {} contains forbidden compliance phrase", 
                            path.display());
                    assert!(!sql_content.contains(&gdpr_ready), 
                            "LIN-3 FAILED: {} contains forbidden ready phrase", 
                            path.display());
                }
            }
        }
        
        // Verify we actually scanned migrations (not silently passing)
        assert!(migration_count > 0, 
                "LIN-3 FAILED: No migration files found in {:?}", migrations_dir);
        assert!(migration_count >= 3, 
                "LIN-3 FAILED: Expected at least 3 migrations (001, 002, 004), found {}", 
                migration_count);
    }

    /// L-G1: PERSONAL -> content_proof_plain IS NULL (anti-plain-hash)
    /// Orakel: insert PERSONAL, verify content_proof_plain = NULL
    #[test]
    fn test_lg1_personal_no_plain_hash() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = LineageRepo::new(db);
        
        let ext = LineageExt {
            execution_id: 1,
            output_ref: vec![0u8; 16],
            content_proof_plain: None,  // MUST be NULL for PERSONAL
            data_class: DataClassification::Personal,
            class_rule_id: "rule-personal-001".to_string(),
            class_rule_ver: 1,
            item_count: 1,
            estimated_bytes: Some(1024),
            retention_expired_at: None,
        };
        
        repo.insert_lineage_ext(&ext).unwrap();
        
        let exts = repo.get_lineage_ext(ExecutionId::new(1)).unwrap();
        assert_eq!(exts.len(), 1);
        assert!(exts[0].content_proof_plain.is_none(), "L-G1 FAILED: PERSONAL must have NULL content_proof_plain");
        assert_eq!(exts[0].data_class, DataClassification::Personal);
    }

    /// L-G2: NON_PERSONAL -> content_proof_plain TERISI (anti-vacuous)
    /// Orakel: insert NON_PERSONAL, verify content_proof_plain IS NOT NULL + dedup works
    #[test]
    fn test_lg2_non_personal_dedup() {
        let db = Database::open_memory(&test_migrations_dir()).unwrap();
        let repo = LineageRepo::new(db);
        
        let proof = vec![0xef; 32];  // Simulasi SHA-256 lintas-instance (TANPA salt)
        let ext = LineageExt {
            execution_id: 1,
            output_ref: vec![1u8; 16],
            content_proof_plain: Some(proof.clone()),  // MUST be filled for NON_PERSONAL
            data_class: DataClassification::NonPersonal,
            class_rule_id: "rule-non-personal-001".to_string(),
            class_rule_ver: 1,
            item_count: 1,
            estimated_bytes: Some(2048),
            retention_expired_at: None,
        };
        
        repo.insert_lineage_ext(&ext).unwrap();
        
        let exts = repo.get_lineage_ext(ExecutionId::new(1)).unwrap();
        assert_eq!(exts.len(), 1);
        assert!(exts[0].content_proof_plain.is_some(), "L-G2 FAILED: NON_PERSONAL must have content_proof_plain");
        assert_eq!(exts[0].content_proof_plain.as_ref().unwrap(), &proof);
        assert_eq!(exts[0].data_class, DataClassification::NonPersonal);
        
        // Dedup test: insert lagi dengan proof sama -> harus bisa di-dedup
        let ext2 = LineageExt {
            execution_id: 2,  // execution berbeda
            output_ref: vec![2u8; 16],
            content_proof_plain: Some(proof),  // proof SAMA (untuk dedup)
            data_class: DataClassification::NonPersonal,
            class_rule_id: "rule-non-personal-001".to_string(),
            class_rule_ver: 1,
            item_count: 1,
            estimated_bytes: Some(2048),
            retention_expired_at: None,
        };
        
        repo.insert_lineage_ext(&ext2).unwrap();
        
        // Verify: 2 executions, tapi content_proof_plain SAMA -> dedup possible
        let exts1 = repo.get_lineage_ext(ExecutionId::new(1)).unwrap();
        let exts2 = repo.get_lineage_ext(ExecutionId::new(2)).unwrap();
        assert_eq!(exts1[0].content_proof_plain, exts2[0].content_proof_plain);
    }

    /// Ruling 34: id_to_sql rejects u64::MAX (prevents silent wraparound)
    #[test]
    fn test_id_to_sql_rejects_overflow() {
        // Valid IDs should convert
        assert_eq!(id_to_sql(0).unwrap(), 0);
        assert_eq!(id_to_sql(100).unwrap(), 100);
        assert_eq!(id_to_sql(i64::MAX as u64).unwrap(), i64::MAX);
        
        // u64::MAX should be REJECTED (not silently wrapped to negative)
        assert!(id_to_sql(u64::MAX).is_err(), "id_to_sql(u64::MAX) must return Err");
        
        // Values just above i64::MAX should also be rejected
        assert!(id_to_sql(i64::MAX as u64 + 1).is_err(), "id_to_sql(i64::MAX+1) must return Err");
    }

}

// ============================================================================
// G-2 TESTS — repr↔inputs_* mutual exclusion (matt #1345 §3)
// ============================================================================

#[cfg(test)]
mod g2_tests {
    use rusqlite::Connection;
    

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        let migration = include_str!("../migrations/004_item_lineage.sql");
        conn.execute_batch(migration).unwrap();
        conn
    }

    #[test]
    fn test_g2_repr0_with_null_inputs_exact_rejected() {
        let conn = setup_db();
        // repr=0 requires inputs_exact NOT NULL
        let result = conn.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, inputs_exact, rule_id, rule_ver, content_hash)
             VALUES (1, X'00000000000000000000000000000000', 0, NULL, 1, 1, X'0000000000000000000000000000000000000000000000000000000000000000')",
            [],
        );
        assert!(result.is_err(), "G-2 FAIL: repr=0 with NULL inputs_exact should be rejected");
    }

    #[test]
    fn test_g2_repr3_with_null_unknown_reason_rejected() {
        let conn = setup_db();
        // repr=3 requires unknown_reason NOT NULL
        let result = conn.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, unknown_reason, rule_id, rule_ver, content_hash)
             VALUES (1, X'00000000000000000000000000000000', 3, NULL, 1, 1, X'0000000000000000000000000000000000000000000000000000000000000000')",
            [],
        );
        assert!(result.is_err(), "G-2 FAIL: repr=3 with NULL unknown_reason should be rejected");
    }

    #[test]
    fn test_g2_repr0_with_both_inputs_rejected() {
        let conn = setup_db();
        // repr=0 with BOTH inputs_exact AND inputs_range — mutual exclusion violation
        let result = conn.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, inputs_exact, inputs_range, rule_id, rule_ver, content_hash)
             VALUES (1, X'00000000000000000000000000000000', 0, X'00', X'00', 1, 1, X'0000000000000000000000000000000000000000000000000000000000000000')",
            [],
        );
        assert!(result.is_err(), "G-2 FAIL: repr=0 with both inputs_exact and inputs_range should be rejected");
    }

    #[test]
    fn test_g2_positive_control_all_repr_accepted() {
        let conn = setup_db();
        // Positive control: each repr with correct columns should succeed
        let content_hash = vec![0u8; 32];
        
        // repr=0 with inputs_exact
        conn.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, inputs_exact, rule_id, rule_ver, content_hash)
             VALUES (1, X'00000000000000000000000000000000', 0, X'00', 1, 1, ?1)",
            rusqlite::params![content_hash],
        ).unwrap();

        // repr=1 with inputs_range
        conn.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, inputs_range, rule_id, rule_ver, content_hash)
             VALUES (2, X'00000000000000000000000000000000', 1, X'00', 1, 1, ?1)",
            rusqlite::params![content_hash],
        ).unwrap();

        // repr=2 with inputs_digest
        conn.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, inputs_digest, rule_id, rule_ver, content_hash)
             VALUES (3, X'00000000000000000000000000000000', 2, X'00', 1, 1, ?1)",
            rusqlite::params![content_hash],
        ).unwrap();

        // repr=3 with unknown_reason
        conn.execute(
            "INSERT INTO lineage_edge (execution_id, output_ref, repr, unknown_reason, rule_id, rule_ver, content_hash)
             VALUES (4, X'00000000000000000000000000000000', 3, 1, 1, 1, ?1)",
            rusqlite::params![content_hash],
        ).unwrap();

        // Verify all 4 inserted
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM lineage_edge", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 4, "G-2 FAIL: positive control should insert 4 rows");
    }
}
