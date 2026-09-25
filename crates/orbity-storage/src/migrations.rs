//! Schema migrations and DDL definitions for SQLite persistence.

pub const SCHEMA_MIGRATION_V1: &str = r#"
-- Teams table
CREATE TABLE IF NOT EXISTS teams (
    name TEXT PRIMARY KEY,
    description TEXT,
    config_yaml TEXT NOT NULL,
    config_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Agents table
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    team_name TEXT,
    state TEXT NOT NULL DEFAULT 'Draft',
    orchestrator_model TEXT,
    prompt_system TEXT,
    plan_strategy TEXT,
    plan_json TEXT,
    finops_budget_usd REAL,
    config_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(team_name) REFERENCES teams(name) ON DELETE SET NULL
);

-- Agent workers table
CREATE TABLE IF NOT EXISTS agent_workers (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    worker_name TEXT NOT NULL,
    role TEXT,
    model TEXT,
    allowed_tools_json TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- Runs table
CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    initiated_at TEXT NOT NULL,
    completed_at TEXT,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost_usd REAL NOT NULL DEFAULT 0.0,
    metadata TEXT
);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    parent_task_id TEXT,
    agent_name TEXT NOT NULL,
    status TEXT NOT NULL,
    input_prompt TEXT,
    output_result TEXT,
    duration_ms INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
);

-- Sandboxes table
CREATE TABLE IF NOT EXISTS sandboxes (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    path TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    destroyed_at TEXT,
    FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
);

-- Token ledger table
CREATE TABLE IF NOT EXISTS token_ledger (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    task_id TEXT,
    agent_name TEXT NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cached_tokens INTEGER NOT NULL DEFAULT 0,
    reasoning_tokens INTEGER NOT NULL DEFAULT 0,
    cost_usd REAL NOT NULL DEFAULT 0.0,
    recorded_at TEXT NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
);

-- Audit events table (Append-Only Hash-Chained Ledger)
CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    task_id TEXT,
    sequence_num INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    previous_hash TEXT NOT NULL,
    current_hash TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    UNIQUE(run_id, sequence_num)
);

-- Performance and Query Indices
CREATE INDEX IF NOT EXISTS idx_audit_events_run_seq ON audit_events(run_id, sequence_num);
CREATE INDEX IF NOT EXISTS idx_audit_events_task ON audit_events(task_id);
CREATE INDEX IF NOT EXISTS idx_tasks_run ON tasks(run_id);
CREATE INDEX IF NOT EXISTS idx_tasks_agent ON tasks(agent_name);
CREATE INDEX IF NOT EXISTS idx_token_ledger_run ON token_ledger(run_id);
CREATE INDEX IF NOT EXISTS idx_token_ledger_agent ON token_ledger(agent_name);
CREATE INDEX IF NOT EXISTS idx_agents_team ON agents(team_name);
CREATE INDEX IF NOT EXISTS idx_agents_state ON agents(state);
"#;
