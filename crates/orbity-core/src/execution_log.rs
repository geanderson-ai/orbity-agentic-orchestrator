//! Layer 2: Execution Logs - Operational actions (commands, tools, filesystem, network).

use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::bus::{EventSink, EventSinkError};
use crate::events::{EventEnvelope, RuntimeEvent};

/// Operational action type for Layer 2 execution logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionActionType {
    Command,
    Tool,
    FileRead,
    FileWrite,
    Network,
    SandboxSetup,
    SandboxTeardown,
}

impl std::fmt::Display for ExecutionActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command => write!(f, "COMMAND"),
            Self::Tool => write!(f, "TOOL"),
            Self::FileRead => write!(f, "FILE_READ"),
            Self::FileWrite => write!(f, "FILE_WRITE"),
            Self::Network => write!(f, "NETWORK"),
            Self::SandboxSetup => write!(f, "SBX_SETUP"),
            Self::SandboxTeardown => write!(f, "SBX_TEARDOWN"),
        }
    }
}

/// Detailed operational payload of an execution event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionDetails {
    Command {
        command: String,
        args: Vec<String>,
        exit_code: i32,
        stdout_preview: Option<String>,
        stderr_preview: Option<String>,
        sandbox_id: Option<String>,
    },
    Tool {
        tool_name: String,
        input: serde_json::Value,
        output: Option<serde_json::Value>,
    },
    FileRead {
        path: String,
        bytes_read: usize,
    },
    FileWrite {
        path: String,
        bytes_written: usize,
        content_hash: String,
    },
    Network {
        url_or_host: String,
        method: String,
        status_code: Option<u16>,
    },
    Sandbox {
        sandbox_id: String,
        workspace_path: Option<String>,
    },
}

/// A structured entry representing an operational execution action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionLogRecord {
    pub timestamp: DateTime<Utc>,
    pub run_id: String,
    pub task_id: Option<String>,
    pub agent_name: String,
    pub action_type: ExecutionActionType,
    pub target: String,
    pub details: ExecutionDetails,
    pub duration_ms: u64,
    pub success: bool,
}

impl ExecutionLogRecord {
    /// Formats the execution entry into a clear, single-line operational string.
    pub fn format_line(&self) -> String {
        let status_str = if self.success { "OK" } else { "FAIL" };
        let task_str = self
            .task_id
            .as_ref()
            .map(|t| format!(" [task:{}]", t))
            .unwrap_or_default();

        match &self.details {
            ExecutionDetails::Command {
                exit_code,
                sandbox_id,
                ..
            } => {
                let sbx_info = sandbox_id
                    .as_deref()
                    .map(|s| format!(" sbx:{}", s))
                    .unwrap_or_default();
                format!(
                    "[{}] [EXEC:{}] [run:{}] [{}]{}: '{}' (exit: {}, {}ms{}) [{}]",
                    self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                    self.action_type,
                    self.run_id,
                    self.agent_name,
                    task_str,
                    self.target,
                    exit_code,
                    self.duration_ms,
                    sbx_info,
                    status_str
                )
            }
            ExecutionDetails::FileWrite {
                bytes_written,
                content_hash,
                ..
            } => {
                let hash_prefix = if content_hash.len() >= 8 {
                    &content_hash[..8]
                } else {
                    content_hash
                };
                format!(
                    "[{}] [EXEC:{}] [run:{}] [{}]{}: wrote '{}' ({} bytes, sha256:{}...) [{}]",
                    self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                    self.action_type,
                    self.run_id,
                    self.agent_name,
                    task_str,
                    self.target,
                    bytes_written,
                    hash_prefix,
                    status_str
                )
            }
            ExecutionDetails::FileRead { bytes_read, .. } => {
                format!(
                    "[{}] [EXEC:{}] [run:{}] [{}]{}: read '{}' ({} bytes) [{}]",
                    self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                    self.action_type,
                    self.run_id,
                    self.agent_name,
                    task_str,
                    self.target,
                    bytes_read,
                    status_str
                )
            }
            ExecutionDetails::Tool { tool_name, .. } => {
                format!(
                    "[{}] [EXEC:{}] [run:{}] [{}]{}: invoked tool '{}' ({}ms) [{}]",
                    self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                    self.action_type,
                    self.run_id,
                    self.agent_name,
                    task_str,
                    tool_name,
                    self.duration_ms,
                    status_str
                )
            }
            ExecutionDetails::Network {
                url_or_host,
                method,
                status_code,
            } => {
                format!(
                    "[{}] [EXEC:{}] [run:{}] [{}]{}: {} {} (status: {:?}) [{}]",
                    self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                    self.action_type,
                    self.run_id,
                    self.agent_name,
                    task_str,
                    method,
                    url_or_host,
                    status_code,
                    status_str
                )
            }
            ExecutionDetails::Sandbox { sandbox_id, .. } => {
                format!(
                    "[{}] [EXEC:{}] [run:{}] [{}]{}: sandbox '{}' ({}) [{}]",
                    self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                    self.action_type,
                    self.run_id,
                    self.agent_name,
                    task_str,
                    sandbox_id,
                    self.target,
                    status_str
                )
            }
        }
    }

    /// Attempts to parse an `ExecutionLogRecord` from an `EventEnvelope`.
    /// Returns `None` if the event does not belong to Layer 2 execution logs.
    pub fn from_envelope(envelope: &EventEnvelope) -> Option<Self> {
        let timestamp = envelope.timestamp;
        let run_id = envelope.run_id.clone();
        let task_id = envelope.task_id.clone();

        match &envelope.event {
            RuntimeEvent::CommandExecuted {
                agent_name,
                command,
                args,
                exit_code,
                duration_ms,
                stdout_preview,
                stderr_preview,
                sandbox_id,
                ..
            } => {
                let full_cmd = if args.is_empty() {
                    command.clone()
                } else {
                    format!("{} {}", command, args.join(" "))
                };
                Some(Self {
                    timestamp,
                    run_id,
                    task_id,
                    agent_name: agent_name.clone(),
                    action_type: ExecutionActionType::Command,
                    target: full_cmd,
                    details: ExecutionDetails::Command {
                        command: command.clone(),
                        args: args.clone(),
                        exit_code: *exit_code,
                        stdout_preview: stdout_preview.clone(),
                        stderr_preview: stderr_preview.clone(),
                        sandbox_id: sandbox_id.clone(),
                    },
                    duration_ms: *duration_ms,
                    success: *exit_code == 0,
                })
            }
            RuntimeEvent::ToolCalled {
                agent_name,
                tool_name,
                input,
                output,
                duration_ms,
                ..
            } => Some(Self {
                timestamp,
                run_id,
                task_id,
                agent_name: agent_name.clone(),
                action_type: ExecutionActionType::Tool,
                target: tool_name.clone(),
                details: ExecutionDetails::Tool {
                    tool_name: tool_name.clone(),
                    input: input.clone(),
                    output: output.clone(),
                },
                duration_ms: *duration_ms,
                success: true,
            }),
            RuntimeEvent::FileRead {
                agent_name,
                file_path,
                bytes_read,
                ..
            } => Some(Self {
                timestamp,
                run_id,
                task_id,
                agent_name: agent_name.clone(),
                action_type: ExecutionActionType::FileRead,
                target: file_path.clone(),
                details: ExecutionDetails::FileRead {
                    path: file_path.clone(),
                    bytes_read: *bytes_read,
                },
                duration_ms: 0,
                success: true,
            }),
            RuntimeEvent::FileWritten {
                agent_name,
                file_path,
                bytes_written,
                content_hash,
                ..
            } => Some(Self {
                timestamp,
                run_id,
                task_id,
                agent_name: agent_name.clone(),
                action_type: ExecutionActionType::FileWrite,
                target: file_path.clone(),
                details: ExecutionDetails::FileWrite {
                    path: file_path.clone(),
                    bytes_written: *bytes_written,
                    content_hash: content_hash.clone(),
                },
                duration_ms: 0,
                success: true,
            }),
            RuntimeEvent::NetworkRequest {
                agent_name,
                url,
                method,
                status_code,
                ..
            } => Some(Self {
                timestamp,
                run_id,
                task_id,
                agent_name: agent_name.clone(),
                action_type: ExecutionActionType::Network,
                target: url.clone(),
                details: ExecutionDetails::Network {
                    url_or_host: url.clone(),
                    method: method.clone(),
                    status_code: *status_code,
                },
                duration_ms: 0,
                success: status_code.map(|s| s < 400).unwrap_or(true),
            }),
            RuntimeEvent::SandboxCreated {
                sandbox_id,
                path,
                provider: _,
                ..
            } => Some(Self {
                timestamp,
                run_id,
                task_id,
                agent_name: "sandbox".to_string(),
                action_type: ExecutionActionType::SandboxSetup,
                target: "Created".to_string(),
                details: ExecutionDetails::Sandbox {
                    sandbox_id: sandbox_id.clone(),
                    workspace_path: Some(path.clone()),
                },
                duration_ms: 0,
                success: true,
            }),
            RuntimeEvent::SandboxDestroyed {
                sandbox_id,
                duration_ms,
                ..
            } => Some(Self {
                timestamp,
                run_id,
                task_id,
                agent_name: "sandbox".to_string(),
                action_type: ExecutionActionType::SandboxTeardown,
                target: "Destroyed".to_string(),
                details: ExecutionDetails::Sandbox {
                    sandbox_id: sandbox_id.clone(),
                    workspace_path: None,
                },
                duration_ms: *duration_ms,
                success: true,
            }),
            _ => None,
        }
    }
}

/// An EventSink dedicated to collecting and filtering Layer 2 Execution Logs.
#[derive(Debug, Clone, Default)]
pub struct ExecutionLogSink {
    name: String,
    records: Arc<Mutex<Vec<ExecutionLogRecord>>>,
}

impl ExecutionLogSink {
    pub fn new() -> Self {
        Self {
            name: "layer2-execution-logs".to_string(),
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns all execution records collected.
    pub async fn records(&self) -> Vec<ExecutionLogRecord> {
        self.records.lock().await.clone()
    }

    /// Filters commands executed.
    pub async fn commands(&self) -> Vec<ExecutionLogRecord> {
        self.records
            .lock()
            .await
            .iter()
            .filter(|r| r.action_type == ExecutionActionType::Command)
            .cloned()
            .collect()
    }

    /// Filters files written and modified.
    pub async fn files_written(&self) -> Vec<ExecutionLogRecord> {
        self.records
            .lock()
            .await
            .iter()
            .filter(|r| r.action_type == ExecutionActionType::FileWrite)
            .cloned()
            .collect()
    }

    /// Returns all failed operational actions (non-zero exit code or network failure).
    pub async fn failures(&self) -> Vec<ExecutionLogRecord> {
        self.records
            .lock()
            .await
            .iter()
            .filter(|r| !r.success)
            .cloned()
            .collect()
    }

    /// Clears recorded entries.
    pub async fn clear(&self) {
        self.records.lock().await.clear();
    }
}

#[async_trait]
impl EventSink for ExecutionLogSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError> {
        if let Some(record) = ExecutionLogRecord::from_envelope(event) {
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
    async fn test_execution_log_captures_commands_and_files() {
        let bus = EventBus::new(EventBusConfig::default());
        let exec_sink = Arc::new(ExecutionLogSink::new());
        bus.register_sink(exec_sink.clone()).await;

        let run_id = "run-exec-01";

        // 1. CommandExecuted (Success)
        bus.emit(
            run_id,
            RuntimeEvent::CommandExecuted {
                run_id: run_id.to_string(),
                task_id: Some("task-build".to_string()),
                agent_name: "codex".to_string(),
                command: "cargo".to_string(),
                args: vec!["build".to_string(), "--release".to_string()],
                exit_code: 0,
                duration_ms: 1200,
                stdout_preview: Some("Finished release".to_string()),
                stderr_preview: None,
                sandbox_id: Some("sbx-44".to_string()),
            },
        )
        .await
        .unwrap();

        // 2. FileWritten
        bus.emit(
            run_id,
            RuntimeEvent::FileWritten {
                run_id: run_id.to_string(),
                task_id: Some("task-build".to_string()),
                agent_name: "codex".to_string(),
                file_path: "src/engine.rs".to_string(),
                bytes_written: 4096,
                content_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
            },
        )
        .await
        .unwrap();

        // 3. CommandExecuted (Failure: exit 101)
        bus.emit(
            run_id,
            RuntimeEvent::CommandExecuted {
                run_id: run_id.to_string(),
                task_id: Some("task-test".to_string()),
                agent_name: "codex".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string()],
                exit_code: 101,
                duration_ms: 450,
                stdout_preview: None,
                stderr_preview: Some("assertion failed".to_string()),
                sandbox_id: Some("sbx-44".to_string()),
            },
        )
        .await
        .unwrap();

        // 4. Runtime event (AgentStarted) - should NOT be captured by ExecutionLogSink
        bus.emit(
            run_id,
            RuntimeEvent::AgentStarted {
                run_id: run_id.to_string(),
                task_id: None,
                agent_id: "agt_hermes".to_string(),
                agent_name: "hermes".to_string(),
            },
        )
        .await
        .unwrap();

        sleep(Duration::from_millis(60)).await;

        let records = exec_sink.records().await;
        assert_eq!(records.len(), 3); // 2 commands + 1 file write

        let commands = exec_sink.commands().await;
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].target, "cargo build --release");
        assert!(commands[0].success);
        assert_eq!(commands[0].duration_ms, 1200);

        let files = exec_sink.files_written().await;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].target, "src/engine.rs");
        if let ExecutionDetails::FileWrite {
            content_hash,
            bytes_written,
            ..
        } = &files[0].details
        {
            assert_eq!(*bytes_written, 4096);
            assert_eq!(
                content_hash,
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
        } else {
            panic!("Expected FileWrite details");
        }

        let failures = exec_sink.failures().await;
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].target, "cargo test");
        assert!(!failures[0].success);

        // Test line formatting
        let line = files[0].format_line();
        assert!(line.contains("[EXEC:FILE_WRITE]"));
        assert!(line.contains("wrote 'src/engine.rs'"));
        assert!(line.contains("4096 bytes"));
    }
}
