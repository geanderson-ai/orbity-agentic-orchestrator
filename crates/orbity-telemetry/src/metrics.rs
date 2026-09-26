//! Telemetry metrics and FinOps token aggregation.

use orbity_core::events::RuntimeEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregated metrics across runs and multi-agent sessions.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TelemetryMetrics {
    pub total_events: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_tokens: u64,
    pub total_cost_usd: f64,
    pub calls_by_agent: HashMap<String, u64>,
    pub commands_count: u64,
    pub tools_count: u64,
    pub files_written_count: u64,
    pub security_denials_count: u64,
    pub errors_count: u64,
    pub command_durations_ms: Vec<u64>,
}

impl TelemetryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records metrics from a structured `RuntimeEvent`.
    pub fn record_event(&mut self, event: &RuntimeEvent) {
        self.total_events += 1;

        if let Some(agent) = event.agent_name() {
            *self.calls_by_agent.entry(agent.to_string()).or_insert(0) += 1;
        }

        match event {
            RuntimeEvent::TokenUsageUpdated {
                input_tokens,
                output_tokens,
                cached_tokens,
                cost_usd,
                ..
            } => {
                self.total_input_tokens += *input_tokens;
                self.total_output_tokens += *output_tokens;
                self.total_cache_tokens += *cached_tokens;
                self.total_cost_usd += *cost_usd;
            }
            RuntimeEvent::CommandExecuted {
                duration_ms,
                exit_code,
                ..
            } => {
                self.commands_count += 1;
                self.command_durations_ms.push(*duration_ms);
                if *exit_code != 0 {
                    self.errors_count += 1;
                }
            }
            RuntimeEvent::ToolCalled { .. } => {
                self.tools_count += 1;
            }
            RuntimeEvent::FileWritten { .. } => {
                self.files_written_count += 1;
            }
            RuntimeEvent::PolicyDenied { .. } => {
                self.security_denials_count += 1;
            }
            RuntimeEvent::AgentFailed { .. } | RuntimeEvent::RunFailed { .. } => {
                self.errors_count += 1;
            }
            _ => {}
        }
    }

    /// Computes the average command duration in milliseconds.
    pub fn avg_command_duration_ms(&self) -> f64 {
        if self.command_durations_ms.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.command_durations_ms.iter().sum();
        sum as f64 / self.command_durations_ms.len() as f64
    }

    /// Computes the 95th percentile (P95) command duration in milliseconds.
    pub fn p95_command_duration_ms(&self) -> u64 {
        if self.command_durations_ms.is_empty() {
            return 0;
        }
        let mut sorted = self.command_durations_ms.clone();
        sorted.sort_unstable();
        let idx = ((sorted.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
        sorted[idx.min(sorted.len() - 1)]
    }

    /// Serializes consolidated telemetry summary into JSON for Loki/Grafana ingestion.
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({
            "total_events": self.total_events,
            "tokens": {
                "input": self.total_input_tokens,
                "output": self.total_output_tokens,
                "cache": self.total_cache_tokens,
                "total": self.total_input_tokens + self.total_output_tokens,
                "estimated_cost_usd": self.total_cost_usd
            },
            "operations": {
                "commands": self.commands_count,
                "tools": self.tools_count,
                "files_written": self.files_written_count,
                "avg_cmd_duration_ms": self.avg_command_duration_ms(),
                "p95_cmd_duration_ms": self.p95_command_duration_ms()
            },
            "security": {
                "policy_denials": self.security_denials_count,
                "errors": self.errors_count
            },
            "agents_activity": self.calls_by_agent
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_metrics_accumulation() {
        let mut metrics = TelemetryMetrics::new();

        metrics.record_event(&RuntimeEvent::TokenUsageUpdated {
            run_id: "run-m-1".to_string(),
            task_id: None,
            agent_name: "codex".to_string(),
            input_tokens: 1000,
            output_tokens: 200,
            cached_tokens: 100,
            reasoning_tokens: 0,
            cost_usd: 0.005,
        });

        metrics.record_event(&RuntimeEvent::CommandExecuted {
            run_id: "run-m-1".to_string(),
            task_id: None,
            agent_name: "codex".to_string(),
            command: "echo".to_string(),
            args: vec!["hi".to_string()],
            exit_code: 0,
            duration_ms: 100,
            stdout_preview: Some("hi".to_string()),
            stderr_preview: None,
            sandbox_id: None,
        });

        metrics.record_event(&RuntimeEvent::CommandExecuted {
            run_id: "run-m-1".to_string(),
            task_id: None,
            agent_name: "codex".to_string(),
            command: "false".to_string(),
            args: vec![],
            exit_code: 1,
            duration_ms: 200,
            stdout_preview: None,
            stderr_preview: None,
            sandbox_id: None,
        });

        assert_eq!(metrics.total_events, 3);
        assert_eq!(metrics.total_input_tokens, 1000);
        assert_eq!(metrics.total_output_tokens, 200);
        assert_eq!(metrics.total_cost_usd, 0.005);
        assert_eq!(metrics.commands_count, 2);
        assert_eq!(metrics.errors_count, 1);
        assert_eq!(metrics.avg_command_duration_ms(), 150.0);
        assert_eq!(metrics.p95_command_duration_ms(), 200);

        let summary = metrics.summary();
        assert_eq!(summary["operations"]["commands"], 2);
        assert_eq!(summary["security"]["errors"], 1);
    }
}
