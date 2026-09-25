//! Historical cryptographic audit verifier.

use crate::audit_chain::{compute_audit_hash, AuditStore, GENESIS_HASH};
use crate::error::StorageError;
use crate::pool::SqliteStoragePool;
use serde::{Deserialize, Serialize};

/// Result of verifying the cryptographic audit trail for a given execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditVerificationResult {
    Valid {
        run_id: String,
        total_events: usize,
        final_hash: String,
    },
    Empty {
        run_id: String,
    },
    Tampered {
        run_id: String,
        event_id: String,
        sequence_num: i64,
        expected_hash: String,
        actual_hash: String,
        reason: String,
    },
}

#[derive(Clone, Debug)]
pub struct AuditVerifier {
    pool: SqliteStoragePool,
}

impl AuditVerifier {
    pub fn new(pool: SqliteStoragePool) -> Self {
        Self { pool }
    }

    /// Validates the complete cryptographic chain of an execution run.
    pub async fn verify_run(&self, run_id: &str) -> Result<AuditVerificationResult, StorageError> {
        let store = AuditStore::new(self.pool.clone());
        let chain = store.get_chain(run_id).await?;

        if chain.is_empty() {
            return Ok(AuditVerificationResult::Empty {
                run_id: run_id.to_string(),
            });
        }

        let mut expected_previous_hash = GENESIS_HASH.to_string();

        for (idx, record) in chain.iter().enumerate() {
            let expected_seq = idx as i64;
            if record.sequence_num != expected_seq {
                return Ok(AuditVerificationResult::Tampered {
                    run_id: run_id.to_string(),
                    event_id: record.id.clone(),
                    sequence_num: record.sequence_num,
                    expected_hash: "".to_string(),
                    actual_hash: "".to_string(),
                    reason: format!(
                        "Broken sequence: expected index {}, found {}",
                        expected_seq, record.sequence_num
                    ),
                });
            }

            if record.previous_hash != expected_previous_hash {
                return Ok(AuditVerificationResult::Tampered {
                    run_id: run_id.to_string(),
                    event_id: record.id.clone(),
                    sequence_num: record.sequence_num,
                    expected_hash: expected_previous_hash,
                    actual_hash: record.previous_hash.clone(),
                    reason: "Broken hash chain: previous_hash does not match earlier block current_hash"
                        .to_string(),
                });
            }

            let recorded_at_str = record.recorded_at.to_rfc3339();
            let expected_current_hash = compute_audit_hash(
                &record.previous_hash,
                record.sequence_num,
                &record.event_type,
                &record.payload_json,
                &recorded_at_str,
            );

            if record.current_hash != expected_current_hash {
                return Ok(AuditVerificationResult::Tampered {
                    run_id: run_id.to_string(),
                    event_id: record.id.clone(),
                    sequence_num: record.sequence_num,
                    expected_hash: expected_current_hash,
                    actual_hash: record.current_hash.clone(),
                    reason: "Cryptographic hash mismatch: payload or block metadata was modified"
                        .to_string(),
                });
            }

            expected_previous_hash = record.current_hash.clone();
        }

        let final_hash = expected_previous_hash;
        Ok(AuditVerificationResult::Valid {
            run_id: run_id.to_string(),
            total_events: chain.len(),
            final_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbity_core::events::RuntimeEvent;

    #[tokio::test]
    async fn test_audit_verification_valid_run() {
        let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
        let store = AuditStore::new(pool.clone());
        let verifier = AuditVerifier::new(pool);
        let run_id = "run-valid-01";

        // Add 5 events
        for i in 0..5 {
            let event = RuntimeEvent::CommandExecuted {
                run_id: run_id.to_string(),
                task_id: Some(format!("task-{}", i)),
                agent_name: "codex".to_string(),
                command: "echo".to_string(),
                args: vec![format!("step {}", i)],
                exit_code: 0,
                duration_ms: 10,
                stdout_preview: Some("ok".to_string()),
                stderr_preview: None,
                sandbox_id: None,
            };
            store.append_event(run_id, Some(&format!("task-{}", i)), &event).await.unwrap();
        }

        let result = verifier.verify_run(run_id).await.unwrap();
        match result {
            AuditVerificationResult::Valid { total_events, .. } => {
                assert_eq!(total_events, 5);
            }
            other => panic!("Expected valid audit result, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_audit_verification_detects_tampered_payload() {
        let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
        let store = AuditStore::new(pool.clone());
        let verifier = AuditVerifier::new(pool.clone());
        let run_id = "run-tampered-01";

        // Add 3 events
        for i in 0..3 {
            let event = RuntimeEvent::CommandExecuted {
                run_id: run_id.to_string(),
                task_id: Some(format!("task-{}", i)),
                agent_name: "codex".to_string(),
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                exit_code: 0,
                duration_ms: 10,
                stdout_preview: None,
                stderr_preview: None,
                sandbox_id: None,
            };
            store.append_event(run_id, None, &event).await.unwrap();
        }

        // Tamper 1 byte directly in the SQLite database for sequence_num = 1
        sqlx::query(
            "UPDATE audit_events SET payload_json = json_set(payload_json, '$.data.exit_code', 1) WHERE run_id = ? AND sequence_num = 1"
        )
        .bind(run_id)
        .execute(pool.inner())
        .await
        .unwrap();

        let result = verifier.verify_run(run_id).await.unwrap();
        match result {
            AuditVerificationResult::Tampered { sequence_num, reason, .. } => {
                assert_eq!(sequence_num, 1);
                assert!(reason.contains("hash mismatch"));
            }
            other => panic!("Expected tampered detection, got {:?}", other),
        }
    }
}
