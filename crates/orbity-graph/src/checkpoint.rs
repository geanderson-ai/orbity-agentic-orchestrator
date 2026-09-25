//! Cryptographic checkpointing for Graph State into SQLite WAL storage.

use crate::types::{GraphStateCheckpoint, NodeId};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

pub const GENESIS_PARENT_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Corrupted checkpoint hash chain at step {step}: expected {expected}, actual {actual}")]
    HashMismatch {
        step: u32,
        expected: String,
        actual: String,
    },
}

type CheckpointDbRow = (String, i64, String, String, String, String, String, String);

/// Cryptographically chained checkpoint manager for graph executions.
pub struct GraphCheckpointStore {
    pool: SqlitePool,
}

impl GraphCheckpointStore {

    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Ensures the DDL table for graph checkpoints exists in SQLite.
    pub async fn init_schema(&self) -> Result<(), CheckpointError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS graph_checkpoints (
                execution_id TEXT NOT NULL,
                step_number INTEGER NOT NULL,
                active_nodes_json TEXT NOT NULL,
                completed_nodes_json TEXT NOT NULL,
                blackboard_snapshot TEXT NOT NULL,
                previous_hash TEXT NOT NULL,
                state_hash TEXT NOT NULL,
                recorded_at TEXT NOT NULL,
                PRIMARY KEY (execution_id, step_number)
            );
            CREATE INDEX IF NOT EXISTS idx_graph_checkpoints_exec ON graph_checkpoints(execution_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Computes cryptographic SHA-256 hash for a checkpoint state.
    pub fn compute_hash(
        execution_id: &Uuid,
        step: u32,
        active_nodes: &[NodeId],
        completed_nodes: &[NodeId],
        snapshot: &serde_json::Value,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(execution_id.as_bytes());
        hasher.update(step.to_be_bytes());
        hasher.update(serde_json::to_string(active_nodes).unwrap_or_default().as_bytes());
        hasher.update(serde_json::to_string(completed_nodes).unwrap_or_default().as_bytes());
        hasher.update(snapshot.to_string().as_bytes());
        hasher.update(prev_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Persists a new checkpoint into SQLite chained to the previous hash.
    pub async fn save_checkpoint(
        &self,
        execution_id: Uuid,
        step_number: u32,
        active_nodes: Vec<NodeId>,
        completed_nodes: Vec<NodeId>,
        blackboard_snapshot: serde_json::Value,
    ) -> Result<GraphStateCheckpoint, CheckpointError> {
        let prev_hash = if step_number == 0 {
            GENESIS_PARENT_HASH.to_string()
        } else {
            // Fetch last checkpoint hash
            let prev_rec: Option<(String,)> = sqlx::query_as(
                "SELECT state_hash FROM graph_checkpoints WHERE execution_id = ? AND step_number = ?"
            )
            .bind(execution_id.to_string())
            .bind((step_number - 1) as i64)
            .fetch_optional(&self.pool)
            .await?;

            prev_rec
                .map(|r| r.0)
                .unwrap_or_else(|| GENESIS_PARENT_HASH.to_string())
        };

        let state_hash = Self::compute_hash(
            &execution_id,
            step_number,
            &active_nodes,
            &completed_nodes,
            &blackboard_snapshot,
            &prev_hash,
        );

        let recorded_at = Utc::now();
        let active_json = serde_json::to_string(&active_nodes)?;
        let completed_json = serde_json::to_string(&completed_nodes)?;
        let snapshot_str = blackboard_snapshot.to_string();

        sqlx::query(
            r#"
            INSERT INTO graph_checkpoints 
            (execution_id, step_number, active_nodes_json, completed_nodes_json, blackboard_snapshot, previous_hash, state_hash, recorded_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(execution_id.to_string())
        .bind(step_number as i64)
        .bind(active_json)
        .bind(completed_json)
        .bind(snapshot_str)
        .bind(&prev_hash)
        .bind(&state_hash)
        .bind(recorded_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(GraphStateCheckpoint {
            execution_id,
            step_number,
            active_nodes,
            completed_nodes,
            blackboard_snapshot,
            previous_hash: prev_hash,
            state_hash,
            recorded_at,
        })
    }

    /// Loads the latest checkpoint for an execution to resume work without data loss.
    pub async fn load_latest_checkpoint(
        &self,
        execution_id: Uuid,
    ) -> Result<Option<GraphStateCheckpoint>, CheckpointError> {
        let row: Option<CheckpointDbRow> = sqlx::query_as(

            r#"
            SELECT execution_id, step_number, active_nodes_json, completed_nodes_json, blackboard_snapshot, previous_hash, state_hash, recorded_at
            FROM graph_checkpoints
            WHERE execution_id = ?
            ORDER BY step_number DESC
            LIMIT 1
            "#,
        )

        .bind(execution_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some((_, step, active_json, completed_json, snapshot_str, prev_hash, state_hash, rec_at)) = row {
            let active_nodes: Vec<NodeId> = serde_json::from_str(&active_json)?;
            let completed_nodes: Vec<NodeId> = serde_json::from_str(&completed_json)?;
            let blackboard_snapshot: serde_json::Value = serde_json::from_str(&snapshot_str)?;
            let recorded_at = chrono::DateTime::parse_from_rfc3339(&rec_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            // Validate integrity
            let expected_hash = Self::compute_hash(
                &execution_id,
                step as u32,
                &active_nodes,
                &completed_nodes,
                &blackboard_snapshot,
                &prev_hash,
            );

            if expected_hash != state_hash {
                return Err(CheckpointError::HashMismatch {
                    step: step as u32,
                    expected: expected_hash,
                    actual: state_hash,
                });
            }

            Ok(Some(GraphStateCheckpoint {
                execution_id,
                step_number: step as u32,
                active_nodes,
                completed_nodes,
                blackboard_snapshot,
                previous_hash: prev_hash,
                state_hash,
                recorded_at,
            }))
        } else {
            Ok(None)
        }
    }
}
