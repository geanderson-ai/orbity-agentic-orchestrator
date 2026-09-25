//! Dynamic conditional edge predicates and evaluation against Blackboard.

use crate::blackboard::Blackboard;
use crate::types::NodeOutput;

/// Evaluates conditional expressions for edge transitions.
pub struct ConditionalEvaluator;

impl ConditionalEvaluator {
    /// Evaluates a simple predicate string against the node's output and the blackboard.
    ///
    /// Supported expressions:
    /// - `"outcome == 'success'"` or `"outcome == 'failed'"`
    /// - `"exit_code == 0"` or `"exit_code != 0"`
    /// - `"stdout.contains('...')"`
    /// - `"loop_count < N"`
    pub async fn evaluate(
        predicate: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
        loop_key: Option<&str>,
    ) -> bool {
        let p = predicate.trim();

        if p.is_empty() || p == "true" || p == "always" {
            return true;
        }

        if p == "false" || p == "never" {
            return false;
        }

        if p == "outcome == 'success'" || p == "success" {
            return node_output.success;
        }

        if p == "outcome == 'failed'" || p == "failed" {
            return !node_output.success;
        }

        if p == "exit_code == 0" {
            return node_output.exit_code == Some(0);
        }

        if p == "exit_code != 0" {
            return node_output.exit_code != Some(0);
        }

        if let Some(rest) = p.strip_prefix("stdout.contains(") {
            if let Some(target) = rest.strip_suffix(')') {
                let cleaned = target.trim_matches(|c| c == '\'' || c == '"');
                return node_output.stdout.contains(cleaned);
            }
        }

        if let Some(rest) = p.strip_prefix("stderr.contains(") {
            if let Some(target) = rest.strip_suffix(')') {
                let cleaned = target.trim_matches(|c| c == '\'' || c == '"');
                return node_output.stderr.contains(cleaned);
            }
        }

        if let Some(rest) = p.strip_prefix("loop_count < ") {
            if let Ok(limit) = rest.trim().parse::<u32>() {
                if let Some(key) = loop_key {
                    let current = blackboard.get_loop_count(key).await;
                    return current < limit;
                }
            }
        }

        // Context variable check: "context.key == 'val'"
        if let Some(rest) = p.strip_prefix("context.") {
            let parts: Vec<&str> = rest.split("==").collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let expected_val = parts[1].trim().trim_matches(|c| c == '\'' || c == '"');
                if let Some(val) = blackboard.get_context(key).await {
                    if let Some(s) = val.as_str() {
                        return s == expected_val;
                    }
                }
            }
        }

        false
    }
}
