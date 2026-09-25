//! AgentRepository and DAO operations for agents and sub-orchestrators.

use crate::error::StorageError;
use crate::pool::SqliteStoragePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDbRecord {
    pub id: String,
    pub name: String,
    pub team_name: Option<String>,
    pub state: String,
    pub orchestrator_model: Option<String>,
    pub prompt_system: Option<String>,
    pub plan_strategy: Option<String>,
    pub plan_json: Option<String>,
    pub finops_budget_usd: Option<f64>,
    pub config_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct AgentDao {
    pool: SqliteStoragePool,
}

impl AgentDao {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, agent: &AgentDbRecord) -> Result<(), StorageError> {
        let created_at_str = agent.created_at.to_rfc3339();
        let updated_at_str = agent.updated_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO agents (
                id, name, team_name, state, orchestrator_model, prompt_system, plan_strategy, plan_json, finops_budget_usd, config_hash, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                team_name = excluded.team_name,
                state = excluded.state,
                orchestrator_model = excluded.orchestrator_model,
                prompt_system = excluded.prompt_system,
                plan_strategy = excluded.plan_strategy,
                plan_json = excluded.plan_json,
                finops_budget_usd = excluded.finops_budget_usd,
                config_hash = excluded.config_hash,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&agent.id)
        .bind(&agent.name)
        .bind(&agent.team_name)
        .bind(&agent.state)
        .bind(&agent.orchestrator_model)
        .bind(&agent.prompt_system)
        .bind(&agent.plan_strategy)
        .bind(&agent.plan_json)
        .bind(agent.finops_budget_usd)
        .bind(&agent.config_hash)
        .bind(&created_at_str)
        .bind(&updated_at_str)
        .execute(self.pool.inner())
        .await?;

        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<AgentDbRecord>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, team_name, state, orchestrator_model, prompt_system, plan_strategy, plan_json, finops_budget_usd, config_hash, created_at, updated_at
            FROM agents WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(self.pool.inner())
        .await?;

        match row {
            Some(row) => {
                let created_at_str: String = row.get(10);
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let updated_at_str: String = row.get(11);
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Some(AgentDbRecord {
                    id: row.get(0),
                    name: row.get(1),
                    team_name: row.get(2),
                    state: row.get(3),
                    orchestrator_model: row.get(4),
                    prompt_system: row.get(5),
                    plan_strategy: row.get(6),
                    plan_json: row.get(7),
                    finops_budget_usd: row.get(8),
                    config_hash: row.get(9),
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_state(&self, id: &str, state: &str) -> Result<(), StorageError> {
        let updated_at_str = Utc::now().to_rfc3339();
        sqlx::query("UPDATE agents SET state = ?, updated_at = ? WHERE id = ?")
            .bind(state)
            .bind(&updated_at_str)
            .bind(id)
            .execute(self.pool.inner())
            .await?;
        Ok(())
    }

    pub async fn list_by_team(&self, team_name: &str) -> Result<Vec<AgentDbRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, team_name, state, orchestrator_model, prompt_system, plan_strategy, plan_json, finops_budget_usd, config_hash, created_at, updated_at
            FROM agents WHERE team_name = ?
            ORDER BY name ASC
            "#
        )
        .bind(team_name)
        .fetch_all(self.pool.inner())
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in rows {
            let created_at_str: String = row.get(10);
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let updated_at_str: String = row.get(11);
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            list.push(AgentDbRecord {
                id: row.get(0),
                name: row.get(1),
                team_name: row.get(2),
                state: row.get(3),
                orchestrator_model: row.get(4),
                prompt_system: row.get(5),
                plan_strategy: row.get(6),
                plan_json: row.get(7),
                finops_budget_usd: row.get(8),
                config_hash: row.get(9),
                created_at,
                updated_at,
            });
        }

        Ok(list)
    }

    pub async fn list_all(&self) -> Result<Vec<AgentDbRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, team_name, state, orchestrator_model, prompt_system, plan_strategy, plan_json, finops_budget_usd, config_hash, created_at, updated_at
            FROM agents ORDER BY name ASC
            "#
        )
        .fetch_all(self.pool.inner())
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in rows {
            let created_at_str: String = row.get(10);
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let updated_at_str: String = row.get(11);
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            list.push(AgentDbRecord {
                id: row.get(0),
                name: row.get(1),
                team_name: row.get(2),
                state: row.get(3),
                orchestrator_model: row.get(4),
                prompt_system: row.get(5),
                plan_strategy: row.get(6),
                plan_json: row.get(7),
                finops_budget_usd: row.get(8),
                config_hash: row.get(9),
                created_at,
                updated_at,
            });
        }

        Ok(list)
    }

    pub async fn delete(&self, id: &str) -> Result<bool, StorageError> {
        let res = sqlx::query("DELETE FROM agents WHERE id = ?")
            .bind(id)
            .execute(self.pool.inner())
            .await?;
        Ok(res.rows_affected() > 0)
    }
}
