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
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
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
                format!(
                    "Topcoat Orchestrator [{}] plan generated with context len {}",
                    engine,
                    injected_context.len()
                )
            }
            NodeKind::Agent { cli, config } => {
                format!(
                    "Agent [{}] ({}) executed task. Context: {}",
                    cli,
                    config.name,
                    injected_context.trim()
                )
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
            NodeKind::JoinBarrier { .. } => "Join barrier synchronization reached.".to_string(),
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
    step_counter: Arc<AtomicU32>,
    completed_nodes: Arc<RwLock<HashSet<NodeId>>>,
    concurrency_semaphore: Option<Arc<Semaphore>>,
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
            step_counter: Arc::new(AtomicU32::new(0)),
            completed_nodes: Arc::new(RwLock::new(HashSet::new())),
            concurrency_semaphore: None,
        }
    }

    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        if max_concurrency > 0 {
            self.concurrency_semaphore = Some(Arc::new(Semaphore::new(max_concurrency)));
        }
        self
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

    pub fn with_initial_state(
        self,
        step: u32,
        _active_nodes: Vec<NodeId>,
        completed: Vec<NodeId>,
    ) -> Self {
        self.step_counter.store(step, Ordering::SeqCst);
        {
            let mut w = self.completed_nodes.blocking_write();
            for c in completed {
                w.insert(c);
            }
        }
        Self {
            completed_nodes: self.completed_nodes,
            ..self
        }
    }

    pub fn execution_id(&self) -> Uuid {
        self.execution_id
    }

    pub fn blackboard(&self) -> &Blackboard {
        &self.blackboard
    }

    /// Validates topology and runs the graph to completion.
    pub async fn execute(&self) -> Result<(), ExecutionError> {
        let start_time = std::time::Instant::now();
        let run_id_str = self.execution_id.to_string();

        // 1. Validate graph topology
        let _plan = TopologyValidator::validate_and_plan(&self.graph)?;

        // 2. Publish RunInitiated event
        if let Some(bus) = &self.event_bus {
            let prompt = self
                .blackboard
                .get_context("user_prompt")
                .await
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| self.graph.name.clone());

            let _ = bus
                .emit(
                    &run_id_str,
                    orbity_core::events::RuntimeEvent::RunInitiated {
                        run_id: run_id_str.clone(),
                        prompt,
                        team_name: Some(self.graph.name.clone()),
                    },
                )
                .await;
        }

        let mut ready_nodes = {
            let comp = self.completed_nodes.read().await;
            if comp.is_empty() {
                vec![self.graph.start_node.clone()]
            } else {
                // If resuming, start node has already run; derive ready nodes from non-completed
                let mut candidates = Vec::new();
                for edge in &self.graph.edges {
                    if comp.contains(&edge.from) && !comp.contains(&edge.to) {
                        candidates.push(edge.to.clone());
                    }
                }
                if candidates.is_empty() {
                    vec![self.graph.start_node.clone()]
                } else {
                    candidates
                }
            }
        };

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
                let sem = self.concurrency_semaphore.clone();
                let bus = self.event_bus.clone();
                let run_id = run_id_str.clone();

                join_set.spawn(async move {
                    let _permit = match &sem {
                        Some(s) => Some(s.acquire().await.map_err(|e| ExecutionError::NodeFailed {
                            node_id: node_id.0.clone(),
                            message: format!("Concurrency semaphore error: {}", e),
                        })?),
                        None => None,
                    };

                    // Check & Reserve FinOps budget atomically
                    let estimated_cost = node.budget_limit_usd.unwrap_or(0.05);
                    finops
                        .reserve_budget(&node_id, estimated_cost, node.budget_limit_usd)
                        .await
                        .map_err(ExecutionError::BudgetExceeded)?;

                    // Inject context from predecessors or user prompt if root node
                    let mut injected_context = blackboard.inject_context(&preds).await;
                    if injected_context.is_empty() {
                        if let Some(user_prompt) = blackboard.get_context("user_prompt").await {
                            if let Some(s) = user_prompt.as_str() {
                                injected_context = s.to_string();
                            }
                        }
                    }

                    // Publish AgentStarted event
                    if let Some(b) = &bus {
                        let _ = b
                            .emit(
                                &run_id,
                                orbity_core::events::RuntimeEvent::AgentStarted {
                                    run_id: run_id.clone(),
                                    task_id: Some(node_id.0.clone()),
                                    agent_id: node_id.0.clone(),
                                    agent_name: node.name().to_string(),
                                },
                            )
                            .await;
                    }

                    // Check declarative Human Gate / YAML policy
                    if let NodeKind::HumanGate { prompt, .. } = &node.kind {
                        let decision = finops.evaluate_approval(prompt).await;
                        match decision {
                            ApprovalDecision::Approved => {
                                // Auto approved via YAML policy
                            }
                            ApprovalDecision::Rejected => {
                                if let Some(b) = &bus {
                                    let _ = b.emit(&run_id, orbity_core::events::RuntimeEvent::ApprovalRejected {
                                        run_id: run_id.clone(),
                                        rejecter: "PolicyEngine".to_string(),
                                        reason: Some(format!("Policy rejected gate: {}", prompt)),
                                    }).await;
                                }
                                return Err(ExecutionError::NodeFailed {
                                    node_id: node_id.0,
                                    message: format!("Policy rejected human gate: {}", prompt),
                                });
                            }
                            ApprovalDecision::NeedsHuman => {
                                if let Some(b) = &bus {
                                    let _ = b.emit(&run_id, orbity_core::events::RuntimeEvent::ApprovalRequired {
                                        run_id: run_id.clone(),
                                        prompt: prompt.clone(),
                                        proposed_cost_usd: Some(estimated_cost),
                                        timeout_seconds: Some(300),
                                    }).await;
                                }
                                return Err(ExecutionError::WaitingHumanApproval {
                                    node_id: node_id.0,
                                    prompt: prompt.clone(),
                                });
                            }
                        }
                    }

                    // Run node with intelligent retries and error context injection
                    let mut attempts = 0;
                    let max_attempts = 1 + node.retries_limit;
                    let mut last_err = String::new();
                    let mut output = NodeOutput::default();
                    let mut current_context = injected_context.clone();

                    while attempts < max_attempts {
                        attempts += 1;
                        if attempts > 1 && !last_err.is_empty() {
                            current_context = format!(
                                "{}\n\n[RETRY ATTEMPT {}/{}]\nPrevious execution failed with error:\n{}\nPlease fix the issue and try again.",
                                injected_context, attempts, max_attempts, last_err
                            );
                            let backoff_ms = (50 * (1 << (attempts - 1))).min(2000);
                            tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                        }

                        match runner.run(&node, &current_context, &blackboard).await {
                            Ok(out) => {
                                let ok = out.success;
                                output = out;
                                if ok {
                                    last_err.clear();
                                    break;
                                } else {
                                    last_err = if !output.stderr.is_empty() {
                                        output.stderr.clone()
                                    } else {
                                        format!("Process exited with status {:?}", output.exit_code)
                                    };
                                }
                            }
                            Err(e) => {
                                last_err = e;
                            }
                        }
                    }

                    if !output.success {
                        if let Some(b) = &bus {
                            let _ = b
                                .emit(
                                    &run_id,
                                    orbity_core::events::RuntimeEvent::AgentFailed {
                                        run_id: run_id.clone(),
                                        task_id: Some(node_id.0.clone()),
                                        agent_id: node_id.0.clone(),
                                        agent_name: node.name().to_string(),
                                        error: last_err.clone(),
                                    },
                                )
                                .await;
                        }
                        return Err(ExecutionError::NodeFailed {
                            node_id: node_id.0,
                            message: if !last_err.is_empty() {
                                last_err
                            } else {
                                "Node execution failed".to_string()
                            },
                        });
                    }

                    // Commit FinOps spend and release reservation
                    finops
                        .commit_spend(
                            &node_id,
                            output.cost_usd,
                            estimated_cost,
                            output.tokens_input + output.tokens_output,
                            0,
                            0,
                        )
                        .await;

                    // Publish AgentFinished & TokenUsageUpdated events
                    if let Some(b) = &bus {
                        let _ = b
                            .emit(
                                &run_id,
                                orbity_core::events::RuntimeEvent::AgentFinished {
                                    run_id: run_id.clone(),
                                    task_id: Some(node_id.0.clone()),
                                    agent_id: node_id.0.clone(),
                                    agent_name: node.name().to_string(),
                                    summary: Some(format!("Output length: {} chars", output.stdout.len())),
                                },
                            )
                            .await;

                        let _ = b
                            .emit(
                                &run_id,
                                orbity_core::events::RuntimeEvent::TokenUsageUpdated {
                                    run_id: run_id.clone(),
                                    task_id: Some(node_id.0.clone()),
                                    agent_name: node.name().to_string(),
                                    input_tokens: output.tokens_input as u64,
                                    output_tokens: output.tokens_output as u64,
                                    cached_tokens: 0,
                                    reasoning_tokens: 0,
                                    cost_usd: output.cost_usd,
                                },
                            )
                            .await;
                    }

                    // Update blackboard with output and structured handoff
                    blackboard
                        .set_node_output(node_id.0.clone(), output.stdout.clone())
                        .await;

                    blackboard
                        .record_handoff(crate::blackboard::HandoffEnvelope {
                            from_node: node_id.0.clone(),
                            summary: format!("Node {} execution finished successfully", node_id.0),
                            artifacts: output.artifacts.keys().cloned().collect(),
                            exit_code: output.exit_code.unwrap_or(0),
                            timestamp_ms: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                        })
                        .await;

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
                    Ok(Err(e)) => {
                        if let Some(bus) = &self.event_bus {
                            let _ = bus.emit(&run_id_str, orbity_core::events::RuntimeEvent::RunFailed {
                                run_id: run_id_str.clone(),
                                error: e.to_string(),
                            }).await;
                        }
                        return Err(e);
                    }
                    Err(join_err) => {
                        let err = ExecutionError::NodeFailed {
                            node_id: "unknown".to_string(),
                            message: format!("Tokio join error: {}", join_err),
                        };
                        if let Some(bus) = &self.event_bus {
                            let _ = bus.emit(&run_id_str, orbity_core::events::RuntimeEvent::RunFailed {
                                run_id: run_id_str.clone(),
                                error: err.to_string(),
                            }).await;
                        }
                        return Err(err);
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
                let step = self.step_counter.fetch_add(1, Ordering::SeqCst) + 1;
                let active = ready_nodes.clone();
                let comp_vec: Vec<NodeId> =
                    self.completed_nodes.read().await.iter().cloned().collect();
                let snapshot = self.blackboard.snapshot().await;
                store
                    .save_checkpoint(self.execution_id, step, active, comp_vec, snapshot)
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
                            )
                            .await;

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
            let ready_filtered = {
                let mut comp = self.completed_nodes.write().await;
                if !feedback_targets.is_empty() {
                    for target in &feedback_targets {
                        comp.remove(target);
                    }
                    for (node_id, _) in &completed_in_batch {
                        comp.remove(node_id);
                    }
                }

                next_candidates
                    .into_iter()
                    .filter(|n| !comp.contains(n) || feedback_targets.contains(n))
                    .collect()
            };

            ready_nodes = ready_filtered;
        }

        // Publish RunCompleted event
        if let Some(bus) = &self.event_bus {
            let total_tokens = self.finops.total_tokens().await as u64;
            let total_cost = self.finops.total_cost().await;
            let duration_ms = start_time.elapsed().as_millis() as u64;

            let _ = bus
                .emit(
                    &run_id_str,
                    orbity_core::events::RuntimeEvent::RunCompleted {
                        run_id: run_id_str.clone(),
                        total_tokens,
                        total_cost_usd: total_cost,
                        duration_ms,
                    },
                )
                .await;
        }

        Ok(())
    }
}
