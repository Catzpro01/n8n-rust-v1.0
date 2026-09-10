-- Migration 005: Add attempt field to node_output (BUG PARITAS fix)
-- 
-- Issue: node_output PK without attempt field → retry overwrites previous attempt material
-- Upstream n8n: IRunData = {[node]: ITaskData[]} → stores ALL attempts as array
-- Fix: Add attempt field to PK, rebuild table (SQLite-compatible pattern)
--
-- Requirements (matt #1586 §5):
-- 1. DEFAULT 0 = marker for existing rows (fail-closed still required for rows without material)
-- 2. Recreate v_orphan_intents view (no explicit indexes for node_output, only implicit PK)
-- 3. Falsifiable proof: count(*) before vs after must match
--
-- Migration numbering: 001, 002, 004 exist (003 missing/skipped) → next is 005

PRAGMA foreign_keys = OFF;

-- Drop views that depend on node_output
DROP VIEW IF EXISTS v_orphan_intents;

-- Create new table with attempt field
CREATE TABLE node_output_new (
    execution_id INTEGER NOT NULL REFERENCES execution(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL,
    output_index INTEGER NOT NULL DEFAULT 0,
    attempt INTEGER NOT NULL DEFAULT 0,  -- NEW: attempt number (0 = first/original)
    storage_kind TEXT NOT NULL CHECK (storage_kind IN ('inline', 'spill', 'blob')),
    spill_path TEXT,
    checksum_blake3 TEXT,
    item_count INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (execution_id, node_id, output_index, attempt)
);

-- Copy existing data with attempt=0 (backward compatible)
INSERT INTO node_output_new (
    execution_id, node_id, output_index, attempt,
    storage_kind, spill_path, checksum_blake3,
    item_count, total_bytes, created_at
)
SELECT 
    execution_id, node_id, output_index, 0 AS attempt,
    storage_kind, spill_path, checksum_blake3,
    item_count, total_bytes, created_at
FROM node_output;

-- Drop old table
DROP TABLE node_output;

-- Rename new table to original name
ALTER TABLE node_output_new RENAME TO node_output;

-- Recreate views that depend on node_output
CREATE VIEW IF NOT EXISTS v_orphan_intents AS
SELECT 
    si.id,
    si.execution_id,
    si.intent_type,
    si.target_path,
    si.created_at
FROM spill_intent si
WHERE NOT EXISTS (
    SELECT 1 FROM node_output no 
    WHERE no.spill_path = si.target_path 
      AND no.execution_id = si.execution_id
);

-- Recreate indexes (none exist explicitly for node_output, but FK and PK are implicit)

-- Re-enable foreign keys
PRAGMA foreign_keys = ON;

-- Verification queries (run after migration):
-- SELECT COUNT(*) FROM node_output WHERE attempt = 0;  -- should match original count
-- .schema node_output  -- should show new PK with attempt field
-- SELECT * FROM v_orphan_intents;  -- view should work
