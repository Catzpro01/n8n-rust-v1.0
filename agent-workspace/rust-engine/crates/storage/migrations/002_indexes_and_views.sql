-- Migration: Useful views and additional indexes

-- View: execution with workflow name
CREATE VIEW IF NOT EXISTS v_execution_detail AS
SELECT 
    e.id,
    e.workflow_id,
    w.name as workflow_name,
    e.mode,
    e.status,
    e.started_at,
    e.finished_at,
    e.error_code,
    e.error_message,
    e.created_at
FROM execution e
JOIN workflow w ON w.id = e.workflow_id;

-- View: task summary per execution
CREATE VIEW IF NOT EXISTS v_execution_task_summary AS
SELECT 
    execution_id,
    status,
    COUNT(*) as count
FROM task
GROUP BY execution_id, status;

-- View: pending spill intents (orphan detection)
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
    AND si.intent_type = 'write_spill'
);
