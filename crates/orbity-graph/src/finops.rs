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
    reserved_cost_usd: f64,
    node_costs: HashMap<String, f64>,
    node_reserved: HashMap<String, f64>,
    total_tokens: u32,
    cached_tokens: u32,
    reasoning_tokens: u32,
    approval_policy: Option<ApprovalPolicy>,
}

impl GraphFinOpsTracker {
    pub fn new(global_budget_usd: Option<f64>, approval_policy: Option<ApprovalPolicy>) -> Self {
        Self {
            state: Arc::new(RwLock::new(FinOpsState {
                global_budget_usd,
                total_cost_usd: 0.0,
                reserved_cost_usd: 0.0,
                node_costs: HashMap::new(),
                node_reserved: HashMap::new(),
                total_tokens: 0,
                cached_tokens: 0,
                reasoning_tokens: 0,
                approval_policy,
            })),
        }
    }

    /// Atomically checks and reserves budget for a node before execution begins,
    /// preventing budget overshoot races during concurrent fan-out tasks.
    pub async fn reserve_budget(
        &self,
        node_id: &NodeId,
        estimated_cost: f64,
        node_limit_usd: Option<f64>,
    ) -> Result<(), String> {
        let mut state = self.state.write().await;

        if let Some(global_limit) = state.global_budget_usd {
            let projected_total = state.total_cost_usd + state.reserved_cost_usd + estimated_cost;
            if projected_total > global_limit {
                return Err(format!(
                    "Global budget tripwire exceeded: projected ${:.4} (${:.4} spent + ${:.4} reserved) > limit ${:.4}",
                    projected_total, state.total_cost_usd, state.reserved_cost_usd, global_limit
                ));
            }
        }

        if let Some(limit) = node_limit_usd {
            let spent = state.node_costs.get(&node_id.0).copied().unwrap_or(0.0);
            let reserved = state.node_reserved.get(&node_id.0).copied().unwrap_or(0.0);
            let projected_node = spent + reserved + estimated_cost;
            if projected_node > limit {
                return Err(format!(
                    "Node '{}' budget exceeded: projected ${:.4} > limit ${:.4}",
                    node_id.0, projected_node, limit
                ));
            }
        }

        state.reserved_cost_usd += estimated_cost;
        *state.node_reserved.entry(node_id.0.clone()).or_insert(0.0) += estimated_cost;

        Ok(())
    }

    /// Commits the actual spend incurred by a node and releases the reservation.
    pub async fn commit_spend(
        &self,
        node_id: &NodeId,
        actual_cost: f64,
        reserved_cost: f64,
        tokens: u32,
        cached_tokens: u32,
        reasoning_tokens: u32,
    ) {
        let mut state = self.state.write().await;

        state.reserved_cost_usd = (state.reserved_cost_usd - reserved_cost).max(0.0);
        if let Some(res) = state.node_reserved.get_mut(&node_id.0) {
            *res = (*res - reserved_cost).max(0.0);
        }

        state.total_cost_usd += actual_cost;
        state.total_tokens += tokens;
        state.cached_tokens += cached_tokens;
        state.reasoning_tokens += reasoning_tokens;
        *state.node_costs.entry(node_id.0.clone()).or_insert(0.0) += actual_cost;
    }

    /// Checks if a proposed node execution exceeds the global budget or node budget.
    pub async fn check_budget(
        &self,
        node_id: &NodeId,
        node_limit_usd: Option<f64>,
    ) -> Result<(), String> {
        let state = self.state.read().await;

        if let Some(global_limit) = state.global_budget_usd {
            if (state.total_cost_usd + state.reserved_cost_usd) >= global_limit {
                return Err(format!(
                    "Global budget tripwire exceeded: spent ${:.4} + reserved ${:.4} >= limit ${:.4}",
                    state.total_cost_usd, state.reserved_cost_usd, global_limit
                ));
            }
        }

        if let Some(limit) = node_limit_usd {
            let spent = state.node_costs.get(&node_id.0).copied().unwrap_or(0.0);
            let reserved = state.node_reserved.get(&node_id.0).copied().unwrap_or(0.0);
            if (spent + reserved) >= limit {
                return Err(format!(
                    "Node '{}' budget exceeded: spent ${:.4} + reserved ${:.4} >= limit ${:.4}",
                    node_id.0, spent, reserved, limit
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

    pub async fn cached_tokens(&self) -> u32 {
        self.state.read().await.cached_tokens
    }

    pub async fn reasoning_tokens(&self) -> u32 {
        self.state.read().await.reasoning_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_atomic_budget_reservation_and_commit() {
        let tracker = GraphFinOpsTracker::new(Some(1.0), None);
        let node_id = NodeId("agent_1".to_string());

        // Reserve $0.40
        assert!(tracker.reserve_budget(&node_id, 0.40, Some(0.80)).await.is_ok());

        // Reserve another $0.50 -> total $0.90 <= $1.0
        assert!(tracker.reserve_budget(&node_id, 0.50, Some(0.80)).await.is_err()); // Node limit exceeded ($0.40 + $0.50 > $0.80)

        // Commit first reservation with actual cost $0.35
        tracker.commit_spend(&node_id, 0.35, 0.40, 1500, 200, 100).await;

        assert!((tracker.total_cost().await - 0.35).abs() < 1e-5);
        assert_eq!(tracker.total_tokens().await, 1500);
        assert_eq!(tracker.cached_tokens().await, 200);
        assert_eq!(tracker.reasoning_tokens().await, 100);
    }
}
