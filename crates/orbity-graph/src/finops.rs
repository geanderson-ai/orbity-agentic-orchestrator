//! FinOps budget tracker, limits and YAML-based governance policy evaluation per node and edge.

use crate::types::NodeId;
use orbity_core::contracts::{ApprovalDecision, ApprovalPolicy};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tracks budget and token expenditure dynamically during graph execution.
#[derive(Debug, Clone, Default)]
pub struct GraphFinOpsTracker {
    state: Arc<RwLock<FinOpsState>>,
}

#[derive(Debug, Clone, Default)]
struct FinOpsState {
    global_budget_usd: Option<f64>,
    total_cost_usd: f64,
    node_costs: HashMap<String, f64>,
    total_tokens: u32,
    approval_policy: Option<ApprovalPolicy>,
}

impl GraphFinOpsTracker {
    pub fn new(global_budget_usd: Option<f64>, approval_policy: Option<ApprovalPolicy>) -> Self {
        Self {
            state: Arc::new(RwLock::new(FinOpsState {
                global_budget_usd,
                total_cost_usd: 0.0,
                node_costs: HashMap::new(),
                total_tokens: 0,
                approval_policy,
            })),
        }
    }

    /// Checks if a proposed node execution exceeds the global budget or node budget.
    pub async fn check_budget(&self, node_id: &NodeId, node_limit_usd: Option<f64>) -> Result<(), String> {
        let state = self.state.read().await;

        if let Some(global_limit) = state.global_budget_usd {
            if state.total_cost_usd >= global_limit {
                return Err(format!(
                    "Global budget tripwire exceeded: spent ${:.4} >= limit ${:.4}",
                    state.total_cost_usd, global_limit
                ));
            }
        }

        if let Some(limit) = node_limit_usd {
            let spent = state.node_costs.get(&node_id.0).copied().unwrap_or(0.0);
            if spent >= limit {
                return Err(format!(
                    "Node '{}' budget exceeded: spent ${:.4} >= limit ${:.4}",
                    node_id.0, spent, limit
                ));
            }
        }

        Ok(())
    }

    /// Records cost and token expenditure incurred by a node.
    pub async fn record_spend(&self, node_id: &NodeId, cost_usd: f64, tokens: u32) {
        let mut state = self.state.write().await;
        state.total_cost_usd += cost_usd;
        state.total_tokens += tokens;
        *state.node_costs.entry(node_id.0.clone()).or_insert(0.0) += cost_usd;
    }

    /// Evaluates declarative YAML approval policy for a sensitive action or command without human pause.
    pub async fn evaluate_approval(&self, command: &str) -> ApprovalDecision {
        let state = self.state.read().await;
        if let Some(policy) = &state.approval_policy {
            policy.evaluate_command(command)
        } else {
            // Default to approved if no strict policy configured
            ApprovalDecision::Approved
        }
    }

    pub async fn total_cost(&self) -> f64 {
        self.state.read().await.total_cost_usd
    }

    pub async fn total_tokens(&self) -> u32 {
        self.state.read().await.total_tokens
    }
}
