//! TokenLedgerRepository and DAO operations for FinOps accounting.

use crate::error::StorageError;
use crate::pool::SqliteStoragePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenLedgerRecord {
    pub id: String,
    pub run_id: String,
    pub task_id: Option<String>,
    pub agent_name: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub reasoning_tokens: u64,
    pub cost_usd: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TokenUsageSummary {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cached_tokens: u64,
    pub total_reasoning_tokens: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub records_count: usize,
}

#[derive(Clone, Debug)]
pub struct TokenLedgerDao {
    pool: SqliteStoragePool,
}

impl TokenLedgerDao {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self { pool }
    }

    pub async fn record_usage(&self, record: &TokenLedgerRecord) -> Result<(), StorageError> {
        let recorded_at_str = record.recorded_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO token_ledger (
                id, run_id, task_id, agent_name, input_tokens, output_tokens, cached_tokens, reasoning_tokens, cost_usd, recorded_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record.id)
        .bind(&record.run_id)
        .bind(&record.task_id)
        .bind(&record.agent_name)
        .bind(record.input_tokens as i64)
        .bind(record.output_tokens as i64)
        .bind(record.cached_tokens as i64)
        .bind(record.reasoning_tokens as i64)
        .bind(record.cost_usd)
        .bind(&recorded_at_str)
        .execute(self.pool.inner())
        .await?;

        Ok(())
    }

    pub async fn get_run_summary(&self, run_id: &str) -> Result<TokenUsageSummary, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cached_tokens), 0),
                COALESCE(SUM(reasoning_tokens), 0),
                COALESCE(SUM(cost_usd), 0.0),
                COUNT(*)
            FROM token_ledger
            WHERE run_id = ?
            "#
        )
        .bind(run_id)
        .fetch_one(self.pool.inner())
        .await?;

        let inp: i64 = row.get(0);
        let out: i64 = row.get(1);
        let cac: i64 = row.get(2);
        let rea: i64 = row.get(3);
        let cost: f64 = row.get(4);
        let count: i64 = row.get(5);

        let total_tokens = (inp + out + cac + rea) as u64;

        Ok(TokenUsageSummary {
            total_input_tokens: inp as u64,
            total_output_tokens: out as u64,
            total_cached_tokens: cac as u64,
            total_reasoning_tokens: rea as u64,
            total_tokens,
            total_cost_usd: cost,
            records_count: count as usize,
        })
    }

    pub async fn get_agent_summary(
        &self,
        run_id: &str,
        agent_name: &str,
    ) -> Result<TokenUsageSummary, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cached_tokens), 0),
                COALESCE(SUM(reasoning_tokens), 0),
                COALESCE(SUM(cost_usd), 0.0),
                COUNT(*)
            FROM token_ledger
            WHERE run_id = ? AND agent_name = ?
            "#
        )
        .bind(run_id)
        .bind(agent_name)
        .fetch_one(self.pool.inner())
        .await?;

        let inp: i64 = row.get(0);
        let out: i64 = row.get(1);
        let cac: i64 = row.get(2);
        let rea: i64 = row.get(3);
        let cost: f64 = row.get(4);
        let count: i64 = row.get(5);

        let total_tokens = (inp + out + cac + rea) as u64;

        Ok(TokenUsageSummary {
            total_input_tokens: inp as u64,
            total_output_tokens: out as u64,
            total_cached_tokens: cac as u64,
            total_reasoning_tokens: rea as u64,
            total_tokens,
            total_cost_usd: cost,
            records_count: count as usize,
        })
    }
}
