-- Migration: Initial schema for n8n Rust engine
-- Author: agent2 (Backend Engineer)
-- Based on: AGENT2_STORAGE_SPEC.md
-- Ruling 28 compliance: numeric_id! types = INTEGER, NodeId/NodeKind = TEXT

-- Enable WAL mode
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT
);

-- Workflow definitions (Ruling 28: WorkflowId = INTEGER)
CREATE TABLE workflow (
    id INTEGER PRIMARY KEY,  -- WorkflowId (numeric_id!, id.rs:34)
    name TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    nodes_json TEXT NOT NULL,
    connections_json TEXT NOT NULL,
    settings_json TEXT NOT NULL DEFAULT '{}',
    static_data TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_workflow_active ON workflow(active);

-- Workflow versioning (for rollback support)
CREATE TABLE workflow_version (
    workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,  -- FK to WorkflowId
    version INTEGER NOT NULL,
    snapshot_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (workflow_id, version)
);

-- Encryption key registry (answers F20)
CREATE TABLE encryption_key (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version INTEGER NOT NULL UNIQUE,
    algorithm TEXT NOT NULL DEFAULT 'aes-256-gcm',
    key_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    rotated_from INTEGER REFERENCES encryption_key(id),
    description TEXT
);

-- Credentials with key versioning (credential.id tetap TEXT, bukan numeric_id!)
CREATE TABLE credential (
    id TEXT PRIMARY KEY,  -- credential ID = TEXT (bukan numeric_id!)
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    data_encrypted BLOB NOT NULL,
    enc_key_version INTEGER NOT NULL DEFAULT 1 REFERENCES encryption_key(version),
    enc_algorithm TEXT NOT NULL DEFAULT 'aes-256-gcm',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Executions (Ruling 28: ExecutionId = INTEGER)
CREATE TABLE execution (
    id INTEGER PRIMARY KEY,  -- ExecutionId (numeric_id!, id.rs:35)
    workflow_id INTEGER NOT NULL REFERENCES workflow(id),  -- FK to WorkflowId
    workflow_version INTEGER NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('regular', 'manual', 'webhook', 'retry')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'waiting', 'success', 'failed', 'needs_review', 'canceled')),
    started_at TEXT,
    finished_at TEXT,
    error_code TEXT,
    error_message TEXT,
    event_log_path TEXT,  -- for event archival to file (F18)
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_execution_workflow ON execution(workflow_id);
CREATE INDEX idx_execution_status ON execution(status);
CREATE INDEX idx_execution_created ON execution(created_at);

-- Event log (append-only, answers F18)
CREATE TABLE event_log (
    execution_id INTEGER NOT NULL REFERENCES execution(id) ON DELETE CASCADE,  -- FK to ExecutionId
    seq INTEGER NOT NULL,
    ts TEXT NOT NULL DEFAULT (datetime('now')),
    schema_version INTEGER NOT NULL DEFAULT 1,
    event_type TEXT NOT NULL,
    event_json TEXT NOT NULL,
    PRIMARY KEY (execution_id, seq)
);

-- Tasks (Ruling 28: TaskId = INTEGER)
CREATE TABLE task (
    id INTEGER PRIMARY KEY,  -- TaskId (numeric_id!, id.rs:36)
    execution_id INTEGER NOT NULL REFERENCES execution(id) ON DELETE CASCADE,  -- FK to ExecutionId
    node_id TEXT NOT NULL,  -- NodeId = TEXT (id.rs:49, bukan numeric_id!)
    input_index INTEGER NOT NULL DEFAULT 0,
    attempt INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('pending', 'ready', 'running', 'waiting', 'success', 'failed', 'in_doubt', 'canceled')),
    hints_json TEXT DEFAULT '{}',
    lease_holder TEXT,
    lease_expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    finished_at TEXT,
    error_json TEXT,
    wake_at TEXT
);

CREATE INDEX idx_task_execution ON task(execution_id);
CREATE INDEX idx_task_status ON task(status);
CREATE INDEX idx_task_wake ON task(wake_at) WHERE wake_at IS NOT NULL;

-- Checkpoints for crash recovery (Ruling 28: CheckpointId = INTEGER)
CREATE TABLE checkpoint (
    id INTEGER PRIMARY KEY,  -- CheckpointId (numeric_id!, id.rs:37)
    execution_id INTEGER NOT NULL REFERENCES execution(id) ON DELETE CASCADE,  -- FK to ExecutionId
    schema_version INTEGER NOT NULL DEFAULT 1,
    completed_ids_json TEXT NOT NULL DEFAULT '[]',
    in_flight_ids_json TEXT NOT NULL DEFAULT '[]',
    waiting_json TEXT DEFAULT '{}',
    event_seq INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Node output references (with BLAKE3 checksum - answers T2)
CREATE TABLE node_output (
    execution_id INTEGER NOT NULL REFERENCES execution(id) ON DELETE CASCADE,  -- FK to ExecutionId
    node_id TEXT NOT NULL,  -- NodeId = TEXT
    output_index INTEGER NOT NULL DEFAULT 0,
    storage_kind TEXT NOT NULL CHECK (storage_kind IN ('inline', 'spill', 'blob')),
    spill_path TEXT,
    checksum_blake3 TEXT,
    item_count INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (execution_id, node_id, output_index)
);

-- Write-ahead intent for crash safety (answers F19)
CREATE TABLE spill_intent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER NOT NULL REFERENCES execution(id) ON DELETE CASCADE,  -- FK to ExecutionId
    intent_type TEXT NOT NULL CHECK (intent_type IN ('write_spill', 'delete_spill', 'write_event', 'update_task')),
    target_path TEXT,
    payload_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_spill_intent_exec ON spill_intent(execution_id);

-- Spill GC tracking
CREATE TABLE spill_gc (
    spill_path TEXT PRIMARY KEY,
    execution_id INTEGER NOT NULL REFERENCES execution(id) ON DELETE CASCADE,  -- FK to ExecutionId
    ref_count INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Variables (global key-value)
CREATE TABLE variable (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'string'
);
