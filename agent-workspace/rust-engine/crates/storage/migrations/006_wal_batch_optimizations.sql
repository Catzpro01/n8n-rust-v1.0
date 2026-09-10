-- Migration 006: SQLite WAL Batch Optimizations (ADR-v2-STORAGE)
--
-- Optimization for high-volume lineage edge tracking (>10k edges/execution)
-- Implements composite index for fast lineage traversal and ensures WAL pragma alignment.

-- Create composite index on lineage_edge for batch lookups and fast ancestor graph queries
CREATE INDEX IF NOT EXISTS idx_lineage_edge_exec_output 
ON lineage_edge (execution_id, output_ref);

CREATE INDEX IF NOT EXISTS idx_lineage_edge_exec_rule 
ON lineage_edge (execution_id, rule_id, rule_ver);

-- Index for erasure log compliance auditing
CREATE INDEX IF NOT EXISTS idx_erasure_log_exec_id 
ON erasure_log (execution_id);
