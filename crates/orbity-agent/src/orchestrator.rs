//! High-level deterministic multi-agent graph orchestrator coordinating tasks across
//! the 5 CLIs (Codex, Claude, Agy, Hermes, Pi) without requiring Astra.

use crate::runners::SandboxCliNodeRunner;
use orbity_core::bus::EventBus;
use orbity_core::contracts::ApprovalPolicy;
use orbity_graph::{Blackboard, GraphCheckpointStore, GraphDefinition, GraphExecutor, GraphFinOpsTracker};
use orbity_sandbox::traits::Sandbox;
use sqlx::SqlitePool;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Graph execution failed: {0}")]
    Execution(#[from] orbity_graph::ExecutionError),
    #[error("Checkpoint setup failed: {0}")]
    Checkpoint(#[from] orbity_graph::CheckpointError),
}

/// Orchestrator for deterministic multi-agent workflows running on Tokio.
pub struct MultiAgentOrchestrator {
    sandbox: Arc<Mutex<dyn Sandbox>>,
    pool: Option<SqlitePool>,
    event_bus: Option<EventBus>,
    approval_policy: Option<ApprovalPolicy>,
    global_budget_usd: Option<f64>,
}

impl MultiAgentOrchestrator {
    pub fn new(sandbox: Arc<Mutex<dyn Sandbox>>) -> Self {
        Self {
            sandbox,
            pool: None,
            event_bus: None,
            approval_policy: None,
            global_budget_usd: None,
        }
    }

    pub fn with_pool(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub fn with_approval_policy(mut self, policy: ApprovalPolicy) -> Self {
        self.approval_policy = Some(policy);
        self
    }

    pub fn with_budget(mut self, budget_usd: f64) -> Self {
        self.global_budget_usd = Some(budget_usd);
        self
    }

    /// Executes the multi-agent graph with automatic checkpoints and FinOps monitoring.
    pub async fn run_graph(&self, graph: GraphDefinition, execution_id: Option<Uuid>) -> Result<Blackboard, OrchestratorError> {
        let blackboard = Blackboard::new();
        let finops = GraphFinOpsTracker::new(self.global_budget_usd, self.approval_policy.clone());
        let runner = Arc::new(SandboxCliNodeRunner::new(self.sandbox.clone()));

        let mut executor = GraphExecutor::new(graph, blackboard.clone(), finops, runner);

        if let Some(id) = execution_id {
            executor = executor.with_execution_id(id);
        }

        if let Some(bus) = &self.event_bus {
            executor = executor.with_event_bus(bus.clone());
        }

        if let Some(pool) = &self.pool {
            let store = GraphCheckpointStore::new(pool.clone());
            store.init_schema().await?;
            executor = executor.with_checkpoints(store);
        }

        executor.execute().await?;

        Ok(blackboard)
    }
}
