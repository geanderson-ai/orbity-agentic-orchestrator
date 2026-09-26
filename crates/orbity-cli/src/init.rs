use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct WorkspaceInit;

impl WorkspaceInit {
    /// Initializes an Orbity workspace in `target_dir`.
    /// Creates `orbity.yaml`, `teams/dev_team.yaml`, `agents/coder.yaml`, `agents/reviewer.yaml`, and `.gitignore`.
    pub fn init_project(target_dir: &Path, project_name: Option<&str>) -> Result<Vec<PathBuf>, InitError> {
        let name = project_name
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                target_dir
                    .canonicalize()
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                    .filter(|s| !s.is_empty() && s != ".")
                    .unwrap_or_else(|| "orbity-workspace".to_string())
            });

        let mut created_files = Vec::new();

        fs::create_dir_all(target_dir.join("teams"))?;
        fs::create_dir_all(target_dir.join("agents"))?;

        let orbity_yaml_path = target_dir.join("orbity.yaml");
        if !orbity_yaml_path.exists() {
            let config_content = format!(
                r#"# ==============================================================================
# Orbity Workspace Configuration
# ==============================================================================
version: "1.0"
workspace:
  name: "{name}"
  description: "Autonomous multi-agent workspace orchestrated by Orbity"
  database: "orbity.db"

defaults:
  team: "dev_team"
  sandbox: "isolated"
  budget_usd: 5.0
  auto_approve: false
"#
            );
            fs::write(&orbity_yaml_path, config_content)?;
            created_files.push(orbity_yaml_path);
        }

        let dev_team_path = target_dir.join("teams").join("dev_team.yaml");
        if !dev_team_path.exists() {
            let team_content = r#"# ==============================================================================
# Equipe Declarativa: dev_team
# Pipeline: coder (Claude Code) -> reviewer (Codex CLI)
# ==============================================================================
name: "dev_team"
description: "Autonomous software engineering team with implementation and code review"
start_node: "coder"
terminal_nodes:
  - "reviewer"

nodes:
  - id: "coder"
    type: "agent"
    cli: "claude"
    description: "Full-Stack Software Engineer responsible for implementation"
    prompt: "You are an expert software developer. Implement clean, robust, and well-tested code."
    timeout_secs: 300
    budget_usd: 2.00
    retries: 2

  - id: "reviewer"
    type: "agent"
    cli: "codex"
    description: "Security and Quality Code Reviewer"
    prompt: "You are an expert code reviewer and security auditor. Verify code quality, test coverage, and security."
    timeout_secs: 180
    budget_usd: 1.00
    retries: 1

edges:
  - from: "coder"
    to: "reviewer"
    type: "direct"
"#;
            fs::write(&dev_team_path, team_content)?;
            created_files.push(dev_team_path);
        }

        let coder_agent_path = target_dir.join("agents").join("coder.yaml");
        if !coder_agent_path.exists() {
            let coder_content = r#"# ==============================================================================
# Agente Declarativo: coder
# ==============================================================================
agent:
  name: "coder"
  role: "Full-Stack Software Engineer"
  cli: "claude"
  model: "claude-3-7-sonnet"
  system_prompt: "You are an expert software developer. Write clean, modular, and maintainable code with unit tests."
  sandbox:
    mode: "isolated"
    timeout_secs: 300
    allow_network: false
"#;
            fs::write(&coder_agent_path, coder_content)?;
            created_files.push(coder_agent_path);
        }

        let reviewer_agent_path = target_dir.join("agents").join("reviewer.yaml");
        if !reviewer_agent_path.exists() {
            let reviewer_content = r#"# ==============================================================================
# Agente Declarativo: reviewer
# ==============================================================================
agent:
  name: "reviewer"
  role: "Code Reviewer & Security Specialist"
  cli: "codex"
  model: "gpt-4o"
  system_prompt: "You are a code reviewer and security specialist. Inspect changes for correctness, edge cases, and safety."
  sandbox:
    mode: "isolated"
    timeout_secs: 180
    allow_network: false
"#;
            fs::write(&reviewer_agent_path, reviewer_content)?;
            created_files.push(reviewer_agent_path);
        }

        let gitignore_path = target_dir.join(".gitignore");
        if !gitignore_path.exists() {
            let gitignore_content = r#"# Orbity database files
orbity.db
orbity.db-shm
orbity.db-wal

# Orbity cache and temporary files
.orbity/
"#;
            fs::write(&gitignore_path, gitignore_content)?;
            created_files.push(gitignore_path);
        }

        Ok(created_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_workspace_scaffolding() {
        let unique = format!("orbity-test-{}", uuid::Uuid::new_v4());
        let target = std::env::temp_dir().join(unique);

        let created = WorkspaceInit::init_project(&target, Some("alfa")).unwrap();
        assert!(!created.is_empty());
        assert!(target.join("orbity.yaml").exists());
        assert!(target.join("teams/dev_team.yaml").exists());
        assert!(target.join("agents/coder.yaml").exists());
        assert!(target.join("agents/reviewer.yaml").exists());
        assert!(target.join(".gitignore").exists());

        // Second init should not duplicate or error
        let created_again = WorkspaceInit::init_project(&target, Some("alfa")).unwrap();
        assert!(created_again.is_empty());

        let _ = std::fs::remove_dir_all(&target);
    }
}
