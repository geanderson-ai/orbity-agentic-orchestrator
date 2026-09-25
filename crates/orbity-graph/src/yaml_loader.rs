use crate::types::{CliType, EdgeKind, GraphDefinition, GraphId, GraphNode, NodeId, NodeKind};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;


#[derive(Debug, Error)]
pub enum GraphYamlError {
    #[error("Failed to parse YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid YAML topology: {0}")]
    InvalidTopology(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlNodeDef {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub engine: Option<String>,
    pub cli: Option<String>,
    pub command: Option<String>,
    pub predicate_expr: Option<String>,
    pub prompt: Option<String>,
    pub timeout_secs: Option<u64>,
    pub quorum: Option<usize>,
    pub retries: Option<u32>,
    pub budget_usd: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlEdgeDef {
    pub from: String,
    pub to: String,
    #[serde(default = "default_edge_type")]
    #[serde(rename = "type")]
    pub edge_type: String,
    pub predicate: Option<String>,
    pub max_iterations: Option<u32>,
}

fn default_edge_type() -> String {
    "direct".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlGraphTeam {
    pub name: String,
    pub description: Option<String>,
    pub start_node: String,
    #[serde(default)]
    pub terminal_nodes: Vec<String>,
    #[serde(default)]
    pub nodes: Vec<YamlNodeDef>,
    #[serde(default)]
    pub edges: Vec<YamlEdgeDef>,
}

pub struct GraphYamlLoader;

impl GraphYamlLoader {
    /// Loads and parses a YAML string into a GraphDefinition.
    pub fn parse_yaml(yaml_content: &str) -> Result<GraphDefinition, GraphYamlError> {
        let team: YamlGraphTeam = serde_yaml::from_str(yaml_content)?;

        let mut builder = GraphDefinition::builder(
            GraphId::new(&team.name),
            &team.name,
            NodeId::new(&team.start_node),
        );

        if let Some(desc) = team.description {
            builder = builder.with_description(desc);
        }

        for term in team.terminal_nodes {
            builder = builder.add_terminal_node(NodeId::new(term));
        }

        for n in team.nodes {
            let kind = match n.node_type.as_str() {
                "orchestrator" => NodeKind::Orchestrator {
                    engine: n.engine.unwrap_or_else(|| "topcoat".to_string()),
                },
                "agent" => {
                    let cli = match n.cli.as_deref() {
                        Some("codex") => CliType::Codex,
                        Some("claude") => CliType::Claude,
                        Some("agy") => CliType::Agy,
                        Some("hermes") => CliType::Hermes,
                        Some("pi") => CliType::Pi,
                        _ => CliType::Custom,
                    };
                    NodeKind::Agent {
                        cli,
                        config: crate::types::AgentNodeSpec {
                            name: n.id.clone(),
                            role: n.description.clone(),
                            prompt_system: n.prompt,
                            timeout_seconds: n.timeout_secs,
                            ..Default::default()
                        },
                    }
                }
                "tool" => NodeKind::Tool {
                    command: n.command.unwrap_or_default(),
                    timeout_secs: n.timeout_secs.unwrap_or(60),
                },
                "conditional_router" => NodeKind::ConditionalRouter {
                    predicate_expr: n.predicate_expr.unwrap_or_default(),
                },
                "human_gate" => NodeKind::HumanGate {
                    prompt: n.prompt.unwrap_or_else(|| "Approval required".to_string()),
                    timeout_secs: n.timeout_secs,
                },
                "join_barrier" => NodeKind::JoinBarrier { quorum: n.quorum },
                other => {
                    return Err(GraphYamlError::InvalidTopology(format!(
                        "Unknown node type: '{}' for node '{}'",
                        other, n.id
                    )));
                }
            };

            let mut node = GraphNode::new(NodeId::new(&n.id), kind);
            if let Some(retries) = n.retries {
                node = node.with_retries(retries);
            }
            if let Some(budget) = n.budget_usd {
                node = node.with_budget(budget);
            }
            if let Some(desc) = n.description {
                node = node.with_description(desc);
            }

            builder = builder.add_node(node);
        }

        for e in team.edges {
            let kind = match e.edge_type.as_str() {
                "direct" => EdgeKind::Direct,
                "parallel_fan_out" | "fan_out" => EdgeKind::ParallelFanOut,
                "barrier_fan_in" | "fan_in" => EdgeKind::BarrierFanIn,
                "conditional" => EdgeKind::Conditional {
                    predicate: e.predicate.unwrap_or_else(|| "outcome == 'success'".to_string()),
                },
                "feedback_loop" | "loop" => EdgeKind::FeedbackLoop {
                    max_iterations: e.max_iterations.unwrap_or(3),
                },
                other => {
                    return Err(GraphYamlError::InvalidTopology(format!(
                        "Unknown edge type: '{}' between '{}' and '{}'",
                        other, e.from, e.to
                    )));
                }
            };

            let edge = crate::types::GraphEdge::new(NodeId::new(e.from), NodeId::new(e.to), kind);
            builder = builder.add_edge(edge);
        }

        Ok(builder.build())
    }

    /// Loads a GraphDefinition from a YAML file.
    pub fn load_file(path: impl AsRef<Path>) -> Result<GraphDefinition, GraphYamlError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_yaml(&content)
    }
}
