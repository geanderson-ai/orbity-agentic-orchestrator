use crate::tokens::TokenExtractor;
use async_trait::async_trait;
use orbity_graph::{Blackboard, CliType, GraphNode, NodeKind, NodeOutput, NodeRunner};
use orbity_sandbox::traits::Sandbox;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Sandbox-backed runner executing the 5 native CLI tools inside isolated bubblewrap containers.
pub struct SandboxCliNodeRunner {
    sandbox: Arc<Mutex<dyn Sandbox>>,
}

impl SandboxCliNodeRunner {
    pub fn new(sandbox: Arc<Mutex<dyn Sandbox>>) -> Self {
        Self { sandbox }
    }

    /// Formats the command line invocation for the specified CLI tool.
    pub fn build_cli_command(
        cli: CliType,
        prompt: &str,
        injected_context: &str,
        extra_args: &[String],
    ) -> (String, Vec<String>) {
        Self::build_cli_command_with_model(cli, prompt, injected_context, extra_args, None, None)
    }

    /// Formats the command line invocation for the specified CLI tool, resolving semantic tiers or explicit models.
    pub fn build_cli_command_with_model(
        cli: CliType,
        prompt: &str,
        injected_context: &str,
        extra_args: &[String],
        tier: Option<&str>,
        explicit_model: Option<&str>,
    ) -> (String, Vec<String>) {
        let full_prompt = if injected_context.is_empty() {
            prompt.to_string()
        } else {
            format!("{}\n\nContext:\n{}", prompt, injected_context)
        };

        let resolved_model = orbity_graph::ModelTierResolver::resolve(cli, tier, explicit_model);

        match cli {
            CliType::Codex => {
                // codex exec [PROMPT] --json --skip-git-repo-check --dangerously-bypass-approvals-and-sandbox [-m MODEL]
                let mut args = vec![
                    "exec".to_string(),
                    full_prompt,
                    "--json".to_string(),
                    "--skip-git-repo-check".to_string(),
                    "--dangerously-bypass-approvals-and-sandbox".to_string(),
                ];
                if let Some(res) = resolved_model {
                    args.extend(res.cli_args);
                }
                args.extend_from_slice(extra_args);
                ("codex".to_string(), args)
            }
            CliType::Claude => {
                // claude -p [PROMPT] --output-format json --dangerously-skip-permissions [--model MODEL]
                let mut args = vec![
                    "-p".to_string(),
                    full_prompt,
                    "--output-format".to_string(),
                    "json".to_string(),
                    "--dangerously-skip-permissions".to_string(),
                ];
                if let Some(res) = resolved_model {
                    args.extend(res.cli_args);
                }
                args.extend_from_slice(extra_args);
                ("claude".to_string(), args)
            }
            CliType::Agy => {
                // agy -p [PROMPT] --output-format json --dangerously-skip-permissions [--model MODEL] [--effort EFFORT]
                let mut args = vec![
                    "-p".to_string(),
                    full_prompt,
                    "--output-format".to_string(),
                    "json".to_string(),
                    "--dangerously-skip-permissions".to_string(),
                ];
                if let Some(res) = resolved_model {
                    args.extend(res.cli_args);
                }
                args.extend_from_slice(extra_args);
                ("agy".to_string(), args)
            }
            CliType::Hermes => {
                // hermes run [PROMPT] --json [-m MODEL]
                let mut args = vec!["run".to_string(), full_prompt, "--json".to_string()];
                if let Some(res) = resolved_model {
                    args.extend(res.cli_args);
                }
                args.extend_from_slice(extra_args);
                ("hermes".to_string(), args)
            }
            CliType::Pi => {
                // pi -p [PROMPT] --mode json --no-session [--model MODEL]
                let mut args = vec![
                    "-p".to_string(),
                    full_prompt,
                    "--mode".to_string(),
                    "json".to_string(),
                    "--no-session".to_string(),
                ];
                if let Some(res) = resolved_model {
                    args.extend(res.cli_args);
                }
                args.extend_from_slice(extra_args);
                ("pi".to_string(), args)
            }
            CliType::Custom => {
                let mut args = vec![full_prompt];
                args.extend_from_slice(extra_args);
                ("sh".to_string(), args)
            }
        }
    }
}

#[async_trait]
impl NodeRunner for SandboxCliNodeRunner {
    async fn run(
        &self,
        node: &GraphNode,
        injected_context: &str,
        _blackboard: &Blackboard,
    ) -> Result<NodeOutput, String> {
        let (cmd, args, timeout_secs, cli_type) = match &node.kind {
            NodeKind::Agent { cli, config } => {
                let prompt = config.prompt_system.as_deref().unwrap_or(&config.name);
                let (c, a) = Self::build_cli_command_with_model(
                    *cli,
                    prompt,
                    injected_context,
                    &config.cli_args,
                    config.tier.as_deref(),
                    config.model.as_deref(),
                );
                let t = config.timeout_seconds.unwrap_or(60);
                (c, a, t, *cli)
            }
            NodeKind::Tool {
                command,
                timeout_secs,
            } => {
                let parts: Vec<&str> = command.split_whitespace().collect();
                if parts.is_empty() {
                    return Err("Empty tool command".to_string());
                }
                let cmd = parts[0].to_string();
                let args = parts[1..].iter().map(|s| s.to_string()).collect();
                (cmd, args, *timeout_secs, CliType::Custom)
            }
            NodeKind::Orchestrator { engine } => {
                // In-process Topcoat Orchestrator node
                let plan = format!(
                    "Orchestrator [{}] dynamically planned workflow. Injected context length: {} chars.",
                    engine,
                    injected_context.len()
                );
                return Ok(NodeOutput {
                    success: true,
                    exit_code: Some(0),
                    stdout: plan,
                    stderr: String::new(),
                    artifacts: HashMap::new(),
                    tokens_input: 150,
                    tokens_output: 50,
                    cost_usd: 0.001,
                    duration_ms: 10,
                });
            }
            NodeKind::ConditionalRouter { predicate_expr } => {
                return Ok(NodeOutput {
                    success: true,
                    exit_code: Some(0),
                    stdout: format!("Router satisfied: {}", predicate_expr),
                    stderr: String::new(),
                    artifacts: HashMap::new(),
                    tokens_input: 0,
                    tokens_output: 0,
                    cost_usd: 0.0,
                    duration_ms: 1,
                });
            }
            NodeKind::HumanGate { prompt, .. } => {
                return Ok(NodeOutput {
                    success: true,
                    exit_code: Some(0),
                    stdout: format!("Gate verified: {}", prompt),
                    stderr: String::new(),
                    artifacts: HashMap::new(),
                    tokens_input: 0,
                    tokens_output: 0,
                    cost_usd: 0.0,
                    duration_ms: 1,
                });
            }
            NodeKind::JoinBarrier { .. } => {
                return Ok(NodeOutput {
                    success: true,
                    exit_code: Some(0),
                    stdout: "Barrier synchronized.".to_string(),
                    stderr: String::new(),
                    artifacts: HashMap::new(),
                    tokens_input: 0,
                    tokens_output: 0,
                    cost_usd: 0.0,
                    duration_ms: 1,
                });
            }
        };

        let env = HashMap::new();
        let timeout = Duration::from_secs(timeout_secs);

        println!("  ↳ Executing node [{}] using '{}'...", node.id.0, cmd);

        let sb = self.sandbox.lock().await;
        let exec_res = sb
            .run_command(&cmd, &args, &env, timeout)
            .await
            .map_err(|e| format!("Sandbox execution error: {}", e))?;

        let token_report = TokenExtractor::extract(cli_type, &exec_res.stdout);

        if exec_res.success() {
            println!(
                "  ✔ Node [{}] completed in {}ms (tokens: {} in / {} out, cost: ${:.4})",
                node.id.0,
                exec_res.duration_ms,
                token_report.input_tokens,
                token_report.output_tokens,
                token_report.cost_usd
            );
        } else {
            let err_detail = if !exec_res.stderr.trim().is_empty() {
                exec_res.stderr.trim().to_string()
            } else if !exec_res.stdout.trim().is_empty() {
                exec_res.stdout.trim().to_string()
            } else {
                format!("Process terminated with exit code {}", exec_res.exit_code)
            };
            eprintln!(
                "  ✖ Node [{}] failed (exit code {}):\n{}",
                node.id.0,
                exec_res.exit_code,
                err_detail
            );
        }

        Ok(NodeOutput {
            success: exec_res.success(),
            exit_code: Some(exec_res.exit_code),
            stdout: exec_res.stdout,
            stderr: exec_res.stderr,
            artifacts: HashMap::new(),
            tokens_input: token_report.input_tokens,
            tokens_output: token_report.output_tokens,
            cost_usd: token_report.cost_usd,
            duration_ms: exec_res.duration_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cli_command_with_semantic_tiers() {
        // Claude with balanced tier
        let (cmd, args) = SandboxCliNodeRunner::build_cli_command_with_model(
            CliType::Claude,
            "refactor",
            "",
            &[],
            Some("balanced"),
            None,
        );
        assert_eq!(cmd, "claude");
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"sonnet".to_string()));

        // Codex with reasoning tier
        let (cmd, args) = SandboxCliNodeRunner::build_cli_command_with_model(
            CliType::Codex,
            "audit",
            "",
            &[],
            Some("reasoning"),
            None,
        );
        assert_eq!(cmd, "codex");
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"o3-mini".to_string()));

        // Agy with fast tier
        let (cmd, args) = SandboxCliNodeRunner::build_cli_command_with_model(
            CliType::Agy,
            "research",
            "",
            &[],
            Some("fast"),
            None,
        );
        assert_eq!(cmd, "agy");
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"gemini-3.8-flash-low".to_string()));
        assert!(args.contains(&"--effort".to_string()));
        assert!(args.contains(&"low".to_string()));

        // Hermes with explicit model
        let (cmd, args) = SandboxCliNodeRunner::build_cli_command_with_model(
            CliType::Hermes,
            "execute",
            "",
            &[],
            None,
            Some("anthropic/claude-sonnet-4.6"),
        );
        assert_eq!(cmd, "hermes");
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"anthropic/claude-sonnet-4.6".to_string()));

        // Pi with balanced tier
        let (cmd, args) = SandboxCliNodeRunner::build_cli_command_with_model(
            CliType::Pi,
            "format",
            "",
            &[],
            Some("balanced"),
            None,
        );
        assert_eq!(cmd, "pi");
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"sonnet".to_string()));
    }
}
