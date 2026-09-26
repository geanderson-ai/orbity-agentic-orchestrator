//! Declarative agent, team and orchestrator contract definitions and YAML parser.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractsError {
    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),
    #[error("I/O error reading contract file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<String>> From<T> for AgentId {
    fn from(s: T) -> Self {
        AgentId(s.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentLifecycleState {
    Draft,
    Spawning,
    Idle,
    Planning,
    Executing,
    Paused,
    Completed,
    Failed,
    Archived,
}

impl std::fmt::Display for AgentLifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Spawning => write!(f, "Spawning"),
            Self::Idle => write!(f, "Idle"),
            Self::Planning => write!(f, "Planning"),
            Self::Executing => write!(f, "Executing"),
            Self::Paused => write!(f, "Paused"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Archived => write!(f, "Archived"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptConfig {
    pub system: String,
    #[serde(default)]
    pub guidelines: Vec<String>,
    #[serde(default)]
    pub context_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanStep {
    pub step_id: Option<String>,
    pub name: Option<String>,
    pub delegate_to: Option<String>,
    pub action: Option<String>,
    pub sandbox_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanConfig {
    pub strategy: String,
    pub steps: Option<Vec<PlanStep>>,
    pub default_pipeline: Option<Vec<PlanStep>>,
    pub max_iterations: Option<u32>,
    pub require_synthesis: Option<bool>,
    pub on_worker_failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkerConfig {
    pub id: String,
    pub name: Option<String>,
    pub role: Option<String>,
    #[serde(default)]
    pub runner: String,
    pub provider: Option<String>,
    pub tier: Option<String>,
    pub model: Option<String>,
    pub cli_command: Option<String>,
    pub cli_subcommand: Option<String>,
    #[serde(default)]
    pub cli_args: Vec<String>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    pub prompt: Option<PromptConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SandboxPolicy {
    pub provider: Option<String>,
    #[serde(default)]
    pub filesystem: HashMap<String, String>,
    pub network: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub memory_limit_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    #[default]
    Automatic,
    Hybrid,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FallbackAction {
    #[default]
    Approve,
    Reject,
    EscalateToHuman,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExpensiveModelAction {
    #[default]
    AutoApprove,
    AutoReject,
    AskHuman,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AutoApprovalRule {
    pub rule: String,
    pub condition: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AutoRejectRule {
    pub rule: String,
    pub condition: Option<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ApprovalPolicy {
    #[serde(default)]
    pub mode: ApprovalMode,
    #[serde(default)]
    pub auto_approve: Vec<AutoApprovalRule>,
    #[serde(default)]
    pub auto_reject: Vec<AutoRejectRule>,
    #[serde(default)]
    pub fallback_action: FallbackAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    NeedsHuman,
}

impl ApprovalPolicy {
    /// Determines whether a command or action is approved, rejected, or needs human approval based on YAML policy.
    pub fn evaluate_command(&self, command: &str) -> ApprovalDecision {
        match self.mode {
            ApprovalMode::Manual => ApprovalDecision::NeedsHuman,
            ApprovalMode::Automatic | ApprovalMode::Hybrid => {
                // Check auto-reject rules first (precedence)
                for r in &self.auto_reject {
                    for pat in &r.patterns {
                        if glob_or_contains(command, pat) {
                            return ApprovalDecision::Rejected;
                        }
                    }
                }
                // Check auto-approve rules
                for a in &self.auto_approve {
                    for pat in &a.patterns {
                        if glob_or_contains(command, pat) {
                            return ApprovalDecision::Approved;
                        }
                    }
                }
                match self.fallback_action {
                    FallbackAction::Approve => ApprovalDecision::Approved,
                    FallbackAction::Reject => ApprovalDecision::Rejected,
                    FallbackAction::EscalateToHuman => {
                        if self.mode == ApprovalMode::Automatic {
                            ApprovalDecision::Approved
                        } else {
                            ApprovalDecision::NeedsHuman
                        }
                    }
                }
            }
        }
    }
}

fn glob_or_contains(haystack: &str, pattern: &str) -> bool {
    let p = pattern.trim();
    if p.starts_with('*') && p.ends_with('*') && p.len() > 2 {
        haystack.contains(&p[1..p.len() - 1])
    } else if let Some(prefix) = p.strip_suffix('*') {
        haystack.starts_with(prefix)
    } else if let Some(suffix) = p.strip_prefix('*') {
        haystack.ends_with(suffix)
    } else {
        haystack.contains(p)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinOpsConfig {
    pub max_budget_usd: Option<f64>,
    pub max_total_tokens: Option<u64>,
    pub expensive_model_approval_threshold_usd: Option<f64>,
    pub expensive_model_action: Option<ExpensiveModelAction>,
    pub alert_at_budget_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrchestratorConfig {
    pub id: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub runner: Option<String>,
    pub r#type: Option<String>,
    #[serde(default)]
    pub cli_options: HashMap<String, serde_json::Value>,
    pub prompt: Option<PromptConfig>,
    pub plan: Option<PlanConfig>,
    #[serde(default)]
    pub team: Vec<WorkerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub provider: Option<String>,
    pub tier: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub approval_policy: Option<ApprovalPolicy>,
    pub orchestrator: OrchestratorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileDefinition {
    pub version: String,
    pub agent: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub finops: Option<FinOpsConfig>,
    pub sandbox_defaults: Option<SandboxPolicy>,
    pub approval_policy: Option<ApprovalPolicy>,
    pub orchestrator: OrchestratorConfig,
    #[serde(default)]
    pub workers: Vec<WorkerConfig>,
    pub graph_topology: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamFileDefinition {
    pub version: String,
    pub team: TeamConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: AgentId,
    pub name: String,
    pub team_name: Option<String>,
    pub state: AgentLifecycleState,
    pub orchestrator: OrchestratorConfig,
    pub config_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Computes SHA-256 hash of a string content
pub fn compute_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Parses an agent declaration YAML content.
pub fn parse_agent_yaml(content: &str) -> Result<AgentFileDefinition, ContractsError> {
    let def: AgentFileDefinition = serde_yaml::from_str(content)?;
    if def.agent.id.trim().is_empty() {
        return Err(ContractsError::ValidationError(
            "agent.id cannot be empty".to_string(),
        ));
    }
    Ok(def)
}

/// Parses a team declaration YAML content.
pub fn parse_team_yaml(content: &str) -> Result<TeamFileDefinition, ContractsError> {
    let def: TeamFileDefinition = serde_yaml::from_str(content)?;
    if def.team.name.trim().is_empty() {
        return Err(ContractsError::ValidationError(
            "team.name cannot be empty".to_string(),
        ));
    }
    Ok(def)
}

/// Loads and validates an agent YAML file from disk.
pub fn load_agent_file(path: impl AsRef<Path>) -> Result<AgentFileDefinition, ContractsError> {
    let content = std::fs::read_to_string(path)?;
    parse_agent_yaml(&content)
}

/// Loads and validates a team YAML file from disk.
pub fn load_team_file(path: impl AsRef<Path>) -> Result<TeamFileDefinition, ContractsError> {
    let content = std::fs::read_to_string(path)?;
    parse_team_yaml(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_example_agent01() {
        let path = std::path::Path::new("../../examples/agents/agente01.yaml");
        let fallback_path = std::path::Path::new("examples/agents/agente01.yaml");
        let target = if path.exists() { path } else { fallback_path };
        let def = load_agent_file(target).expect("Failed to parse agente01.yaml");

        assert_eq!(def.agent.id, "agt_01h89x2k");
        assert_eq!(def.agent.name, "Agente01");
        assert_eq!(def.agent.orchestrator.team.len(), 3);
        assert_eq!(def.agent.orchestrator.team[0].id, "codex_coder");
        assert_eq!(def.agent.orchestrator.team[0].runner, "codex");
        assert!(def.agent.approval_policy.is_some());
        let pol = def.agent.approval_policy.unwrap();
        assert_eq!(pol.mode, ApprovalMode::Automatic);
        assert_eq!(
            pol.evaluate_command("cargo test"),
            ApprovalDecision::Approved
        );
    }

    #[test]
    fn test_parse_example_agent02() {
        let path = std::path::Path::new("../../examples/agents/agente02.yaml");
        let fallback_path = std::path::Path::new("examples/agents/agente02.yaml");
        let target = if path.exists() { path } else { fallback_path };
        let def = load_agent_file(target).expect("Failed to parse agente02.yaml");

        assert_eq!(def.agent.id, "agt_02m84k1z");
        assert_eq!(def.agent.orchestrator.team.len(), 2);
        assert!(def.agent.approval_policy.is_some());
    }

    #[test]
    fn test_parse_example_forester_team() {
        let path = std::path::Path::new("../../examples/teams/forester.yaml");
        let fallback_path = std::path::Path::new("examples/teams/forester.yaml");
        let target = if path.exists() { path } else { fallback_path };
        let def = load_team_file(target).expect("Failed to parse forester.yaml");

        assert_eq!(def.team.name, "forester");
        assert_eq!(def.team.workers.len(), 5);
        assert!(def.team.finops.is_some());
        let finops = def.team.finops.as_ref().unwrap();
        assert_eq!(finops.max_budget_usd, Some(2.00));
        assert_eq!(
            finops.expensive_model_action,
            Some(ExpensiveModelAction::AutoApprove)
        );
        assert!(def.team.approval_policy.is_some());
        let team_pol = def.team.approval_policy.as_ref().unwrap();
        assert_eq!(team_pol.mode, ApprovalMode::Automatic);
        assert_eq!(
            team_pol.evaluate_command("cargo test"),
            ApprovalDecision::Approved
        );
        assert_eq!(
            team_pol.evaluate_command("rm -rf /"),
            ApprovalDecision::Rejected
        );
        assert!(def.team.graph_topology.is_some());
    }

    #[test]
    fn test_compute_sha256() {
        let hash = compute_sha256("test-content");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_approval_policy_evaluation() {
        let policy = ApprovalPolicy {
            mode: ApprovalMode::Automatic,
            auto_approve: vec![AutoApprovalRule {
                rule: "safe_test_commands".to_string(),
                condition: None,
                tools: vec!["codex".to_string()],
                patterns: vec!["cargo test*".to_string(), "git status".to_string()],
            }],
            auto_reject: vec![AutoRejectRule {
                rule: "forbidden_destructive".to_string(),
                condition: None,
                patterns: vec!["rm -rf /".to_string(), "*id_rsa*".to_string()],
            }],
            fallback_action: FallbackAction::Reject,
        };

        // Auto-approve matches
        assert_eq!(
            policy.evaluate_command("cargo test --workspace"),
            ApprovalDecision::Approved
        );
        assert_eq!(
            policy.evaluate_command("git status"),
            ApprovalDecision::Approved
        );

        // Auto-reject matches
        assert_eq!(
            policy.evaluate_command("rm -rf /"),
            ApprovalDecision::Rejected
        );
        assert_eq!(
            policy.evaluate_command("cat ~/.ssh/id_rsa"),
            ApprovalDecision::Rejected
        );

        // Fallback for unlisted command in Automatic mode
        assert_eq!(
            policy.evaluate_command("python3 script.py"),
            ApprovalDecision::Rejected
        );
    }

    #[test]
    fn test_approval_policy_manual_and_hybrid_mode() {
        let mut policy = ApprovalPolicy {
            mode: ApprovalMode::Manual,
            auto_approve: vec![],
            auto_reject: vec![],
            fallback_action: FallbackAction::EscalateToHuman,
        };
        // In manual mode, always escalates to human
        assert_eq!(
            policy.evaluate_command("cargo test"),
            ApprovalDecision::NeedsHuman
        );

        // In hybrid mode with escalate_to_human fallback
        policy.mode = ApprovalMode::Hybrid;
        policy.auto_approve.push(AutoApprovalRule {
            rule: "approved_tests".to_string(),
            condition: None,
            tools: vec![],
            patterns: vec!["cargo test*".to_string()],
        });
        assert_eq!(
            policy.evaluate_command("cargo test"),
            ApprovalDecision::Approved
        );
        assert_eq!(
            policy.evaluate_command("unknown_tool --do-stuff"),
            ApprovalDecision::NeedsHuman
        );
    }
}
