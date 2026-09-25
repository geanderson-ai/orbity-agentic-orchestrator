//! Asynchronous Graph Execution Engine in Tokio with parallel fan-out, barrier fan-in,
//! feedback loops, and deterministic orchestration without Astra.

use crate::blackboard::Blackboard;
use crate::checkpoint::GraphCheckpointStore;
use crate::finops::GraphFinOpsTracker;
use crate::predicate::ConditionalEvaluator;
use crate::topology::{TopologyError, TopologyValidator};
use crate::types::{EdgeKind, GraphDefinition, NodeId, NodeKind, NodeOutput};
use async_trait::async_trait;
use orbity_core::bus::EventBus;
use orbity_core::contracts::ApprovalDecision;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;


#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Topology validation error: {0}")]
    Topology(#[from] TopologyError),
    #[error("Node execution failed: {node_id} - {message}")]
    NodeFailed { node_id: String, message: String },
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),
    #[error("Loop iterations exceeded limit of {limit} for loop '{loop_id}'")]
    LoopBudgetExceeded { loop_id: String, limit: u32 },
    #[error("Execution suspended waiting for human approval: {node_id}")]
    WaitingHumanApproval { node_id: String, prompt: String },
    #[error("Checkpoint error: {0}")]
    Checkpoint(#[from] crate::checkpoint::CheckpointError),
}

/// Handler trait for executing specific graph node implementations.
#[async_trait]
pub trait NodeRunner: Send + Sync {
    async fn run(
        &self,
        node: &crate::types::GraphNode,
        injected_context: &str,
        blackboard: &Blackboard,
    ) -> Result<NodeOutput, String>;
}

/// Default mock/in-memory runner for tests and simulation.
pub struct DefaultNodeRunner;

#[async_trait]
impl NodeRunner for DefaultNodeRunner {
    async fn run(
        &self,
        node: &crate::types::GraphNode,
        injected_context: &str,
        _blackboard: &Blackboard,
    ) -> Result<NodeOutput, String> {
        let stdout = match &node.kind {
            NodeKind::Orchestrator { engine } => {
                format!("Topcoat Orchestrator [{}] plan generated with context len {}", engine, injected_context.len())
            }
            NodeKind::Agent { cli, config } => {
                format!("Agent [{}] ({}) executed task. Context: {}", cli, config.name, injected_context.trim())
            }
            NodeKind::Tool { command, .. } => {
                format!("Tool command '{}' executed successfully.", command)
            }
            NodeKind::ConditionalRouter { predicate_expr } => {
                format!("Evaluated router: {}", predicate_expr)
            }
            NodeKind::HumanGate { prompt, .. } => {
                format!("Human Gate approved: {}", prompt)
            }
            NodeKind::JoinBarrier { .. } => {
                "Join barrier synchronization reached.".to_string()
            }
        };

        Ok(NodeOutput {
            success: true,
            exit_code: Some(0),
            stdout,
            stderr: String::new(),
            artifacts: HashMap::new(),
            tokens_input: 100,
            tokens_output: 50,
            cost_usd: 0.001,
            duration_ms: 10,
        })
    }
}

/// The Tokio-native Graph Execution Engine.
pub struct GraphExecutor {
    graph: GraphDefinition,
    blackboard: Blackboard,
    finops: GraphFinOpsTracker,
    checkpoint_store: Option<GraphCheckpointStore>,
    event_bus: Option<EventBus>,
    runner: Arc<dyn NodeRunner>,
    execution_id: Uuid,
    step_counter: Mutex<u32>,
    completed_nodes: Arc<RwLock<HashSet<NodeId>>>,
}

impl GraphExecutor {
    pub fn new(
        graph: GraphDefinition,
        blackboard: Blackboard,
        finops: GraphFinOpsTracker,
        runner: Arc<dyn NodeRunner>,
    ) -> Self {
        Self {
            graph,
            blackboard,
            finops,
            checkpoint_store: None,
            event_bus: None,
            runner,
            execution_id: Uuid::new_v4(),
            step_counter: Mutex::new(0),
            completed_nodes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn with_checkpoints(mut self, store: GraphCheckpointStore) -> Self {
        self.checkpoint_store = Some(store);
        self
    }

    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub fn with_execution_id(mut self, id: Uuid) -> Self {
        self.execution_id = id;
        self
    }

    pub fn execution_id(&self) -> Uuid {
        self.execution_id
    }

    pub fn blackboard(&self) -> &Blackboard {
        &self.blackboard
    }

    /// Validates topology and runs the graph to completion.
    pub async fn execute(&self) -> Result<(), ExecutionError> {
        // Validate graph topology (excluding permitted feedback loop edges)
        let _plan = TopologyValidator::validate_and_plan(&self.graph)?;

        let mut ready_nodes = vec![self.graph.start_node.clone()];

        while !ready_nodes.is_empty() {
            let current_batch = std::mem::take(&mut ready_nodes);

            // Execute batch in parallel (Fan-out)
            let mut join_set = tokio::task::JoinSet::new();

            for node_id in current_batch {
                let node = match self.graph.nodes.get(&node_id) {
                    Some(n) => n.clone(),
                    None => continue,
                };

                let blackboard = self.blackboard.clone();
                let finops = self.finops.clone();
                let runner = self.runner.clone();
                let preds = TopologyValidator::get_predecessors(&self.graph, &node_id);

                join_set.spawn(async move {
                    // Check FinOps
                    finops.check_budget(&node_id, node.budget_limit_usd).await
                        .map_err(ExecutionError::BudgetExceeded)?;


                    // Inject context from predecessors
                    let injected_context = blackboard.inject_context(&preds).await;

                    // Check declarative Human Gate / YAML policy
                    if let NodeKind::HumanGate { prompt, .. } = &node.kind {
                        let decision = finops.evaluate_approval(prompt).await;
                        match decision {
                            ApprovalDecision::Approved => {
                                // Auto approved via YAML policy
                            }
                            ApprovalDecision::Rejected => {
                                return Err(ExecutionError::NodeFailed {
                                    node_id: node_id.0,
                                    message: format!("Policy rejected human gate: {}", prompt),
                                });
                            }
                            ApprovalDecision::NeedsHuman => {
                                return Err(ExecutionError::WaitingHumanApproval {
                                    node_id: node_id.0,
                                    prompt: prompt.clone(),
                                });
                            }
                        }
                    }

                    // Run node with retries
                    let mut attempts = 0;
                    let max_attempts = node.retries_limit.max(1);
                    let mut last_err = String::new();
                    let mut output = NodeOutput::default();

                    while attempts < max_attempts {
                        attempts += 1;
                        match runner.run(&node, &injected_context, &blackboard).await {
                            Ok(out) => {
                                output = out;
                                break;
                            }
                            Err(e) => {
                                last_err = e;
                                tokio::time::sleep(tokio::time::Duration::from_millis(50 * (attempts as u64))).await;
                            }
                        }
                    }

                    if !last_err.is_empty() && !output.success {
                        return Err(ExecutionError::NodeFailed {
                            node_id: node_id.0,
                            message: last_err,
                        });
                    }

                    // Record FinOps spend
                    finops.record_spend(&node_id, output.cost_usd, output.tokens_input + output.tokens_output).await;

                    // Update blackboard
                    blackboard.set_node_output(node_id.0.clone(), output.stdout.clone()).await;

                    Ok((node_id, output))
                });
            }

            // Wait for all concurrent nodes in the batch (Fan-in synchronization)
            let mut completed_in_batch = Vec::new();

            while let Some(res) = join_set.join_next().await {
                match res {
                    Ok(Ok((node_id, output))) => {
                        completed_in_batch.push((node_id, output));
                    }
                    Ok(Err(e)) => return Err(e),
                    Err(join_err) => {
                        return Err(ExecutionError::NodeFailed {
                            node_id: "unknown".to_string(),
                            message: format!("Tokio join error: {}", join_err),
                        });
                    }
                }
            }

            // Mark nodes completed & record checkpoint
            {
                let mut comp = self.completed_nodes.write().await;
                for (nid, _) in &completed_in_batch {
                    comp.insert(nid.clone());
                }
            }

            // Checkpoint if configured
            if let Some(store) = &self.checkpoint_store {
                let mut step = self.step_counter.lock().await;
                *step += 1;
                let active = ready_nodes.clone();
                let comp_vec: Vec<NodeId> = self.completed_nodes.read().await.iter().cloned().collect();
                let snapshot = self.blackboard.snapshot().await;
                store
                    .save_checkpoint(self.execution_id, *step, active, comp_vec, snapshot)
                    .await?;
            }

            // Determine next nodes via outgoing edges and conditional evaluation
            let mut next_candidates: HashSet<NodeId> = HashSet::new();

            for (node_id, output) in &completed_in_batch {
                let edges = TopologyValidator::get_outgoing_edges(&self.graph, node_id);


                for edge in edges {
                    match &edge.kind {
                        EdgeKind::Direct | EdgeKind::ParallelFanOut => {
                            next_candidates.insert(edge.to.clone());
                        }
                        EdgeKind::BarrierFanIn => {
                            // Check if all predecessors are satisfied
                            let preds = TopologyValidator::get_predecessors(&self.graph, &edge.to);
                            let comp = self.completed_nodes.read().await;
                            if preds.iter().all(|p| comp.contains(p)) {
                                next_candidates.insert(edge.to.clone());
                            }
                        }
                        EdgeKind::Conditional { predicate } => {
                            let matches = ConditionalEvaluator::evaluate(
                                predicate,
                                output,
                                &self.blackboard,
                                None,
                            ).await;

                            if matches {
                                next_candidates.insert(edge.to.clone());
                            }
                        }
                        EdgeKind::FeedbackLoop { max_iterations } => {
                            let loop_key = format!("{}_to_{}", edge.from.0, edge.to.0);
                            let count = self.blackboard.increment_loop_count(&loop_key).await;
                            if count <= *max_iterations {
                                next_candidates.insert(edge.to.clone());
                            } else {
                                return Err(ExecutionError::LoopBudgetExceeded {
                                    loop_id: loop_key,
                                    limit: *max_iterations,
                                });
                            }
                        }
                    }
                }
            }

            // Determine which nodes were triggered via feedback loops
            let mut feedback_targets = HashSet::new();
            for (node_id, _) in &completed_in_batch {
                for edge in TopologyValidator::get_outgoing_edges(&self.graph, node_id) {
                    if matches!(edge.kind, EdgeKind::FeedbackLoop { .. }) {
                        feedback_targets.insert(edge.to.clone());
                    }
                }
            }

            // If any feedback loop fired, reset completed state for target and its dependencies so loop re-runs
            if !feedback_targets.is_empty() {
                let mut comp = self.completed_nodes.write().await;
                for target in &feedback_targets {
                    comp.remove(target);
                }
                for (node_id, _) in &completed_in_batch {
                    comp.remove(node_id);
                }
            }

            let comp = self.completed_nodes.read().await;
            ready_nodes = next_candidates
                .into_iter()
                .filter(|n| !comp.contains(n) || feedback_targets.contains(n))
                .collect();
        }

        Ok(())
    }
}

