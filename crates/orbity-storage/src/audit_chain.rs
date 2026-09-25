//! Append-Only cryptographic hash-chained audit engine.

use crate::error::StorageError;
use crate::pool::SqliteStoragePool;
use chrono::{DateTime, Utc};
use orbity_core::events::RuntimeEvent;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Canonical record of an event in the cryptographic audit ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: String,
    pub run_id: String,
    pub task_id: Option<String>,
    pub sequence_num: i64,
    pub event_type: String,
    pub payload_json: String,
    pub previous_hash: String,
    pub current_hash: String,
    pub recorded_at: DateTime<Utc>,
}

/// Computes the cryptographic SHA-256 hash chaining previous block with current payload.
/// Format: SHA256(previous_hash || sequence_num || event_type || payload_json || recorded_at)
pub fn compute_audit_hash(
    previous_hash: &str,
    sequence_num: i64,
    event_type: &str,
    payload_json: &str,
    recorded_at: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(previous_hash.as_bytes());
    hasher.update(sequence_num.to_string().as_bytes());
    hasher.update(event_type.as_bytes());
    hasher.update(payload_json.as_bytes());
    hasher.update(recorded_at.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Clone, Debug)]
pub struct AuditStore {
    pool: SqliteStoragePool,
}

impl AuditStore {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self { pool }
    }

    /// Appends a new runtime event to the audit ledger of a run atomically.
    pub async fn append_event(
        &self,
        run_id: &str,
        task_id: Option<&str>,
        event: &RuntimeEvent,
    ) -> Result<AuditRecord, StorageError> {
        let payload_json = serde_json::to_string(event)?;
        let event_type = event.event_type_name().to_string();

        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 50;

        loop {
            attempts += 1;
            let mut tx = self.pool.inner().begin().await?;

            // Query the latest sequence number and hash for this run
            let last_row = sqlx::query(
                "SELECT sequence_num, current_hash FROM audit_events WHERE run_id = ? ORDER BY sequence_num DESC LIMIT 1"
            )
            .bind(run_id)
            .fetch_optional(&mut *tx)
            .await?;

            let (sequence_num, previous_hash) = match last_row {
                Some(row) => {
                    let seq: i64 = row.get(0);
                    let hash: String = row.get(1);
                    (seq + 1, hash)
                }
                None => (0, GENESIS_HASH.to_string()),
            };

            let record_id = Uuid::new_v4().to_string();
            let recorded_at = Utc::now();
            let recorded_at_str = recorded_at.to_rfc3339();

            let current_hash = compute_audit_hash(
                &previous_hash,
                sequence_num,
                &event_type,
                &payload_json,
                &recorded_at_str,
            );

            let insert_res = sqlx::query(
                r#"
                INSERT INTO audit_events (
                    id, run_id, task_id, sequence_num, event_type, payload_json, previous_hash, current_hash, recorded_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&record_id)
            .bind(run_id)
            .bind(task_id)
            .bind(sequence_num)
            .bind(&event_type)
            .bind(&payload_json)
            .bind(&previous_hash)
            .bind(&current_hash)
            .bind(&recorded_at_str)
            .execute(&mut *tx)
            .await;

            match insert_res {
                Ok(_) => {
                    if let Err(e) = tx.commit().await {
                        if attempts < MAX_ATTEMPTS {
                            tokio::time::sleep(tokio::time::Duration::from_millis(attempts as u64 * 3)).await;
                            continue;
                        }
                        return Err(StorageError::DatabaseError(e));
                    }
                    return Ok(AuditRecord {
                        id: record_id,
                        run_id: run_id.to_string(),
                        task_id: task_id.map(|s| s.to_string()),
                        sequence_num,
                        event_type,
                        payload_json,
                        previous_hash,
                        current_hash,
                        recorded_at,
                    });
                }
                Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() || db_err.message().contains("UNIQUE") => {
                    let _ = tx.rollback().await;
                    if attempts < MAX_ATTEMPTS {
                        tokio::time::sleep(tokio::time::Duration::from_millis(attempts as u64 * 3)).await;
                        continue;
                    }
                    return Err(StorageError::Conflict(format!(
                        "Concurrent audit sequence conflict after {} attempts: {}",
                        attempts, db_err
                    )));
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    if attempts < MAX_ATTEMPTS {
                        tokio::time::sleep(tokio::time::Duration::from_millis(attempts as u64 * 3)).await;
                        continue;
                    }
                    return Err(StorageError::DatabaseError(e));
                }
            }
        }
    }

    /// Explicitly appends a pre-constructed AuditRecord validating hash integrity and chain order.
    pub async fn append_record(&self, record: &AuditRecord) -> Result<(), StorageError> {
        let recorded_at_str = record.recorded_at.to_rfc3339();
        let expected_hash = compute_audit_hash(
            &record.previous_hash,
            record.sequence_num,
            &record.event_type,
            &record.payload_json,
            &recorded_at_str,
        );

        if record.current_hash != expected_hash {
            return Err(StorageError::AuditTampered {
                event_id: record.id.clone(),
                sequence_num: record.sequence_num,
                expected_hash,
                actual_hash: record.current_hash.clone(),
            });
        }

        let mut tx = self.pool.inner().begin().await?;

        let last_row = sqlx::query(
            "SELECT sequence_num, current_hash FROM audit_events WHERE run_id = ? ORDER BY sequence_num DESC LIMIT 1"
        )
        .bind(&record.run_id)
        .fetch_optional(&mut *tx)
        .await?;

        match last_row {
            Some(row) => {
                let last_seq: i64 = row.get(0);
                let last_hash: String = row.get(1);

                if record.sequence_num != last_seq + 1 {
                    return Err(StorageError::InvalidSequence {
                        expected: last_seq + 1,
                        actual: record.sequence_num,
                    });
                }

                if record.previous_hash != last_hash {
                    return Err(StorageError::InvalidPreviousHash {
                        expected: last_hash,
                        actual: record.previous_hash.clone(),
                    });
                }
            }
            None => {
                if record.sequence_num != 0 {
                    return Err(StorageError::InvalidSequence {
                        expected: 0,
                        actual: record.sequence_num,
                    });
                }
                if record.previous_hash != GENESIS_HASH {
                    return Err(StorageError::InvalidPreviousHash {
                        expected: GENESIS_HASH.to_string(),
                        actual: record.previous_hash.clone(),
                    });
                }
            }
        }

        sqlx::query(
            r#"
            INSERT INTO audit_events (
                id, run_id, task_id, sequence_num, event_type, payload_json, previous_hash, current_hash, recorded_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record.id)
        .bind(&record.run_id)
        .bind(&record.task_id)
        .bind(record.sequence_num)
        .bind(&record.event_type)
        .bind(&record.payload_json)
        .bind(&record.previous_hash)
        .bind(&record.current_hash)
        .bind(&recorded_at_str)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Fetches the complete audit chain for a run ordered by sequence number.
    pub async fn get_chain(&self, run_id: &str) -> Result<Vec<AuditRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, run_id, task_id, sequence_num, event_type, payload_json, previous_hash, current_hash, recorded_at
            FROM audit_events
            WHERE run_id = ?
            ORDER BY sequence_num ASC
            "#
        )
        .bind(run_id)
        .fetch_all(self.pool.inner())
        .await?;

        let mut chain = Vec::with_capacity(rows.len());
        for row in rows {
            let recorded_at_str: String = row.get(8);
            let recorded_at = DateTime::parse_from_rfc3339(&recorded_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            chain.push(AuditRecord {
                id: row.get(0),
                run_id: row.get(1),
                task_id: row.get(2),
                sequence_num: row.get(3),
                event_type: row.get(4),
                payload_json: row.get(5),
                previous_hash: row.get(6),
                current_hash: row.get(7),
                recorded_at,
            });
        }

        Ok(chain)
    }

    /// Fetches the latest audit record of a run if available.
    pub async fn get_latest_record(&self, run_id: &str) -> Result<Option<AuditRecord>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, run_id, task_id, sequence_num, event_type, payload_json, previous_hash, current_hash, recorded_at
            FROM audit_events
            WHERE run_id = ?
            ORDER BY sequence_num DESC
            LIMIT 1
            "#
        )
        .bind(run_id)
        .fetch_optional(self.pool.inner())
        .await?;

        match row {
            Some(row) => {
                let recorded_at_str: String = row.get(8);
                let recorded_at = DateTime::parse_from_rfc3339(&recorded_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Some(AuditRecord {
                    id: row.get(0),
                    run_id: row.get(1),
                    task_id: row.get(2),
                    sequence_num: row.get(3),
                    event_type: row.get(4),
                    payload_json: row.get(5),
                    previous_hash: row.get(6),
                    current_hash: row.get(7),
                    recorded_at,
                }))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_store_append_and_chain() {
        let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
        let store = AuditStore::new(pool);

        let run_id = "run-test-01";

        // 1. Genesis event
        let ev0 = RuntimeEvent::RunInitiated {
            run_id: run_id.to_string(),
            prompt: "Init test".to_string(),
            team_name: Some("forester".to_string()),
        };
        let rec0 = store.append_event(run_id, None, &ev0).await.unwrap();
        assert_eq!(rec0.sequence_num, 0);
        assert_eq!(rec0.previous_hash, GENESIS_HASH);

        // 2. Second event
        let ev1 = RuntimeEvent::AgentStarted {
            run_id: run_id.to_string(),
            task_id: Some("t-1".to_string()),
            agent_id: "codex-01".to_string(),
            agent_name: "codex".to_string(),
        };
        let rec1 = store.append_event(run_id, Some("t-1"), &ev1).await.unwrap();
        assert_eq!(rec1.sequence_num, 1);
        assert_eq!(rec1.previous_hash, rec0.current_hash);

        // 3. Verify retrieved chain
        let chain = store.get_chain(run_id).await.unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].current_hash, chain[1].previous_hash);
    }

    #[tokio::test]
    async fn test_audit_store_rejects_out_of_order() {
        let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
        let store = AuditStore::new(pool);
        let run_id = "run-test-order";

        // Try to insert sequence 1 when none exists
        let recorded_at = Utc::now();
        let recorded_at_str = recorded_at.to_rfc3339();
        let bad_record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            run_id: run_id.to_string(),
            task_id: None,
            sequence_num: 1,
            event_type: "Test".to_string(),
            payload_json: "{}".to_string(),
            previous_hash: GENESIS_HASH.to_string(),
            current_hash: compute_audit_hash(GENESIS_HASH, 1, "Test", "{}", &recorded_at_str),
            recorded_at,
        };

        let result = store.append_record(&bad_record).await;
        assert!(matches!(result, Err(StorageError::InvalidSequence { expected: 0, actual: 1 })));
    }
}
