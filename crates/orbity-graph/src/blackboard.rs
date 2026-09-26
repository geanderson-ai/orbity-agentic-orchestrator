//! Shared Blackboard memory pattern for asynchronous graph execution and inter-node context passing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// An artifact produced by an agent or tool during graph execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskArtifact {
    pub name: String,
    pub content: String,
    pub mime_type: String,
    pub producer_node: String,
}

/// A structured handoff envelope passed between nodes during pipeline transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffEnvelope {
    pub from_node: String,
    pub summary: String,
    pub artifacts: Vec<String>,
    pub exit_code: i32,
    pub timestamp_ms: u64,
}

/// Shared in-memory Blackboard for inter-node state passing, results aggregation and context injection.
#[derive(Debug, Clone, Default)]
pub struct Blackboard {
    data: Arc<RwLock<BlackboardState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlackboardState {
    pub global_context: HashMap<String, serde_json::Value>,
    pub node_outputs: HashMap<String, String>,
    pub node_data: HashMap<String, serde_json::Value>,
    pub artifacts: HashMap<String, TaskArtifact>,
    pub handoffs: HashMap<String, HandoffEnvelope>,
    pub loop_iteration_counts: HashMap<String, u32>,
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(BlackboardState::default())),
        }
    }

    /// Sets a global context value.
    pub async fn set_context(&self, key: impl Into<String>, value: serde_json::Value) {
        let mut state = self.data.write().await;
        state.global_context.insert(key.into(), value);
    }

    /// Gets a global context value.
    pub async fn get_context(&self, key: &str) -> Option<serde_json::Value> {
        let state = self.data.read().await;
        state.global_context.get(key).cloned()
    }

    /// Records text output from a finished node.
    pub async fn set_node_output(&self, node_id: impl Into<String>, output: impl Into<String>) {
        let mut state = self.data.write().await;
        state.node_outputs.insert(node_id.into(), output.into());
    }

    /// Retrieves text output of a node.
    pub async fn get_node_output(&self, node_id: &str) -> Option<String> {
        let state = self.data.read().await;
        state.node_outputs.get(node_id).cloned()
    }

    /// Records structured json data produced by a node.
    pub async fn set_node_data(&self, node_id: impl Into<String>, data: serde_json::Value) {
        let mut state = self.data.write().await;
        state.node_data.insert(node_id.into(), data);
    }

    /// Retrieves structured json data of a node.
    pub async fn get_node_data(&self, node_id: &str) -> Option<serde_json::Value> {
        let state = self.data.read().await;
        state.node_data.get(node_id).cloned()
    }

    /// Records a structured handoff envelope from a node.
    pub async fn record_handoff(&self, envelope: HandoffEnvelope) {
        let mut state = self.data.write().await;
        state.handoffs.insert(envelope.from_node.clone(), envelope);
    }

    /// Retrieves a structured handoff envelope for a node.
    pub async fn get_handoff(&self, node_id: &str) -> Option<HandoffEnvelope> {
        let state = self.data.read().await;
        state.handoffs.get(node_id).cloned()
    }

    /// Adds an artifact produced during execution.
    pub async fn add_artifact(&self, artifact: TaskArtifact) {
        let mut state = self.data.write().await;
        state.artifacts.insert(artifact.name.clone(), artifact);
    }

    /// Gets an artifact by name.
    pub async fn get_artifact(&self, name: &str) -> Option<TaskArtifact> {
        let state = self.data.read().await;
        state.artifacts.get(name).cloned()
    }

    /// Increments loop count for an edge/node and returns current count.
    pub async fn increment_loop_count(&self, loop_key: &str) -> u32 {
        let mut state = self.data.write().await;
        let count = state
            .loop_iteration_counts
            .entry(loop_key.to_string())
            .or_insert(0);
        *count += 1;
        *count
    }

    /// Gets the current iteration count for a loop key.
    pub async fn get_loop_count(&self, loop_key: &str) -> u32 {
        let state = self.data.read().await;
        state
            .loop_iteration_counts
            .get(loop_key)
            .copied()
            .unwrap_or(0)
    }

    /// Captures a complete serializable snapshot of the blackboard state for SQLite checkpointing.
    pub async fn snapshot(&self) -> serde_json::Value {
        let state = self.data.read().await;
        serde_json::to_value(&*state).unwrap_or(serde_json::Value::Null)
    }

    /// Restores the blackboard state from a serialized snapshot.
    pub async fn restore_snapshot(
        &self,
        snapshot: serde_json::Value,
    ) -> Result<(), serde_json::Error> {
        let restored_state: BlackboardState = serde_json::from_value(snapshot)?;
        let mut state = self.data.write().await;
        *state = restored_state;
        Ok(())
    }

    /// Formats the accumulated context from upstream predecessors and global user prompt,
    /// enforcing strict token and character budget caps to prevent context window blowout.
    pub async fn inject_context(&self, predecessor_ids: &[crate::types::NodeId]) -> String {
        const MAX_NODE_OUTPUT_CHARS: usize = 24_000;
        const MAX_TOTAL_CONTEXT_CHARS: usize = 64_000;

        let state = self.data.read().await;
        let mut context_builder = String::new();

        if let Some(val) = state.global_context.get("user_prompt") {
            if let Some(user_prompt) = val.as_str() {
                if !user_prompt.trim().is_empty() {
                    let truncated_prompt = truncate_middle(user_prompt.trim(), MAX_NODE_OUTPUT_CHARS);
                    context_builder.push_str(&format!(
                        "--- User Task / Request ---\n{}\n\n",
                        truncated_prompt
                    ));
                }
            }
        }

        for pred in predecessor_ids {
            // Check if structured handoff exists
            if let Some(handoff) = state.handoffs.get(&pred.0) {
                context_builder.push_str(&format!(
                    "--- Handoff from [{}] (Exit Code: {}) ---\nSummary: {}\nArtifacts: {:?}\n\n",
                    handoff.from_node, handoff.exit_code, handoff.summary, handoff.artifacts
                ));
            }

            if let Some(out) = state.node_outputs.get(&pred.0) {
                let trimmed = out.trim();
                if !trimmed.is_empty() {
                    let truncated = truncate_middle(trimmed, MAX_NODE_OUTPUT_CHARS);
                    context_builder.push_str(&format!(
                        "--- Output from Node [{}] ---\n{}\n\n",
                        pred.0, truncated
                    ));
                }
            }
        }

        if context_builder.len() > MAX_TOTAL_CONTEXT_CHARS {
            truncate_middle(&context_builder, MAX_TOTAL_CONTEXT_CHARS)
        } else {
            context_builder
        }
    }
}

/// Truncates strings in the middle preserving essential beginning and end context.
fn truncate_middle(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let head_count = max_chars / 2;
    let tail_count = max_chars - head_count;

    let chars: Vec<char> = input.chars().collect();
    let total = chars.len();
    let omitted = total.saturating_sub(head_count + tail_count);

    let head: String = chars[..head_count].iter().collect();
    let tail: String = chars[total - tail_count..].iter().collect();

    format!(
        "{}\n\n[... Truncated {} characters for LLM context protection ...]\n\n{}",
        head, omitted, tail
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blackboard_handoff_and_truncation() {
        let bb = Blackboard::new();
        bb.record_handoff(HandoffEnvelope {
            from_node: "planner".to_string(),
            summary: "Planned 3 steps".to_string(),
            artifacts: vec!["plan.md".to_string()],
            exit_code: 0,
            timestamp_ms: 1000,
        }).await;

        let huge_output = "a".repeat(50_000);
        bb.set_node_output("planner", huge_output).await;

        let ctx = bb.inject_context(&[crate::types::NodeId("planner".to_string())]).await;
        assert!(ctx.contains("--- Handoff from [planner]"));
        assert!(ctx.contains("Truncated"));
        assert!(ctx.len() < 40_000);
    }
}
