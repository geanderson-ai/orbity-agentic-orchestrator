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

fn default_agent_kind() -> String {
    "agent".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YamlTopologyNode {
    pub id: String,
    #[serde(default = "default_agent_kind")]
    pub kind: String,
    pub worker_ref: Option<String>,
    pub command: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub prompt: Option<String>,
    pub approval_rule: Option<serde_yaml::Value>,
    pub retries: Option<u32>,
    pub budget_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YamlTopologyEdge {
    pub from: serde_yaml::Value,
    pub to: serde_yaml::Value,
    #[serde(rename = "type", default = "default_edge_type")]
    pub edge_type: String,
    pub condition: Option<String>,
    pub inject_context: Option<Vec<String>>,
    pub max_iterations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YamlGraphTopology {
    pub entrypoint_node: Option<String>,
    pub start_node: Option<String>,
    #[serde(default)]
    pub terminal_nodes: Vec<String>,
    #[serde(default)]
    pub nodes: Vec<YamlTopologyNode>,
    #[serde(default)]
    pub edges: Vec<YamlTopologyEdge>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YamlSimpleAgent {
    pub id: Option<String>,
    pub name: String,
    pub role: Option<String>,
    pub cli: Option<String>,
    pub provider: Option<String>,
    pub runner: Option<String>,
    pub orchestrator: Option<orbity_core::contracts::OrchestratorConfig>,
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

        // Specific fallback error based on YAML content signature
        if yaml_content.contains("agent:") {
            let err: Result<orbity_core::contracts::AgentFileDefinition, _> = serde_yaml::from_str(yaml_content);
            if let Err(e) = err {
                return Err(GraphYamlError::Yaml(e));
            }
        } else if yaml_content.contains("team:") {
            let err: Result<orbity_core::contracts::TeamFileDefinition, _> = serde_yaml::from_str(yaml_content);
            if let Err(e) = err {
                return Err(GraphYamlError::Yaml(e));
            }
        }

        let err: Result<YamlGraphTeam, _> = serde_yaml::from_str(yaml_content);
        Err(GraphYamlError::Yaml(err.unwrap_err()))
    }

    fn build_from_simple_agent(
        agent: YamlSimpleAgent,
    ) -> Result<GraphDefinition, GraphYamlError> {
        let agent_id = agent.id.unwrap_or_else(|| agent.name.to_lowercase().replace(' ', "_"));
        let cli = agent
            .cli
            .or(agent.provider)
            .or(agent.runner)
            .or_else(|| agent.orchestrator.as_ref().and_then(|o| o.runner.clone()))
            .unwrap_or_else(|| "codex".to_string());
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

        let default_cli = def.agent.provider
            .or_else(|| def.agent.orchestrator.runner.clone())
            .unwrap_or_else(|| "agy".to_string());

        let steps = def
            .agent
            .orchestrator
            .plan
            .as_ref()
            .and_then(|p| p.steps.clone().or_else(|| p.default_pipeline.clone()))
            .unwrap_or_default();

        if !steps.is_empty() {
            let mut nodes = Vec::new();
            let mut edges = Vec::new();

            for (idx, step) in steps.iter().enumerate() {
                let step_id = step
                    .step_id
                    .clone()
                    .unwrap_or_else(|| format!("{}_step_{}", agent_id, idx));
                let delegate = step.delegate_to.as_deref().unwrap_or("");
                let action = step.action.as_deref().unwrap_or("");

                let cli = if delegate.contains("codex") || action.contains("codex") {
                    "codex".to_string()
                } else if delegate.contains("claude") || action.contains("claude") {
                    "claude".to_string()
                } else if delegate.contains("hermes") || action.contains("hermes") {
                    "hermes".to_string()
                } else if delegate.contains("pi") || action.contains("pi") {
                    "pi".to_string()
                } else if delegate.contains("agy") || action.contains("agy") {
                    "agy".to_string()
                } else {
                    default_cli.clone()
                };

                nodes.push(YamlNodeDef {
                    id: step_id.clone(),
                    node_type: "agent".to_string(),
                    engine: None,
                    cli: Some(cli.clone()),
                    provider: Some(cli),
                    tier: def.agent.tier.clone(),
                    model: def.agent.model.clone(),
                    command: None,
                    predicate_expr: None,
                    prompt: step.prompt_system.clone().or_else(|| step.prompt.as_ref().map(|p| p.system.clone())),
                    timeout_secs: Some(300),
                    quorum: None,
                    retries: Some(1),
                    budget_usd: Some(2.0),
                    description: step.name.clone().or_else(|| Some(format!("Step {}", idx + 1))),
                });

                if idx > 0 {
                    let prev_id = steps[idx - 1]
                        .step_id
                        .clone()
                        .unwrap_or_else(|| format!("{}_step_{}", agent_id, idx - 1));
                    edges.push(YamlEdgeDef {
                        from: prev_id,
                        to: step_id,
                        edge_type: "direct".to_string(),
                        predicate: None,
                        max_iterations: None,
                    });
                }
            }

            let start_node = nodes.first().unwrap().id.clone();
            let terminal_nodes = vec![nodes.last().unwrap().id.clone()];

            let yaml_team = YamlGraphTeam {
                name: def.agent.name,
                description: Some("Autonomous agent execution graph".to_string()),
                start_node,
                terminal_nodes,
                nodes,
                edges,
            };

            return Self::build_from_yaml_team(yaml_team);
        }

        let prompt_str = def.agent.orchestrator.prompt.map(|p| p.system);

        let node = YamlNodeDef {
            id: agent_id.clone(),
            node_type: "agent".to_string(),
            engine: None,
            cli: Some(default_cli.clone()),
            provider: Some(default_cli),
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

        // 1. If canonical graph_topology is provided, compile directly from rich graph topology
        if let Some(ref topology_val) = def.team.graph_topology {
            if !topology_val.is_null() {
                if let Ok(graph) = Self::build_from_graph_topology(team_name.clone(), description.clone(), &def.team, topology_val) {
                    return Ok(graph);
                }
            }
        }

        // 2. Extract steps from orchestrator plan default_pipeline or steps
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
                prompt: step.prompt_system.clone().or_else(|| step.prompt.as_ref().map(|p| p.system.clone())),
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

    fn build_from_graph_topology(
        team_name: String,
        description: Option<String>,
        team_config: &orbity_core::contracts::TeamConfig,
        topology_value: &serde_yaml::Value,
    ) -> Result<GraphDefinition, GraphYamlError> {
        let topology: YamlGraphTopology = serde_yaml::from_value(topology_value.clone())
            .map_err(|e| GraphYamlError::InvalidTopology(format!("Failed to parse graph_topology: {}", e)))?;

        if topology.nodes.is_empty() {
            return Err(GraphYamlError::InvalidTopology("graph_topology contains no nodes".to_string()));
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let find_worker = |ref_name: &str| -> Option<&orbity_core::contracts::WorkerConfig> {
            let clean = ref_name.to_lowercase().replace('_', "-");
            if let Some(w) = team_config.workers.iter().find(|w| w.id == ref_name || w.name.as_deref() == Some(ref_name)) {
                return Some(w);
            }
            for w in &team_config.workers {
                let w_id = w.id.to_lowercase().replace('_', "-");
                let w_runner = w.runner.to_lowercase();
                let w_provider = w.provider.as_deref().unwrap_or("").to_lowercase();
                if clean.contains(&w_runner) || (!w_provider.is_empty() && clean.contains(&w_provider)) {
                    return Some(w);
                }
                if w_id.contains("codex") && clean.contains("codex") {
                    return Some(w);
                }
                if (w_id.contains("claude") || w_id.contains("review") || w_id.contains("audit")) && (clean.contains("claude") || clean.contains("review") || clean.contains("audit")) {
                    return Some(w);
                }
                if (w_id.contains("hermes") || w_id.contains("scout") || w_id.contains("research")) && (clean.contains("hermes") || clean.contains("scout") || clean.contains("research")) {
                    return Some(w);
                }
                if (w_id.contains("pi") || w_id.contains("fix")) && (clean.contains("pi") || clean.contains("fix")) {
                    return Some(w);
                }
                if (w_id.contains("agy") || w_id.contains("lead") || w_id.contains("plan")) && (clean.contains("agy") || clean.contains("lead") || clean.contains("plan")) {
                    return Some(w);
                }
            }
            None
        };

        for node_def in &topology.nodes {
            let node_id = node_def.id.clone();
            let kind = node_def.kind.to_lowercase();
            let matched_worker = node_def.worker_ref.as_deref().and_then(find_worker);

            let (node_type, cli, tier, model, prompt_str, command, predicate_expr) = match kind.as_str() {
                "tool" => {
                    ("tool".to_string(), None, None, None, None, node_def.command.clone(), None)
                }
                "human_gate" => {
                    ("human_gate".to_string(), None, None, None, node_def.prompt.clone(), None, None)
                }
                "router" | "conditional" => {
                    ("conditional_router".to_string(), None, None, None, None, None, node_def.prompt.clone())
                }
                "join" | "barrier" => {
                    ("join_barrier".to_string(), None, None, None, None, None, None)
                }
                "supervisor" => {
                    let orch_runner = team_config.orchestrator.runner.clone().unwrap_or_else(|| "agy".to_string());
                    let p = node_def.prompt.clone().or_else(|| team_config.orchestrator.prompt.as_ref().map(|p| p.system.clone()));
                    ("agent".to_string(), Some(orch_runner.clone()), None, None, p, None, None)
                }
                _ => {
                    let (cli_val, tier_val, model_val, p_val) = if let Some(w) = matched_worker {
                        let c = if !w.runner.is_empty() {
                            w.runner.clone()
                        } else {
                            w.provider.clone().unwrap_or_else(|| "codex".to_string())
                        };
                        let t = w.tier.clone();
                        let m = w.model.clone();
                        let p = node_def.prompt.clone().or_else(|| w.prompt.as_ref().map(|p| p.system.clone()));
                        (c, t, m, p)
                    } else {
                        let ref_str = node_def.worker_ref.as_deref().unwrap_or(&node_def.id);
                        let c = if ref_str.contains("codex") {
                            "codex".to_string()
                        } else if ref_str.contains("claude") {
                            "claude".to_string()
                        } else if ref_str.contains("hermes") {
                            "hermes".to_string()
                        } else if ref_str.contains("pi") {
                            "pi".to_string()
                        } else {
                            "agy".to_string()
                        };
                        (c, None, None, node_def.prompt.clone())
                    };
                    ("agent".to_string(), Some(cli_val.clone()), tier_val, model_val, p_val, None, None)
                }
            };

            nodes.push(YamlNodeDef {
                id: node_id.clone(),
                node_type,
                engine: None,
                cli: cli.clone(),
                provider: cli,
                tier,
                model,
                command,
                predicate_expr,
                prompt: prompt_str,
                timeout_secs: node_def.timeout_seconds.or(Some(300)),
                quorum: None,
                retries: node_def.retries.or(Some(1)),
                budget_usd: node_def.budget_usd.or(Some(1.0)),
                description: Some(node_def.id.clone()),
            });
        }

        for edge_def in &topology.edges {
            let from_nodes: Vec<String> = match &edge_def.from {
                serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
                serde_yaml::Value::String(s) => vec![s.clone()],
                _ => Vec::new(),
            };

            let to_nodes: Vec<String> = match &edge_def.to {
                serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
                serde_yaml::Value::String(s) => vec![s.clone()],
                _ => Vec::new(),
            };

            for from_n in &from_nodes {
                for to_n in &to_nodes {
                    edges.push(YamlEdgeDef {
                        from: from_n.clone(),
                        to: to_n.clone(),
                        edge_type: edge_def.edge_type.clone(),
                        predicate: edge_def.condition.clone(),
                        max_iterations: edge_def.max_iterations,
                    });
                }
            }
        }

        let start_node = topology.entrypoint_node
            .or(topology.start_node)
            .unwrap_or_else(|| nodes.first().unwrap().id.clone());

        let terminal_nodes = if !topology.terminal_nodes.is_empty() {
            topology.terminal_nodes
        } else {
            vec![nodes.last().unwrap().id.clone()]
        };

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
