//! Layer 4: Telemetry Sink - OpenTelemetry spans and metrics collector.

use std::sync::Arc;
use async_trait::async_trait;
use orbity_core::bus::{EventSink, EventSinkError};
use orbity_core::events::{EventEnvelope, RuntimeEvent};
use tokio::sync::Mutex;

use crate::metrics::TelemetryMetrics;
use crate::spans::{SpanStatus, SpanTree};

/// Telemetry Sink implementing OpenTelemetry-compatible span tracking and FinOps metrics.
#[derive(Debug, Clone, Default)]
pub struct TelemetrySink {
    name: String,
    metrics: Arc<Mutex<TelemetryMetrics>>,
    span_tree: Arc<Mutex<SpanTree>>,
}

impl TelemetrySink {
    pub fn new() -> Self {
        Self {
            name: "layer4-telemetry-otel".to_string(),
            metrics: Arc::new(Mutex::new(TelemetryMetrics::new())),
            span_tree: Arc::new(Mutex::new(SpanTree::new())),
        }
    }

    /// Returns a snapshot of the accumulated telemetry and FinOps metrics.
    pub async fn metrics(&self) -> TelemetryMetrics {
        self.metrics.lock().await.clone()
    }

    /// Returns a copy of the active span tree.
    pub async fn span_tree(&self) -> SpanTree {
        self.span_tree.lock().await.clone()
    }

    /// Exports all spans and metrics as standard OpenTelemetry (OTLP) JSON.
    pub async fn export_otlp_json(&self, service_name: &str) -> serde_json::Value {
        let tree = self.span_tree.lock().await;
        let metrics = self.metrics.lock().await;
        let spans = tree.all_spans();

        let otlp_spans: Vec<serde_json::Value> = spans
            .iter()
            .map(|s| {
                serde_json::json!({
                    "spanId": s.span_id,
                    "parentSpanId": s.parent_span_id,
                    "name": s.name,
                    "status": {
                        "code": match s.status {
                            SpanStatus::Ok => 1,
                            SpanStatus::Error => 2,
                            SpanStatus::Running => 0,
                        },
                        "message": s.status.to_string()
                    },
                    "startTimeUnixNano": s.started_at.timestamp_nanos_opt().unwrap_or(0),
                    "endTimeUnixNano": s.ended_at.and_then(|t| t.timestamp_nanos_opt()).unwrap_or(0),
                    "durationMs": s.duration_ms,
                    "attributes": s.attributes
                })
            })
            .collect();

        serde_json::json!({
            "resourceSpans": [{
                "resource": {
                    "attributes": [
                        { "key": "service.name", "value": { "stringValue": service_name } },
                        { "key": "telemetry.sdk.name", "value": { "stringValue": "orbity-telemetry" } }
                    ]
                },
                "scopeSpans": [{
                    "scope": { "name": "orbity.agent.orchestrator", "version": "0.1.0" },
                    "spans": otlp_spans
                }]
            }],
            "metricsSummary": metrics.summary()
        })
    }
}

#[async_trait]
impl EventSink for TelemetrySink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError> {
        // 1. Update Metrics
        {
            let mut m = self.metrics.lock().await;
            m.record_event(&event.event);
        }

        // 2. Update Hierarchical Spans
        {
            let mut tree = self.span_tree.lock().await;
            match &event.event {
                RuntimeEvent::RunInitiated {
                    run_id, team_name, ..
                } => {
                    let root_id = tree.start_run_span(run_id);
                    if let Some(team) = team_name {
                        // Start supervisor span under root
                        let sup_id = tree.start_supervisor_span(run_id, "astra");
                        tracing::info!(
                            target: "orbity::telemetry",
                            run_id = %run_id,
                            team = %team,
                            root_span = %root_id,
                            supervisor_span = %sup_id,
                            "Initialized execution tree for run"
                        );
                    }
                }
                RuntimeEvent::AgentStarted {
                    run_id,
                    agent_name,
                    task_id,
                    ..
                } => {
                    let span_id = tree.start_agent_span(run_id, agent_name, task_id.clone(), None);
                    tracing::info!(
                        target: "orbity::telemetry",
                        run_id = %run_id,
                        agent = %agent_name,
                        span_id = %span_id,
                        task_id = ?task_id,
                        "Worker agent span started"
                    );
                }
                RuntimeEvent::AgentFinished {
                    run_id, agent_name, ..
                } => {
                    tree.close_agent_span(run_id, agent_name, SpanStatus::Ok);
                    tracing::info!(
                        target: "orbity::telemetry",
                        run_id = %run_id,
                        agent = %agent_name,
                        "Worker agent span finished OK"
                    );
                }
                RuntimeEvent::AgentFailed {
                    run_id,
                    agent_name,
                    error,
                    ..
                } => {
                    tree.close_agent_span(run_id, agent_name, SpanStatus::Error);
                    tracing::warn!(
                        target: "orbity::telemetry",
                        run_id = %run_id,
                        agent = %agent_name,
                        error = %error,
                        "Worker agent span finished with ERROR"
                    );
                }
                RuntimeEvent::RunCompleted {
                    run_id,
                    total_tokens,
                    total_cost_usd,
                    duration_ms,
                } => {
                    tree.close_run_span(run_id, SpanStatus::Ok);
                    tracing::info!(
                        target: "orbity::telemetry",
                        run_id = %run_id,
                        tokens = %total_tokens,
                        cost = %total_cost_usd,
                        duration_ms = %duration_ms,
                        "Run span closed successfully"
                    );
                }
                RuntimeEvent::RunFailed { run_id, error } => {
                    tree.close_run_span(run_id, SpanStatus::Error);
                    tracing::error!(
                        target: "orbity::telemetry",
                        run_id = %run_id,
                        error = %error,
                        "Run span closed with FAILURE"
                    );
                }
                RuntimeEvent::CommandExecuted {
                    agent_name,
                    command,
                    exit_code,
                    duration_ms,
                    ..
                } => {
                    tracing::debug!(
                        target: "orbity::telemetry",
                        agent = %agent_name,
                        command = %command,
                        exit_code = %exit_code,
                        duration_ms = %duration_ms,
                        "Command execution telemetry captured"
                    );
                }
                RuntimeEvent::TokenUsageUpdated {
                    agent_name,
                    input_tokens,
                    output_tokens,
                    cost_usd,
                    ..
                } => {
                    tracing::debug!(
                        target: "orbity::telemetry",
                        agent = %agent_name,
                        input = %input_tokens,
                        output = %output_tokens,
                        cost = %cost_usd,
                        "FinOps token telemetry updated"
                    );
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbity_core::bus::{EventBus, EventBusConfig};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_telemetry_sink_hierarchical_spans_and_metrics() {
        let bus = EventBus::new(EventBusConfig::default());
        let telemetry_sink = Arc::new(TelemetrySink::new());
        bus.register_sink(telemetry_sink.clone()).await;

        let run_id = "run-otel-01";

        // 1. Run Initiated with team forester (creates root & supervisor spans)
        bus.emit(
            run_id,
            RuntimeEvent::RunInitiated {
                run_id: run_id.to_string(),
                prompt: "Build and test telemetry".to_string(),
                team_name: Some("forester".to_string()),
            },
        )
        .await
        .unwrap();

        // 2. Start worker agent (codex)
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

        // 3. Command Executed
        bus.emit(
            run_id,
            RuntimeEvent::CommandExecuted {
                run_id: run_id.to_string(),
                task_id: Some("task-01".to_string()),
                agent_name: "codex".to_string(),
                command: "cargo".to_string(),
                args: vec!["check".to_string()],
                exit_code: 0,
                duration_ms: 320,
                stdout_preview: Some("ok".to_string()),
                stderr_preview: None,
                sandbox_id: Some("sbx-1".to_string()),
            },
        )
        .await
        .unwrap();

        // 4. Token usage updated
        bus.emit(
            run_id,
            RuntimeEvent::TokenUsageUpdated {
                run_id: run_id.to_string(),
                task_id: Some("task-01".to_string()),
                agent_name: "codex".to_string(),
                input_tokens: 1500,
                output_tokens: 450,
                cached_tokens: 200,
                reasoning_tokens: 0,
                cost_usd: 0.0085,
            },
        )
        .await
        .unwrap();

        // 5. Worker agent finished
        bus.emit(
            run_id,
            RuntimeEvent::AgentFinished {
                run_id: run_id.to_string(),
                task_id: Some("task-01".to_string()),
                agent_id: "agt_codex".to_string(),
                agent_name: "codex".to_string(),
                summary: Some("Check passed".to_string()),
            },
        )
        .await
        .unwrap();

        // 6. Run completed
        bus.emit(
            run_id,
            RuntimeEvent::RunCompleted {
                run_id: run_id.to_string(),
                total_tokens: 1950,
                total_cost_usd: 0.0085,
                duration_ms: 1500,
            },
        )
        .await
        .unwrap();

        sleep(Duration::from_millis(80)).await;

        let metrics = telemetry_sink.metrics().await;
        assert_eq!(metrics.total_events, 6);
        assert_eq!(metrics.total_input_tokens, 1500);
        assert_eq!(metrics.total_output_tokens, 450);
        assert_eq!(metrics.total_cost_usd, 0.0085);
        assert_eq!(metrics.commands_count, 1);
        assert_eq!(metrics.avg_command_duration_ms(), 320.0);

        // Export OTLP JSON
        let otlp = telemetry_sink.export_otlp_json("orbity-test").await;
        assert!(otlp["resourceSpans"].is_array());
        assert_eq!(otlp["metricsSummary"]["tokens"]["input"], 1500);
    }
}
