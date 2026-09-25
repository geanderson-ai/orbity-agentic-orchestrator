//! TaskRepository and DAO operations for orchestrator tasks.

use crate::error::StorageError;
use crate::pool::SqliteStoragePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub run_id: String,
    pub parent_task_id: Option<String>,
    pub agent_name: String,
    pub status: String,
    pub input_prompt: Option<String>,
    pub output_result: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct TaskDao {
    pool: SqliteStoragePool,
}

impl TaskDao {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, task: &TaskRecord) -> Result<(), StorageError> {
        let created_at_str = task.created_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO tasks (id, run_id, parent_task_id, agent_name, status, input_prompt, output_result, duration_ms, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&task.id)
        .bind(&task.run_id)
        .bind(&task.parent_task_id)
        .bind(&task.agent_name)
        .bind(&task.status)
        .bind(&task.input_prompt)
        .bind(&task.output_result)
        .bind(task.duration_ms)
        .bind(&created_at_str)
        .execute(self.pool.inner())
        .await?;

        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<TaskRecord>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, run_id, parent_task_id, agent_name, status, input_prompt, output_result, duration_ms, created_at
            FROM tasks WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(self.pool.inner())
        .await?;

        match row {
            Some(row) => {
                let created_at_str: String = row.get(8);
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Some(TaskRecord {
                    id: row.get(0),
                    run_id: row.get(1),
                    parent_task_id: row.get(2),
                    agent_name: row.get(3),
                    status: row.get(4),
                    input_prompt: row.get(5),
                    output_result: row.get(6),
                    duration_ms: row.get(7),
                    created_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        output_result: Option<&str>,
        duration_ms: Option<i64>,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE tasks
            SET status = ?, output_result = COALESCE(?, output_result), duration_ms = COALESCE(?, duration_ms)
            WHERE id = ?
            "#
        )
        .bind(status)
        .bind(output_result)
        .bind(duration_ms)
        .bind(id)
        .execute(self.pool.inner())
        .await?;

        Ok(())
    }

    pub async fn list_by_run(&self, run_id: &str) -> Result<Vec<TaskRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, run_id, parent_task_id, agent_name, status, input_prompt, output_result, duration_ms, created_at
            FROM tasks
            WHERE run_id = ?
            ORDER BY created_at ASC
            "#
        )
        .bind(run_id)
        .fetch_all(self.pool.inner())
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in rows {
            let created_at_str: String = row.get(8);
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            list.push(TaskRecord {
                id: row.get(0),
                run_id: row.get(1),
                parent_task_id: row.get(2),
                agent_name: row.get(3),
                status: row.get(4),
                input_prompt: row.get(5),
                output_result: row.get(6),
                duration_ms: row.get(7),
                created_at,
            });
        }

        Ok(list)
    }
}
