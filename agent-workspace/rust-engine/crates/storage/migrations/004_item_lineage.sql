-- W3-ITEM-LINEAGE Migration 004
-- Baseline: AGENT10-ITEM-LINEAGE-SPEC.md v1.0 §7
-- Extension: AGENT10-W3-ITEM-LINEAGE-SPEC-v0.3.4.md §3 (T-1 + T-2)
-- Compliance: Ruling 18 (Ruling 14 ditarik) - salted-only + klasifikasi
-- Owner: agent2 (ROLE_STORAGE)

-- ============================================================================
-- BASELINE v1.0 §7: lineage_edge (pseudonymized lineage tracking)
-- Ruling 18: content_hash = BLAKE3-256(salt_exec || kanon(isi)) SAJA
-- Tidak ada hash polos atas isi di tabel ini
-- ============================================================================
CREATE TABLE IF NOT EXISTS lineage_edge (
    execution_id   INTEGER NOT NULL,  -- ExecutionId (numeric_id from id.rs:35)
    output_ref     BLOB    NOT NULL,  -- 16-byte BLAKE3-128 (ItemRef)
    repr           INTEGER NOT NULL CHECK (
        (repr = 0 AND inputs_exact   IS NOT NULL AND inputs_range  IS NULL AND inputs_digest IS NULL AND unknown_reason IS NULL) OR
        (repr = 1 AND inputs_range   IS NOT NULL AND inputs_exact  IS NULL AND inputs_digest IS NULL AND unknown_reason IS NULL) OR
        (repr = 2 AND inputs_digest  IS NOT NULL AND inputs_exact  IS NULL AND inputs_range  IS NULL AND unknown_reason IS NULL) OR
        (repr = 3 AND unknown_reason IS NOT NULL AND inputs_exact  IS NULL AND inputs_range  IS NULL AND inputs_digest IS NULL)
    ),  -- G-2: repr↔inputs_* mutual exclusion (matt #1345 §3)
    inputs_exact   BLOB,              -- [ItemRef] serialized, repr=0 only
    inputs_range   BLOB,              -- {node_id,seq_base,count,hash_root}, repr=1 only
    inputs_digest  BLOB,              -- {merkle_root,count,sample}, repr=2 only
    unknown_reason INTEGER,           -- GC|RULE_GAP|LOSSY_UPSTREAM, repr=3 only
    rule_id        INTEGER NOT NULL,
    rule_ver       INTEGER NOT NULL,
    content_hash   BLOB    NOT NULL,  -- 32-byte BLAKE3-256(salt_exec || kanon(isi)) - SALTED ONLY
    PRIMARY KEY (execution_id, output_ref)
);

-- Index untuk fast lookup by execution
CREATE INDEX IF NOT EXISTS idx_lineage_edge_exec ON lineage_edge(execution_id);

-- G-4: Reverse lookup index (matt #1345 §4)
-- Cost asymmetry: free now (004 not in production), expensive later (high-volume table)
CREATE INDEX IF NOT EXISTS idx_lineage_edge_output_ref ON lineage_edge(output_ref);

-- ============================================================================
-- BASELINE v1.0 §7: erasure_log (GDPR Art.17 compliance, APPEND-ONLY)
-- ============================================================================
CREATE TABLE IF NOT EXISTS erasure_log (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_request_id TEXT  NOT NULL,
    execution_id     INTEGER NOT NULL,
    destroyed_at     TEXT    NOT NULL,  -- ISO-8601 timestamp
    actor            TEXT    NOT NULL,  -- siapa mengeksekusi
    authority        TEXT    NOT NULL,  -- dasar hukum/tiket
    salt_fingerprint BLOB    NOT NULL   -- BLAKE3(salt_exec) SEBELUM dihancurkan
);

-- Index untuk audit query
CREATE INDEX IF NOT EXISTS idx_erasure_log_exec ON erasure_log(execution_id);
CREATE INDEX IF NOT EXISTS idx_erasure_log_subject ON erasure_log(subject_request_id);

-- ============================================================================
-- EXTENSION T-1 (v0.3.4 §3): lineage_lookup (audit query performance)
-- ============================================================================
CREATE TABLE IF NOT EXISTS lineage_lookup (
    execution_id   INTEGER NOT NULL,
    node_id        TEXT    NOT NULL,
    attempt        INTEGER NOT NULL,
    output_index   INTEGER NOT NULL DEFAULT 0,
    seq_start      INTEGER NOT NULL,
    seq_end        INTEGER NOT NULL,
    output_ref     BLOB    NOT NULL,
    PRIMARY KEY (execution_id, output_ref)
);

-- Critical index untuk LIN-1: query per (node, attempt, output_index) tanpa table scan
CREATE INDEX IF NOT EXISTS idx_lookup_query ON lineage_lookup(
    execution_id, node_id, attempt, output_index, seq_start
);

-- ============================================================================
-- EXTENSION T-2: lineage_ext (CASD + klasifikasi per Ruling 18)
-- content_proof_plain = SHA-256 lintas-instance (TANPA salt) - NULL untuk PERSONAL
-- data_class + class_rule_id + class_rule_ver WAJIB terekam
-- content_proof_plain NULL-able TANPA default auto-fill
-- ============================================================================
CREATE TABLE IF NOT EXISTS lineage_ext (
    execution_id          INTEGER NOT NULL,
    output_ref            BLOB    NOT NULL,
    content_proof_plain   BLOB,              -- SHA-256 lintas-instance (TANPA salt) - NULL untuk PERSONAL
    data_class            TEXT    NOT NULL CHECK (data_class IN ('PERSONAL', 'NON_PERSONAL')),
    class_rule_id         TEXT    NOT NULL,  -- rule yang menentukan klasifikasi (TEXT: bisa non-numerik)
    class_rule_ver        INTEGER NOT NULL,  -- versi rule klasifikasi
    item_count            INTEGER NOT NULL DEFAULT 1,
    estimated_bytes       INTEGER,
    retention_expired_at  INTEGER,           -- untuk cleanup jobs (retensi terpisah D-L7)
    PRIMARY KEY (execution_id, output_ref),
    -- PERSONAL wajib content_proof_plain NULL; NON_PERSONAL wajib content_proof_plain terisi
    CHECK (
        (data_class = 'PERSONAL' AND content_proof_plain IS NULL)
        OR (data_class = 'NON_PERSONAL' AND content_proof_plain IS NOT NULL)
    )
);

-- Index untuk purge jobs
CREATE INDEX IF NOT EXISTS idx_ext_purge ON lineage_ext(execution_id) 
WHERE retention_expired_at IS NOT NULL;

-- Index untuk L-G2: query NON_PERSONAL untuk verifikasi dedup lintas-instance
CREATE INDEX IF NOT EXISTS idx_ext_proof ON lineage_ext(content_proof_plain)
WHERE data_class = 'NON_PERSONAL' AND content_proof_plain IS NOT NULL;
