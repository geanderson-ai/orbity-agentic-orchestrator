//! Layer 3: Audit Logs - Asynchronous EventSink bridge to SQLite SHA-256 hash-chain ledger.

use async_trait::async_trait;
use orbity_core::bus::{EventSink, EventSinkError};
use orbity_core::events::EventEnvelope;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::audit_chain::AuditStore;
use crate::audit_verifier::{AuditVerificationResult, AuditVerifier};
use crate::error::StorageError;

/// Filtering mode for the Audit Sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuditFilterMode {
    /// Audits all events published to the bus.
    #[default]
    AllEvents,
    /// Audits only governance, security policies, and lifecycle events (Layer 3).
    SecurityAndGovernanceOnly,
}

/// An EventSink that persists events into the SQLite cryptographic audit chain ledger.
#[derive(Debug, Clone)]
pub struct AuditLogSink {
    name: String,
    audit_store: AuditStore,
    filter_mode: AuditFilterMode,
    audited_count: Arc<AtomicU64>,
}

impl AuditLogSink {
    /// Creates a new `AuditLogSink` with default filtering (AllEvents).
    pub fn new(audit_store: AuditStore) -> Self {
        Self {
            name: "layer3-audit-sqlite".to_string(),
            audit_store,
            filter_mode: AuditFilterMode::AllEvents,
            audited_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Creates a new `AuditLogSink` with specified filtering mode.
    pub fn with_filter_mode(audit_store: AuditStore, filter_mode: AuditFilterMode) -> Self {
        Self {
            name: "layer3-audit-sqlite".to_string(),
            audit_store,
            filter_mode,
            audited_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Returns a reference to the underlying `AuditStore`.
    pub fn audit_store(&self) -> &AuditStore {
        &self.audit_store
    }

    /// Returns the total number of events successfully appended to SQLite audit ledger.
    pub fn audited_count(&self) -> u64 {
        self.audited_count.load(Ordering::Relaxed)
    }

    /// Verifies the cryptographic hash-chain integrity of a specific run in SQLite.
    pub async fn verify_run_integrity(
        &self,
        run_id: &str,
    ) -> Result<AuditVerificationResult, StorageError> {
        let verifier = AuditVerifier::new(self.audit_store.pool().clone());
        verifier.verify_run(run_id).await
    }
}

#[async_trait]
impl EventSink for AuditLogSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError> {
        // Apply filter mode
        let should_audit = match self.filter_mode {
            AuditFilterMode::AllEvents => true,
            AuditFilterMode::SecurityAndGovernanceOnly => event.event.is_audit_security(),
        };

        if !should_audit {
            return Ok(());
        }

        self.audit_store
            .append_event(&event.run_id, event.task_id.as_deref(), &event.event)
            .await
            .map_err(|e| {
                EventSinkError::Delivery(format!("Failed to record audit event: {}", e))
            })?;

        self.audited_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::SqliteStoragePool;
    use orbity_core::bus::{EventBus, EventBusConfig};
    use orbity_core::events::RuntimeEvent;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_audit_sink_persists_and_verifies_hash_chain() {
        let pool = SqliteStoragePool::connect_in_memory()
            .await
            .expect("Connect in-memory");
        let audit_store = AuditStore::new(pool.clone());
        let audit_sink = Arc::new(AuditLogSink::new(audit_store));

        let bus = EventBus::new(EventBusConfig::default());
        bus.register_sink(audit_sink.clone()).await;

        let run_id = "run-audit-sink-01";

        // 1. RunInitiated
        bus.emit(
            run_id,
            RuntimeEvent::RunInitiated {
                run_id: run_id.to_string(),
                prompt: "Security Audit Test".to_string(),
                team_name: Some("forester".to_string()),
            },
        )
        .await
        .unwrap();

        // 2. PolicyDenied (Security event)
        bus.emit(
            run_id,
            RuntimeEvent::PolicyDenied {
                run_id: run_id.to_string(),
                agent_name: "hermes".to_string(),
                action: "network_connect".to_string(),
                resource: "192.168.1.50".to_string(),
                reason: "Attempted outbound connection in isolated sandbox".to_string(),
            },
        )
        .await
        .unwrap();

        // 3. SecretRequested
        bus.emit(
            run_id,
            RuntimeEvent::SecretRequested {
                run_id: run_id.to_string(),
                agent_name: "codex".to_string(),
                secret_id: "OPENAI_API_KEY".to_string(),
            },
        )
        .await
        .unwrap();

        // 4. FileWritten
        bus.emit(
            run_id,
            RuntimeEvent::FileWritten {
                run_id: run_id.to_string(),
                task_id: Some("task-01".to_string()),
                agent_name: "codex".to_string(),
                file_path: "crates/lib.rs".to_string(),
                bytes_written: 1024,
                content_hash: "a1b2c3d4e5f6".to_string(),
            },
        )
        .await
        .unwrap();

        // Allow worker to process
        sleep(Duration::from_millis(80)).await;

        assert_eq!(audit_sink.audited_count(), 4);

        // Verify cryptographic hash chain with AuditVerifier
        let result = audit_sink
            .verify_run_integrity(run_id)
            .await
            .expect("Verification failed");

        match result {
            AuditVerificationResult::Valid {
                total_events,
                final_hash,
                ..
            } => {
                assert_eq!(total_events, 4);
                assert!(!final_hash.is_empty());
            }
            _ => panic!("Expected Valid hash chain, got {:?}", result),
        }
    }
}
