//! RunRepository and DAO operations for orchestrator runs.

use crate::error::StorageError;
use crate::pool::SqliteStoragePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub status: String,
    pub initiated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub metadata: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RunDao {
    pool: SqliteStoragePool,
}

impl RunDao {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, run: &RunRecord) -> Result<(), StorageError> {
        let initiated_at_str = run.initiated_at.to_rfc3339();
        let completed_at_str = run.completed_at.map(|dt| dt.to_rfc3339());
        let total_tokens_i64 = run.total_tokens as i64;

        sqlx::query(
            r#"
            INSERT INTO runs (id, status, initiated_at, completed_at, total_tokens, total_cost_usd, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&run.id)
        .bind(&run.status)
        .bind(&initiated_at_str)
        .bind(&completed_at_str)
        .bind(total_tokens_i64)
        .bind(run.total_cost_usd)
        .bind(&run.metadata)
        .execute(self.pool.inner())
        .await?;

        Ok(())
    }

    fn map_row(row: &sqlx::sqlite::SqliteRow) -> Result<RunRecord, StorageError> {
        let initiated_at_str: String = row.get(2);
        let initiated_at = DateTime::parse_from_rfc3339(&initiated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let completed_at_str: Option<String> = row.get(3);
        let completed_at = completed_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        let total_tokens_i64: i64 = row.get(4);
        let total_cost_usd: f64 = row.get(5);
        let metadata: Option<String> = row.get(6);

        Ok(RunRecord {
            id: row.get(0),
            status: row.get(1),
            initiated_at,
            completed_at,
            total_tokens: total_tokens_i64 as u64,
            total_cost_usd,
            metadata,
        })
    }

    pub async fn get(&self, id: &str) -> Result<Option<RunRecord>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, status, initiated_at, completed_at, total_tokens, total_cost_usd, metadata
            FROM runs WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool.inner())
        .await?;

        match row {
            Some(row) => Ok(Some(Self::map_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        completed_at: Option<DateTime<Utc>>,
    ) -> Result<(), StorageError> {
        let completed_at_str = completed_at.map(|dt| dt.to_rfc3339());
        sqlx::query("UPDATE runs SET status = ?, completed_at = ? WHERE id = ?")
            .bind(status)
            .bind(completed_at_str)
            .bind(id)
            .execute(self.pool.inner())
            .await?;
        Ok(())
    }

    pub async fn update_tokens_and_cost(
        &self,
        id: &str,
        total_tokens: u64,
        total_cost_usd: f64,
    ) -> Result<(), StorageError> {
        sqlx::query("UPDATE runs SET total_tokens = ?, total_cost_usd = ? WHERE id = ?")
            .bind(total_tokens as i64)
            .bind(total_cost_usd)
            .bind(id)
            .execute(self.pool.inner())
            .await?;
        Ok(())
    }

    pub async fn list(&self, limit: usize) -> Result<Vec<RunRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, status, initiated_at, completed_at, total_tokens, total_cost_usd, metadata
            FROM runs
            ORDER BY initiated_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit as i64)
        .fetch_all(self.pool.inner())
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in rows {
            list.push(Self::map_row(&row)?);
        }

        Ok(list)
    }
}
