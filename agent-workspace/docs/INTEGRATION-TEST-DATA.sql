-- Integration Test Data for Multi-Attempt Replay Scenarios
-- Prepared by: agent2 (Storage Owner)
-- Purpose: Test data untuk agent10 query-layer + agent1 reversion integration
-- Reference: agent1 #1635 (accepted test data offer)

-- ========================================
-- SCENARIO 1: Single Attempt (Backward Compatibility)
-- ========================================
INSERT INTO execution (id, status) VALUES (100, 'completed');
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (100, 'set_node', 0, 0, 'inline', 'hash_single_attempt', 5, 500);

-- Expected: Query attempt<=0 returns 1 row
-- Expected: Query attempt<=1 returns 1 row (no attempt=1)

-- ========================================
-- SCENARIO 2: Multiple Attempts (Retry Success)
-- ========================================
INSERT INTO execution (id, status) VALUES (101, 'completed');
-- Attempt 0: FAILED (item_count=0)
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (101, 'http_request', 0, 0, 'inline', 'hash_failed_attempt', 0, 0);
-- Attempt 1: SUCCESS
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (101, 'http_request', 0, 1, 'inline', 'hash_success_attempt', 10, 1000);

-- Expected: Query attempt<=0 returns attempt=0 (fail-closed should ERROR)
-- Expected: Query attempt<=1 returns both attempts
-- Expected: Query attempt=1 returns only successful attempt

-- ========================================
-- SCENARIO 3: Three Attempts (Multiple Retries)
-- ========================================
INSERT INTO execution (id, status) VALUES (102, 'completed');
-- Attempt 0: FAILED
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (102, 'api_call', 0, 0, 'inline', 'hash_attempt_0', 0, 0);
-- Attempt 1: FAILED
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (102, 'api_call', 0, 1, 'inline', 'hash_attempt_1', 0, 0);
-- Attempt 2: SUCCESS
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (102, 'api_call', 0, 2, 'inline', 'hash_attempt_2', 15, 1500);

-- Expected: Query attempt<=1 returns attempts 0,1 (both failed)
-- Expected: Query attempt<=2 returns all three
-- Expected: Replay logic uses attempt=2 (final successful)

-- ========================================
-- SCENARIO 4: Mixed Storage Kinds
-- ========================================
INSERT INTO execution (id, status) VALUES (103, 'completed');
-- Attempt 0: Inline storage
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (103, 'transform', 0, 0, 'inline', 'hash_inline', 5, 500);
-- Attempt 1: Spill storage (large data)
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, spill_path, checksum_blake3, item_count, total_bytes)
VALUES (103, 'transform', 0, 1, 'spill', '/tmp/spill_103_transform', 'hash_spill', 100, 10000);

-- Expected: Query returns both attempts with different storage_kind
-- Expected: Replay logic must handle inline vs spill

-- ========================================
-- SCENARIO 5: Multiple Nodes in Same Execution
-- ========================================
INSERT INTO execution (id, status) VALUES (104, 'completed');
-- Node A: attempt=0 only
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (104, 'node_a', 0, 0, 'inline', 'hash_node_a', 5, 500);
-- Node B: attempt=0 and attempt=1
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (104, 'node_b', 0, 0, 'inline', 'hash_node_b_0', 0, 0);
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (104, 'node_b', 0, 1, 'inline', 'hash_node_b_1', 10, 1000);
-- Node C: attempt=0, 1, 2
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (104, 'node_c', 0, 0, 'inline', 'hash_node_c_0', 0, 0);
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (104, 'node_c', 0, 1, 'inline', 'hash_node_c_1', 0, 0);
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (104, 'node_c', 0, 2, 'inline', 'hash_node_c_2', 20, 2000);

-- Expected: Query returns 6 rows total
-- Expected: Per-node queries return correct attempts
-- Expected: Replay uses latest attempt per node

-- ========================================
-- SCENARIO 6: Multiple Output Indexes
-- ========================================
INSERT INTO execution (id, status) VALUES (105, 'completed');
-- Node with multiple outputs (output_index 0 and 1)
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (105, 'split_node', 0, 0, 'inline', 'hash_output_0', 5, 500);
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (105, 'split_node', 1, 0, 'inline', 'hash_output_1', 3, 300);
-- Retry with both outputs
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (105, 'split_node', 0, 1, 'inline', 'hash_output_0_retry', 6, 600);
INSERT INTO node_output (execution_id, node_id, output_index, attempt, storage_kind, checksum_blake3, item_count, total_bytes)
VALUES (105, 'split_node', 1, 1, 'inline', 'hash_output_1_retry', 4, 400);

-- Expected: Query returns 4 rows
-- Expected: Per-output_index queries return correct attempts

-- ========================================
-- VERIFICATION QUERIES
-- ========================================
-- Single attempt
SELECT 'Scenario 1' AS test, COUNT(*) AS result FROM node_output WHERE execution_id=100 AND attempt<=0;
-- Expected: 1

-- Multiple attempts - fail-closed
SELECT 'Scenario 2 (fail-closed)' AS test, COUNT(*) AS result FROM node_output WHERE execution_id=101 AND attempt<=0;
-- Expected: 1 (but fail-closed should ERROR because item_count=0)

-- Multiple attempts - success
SELECT 'Scenario 2 (success)' AS test, COUNT(*) AS result FROM node_output WHERE execution_id=101 AND attempt<=1;
-- Expected: 2

-- Three attempts
SELECT 'Scenario 3 (all)' AS test, COUNT(*) AS result FROM node_output WHERE execution_id=102 AND attempt<=2;
-- Expected: 3

-- Mixed storage
SELECT 'Scenario 4 (mixed)' AS test, storage_kind, COUNT(*) AS count FROM node_output WHERE execution_id=103 GROUP BY storage_kind;
-- Expected: inline=1, spill=1

-- Multiple nodes
SELECT 'Scenario 5 (per-node)' AS test, node_id, COUNT(*) AS attempts FROM node_output WHERE execution_id=104 GROUP BY node_id;
-- Expected: node_a=1, node_b=2, node_c=3

-- Multiple outputs
SELECT 'Scenario 6 (per-output)' AS test, output_index, COUNT(*) AS attempts FROM node_output WHERE execution_id=105 GROUP BY output_index;
-- Expected: output_0=2, output_1=2

-- ========================================
-- CLEANUP (for testing)
-- ========================================
-- DELETE FROM node_output WHERE execution_id BETWEEN 100 AND 105;
-- DELETE FROM execution WHERE id BETWEEN 100 AND 105;
