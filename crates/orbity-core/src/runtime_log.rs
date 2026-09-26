//! Layer 1: Runtime Logs - Agent and Process Lifecycle Observability.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::bus::{EventSink, EventSinkError};
use crate::events::{EventEnvelope, RuntimeEvent};

/// Log level severity for runtime lifecycle entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeLogLevel {
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for RuntimeLogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A structured entry representing an agent or process lifecycle transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeLogRecord {
    pub timestamp: DateTime<Utc>,
    pub level: RuntimeLogLevel,
    pub run_id: String,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_name: Option<String>,
    pub action: String,
    pub message: String,
    pub raw_event_type: String,
}

impl RuntimeLogRecord {
    /// Formats the log entry into a clean, canonical human-readable line.
    pub fn format_line(&self) -> String {
        let agent = self
            .agent_name
            .as_deref()
            .or(self.agent_id.as_deref())
            .unwrap_or("system");

        let task_str = self
            .task_id
            .as_ref()
            .map(|t| format!(" [task:{}]", t))
            .unwrap_or_default();

        format!(
            "[{}] [{}] [run:{}] [{}]{}: {}",
            self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
            self.level,
            self.run_id,
            agent,
            task_str,
            self.message
        )
    }

    /// Attempts to extract a `RuntimeLogRecord` from an `EventEnvelope`.
    /// Returns `None` if the event is not a Layer 1 runtime lifecycle event.
    pub fn from_envelope(envelope: &EventEnvelope) -> Option<Self> {
        let timestamp = envelope.timestamp;
        let run_id = envelope.run_id.clone();
        let task_id = envelope.task_id.clone();

        match &envelope.event {
            RuntimeEvent::RunInitiated {
                run_id: _,
                prompt,
                team_name,
            } => {
                let team_info = team_name
                    .as_deref()
                    .map(|t| format!(" with team '{}'", t))
                    .unwrap_or_default();
                Some(Self {
                    timestamp,
                    level: RuntimeLogLevel::Info,
                    run_id,
                    task_id,
                    agent_id: None,
                    agent_name: None,
                    action: "RunInitiated".to_string(),
                    message: format!("Run initiated{}: \"{}\"", team_info, prompt),
                    raw_event_type: "RunInitiated".to_string(),
                })
            }
            RuntimeEvent::RunCompleted {
                total_tokens,
                total_cost_usd,
                duration_ms,
                ..
            } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Info,
                run_id,
                task_id,
                agent_id: None,
                agent_name: None,
                action: "RunCompleted".to_string(),
                message: format!(
                    "Run completed successfully in {}ms (tokens: {}, cost: ${:.4})",
                    duration_ms, total_tokens, total_cost_usd
                ),
                raw_event_type: "RunCompleted".to_string(),
            }),
            RuntimeEvent::RunFailed { error, .. } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Error,
                run_id,
                task_id,
                agent_id: None,
                agent_name: None,
                action: "RunFailed".to_string(),
                message: format!("Run failed critically: {}", error),
                raw_event_type: "RunFailed".to_string(),
            }),
            RuntimeEvent::AgentCreated {
                agent_id,
                name,
                team_name,
            } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Info,
                run_id,
                task_id,
                agent_id: Some(agent_id.clone()),
                agent_name: Some(name.clone()),
                action: "AgentCreated".to_string(),
                message: format!(
                    "Agent '{}' (ID: {}) created for team '{:?}'",
                    name, agent_id, team_name
                ),
                raw_event_type: "AgentCreated".to_string(),
            }),
            RuntimeEvent::AgentStarted {
                agent_id,
                agent_name,
                ..
            } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Info,
                run_id,
                task_id,
                agent_id: Some(agent_id.clone()),
                agent_name: Some(agent_name.clone()),
                action: "AgentStarted".to_string(),
                message: format!("Agent '{}' transitioned to active execution", agent_name),
                raw_event_type: "AgentStarted".to_string(),
            }),
            RuntimeEvent::AgentFinished {
                agent_id,
                agent_name,
                summary,
                ..
            } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Info,
                run_id,
                task_id,
                agent_id: Some(agent_id.clone()),
                agent_name: Some(agent_name.clone()),
                action: "AgentFinished".to_string(),
                message: format!(
                    "Agent '{}' finished: {}",
                    agent_name,
                    summary.as_deref().unwrap_or("no summary provided")
                ),
                raw_event_type: "AgentFinished".to_string(),
            }),
            RuntimeEvent::AgentFailed {
                agent_id,
                agent_name,
                error,
                ..
            } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Error,
                run_id,
                task_id,
                agent_id: Some(agent_id.clone()),
                agent_name: Some(agent_name.clone()),
                action: "AgentFailed".to_string(),
                message: format!("Agent '{}' encountered error: {}", agent_name, error),
                raw_event_type: "AgentFailed".to_string(),
            }),
            RuntimeEvent::ApprovalRequired {
                prompt,
                proposed_cost_usd,
                timeout_seconds,
                ..
            } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Warn,
                run_id,
                task_id,
                agent_id: None,
                agent_name: None,
                action: "ApprovalRequired".to_string(),
                message: format!(
                    "HITL approval required: \"{}\" (cost: ${:?}, timeout: {:?}s)",
                    prompt, proposed_cost_usd, timeout_seconds
                ),
                raw_event_type: "ApprovalRequired".to_string(),
            }),
            RuntimeEvent::ApprovalGranted { approver, .. } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Info,
                run_id,
                task_id,
                agent_id: None,
                agent_name: None,
                action: "ApprovalGranted".to_string(),
                message: format!("HITL approval granted by '{}'", approver),
                raw_event_type: "ApprovalGranted".to_string(),
            }),
            RuntimeEvent::ApprovalRejected {
                rejecter, reason, ..
            } => Some(Self {
                timestamp,
                level: RuntimeLogLevel::Warn,
                run_id,
                task_id,
                agent_id: None,
                agent_name: None,
                action: "ApprovalRejected".to_string(),
                message: format!(
                    "HITL approval rejected by '{}' (reason: {})",
                    rejecter,
                    reason.as_deref().unwrap_or("none")
                ),
                raw_event_type: "ApprovalRejected".to_string(),
            }),
            _ => None,
        }
    }
}

/// An EventSink dedicated to collecting and filtering Layer 1 Runtime Logs.
#[derive(Debug, Clone, Default)]
pub struct RuntimeLogSink {
    name: String,
    records: Arc<Mutex<Vec<RuntimeLogRecord>>>,
}

impl RuntimeLogSink {
    pub fn new() -> Self {
        Self {
            name: "layer1-runtime-logs".to_string(),
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Retrieves all recorded runtime log entries.
    pub async fn records(&self) -> Vec<RuntimeLogRecord> {
        self.records.lock().await.clone()
    }

    /// Filters entries by a specific agent name.
    pub async fn by_agent(&self, agent_name: &str) -> Vec<RuntimeLogRecord> {
        self.records
            .lock()
            .await
            .iter()
            .filter(|r| r.agent_name.as_deref() == Some(agent_name))
            .cloned()
            .collect()
    }

    /// Returns only error records.
    pub async fn errors(&self) -> Vec<RuntimeLogRecord> {
        self.records
            .lock()
            .await
            .iter()
            .filter(|r| r.level == RuntimeLogLevel::Error)
            .cloned()
            .collect()
    }

    /// Clears all stored records.
    pub async fn clear(&self) {
        self.records.lock().await.clear();
    }
}

#[async_trait]
impl EventSink for RuntimeLogSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError> {
        if let Some(record) = RuntimeLogRecord::from_envelope(event) {
            self.records.lock().await.push(record);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::{EventBus, EventBusConfig};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_runtime_log_lifecycle_tracking() {
        let bus = EventBus::new(EventBusConfig::default());
        let runtime_sink = Arc::new(RuntimeLogSink::new());
        bus.register_sink(runtime_sink.clone()).await;

        let run_id = "run-rt-01";

        // 1. Run Initiated
        bus.emit(
            run_id,
            RuntimeEvent::RunInitiated {
                run_id: run_id.to_string(),
                prompt: "Build compiler".to_string(),
                team_name: Some("forester".to_string()),
            },
        )
        .await
        .unwrap();

        // 2. Agent Started
        bus.emit(
            run_id,
            RuntimeEvent::AgentStarted {
                run_id: run_id.to_string(),
                task_id: Some("task-01".to_string()),
                agent_id: "agt_codex".to_string(),
                agent_name: "codex".to_string(),
            },
        )
        .await
        .unwrap();

        // 3. Operational command (Layer 2) - should NOT create runtime log
        bus.emit(
            run_id,
            RuntimeEvent::CommandExecuted {
                run_id: run_id.to_string(),
                task_id: Some("task-01".to_string()),
                agent_name: "codex".to_string(),
                command: "cargo".to_string(),
                args: vec!["build".to_string()],
                exit_code: 0,
                duration_ms: 150,
                stdout_preview: Some("ok".to_string()),
                stderr_preview: None,
                sandbox_id: None,
            },
        )
        .await
        .unwrap();

        // 4. Agent Finished
        bus.emit(
            run_id,
            RuntimeEvent::AgentFinished {
                run_id: run_id.to_string(),
                task_id: Some("task-01".to_string()),
                agent_id: "agt_codex".to_string(),
                agent_name: "codex".to_string(),
                summary: Some("Build succeeded".to_string()),
            },
        )
        .await
        .unwrap();

        // 5. Agent Failed
        bus.emit(
            run_id,
            RuntimeEvent::AgentFailed {
                run_id: run_id.to_string(),
                task_id: Some("task-02".to_string()),
                agent_id: "agt_claude".to_string(),
                agent_name: "claude".to_string(),
                error: "Review timeout".to_string(),
            },
        )
        .await
        .unwrap();

        sleep(Duration::from_millis(60)).await;

        let records = runtime_sink.records().await;
        // RunInitiated + AgentStarted + AgentFinished + AgentFailed = 4 records
        assert_eq!(records.len(), 4);

        assert_eq!(records[0].action, "RunInitiated");
        assert_eq!(records[0].level, RuntimeLogLevel::Info);

        assert_eq!(records[1].agent_name.as_deref(), Some("codex"));
        assert_eq!(records[1].action, "AgentStarted");

        assert_eq!(records[2].agent_name.as_deref(), Some("codex"));
        assert_eq!(records[2].action, "AgentFinished");

        assert_eq!(records[3].agent_name.as_deref(), Some("claude"));
        assert_eq!(records[3].level, RuntimeLogLevel::Error);

        let errors = runtime_sink.errors().await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].agent_name.as_deref(), Some("claude"));

        // Test line formatting
        let line = records[1].format_line();
        assert!(line.contains("[INFO]"));
        assert!(line.contains("[codex]"));
        assert!(line.contains("active execution"));
    }
}
