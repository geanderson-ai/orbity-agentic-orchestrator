//! TeamRepository and DAO operations for team definitions.

use crate::error::StorageError;
use crate::pool::SqliteStoragePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamRecord {
    pub name: String,
    pub description: Option<String>,
    pub config_yaml: String,
    pub config_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct TeamDao {
    pool: SqliteStoragePool,
}

impl TeamDao {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, team: &TeamRecord) -> Result<(), StorageError> {
        let created_at_str = team.created_at.to_rfc3339();
        let updated_at_str = team.updated_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO teams (name, description, config_yaml, config_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                description = excluded.description,
                config_yaml = excluded.config_yaml,
                config_hash = excluded.config_hash,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&team.name)
        .bind(&team.description)
        .bind(&team.config_yaml)
        .bind(&team.config_hash)
        .bind(&created_at_str)
        .bind(&updated_at_str)
        .execute(self.pool.inner())
        .await?;

        Ok(())
    }

    pub async fn get(&self, name: &str) -> Result<Option<TeamRecord>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT name, description, config_yaml, config_hash, created_at, updated_at
            FROM teams WHERE name = ?
            "#
        )
        .bind(name)
        .fetch_optional(self.pool.inner())
        .await?;

        match row {
            Some(row) => {
                let created_at_str: String = row.get(4);
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let updated_at_str: String = row.get(5);
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Some(TeamRecord {
                    name: row.get(0),
                    description: row.get(1),
                    config_yaml: row.get(2),
                    config_hash: row.get(3),
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list(&self) -> Result<Vec<TeamRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT name, description, config_yaml, config_hash, created_at, updated_at
            FROM teams ORDER BY name ASC
            "#
        )
        .fetch_all(self.pool.inner())
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in rows {
            let created_at_str: String = row.get(4);
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let updated_at_str: String = row.get(5);
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            list.push(TeamRecord {
                name: row.get(0),
                description: row.get(1),
                config_yaml: row.get(2),
                config_hash: row.get(3),
                created_at,
                updated_at,
            });
        }

        Ok(list)
    }

    pub async fn delete(&self, name: &str) -> Result<bool, StorageError> {
        let res = sqlx::query("DELETE FROM teams WHERE name = ?")
            .bind(name)
            .execute(self.pool.inner())
            .await?;
        Ok(res.rows_affected() > 0)
    }
}
