//! Hierarchical tracing spans for multi-agent execution trees.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Execution status of a tracing span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    Running,
    Ok,
    Error,
}

impl std::fmt::Display for SpanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "RUNNING"),
            Self::Ok => write!(f, "OK"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A structured record representing an execution span in the agentic hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanRecord {
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub run_id: String,
    pub name: String,
    pub agent_name: Option<String>,
    pub task_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, serde_json::Value>,
}

impl SpanRecord {
    pub fn new(
        name: impl Into<String>,
        run_id: impl Into<String>,
        parent_span_id: Option<String>,
        agent_name: Option<String>,
        task_id: Option<String>,
    ) -> Self {
        Self {
            span_id: format!("span-{}", Uuid::new_v4().simple()),
            parent_span_id,
            run_id: run_id.into(),
            name: name.into(),
            agent_name,
            task_id,
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: None,
            status: SpanStatus::Running,
            attributes: HashMap::new(),
        }
    }

    /// Closes the span with given status and computes duration in ms.
    pub fn close(&mut self, status: SpanStatus) {
        let now = Utc::now();
        self.ended_at = Some(now);
        self.status = status;
        let duration = (now - self.started_at).num_milliseconds();
        self.duration_ms = Some(if duration >= 0 { duration as u64 } else { 0 });
    }

    /// Inserts an attribute into the span.
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.attributes.insert(key.into(), value.into());
    }
}

/// In-memory tree manager for hierarchical agent spans.
#[derive(Debug, Clone, Default)]
pub struct SpanTree {
    spans: HashMap<String, SpanRecord>,
    // Mapping from run_id -> root span_id
    run_root_spans: HashMap<String, String>,
    // Mapping from (run_id, agent_name) -> active worker span_id
    active_agent_spans: HashMap<(String, String), String>,
}

impl SpanTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a root span for a run (e.g. `run:run-123`).
    pub fn start_run_span(&mut self, run_id: &str) -> String {
        let span = SpanRecord::new(format!("run:{}", run_id), run_id, None, None, None);
        let span_id = span.span_id.clone();
        self.run_root_spans
            .insert(run_id.to_string(), span_id.clone());
        self.spans.insert(span_id.clone(), span);
        span_id
    }

    /// Starts a supervisor/orchestrator span under the run span (e.g. `supervisor:topcoat`).
    pub fn start_supervisor_span(&mut self, run_id: &str, supervisor_name: &str) -> String {
        let parent_id = self.run_root_spans.get(run_id).cloned();
        let mut span = SpanRecord::new(
            format!("supervisor:{}", supervisor_name),
            run_id,
            parent_id,
            Some(supervisor_name.to_string()),
            None,
        );
        span.set_attribute("role", "supervisor");
        let span_id = span.span_id.clone();
        self.spans.insert(span_id.clone(), span);
        span_id
    }

    /// Starts a worker agent span under either the supervisor span or the run span.
    pub fn start_agent_span(
        &mut self,
        run_id: &str,
        agent_name: &str,
        task_id: Option<String>,
        parent_span_id: Option<String>,
    ) -> String {
        let parent = parent_span_id.or_else(|| self.run_root_spans.get(run_id).cloned());

        let mut span = SpanRecord::new(
            format!("worker:{}", agent_name),
            run_id,
            parent,
            Some(agent_name.to_string()),
            task_id,
        );
        span.set_attribute("role", "worker");
        let span_id = span.span_id.clone();

        self.active_agent_spans.insert(
            (run_id.to_string(), agent_name.to_string()),
            span_id.clone(),
        );
        self.spans.insert(span_id.clone(), span);
        span_id
    }

    /// Closes an active agent span.
    pub fn close_agent_span(&mut self, run_id: &str, agent_name: &str, status: SpanStatus) {
        if let Some(span_id) = self
            .active_agent_spans
            .remove(&(run_id.to_string(), agent_name.to_string()))
        {
            if let Some(span) = self.spans.get_mut(&span_id) {
                span.close(status);
            }
        }
    }

    /// Closes a run span and all remaining open children.
    pub fn close_run_span(&mut self, run_id: &str, status: SpanStatus) {
        if let Some(root_id) = self.run_root_spans.get(run_id) {
            if let Some(span) = self.spans.get_mut(root_id) {
                span.close(status);
            }
        }
    }

    /// Retrieves a span by ID.
    pub fn get_span(&self, span_id: &str) -> Option<&SpanRecord> {
        self.spans.get(span_id)
    }

    /// Retrieves all spans.
    pub fn all_spans(&self) -> Vec<SpanRecord> {
        self.spans.values().cloned().collect()
    }

    /// Builds a nested JSON representation of the span tree for a given run.
    pub fn to_nested_tree(&self, run_id: &str) -> Option<serde_json::Value> {
        let root_span_id = self.run_root_spans.get(run_id)?;
        let root_span = self.spans.get(root_span_id)?;

        fn build_node(span: &SpanRecord, all: &HashMap<String, SpanRecord>) -> serde_json::Value {
            let children: Vec<serde_json::Value> = all
                .values()
                .filter(|s| s.parent_span_id.as_deref() == Some(&span.span_id))
                .map(|child| build_node(child, all))
                .collect();

            serde_json::json!({
                "span_id": span.span_id,
                "name": span.name,
                "agent_name": span.agent_name,
                "task_id": span.task_id,
                "status": span.status.to_string(),
                "duration_ms": span.duration_ms,
                "attributes": span.attributes,
                "children": children
            })
        }

        Some(build_node(root_span, &self.spans))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_hierarchy_creation_and_closing() {
        let mut tree = SpanTree::new();
        let run_id = "run-span-tree-01";

        let root_id = tree.start_run_span(run_id);
        let supervisor_id = tree.start_supervisor_span(run_id, "topcoat");
        let worker_id = tree.start_agent_span(
            run_id,
            "codex",
            Some("task-01".to_string()),
            Some(supervisor_id.clone()),
        );

        let worker_span = tree.get_span(&worker_id).expect("worker span exists");
        assert_eq!(
            worker_span.parent_span_id.as_deref(),
            Some(supervisor_id.as_str())
        );
        assert_eq!(worker_span.agent_name.as_deref(), Some("codex"));
        assert_eq!(worker_span.status, SpanStatus::Running);

        tree.close_agent_span(run_id, "codex", SpanStatus::Ok);
        let closed_worker = tree.get_span(&worker_id).unwrap();
        assert_eq!(closed_worker.status, SpanStatus::Ok);
        assert!(closed_worker.duration_ms.is_some());

        tree.close_run_span(run_id, SpanStatus::Ok);
        let closed_root = tree.get_span(&root_id).unwrap();
        assert_eq!(closed_root.status, SpanStatus::Ok);

        let nested = tree.to_nested_tree(run_id).expect("nested tree exists");
        assert_eq!(nested["name"], format!("run:{}", run_id));
        assert!(nested["children"].is_array());
    }
}
