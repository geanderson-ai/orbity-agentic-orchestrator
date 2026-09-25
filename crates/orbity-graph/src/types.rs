//! Canonical Graph Engineering data models for DAG topology and execution state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Unique identifier of a node in a graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<String>> From<T> for NodeId {
    fn from(s: T) -> Self {
        NodeId(s.into())
    }
}

/// Unique identifier of a graph definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub String);

impl GraphId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<String>> From<T> for GraphId {
    fn from(s: T) -> Self {
        GraphId(s.into())
    }
}

/// Supported native CLI agent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliType {
    Codex,
    Claude,
    Agy,
    Hermes,
    Pi,
    Custom,
}

impl std::fmt::Display for CliType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Codex => write!(f, "codex"),
            Self::Claude => write!(f, "claude"),
            Self::Agy => write!(f, "agy"),
            Self::Hermes => write!(f, "hermes"),
            Self::Pi => write!(f, "pi"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Specific configuration for an Agent node in the graph.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentNodeSpec {
    pub name: String,
    pub role: Option<String>,
    pub model: Option<String>,
    pub prompt_system: Option<String>,
    #[serde(default)]
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub cli_args: Vec<String>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

/// The kind and behavior of a graph node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    /// Deterministic workflow orchestrator node (e.g. Tokio Topcoat coordinator).
    Orchestrator { engine: String },
    /// Specialized worker agent running in a sandbox with a specific CLI.
    Agent {
        cli: CliType,
        #[serde(default)]
        config: AgentNodeSpec,
    },
    /// Shell command or tool executed in sandbox.
    Tool {
        command: String,
        #[serde(default = "default_tool_timeout")]
        timeout_secs: u64,
    },
    /// Dynamic conditional branch router evaluating an expression against the blackboard.
    ConditionalRouter { predicate_expr: String },
    /// Human-in-the-loop gate requiring interactive or policy-based approval.
    HumanGate {
        prompt: String,
        #[serde(default)]
        timeout_secs: Option<u64>,
    },
    /// Synchronization barrier waiting for upstream nodes.
    JoinBarrier {
        #[serde(default)]
        quorum: Option<usize>,
    },
}

fn default_tool_timeout() -> u64 {
    60
}

/// Node definition in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub kind: NodeKind,
    #[serde(default)]
    pub retries_limit: u32,
    #[serde(default)]
    pub budget_limit_usd: Option<f64>,
    #[serde(default)]
    pub description: Option<String>,
}

impl GraphNode {
    pub fn new(id: impl Into<NodeId>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            kind,
            retries_limit: 0,
            budget_limit_usd: None,
            description: None,
        }
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries_limit = retries;
        self
    }

    pub fn with_budget(mut self, budget_usd: f64) -> Self {
        self.budget_limit_usd = Some(budget_usd);
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// The kind of transition edge between nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeKind {
    /// Sequential transition: A completes successfully -> B starts.
    Direct,
    /// Parallel Fan-Out: A completes -> branches [B, C, D] start concurrently.
    ParallelFanOut,
    /// Barrier Fan-In: upstream nodes finish -> downstream target unlocks.
    BarrierFanIn,
    /// Conditional edge: traversal depends on predicate evaluation against blackboard.
    Conditional { predicate: String },
    /// Controlled feedback loop for self-healing code and iterative refinement.
    FeedbackLoop { max_iterations: u32 },
}

/// Directed edge connecting two nodes in the graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
    #[serde(default)]
    pub payload_filter: Option<Vec<String>>,
}

impl GraphEdge {
    pub fn new(from: impl Into<NodeId>, to: impl Into<NodeId>, kind: EdgeKind) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            kind,
            payload_filter: None,
        }
    }

    pub fn direct(from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        Self::new(from, to, EdgeKind::Direct)
    }

    pub fn fan_out(from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        Self::new(from, to, EdgeKind::ParallelFanOut)
    }

    pub fn fan_in(from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        Self::new(from, to, EdgeKind::BarrierFanIn)
    }

    pub fn conditional(from: impl Into<NodeId>, to: impl Into<NodeId>, predicate: impl Into<String>) -> Self {
        Self::new(from, to, EdgeKind::Conditional { predicate: predicate.into() })
    }

    pub fn feedback_loop(from: impl Into<NodeId>, to: impl Into<NodeId>, max_iterations: u32) -> Self {
        Self::new(from, to, EdgeKind::FeedbackLoop { max_iterations })
    }
}

/// Complete declarative graph topology definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDefinition {
    pub id: GraphId,
    pub name: String,
    pub nodes: HashMap<NodeId, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub start_node: NodeId,
    pub terminal_nodes: HashSet<NodeId>,
    #[serde(default)]
    pub description: Option<String>,
}

impl GraphDefinition {
    pub fn builder(id: impl Into<GraphId>, name: impl Into<String>, start_node: impl Into<NodeId>) -> GraphBuilder {
        GraphBuilder::new(id, name, start_node)
    }
}

/// Fluent builder for constructing graph topologies programmatically.
pub struct GraphBuilder {
    id: GraphId,
    name: String,
    start_node: NodeId,
    nodes: HashMap<NodeId, GraphNode>,
    edges: Vec<GraphEdge>,
    terminal_nodes: HashSet<NodeId>,
    description: Option<String>,
}

impl GraphBuilder {
    pub fn new(id: impl Into<GraphId>, name: impl Into<String>, start_node: impl Into<NodeId>) -> Self {
        let start = start_node.into();
        Self {
            id: id.into(),
            name: name.into(),
            start_node: start,
            nodes: HashMap::new(),
            edges: Vec::new(),
            terminal_nodes: HashSet::new(),
            description: None,
        }
    }

    pub fn add_node(mut self, node: GraphNode) -> Self {
        self.nodes.insert(node.id.clone(), node);
        self
    }

    pub fn add_edge(mut self, edge: GraphEdge) -> Self {
        self.edges.push(edge);
        self
    }

    pub fn add_terminal_node(mut self, node_id: impl Into<NodeId>) -> Self {
        self.terminal_nodes.insert(node_id.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn build(self) -> GraphDefinition {
        GraphDefinition {
            id: self.id,
            name: self.name,
            nodes: self.nodes,
            edges: self.edges,
            start_node: self.start_node,
            terminal_nodes: self.terminal_nodes,
            description: self.description,
        }
    }
}

/// Execution status of an individual graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeExecutionStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Skipped,
    WaitingHumanApproval,
}

/// Execution output produced by a node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    #[serde(default)]
    pub artifacts: HashMap<String, String>,
    #[serde(default)]
    pub tokens_input: u32,
    #[serde(default)]
    pub tokens_output: u32,
    #[serde(default)]
    pub cost_usd: f64,
    #[serde(default)]
    pub duration_ms: u64,
}

/// Cryptographic state checkpoint of a running graph for crash resilience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStateCheckpoint {
    pub execution_id: Uuid,
    pub step_number: u32,
    pub active_nodes: Vec<NodeId>,
    pub completed_nodes: Vec<NodeId>,
    pub blackboard_snapshot: serde_json::Value,
    pub previous_hash: String,
    pub state_hash: String,
    pub recorded_at: DateTime<Utc>,
}
