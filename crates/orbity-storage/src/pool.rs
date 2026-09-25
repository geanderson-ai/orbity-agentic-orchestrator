//! SQLite connection pool with WAL mode and performance pragmas.

use crate::error::StorageError;
use crate::migrations::SCHEMA_MIGRATION_V1;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Row, Sqlite};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SqliteStoragePool {
    pool: Pool<Sqlite>,
}

impl SqliteStoragePool {
    /// Creates a pool from an existing SQLx pool
    pub fn from_pool(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Connects to an in-memory SQLite database (useful for isolated tests)
    pub async fn connect_in_memory() -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await?;

        let instance = Self { pool };
        instance.run_migrations().await?;
        Ok(instance)
    }

    /// Connects to a SQLite database file with WAL mode enabled
    pub async fn connect_file(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(16)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await?;

        let instance = Self { pool };
        instance.run_migrations().await?;
        Ok(instance)
    }

    /// Connects using a generic database URL string
    pub async fn connect(url: &str) -> Result<Self, StorageError> {
        if url.contains(":memory:") {
            Self::connect_in_memory().await
        } else {
            let path_str = url.trim_start_matches("sqlite://").trim_start_matches("sqlite:");
            Self::connect_file(path_str).await
        }
    }

    /// Returns a reference to the underlying sqlx Pool<Sqlite>
    pub fn inner(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Runs all pending schema migrations
    pub async fn run_migrations(&self) -> Result<(), StorageError> {
        // Execute schema DDL
        sqlx::raw_sql(SCHEMA_MIGRATION_V1)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Verifies that foreign keys and pragmas are properly configured
    pub async fn verify_pragmas(&self) -> Result<PragmaStatus, StorageError> {
        let fk_row = sqlx::query("PRAGMA foreign_keys;")
            .fetch_one(&self.pool)
            .await?;
        let foreign_keys_enabled: i32 = fk_row.get(0);

        let jm_row = sqlx::query("PRAGMA journal_mode;")
            .fetch_one(&self.pool)
            .await?;
        let journal_mode: String = jm_row.get(0);

        let syn_row = sqlx::query("PRAGMA synchronous;")
            .fetch_one(&self.pool)
            .await?;
        let synchronous: i32 = syn_row.get(0);

        Ok(PragmaStatus {
            foreign_keys_enabled: foreign_keys_enabled == 1,
            journal_mode: journal_mode.to_uppercase(),
            synchronous,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PragmaStatus {
    pub foreign_keys_enabled: bool,
    pub journal_mode: String,
    pub synchronous: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_pool_migration() {
        let pool = SqliteStoragePool::connect_in_memory()
            .await
            .expect("Failed to connect in memory");

        let status = pool.verify_pragmas().await.expect("Failed to check pragmas");
        assert!(status.foreign_keys_enabled);

        // Verify table exists
        let row = sqlx::query("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='audit_events'")
            .fetch_one(pool.inner())
            .await
            .expect("query master");
        let count: i64 = row.get(0);
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_file_pool_wal_mode() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_orbity_{}.db", uuid::Uuid::new_v4()));

        let pool = SqliteStoragePool::connect_file(&db_path)
            .await
            .expect("connect file");

        let status = pool.verify_pragmas().await.expect("verify pragmas");
        assert!(status.foreign_keys_enabled);
        assert_eq!(status.journal_mode, "WAL");

        let _ = std::fs::remove_file(&db_path);
    }
}
