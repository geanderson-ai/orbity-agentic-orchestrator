//! Canonical structured events taxonomy for the Orbity Agentic Platform.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum RuntimeEvent {
    // Lifecycle
    AgentCreated {
        agent_id: String,
        name: String,
        team_name: Option<String>,
    },
    AgentStarted {
        run_id: String,
        task_id: Option<String>,
        agent_id: String,
        agent_name: String,
    },
    AgentFinished {
        run_id: String,
        task_id: Option<String>,
        agent_id: String,
        agent_name: String,
        summary: Option<String>,
    },
    AgentFailed {
        run_id: String,
        task_id: Option<String>,
        agent_id: String,
        agent_name: String,
        error: String,
    },
    RunInitiated {
        run_id: String,
        prompt: String,
        team_name: Option<String>,
    },
    RunCompleted {
        run_id: String,
        total_tokens: u64,
        total_cost_usd: f64,
        duration_ms: u64,
    },
    RunFailed {
        run_id: String,
        error: String,
    },

    // Execution
    CommandExecuted {
        run_id: String,
        task_id: Option<String>,
        agent_name: String,
        command: String,
        args: Vec<String>,
        exit_code: i32,
        duration_ms: u64,
        stdout_preview: Option<String>,
        stderr_preview: Option<String>,
        sandbox_id: Option<String>,
    },
    ToolCalled {
        run_id: String,
        task_id: Option<String>,
        agent_name: String,
        tool_name: String,
        input: serde_json::Value,
        output: Option<serde_json::Value>,
        duration_ms: u64,
    },
    FileRead {
        run_id: String,
        task_id: Option<String>,
        agent_name: String,
        file_path: String,
        bytes_read: usize,
    },
    FileWritten {
        run_id: String,
        task_id: Option<String>,
        agent_name: String,
        file_path: String,
        bytes_written: usize,
        content_hash: String,
    },
    NetworkRequest {
        run_id: String,
        task_id: Option<String>,
        agent_name: String,
        url: String,
        method: String,
        status_code: Option<u16>,
        duration_ms: u64,
    },

    // Policies & Security
    PolicyAllowed {
        run_id: String,
        agent_name: String,
        action: String,
        resource: String,
    },
    PolicyDenied {
        run_id: String,
        agent_name: String,
        action: String,
        resource: String,
        reason: String,
    },
    SecretRequested {
        run_id: String,
        agent_name: String,
        secret_id: String,
    },
    SecretGranted {
        run_id: String,
        agent_name: String,
        secret_id: String,
        value_logged: bool,
    },

    // Sandbox
    SandboxCreated {
        sandbox_id: String,
        run_id: String,
        path: String,
        provider: String,
    },
    SandboxDestroyed {
        sandbox_id: String,
        run_id: String,
        duration_ms: u64,
    },

    // FinOps
    TokenUsageUpdated {
        run_id: String,
        task_id: Option<String>,
        agent_name: String,
        input_tokens: u64,
        output_tokens: u64,
        cached_tokens: u64,
        reasoning_tokens: u64,
        cost_usd: f64,
    },
    BudgetThresholdReached {
        run_id: String,
        current_cost_usd: f64,
        threshold_usd: f64,
        percentage: f64,
    },
    BudgetExceeded {
        run_id: String,
        current_cost_usd: f64,
        max_budget_usd: f64,
    },
    ContextWindowThresholdReached {
        run_id: String,
        agent_name: String,
        current_tokens: u64,
        max_context_tokens: u64,
    },

    // Sync & HITL
    AgentSyncedFromYaml {
        file_path: String,
        agent_id: String,
        config_hash: String,
    },
    ApprovalRequired {
        run_id: String,
        prompt: String,
        proposed_cost_usd: Option<f64>,
        timeout_seconds: Option<u64>,
    },
    ApprovalGranted {
        run_id: String,
        approver: String,
        approved_at: DateTime<Utc>,
    },
    ApprovalRejected {
        run_id: String,
        rejecter: String,
        reason: Option<String>,
    },
}

impl RuntimeEvent {
    /// Returns the canonical event type name string
    pub fn event_type_name(&self) -> &'static str {
        match self {
            Self::AgentCreated { .. } => "AgentCreated",
            Self::AgentStarted { .. } => "AgentStarted",
            Self::AgentFinished { .. } => "AgentFinished",
            Self::AgentFailed { .. } => "AgentFailed",
            Self::RunInitiated { .. } => "RunInitiated",
            Self::RunCompleted { .. } => "RunCompleted",
            Self::RunFailed { .. } => "RunFailed",
            Self::CommandExecuted { .. } => "CommandExecuted",
            Self::ToolCalled { .. } => "ToolCalled",
            Self::FileRead { .. } => "FileRead",
            Self::FileWritten { .. } => "FileWritten",
            Self::NetworkRequest { .. } => "NetworkRequest",
            Self::PolicyAllowed { .. } => "PolicyAllowed",
            Self::PolicyDenied { .. } => "PolicyDenied",
            Self::SecretRequested { .. } => "SecretRequested",
            Self::SecretGranted { .. } => "SecretGranted",
            Self::SandboxCreated { .. } => "SandboxCreated",
            Self::SandboxDestroyed { .. } => "SandboxDestroyed",
            Self::TokenUsageUpdated { .. } => "TokenUsageUpdated",
            Self::BudgetThresholdReached { .. } => "BudgetThresholdReached",
            Self::BudgetExceeded { .. } => "BudgetExceeded",
            Self::ContextWindowThresholdReached { .. } => "ContextWindowThresholdReached",
            Self::AgentSyncedFromYaml { .. } => "AgentSyncedFromYaml",
            Self::ApprovalRequired { .. } => "ApprovalRequired",
            Self::ApprovalGranted { .. } => "ApprovalGranted",
            Self::ApprovalRejected { .. } => "ApprovalRejected",
        }
    }

    /// Returns the associated run_id if any
    pub fn run_id(&self) -> Option<&str> {
        match self {
            Self::AgentCreated { .. } | Self::AgentSyncedFromYaml { .. } => None,
            Self::AgentStarted { run_id, .. }
            | Self::AgentFinished { run_id, .. }
            | Self::AgentFailed { run_id, .. }
            | Self::RunInitiated { run_id, .. }
            | Self::RunCompleted { run_id, .. }
            | Self::RunFailed { run_id, .. }
            | Self::CommandExecuted { run_id, .. }
            | Self::ToolCalled { run_id, .. }
            | Self::FileRead { run_id, .. }
            | Self::FileWritten { run_id, .. }
            | Self::NetworkRequest { run_id, .. }
            | Self::PolicyAllowed { run_id, .. }
            | Self::PolicyDenied { run_id, .. }
            | Self::SecretRequested { run_id, .. }
            | Self::SecretGranted { run_id, .. }
            | Self::SandboxCreated { run_id, .. }
            | Self::SandboxDestroyed { run_id, .. }
            | Self::TokenUsageUpdated { run_id, .. }
            | Self::BudgetThresholdReached { run_id, .. }
            | Self::BudgetExceeded { run_id, .. }
            | Self::ContextWindowThresholdReached { run_id, .. }
            | Self::ApprovalRequired { run_id, .. }
            | Self::ApprovalGranted { run_id, .. }
            | Self::ApprovalRejected { run_id, .. } => Some(run_id),
        }
    }

    /// Returns the associated task_id if any
    pub fn task_id(&self) -> Option<&str> {
        match self {
            Self::AgentStarted { task_id, .. }
            | Self::AgentFinished { task_id, .. }
            | Self::AgentFailed { task_id, .. }
            | Self::CommandExecuted { task_id, .. }
            | Self::ToolCalled { task_id, .. }
            | Self::FileRead { task_id, .. }
            | Self::FileWritten { task_id, .. }
            | Self::NetworkRequest { task_id, .. }
            | Self::TokenUsageUpdated { task_id, .. } => task_id.as_deref(),
            _ => None,
        }
    }

    /// Returns the associated agent_name if any
    pub fn agent_name(&self) -> Option<&str> {
        match self {
            Self::AgentCreated { name, .. } => Some(name),
            Self::AgentStarted { agent_name, .. }
            | Self::AgentFinished { agent_name, .. }
            | Self::AgentFailed { agent_name, .. }
            | Self::CommandExecuted { agent_name, .. }
            | Self::ToolCalled { agent_name, .. }
            | Self::FileRead { agent_name, .. }
            | Self::FileWritten { agent_name, .. }
            | Self::NetworkRequest { agent_name, .. }
            | Self::PolicyAllowed { agent_name, .. }
            | Self::PolicyDenied { agent_name, .. }
            | Self::SecretRequested { agent_name, .. }
            | Self::SecretGranted { agent_name, .. }
            | Self::TokenUsageUpdated { agent_name, .. } => Some(agent_name),
            _ => None,
        }
    }

    /// Whether this event belongs to Layer 1: Runtime lifecycle
    pub fn is_runtime_lifecycle(&self) -> bool {
        matches!(
            self,
            Self::AgentCreated { .. }
                | Self::AgentStarted { .. }
                | Self::AgentFinished { .. }
                | Self::AgentFailed { .. }
                | Self::RunInitiated { .. }
                | Self::RunCompleted { .. }
                | Self::RunFailed { .. }
                | Self::ApprovalRequired { .. }
                | Self::ApprovalGranted { .. }
                | Self::ApprovalRejected { .. }
        )
    }

    /// Whether this event belongs to Layer 2: Operational execution
    pub fn is_execution(&self) -> bool {
        matches!(
            self,
            Self::CommandExecuted { .. }
                | Self::ToolCalled { .. }
                | Self::FileRead { .. }
                | Self::FileWritten { .. }
                | Self::NetworkRequest { .. }
                | Self::SandboxCreated { .. }
                | Self::SandboxDestroyed { .. }
        )
    }

    /// Whether this event belongs to Layer 3: Audit, security and immutable governance
    pub fn is_audit_security(&self) -> bool {
        matches!(
            self,
            Self::RunInitiated { .. }
                | Self::RunCompleted { .. }
                | Self::RunFailed { .. }
                | Self::PolicyAllowed { .. }
                | Self::PolicyDenied { .. }
                | Self::SecretRequested { .. }
                | Self::SecretGranted { .. }
                | Self::FileWritten { .. }
                | Self::BudgetExceeded { .. }
                | Self::ApprovalRequired { .. }
                | Self::ApprovalGranted { .. }
                | Self::ApprovalRejected { .. }
        )
    }

    /// Whether this event belongs to Layer 4: Telemetry & FinOps metrics
    pub fn is_telemetry(&self) -> bool {
        matches!(
            self,
            Self::TokenUsageUpdated { .. }
                | Self::BudgetThresholdReached { .. }
                | Self::BudgetExceeded { .. }
                | Self::ContextWindowThresholdReached { .. }
                | Self::CommandExecuted { .. }
                | Self::ToolCalled { .. }
                | Self::RunCompleted { .. }
        )
    }
}

/// An envelope wrapping an event with unique metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub run_id: String,
    pub task_id: Option<String>,
    pub sequence_num: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub event: RuntimeEvent,
}

impl EventEnvelope {
    pub fn new(run_id: impl Into<String>, event: RuntimeEvent) -> Self {
        let task_id = event.task_id().map(|s| s.to_string());
        Self {
            id: Uuid::new_v4(),
            run_id: run_id.into(),
            task_id,
            sequence_num: None,
            timestamp: Utc::now(),
            event,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_events_serialization_roundtrip() {
        let event = RuntimeEvent::CommandExecuted {
            run_id: "run-123".to_string(),
            task_id: Some("task-456".to_string()),
            agent_name: "codex".to_string(),
            command: "cargo".to_string(),
            args: vec!["test".to_string(), "--workspace".to_string()],
            exit_code: 0,
            duration_ms: 1240,
            stdout_preview: Some("test result: ok".to_string()),
            stderr_preview: None,
            sandbox_id: Some("sbx-789".to_string()),
        };

        let json = serde_json::to_string(&event).expect("serialize event");
        assert!(json.contains("\"event_type\":\"CommandExecuted\""));
        assert!(json.contains("\"exit_code\":0"));

        let deserialized: RuntimeEvent = serde_json::from_str(&json).expect("deserialize event");
        assert_eq!(event, deserialized);
        assert_eq!(deserialized.event_type_name(), "CommandExecuted");
        assert_eq!(deserialized.run_id(), Some("run-123"));
        assert_eq!(deserialized.task_id(), Some("task-456"));
    }

    #[test]
    fn test_token_usage_event_roundtrip() {
        let event = RuntimeEvent::TokenUsageUpdated {
            run_id: "run-xyz".to_string(),
            task_id: Some("t-1".to_string()),
            agent_name: "claude".to_string(),
            input_tokens: 1500,
            output_tokens: 450,
            cached_tokens: 200,
            reasoning_tokens: 0,
            cost_usd: 0.0125,
        };

        let json = serde_json::to_string(&event).expect("serialize");
        let deserialized: RuntimeEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, deserialized);
        assert_eq!(deserialized.event_type_name(), "TokenUsageUpdated");
    }

    #[test]
    fn test_event_envelope() {
        let event = RuntimeEvent::RunInitiated {
            run_id: "run-001".to_string(),
            prompt: "Fix SQLite WAL leak".to_string(),
            team_name: Some("forester".to_string()),
        };
        let envelope = EventEnvelope::new("run-001", event.clone());
        assert_eq!(envelope.run_id, "run-001");
        assert_eq!(envelope.event, event);

        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, envelope.id);
        assert_eq!(decoded.event, envelope.event);
    }
}
