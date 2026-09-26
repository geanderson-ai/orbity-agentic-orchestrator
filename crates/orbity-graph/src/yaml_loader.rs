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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YamlNodeDef {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub engine: Option<String>,
    pub cli: Option<String>,
    pub provider: Option<String>,
    pub tier: Option<String>,
    pub model: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlNestedGraphTeam {
    pub team: YamlGraphTeam,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlSimpleAgentWrapper {
    pub agent: YamlSimpleAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlSimpleAgent {
    pub id: Option<String>,
    pub name: String,
    pub role: Option<String>,
    pub cli: Option<String>,
    pub provider: Option<String>,
    pub tier: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub prompt: Option<serde_yaml::Value>,
    pub timeout_secs: Option<u64>,
}

pub struct GraphYamlLoader;

impl GraphYamlLoader {
    /// Loads and parses a YAML string into a GraphDefinition.
    pub fn parse_yaml(yaml_content: &str) -> Result<GraphDefinition, GraphYamlError> {
        // 1. Try direct YamlGraphTeam
        if let Ok(team) = serde_yaml::from_str::<YamlGraphTeam>(yaml_content) {
            return Self::build_from_yaml_team(team);
        }

        // 2. Try nested team wrapper { team: { name, start_node, ... } }
        if let Ok(wrapper) = serde_yaml::from_str::<YamlNestedGraphTeam>(yaml_content) {
            return Self::build_from_yaml_team(wrapper.team);
        }

        // 3. Try TeamFileDefinition (canonical forester.yaml format)
        if let Ok(team_file) =
            serde_yaml::from_str::<orbity_core::contracts::TeamFileDefinition>(yaml_content)
        {
            return Self::build_from_team_file_def(team_file);
        }

        // 4. Try AgentFileDefinition (canonical agente01.yaml format)
        if let Ok(agent_file) =
            serde_yaml::from_str::<orbity_core::contracts::AgentFileDefinition>(yaml_content)
        {
            return Self::build_from_agent_file_def(agent_file);
        }

        // 5. Try simple agent wrapper { agent: { name, cli, ... } }
        if let Ok(agent_wrap) = serde_yaml::from_str::<YamlSimpleAgentWrapper>(yaml_content) {
            return Self::build_from_simple_agent(agent_wrap.agent);
        }

        // Fallback: return the original deserialization error from YamlGraphTeam
        let err: Result<YamlGraphTeam, _> = serde_yaml::from_str(yaml_content);
        Err(GraphYamlError::Yaml(err.unwrap_err()))
    }

    fn build_from_simple_agent(
        agent: YamlSimpleAgent,
    ) -> Result<GraphDefinition, GraphYamlError> {
        let agent_id = agent.id.unwrap_or_else(|| agent.name.to_lowercase().replace(' ', "_"));
        let cli = agent.cli.or(agent.provider).unwrap_or_else(|| "claude".to_string());
        let prompt_str = agent.system_prompt.or_else(|| {
            agent.prompt.and_then(|p| match p {
                serde_yaml::Value::String(s) => Some(s),
                serde_yaml::Value::Mapping(m) => m.get(&serde_yaml::Value::String("system".to_string()))
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                _ => None,
            })
        });

        let node = YamlNodeDef {
            id: agent_id.clone(),
            node_type: "agent".to_string(),
            engine: None,
            cli: Some(cli.clone()),
            provider: Some(cli),
            tier: agent.tier,
            model: agent.model,
            command: None,
            predicate_expr: None,
            prompt: prompt_str,
            timeout_secs: agent.timeout_secs.or(Some(300)),
            quorum: None,
            retries: Some(1),
            budget_usd: Some(5.0),
            description: agent.role.or(Some(agent.name.clone())),
        };

        let yaml_team = YamlGraphTeam {
            name: agent.name,
            description: Some("Single-agent execution graph".to_string()),
            start_node: agent_id.clone(),
            terminal_nodes: vec![agent_id],
            nodes: vec![node],
            edges: Vec::new(),
        };

        Self::build_from_yaml_team(yaml_team)
    }

    fn build_from_agent_file_def(
        def: orbity_core::contracts::AgentFileDefinition,
    ) -> Result<GraphDefinition, GraphYamlError> {
        let agent_id = if !def.agent.id.is_empty() {
            def.agent.id.clone()
        } else {
            def.agent.name.to_lowercase().replace(' ', "_")
        };

        let cli = def.agent.provider
            .or_else(|| def.agent.orchestrator.runner.clone())
            .unwrap_or_else(|| "agy".to_string());

        let prompt_str = def.agent.orchestrator.prompt.map(|p| p.system);

        let node = YamlNodeDef {
            id: agent_id.clone(),
            node_type: "agent".to_string(),
            engine: None,
            cli: Some(cli.clone()),
            provider: Some(cli),
            tier: def.agent.tier,
            model: def.agent.model,
            command: None,
            predicate_expr: None,
            prompt: prompt_str,
            timeout_secs: Some(300),
            quorum: None,
            retries: Some(1),
            budget_usd: Some(5.0),
            description: Some(def.agent.name.clone()),
        };

        let yaml_team = YamlGraphTeam {
            name: def.agent.name,
            description: Some("Autonomous agent execution graph".to_string()),
            start_node: agent_id.clone(),
            terminal_nodes: vec![agent_id],
            nodes: vec![node],
            edges: Vec::new(),
        };

        Self::build_from_yaml_team(yaml_team)
    }

    fn build_from_team_file_def(
        def: orbity_core::contracts::TeamFileDefinition,
    ) -> Result<GraphDefinition, GraphYamlError> {
        let team_name = def.team.name.clone();
        let description = def.team.description.clone();

        // Extract steps from orchestrator plan default_pipeline or steps
        let steps = def
            .team
            .orchestrator
            .plan
            .as_ref()
            .and_then(|p| p.default_pipeline.clone().or_else(|| p.steps.clone()))
            .unwrap_or_default();

        if steps.is_empty() {
            // Fallback: use workers as sequential nodes if available
            if !def.team.workers.is_empty() {
                let mut nodes = Vec::new();
                let mut edges = Vec::new();

                for (idx, w) in def.team.workers.iter().enumerate() {
                    let provider = w.provider.clone().or_else(|| {
                        if !w.runner.is_empty() {
                            Some(w.runner.clone())
                        } else {
                            None
                        }
                    });
                    nodes.push(YamlNodeDef {
                        id: w.id.clone(),
                        node_type: "agent".to_string(),
                        engine: None,
                        cli: provider.clone(),
                        provider,
                        tier: w.tier.clone(),
                        model: w.model.clone(),
                        command: None,
                        predicate_expr: None,
                        prompt: w.prompt.as_ref().map(|p| p.system.clone()),
                        timeout_secs: Some(300),
                        quorum: None,
                        retries: Some(1),
                        budget_usd: Some(1.0),
                        description: w.role.clone().or(w.name.clone()),
                    });

                    if idx > 0 {
                        edges.push(YamlEdgeDef {
                            from: def.team.workers[idx - 1].id.clone(),
                            to: w.id.clone(),
                            edge_type: "direct".to_string(),
                            predicate: None,
                            max_iterations: None,
                        });
                    }
                }

                let start_node = def.team.workers[0].id.clone();
                let terminal_nodes = vec![def.team.workers.last().unwrap().id.clone()];

                let yaml_team = YamlGraphTeam {
                    name: team_name,
                    description,
                    start_node,
                    terminal_nodes,
                    nodes,
                    edges,
                };
                return Self::build_from_yaml_team(yaml_team);
            }

            return Err(GraphYamlError::InvalidTopology(format!(
                "Team '{}' contains no pipeline steps or workers to form a graph",
                team_name
            )));
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for (idx, step) in steps.iter().enumerate() {
            let step_id = step
                .step_id
                .clone()
                .unwrap_or_else(|| format!("step_{}", idx));
            let delegate = step.delegate_to.as_deref().unwrap_or("");

            // Infer CLI from delegate_to or action
            let cli = if delegate.contains("codex")
                || step.action.as_deref().unwrap_or("").contains("codex")
            {
                "codex".to_string()
            } else if delegate.contains("claude")
                || step.action.as_deref().unwrap_or("").contains("claude")
            {
                "claude".to_string()
            } else if delegate.contains("hermes")
                || step.action.as_deref().unwrap_or("").contains("hermes")
            {
                "hermes".to_string()
            } else if delegate.contains("pi") || step.action.as_deref().unwrap_or("").contains("pi")
            {
                "pi".to_string()
            } else {
                "agy".to_string()
            };

            nodes.push(YamlNodeDef {
                id: step_id.clone(),
                node_type: "agent".to_string(),
                engine: None,
                cli: Some(cli.clone()),
                provider: Some(cli),
                tier: None,
                model: None,
                command: step.sandbox_action.clone(),
                predicate_expr: None,
                prompt: None,
                timeout_secs: Some(180),
                quorum: None,
                retries: Some(1),
                budget_usd: Some(0.50),
                description: step.name.clone(),
            });

            if idx > 0 {
                let prev_id = steps[idx - 1]
                    .step_id
                    .clone()
                    .unwrap_or_else(|| format!("step_{}", idx - 1));
                edges.push(YamlEdgeDef {
                    from: prev_id,
                    to: step_id.clone(),
                    edge_type: "direct".to_string(),
                    predicate: None,
                    max_iterations: None,
                });
            }
        }

        let start_node = steps[0]
            .step_id
            .clone()
            .unwrap_or_else(|| "step_0".to_string());
        let terminal_nodes = vec![steps
            .last()
            .unwrap()
            .step_id
            .clone()
            .unwrap_or_else(|| format!("step_{}", steps.len() - 1))];

        let yaml_team = YamlGraphTeam {
            name: team_name,
            description,
            start_node,
            terminal_nodes,
            nodes,
            edges,
        };

        Self::build_from_yaml_team(yaml_team)
    }

    fn build_from_yaml_team(team: YamlGraphTeam) -> Result<GraphDefinition, GraphYamlError> {
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
                    let cli_str = n.provider.as_deref().or(n.cli.as_deref());
                    let cli = match cli_str {
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
                            provider: n.provider.or(n.cli),
                            tier: n.tier,
                            model: n.model,
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
                    predicate: e
                        .predicate
                        .unwrap_or_else(|| "outcome == 'success'".to_string()),
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
