//! Semantic Model Tier resolution for multi-agent CLI engines.
//!
//! Provides deterministic resolution of high-level semantic tiers (`fast`, `balanced`,
//! `reasoning`, `latest`) into canonical CLI arguments and engine-specific flags,
//! preventing YAML definitions from decaying as providers deprecate legacy models.

use crate::types::CliType;
use serde::{Deserialize, Serialize};

/// High-level semantic model tier representing cognitive capability and cost profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    /// High-throughput, cost-efficient model for lightweight tasks, checks, and lints.
    Fast,
    /// Balanced intelligence and performance for routine feature work and refactoring.
    Balanced,
    /// Deep reasoning, advanced architecture analysis, and mission-critical review.
    Reasoning,
    /// Preserves the CLI's native default and floating installation settings.
    Latest,
}

impl ModelTier {
    /// Parses a string into a semantic model tier.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "fast" | "light" | "quick" | "mini" => Some(Self::Fast),
            "balanced" | "standard" | "mid" => Some(Self::Balanced),
            "reasoning" | "deep" | "heavy" | "thinking" => Some(Self::Reasoning),
            "latest" | "default" => Some(Self::Latest),
            _ => None,
        }
    }
}

/// Fully resolved CLI model specification ready for execution in sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedModel {
    /// Canonical model identifier or alias.
    pub model_id: String,
    /// Primary CLI flag name (e.g. `--model` or `-m`).
    pub flag_name: String,
    /// Full CLI arguments to be appended to the command.
    pub cli_args: Vec<String>,
    /// Optional explicit reasoning effort level.
    pub reasoning_effort: Option<String>,
}

pub struct ModelTierResolver;

impl ModelTierResolver {
    /// Resolves a semantic tier or explicit model override for the given CLI type.
    pub fn resolve(
        cli: CliType,
        tier: Option<&str>,
        explicit_model: Option<&str>,
    ) -> Option<ResolvedModel> {
        // 1. Explicit model takes precedence if provided and non-empty
        if let Some(m) = explicit_model.filter(|s| !s.trim().is_empty()) {
            return Some(Self::format_explicit_model(cli, m));
        }

        // 2. Resolve semantic tier if provided
        let tier_val = tier.and_then(ModelTier::from_str_loose)?;

        match (cli, tier_val) {
            // Latest / Default tier: don't inject rigid flags, keep CLI default
            (_, ModelTier::Latest) => None,

            // Claude Code
            (CliType::Claude, ModelTier::Fast) => Some(ResolvedModel {
                model_id: "haiku".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec!["--model".to_string(), "haiku".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Claude, ModelTier::Balanced) => Some(ResolvedModel {
                model_id: "sonnet".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec!["--model".to_string(), "sonnet".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Claude, ModelTier::Reasoning) => Some(ResolvedModel {
                model_id: "opus".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec!["--model".to_string(), "opus".to_string()],
                reasoning_effort: Some("high".to_string()),
            }),

            // OpenAI Codex
            (CliType::Codex, ModelTier::Fast) => Some(ResolvedModel {
                model_id: "gpt-4o-mini".to_string(),
                flag_name: "-m".to_string(),
                cli_args: vec!["-m".to_string(), "gpt-4o-mini".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Codex, ModelTier::Balanced) => Some(ResolvedModel {
                model_id: "gpt-4o".to_string(),
                flag_name: "-m".to_string(),
                cli_args: vec!["-m".to_string(), "gpt-4o".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Codex, ModelTier::Reasoning) => Some(ResolvedModel {
                model_id: "o3-mini".to_string(),
                flag_name: "-m".to_string(),
                cli_args: vec!["-m".to_string(), "o3-mini".to_string()],
                reasoning_effort: Some("high".to_string()),
            }),

            // Antigravity (Agy)
            (CliType::Agy, ModelTier::Fast) => Some(ResolvedModel {
                model_id: "gemini-3.8-flash-low".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec![
                    "--model".to_string(),
                    "gemini-3.8-flash-low".to_string(),
                    "--effort".to_string(),
                    "low".to_string(),
                ],
                reasoning_effort: Some("low".to_string()),
            }),
            (CliType::Agy, ModelTier::Balanced) => Some(ResolvedModel {
                model_id: "gemini-3.8-flash-high".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec![
                    "--model".to_string(),
                    "gemini-3.8-flash-high".to_string(),
                    "--effort".to_string(),
                    "medium".to_string(),
                ],
                reasoning_effort: Some("medium".to_string()),
            }),
            (CliType::Agy, ModelTier::Reasoning) => Some(ResolvedModel {
                model_id: "gemini-3.1-pro-high".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec![
                    "--model".to_string(),
                    "gemini-3.1-pro-high".to_string(),
                    "--effort".to_string(),
                    "high".to_string(),
                ],
                reasoning_effort: Some("high".to_string()),
            }),

            // Hermes Agent
            (CliType::Hermes, ModelTier::Fast) => Some(ResolvedModel {
                model_id: "openrouter/auto-fast".to_string(),
                flag_name: "-m".to_string(),
                cli_args: vec!["-m".to_string(), "openrouter/auto-fast".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Hermes, ModelTier::Balanced) => Some(ResolvedModel {
                model_id: "anthropic/claude-sonnet-4.6".to_string(),
                flag_name: "-m".to_string(),
                cli_args: vec!["-m".to_string(), "anthropic/claude-sonnet-4.6".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Hermes, ModelTier::Reasoning) => Some(ResolvedModel {
                model_id: "anthropic/claude-sonnet-4.6".to_string(),
                flag_name: "-m".to_string(),
                cli_args: vec![
                    "-m".to_string(),
                    "anthropic/claude-sonnet-4.6".to_string(),
                    "--reasoning".to_string(),
                    "high".to_string(),
                ],
                reasoning_effort: Some("high".to_string()),
            }),

            // Pi Coding Agent
            (CliType::Pi, ModelTier::Fast) => Some(ResolvedModel {
                model_id: "llama-cpp".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec!["--model".to_string(), "llama-cpp".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Pi, ModelTier::Balanced) => Some(ResolvedModel {
                model_id: "sonnet".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec!["--model".to_string(), "sonnet".to_string()],
                reasoning_effort: None,
            }),
            (CliType::Pi, ModelTier::Reasoning) => Some(ResolvedModel {
                model_id: "sonnet:high".to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec!["--model".to_string(), "sonnet:high".to_string()],
                reasoning_effort: Some("high".to_string()),
            }),

            // Custom tools
            (CliType::Custom, _) => None,
        }
    }

    fn format_explicit_model(cli: CliType, model: &str) -> ResolvedModel {
        match cli {
            CliType::Codex | CliType::Hermes => ResolvedModel {
                model_id: model.to_string(),
                flag_name: "-m".to_string(),
                cli_args: vec!["-m".to_string(), model.to_string()],
                reasoning_effort: None,
            },
            CliType::Claude | CliType::Agy | CliType::Pi | CliType::Custom => ResolvedModel {
                model_id: model.to_string(),
                flag_name: "--model".to_string(),
                cli_args: vec!["--model".to_string(), model.to_string()],
                reasoning_effort: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_claude_tiers() {
        let fast = ModelTierResolver::resolve(CliType::Claude, Some("fast"), None).unwrap();
        assert_eq!(fast.model_id, "haiku");
        assert_eq!(fast.cli_args, vec!["--model", "haiku"]);

        let balanced = ModelTierResolver::resolve(CliType::Claude, Some("balanced"), None).unwrap();
        assert_eq!(balanced.model_id, "sonnet");
        assert_eq!(balanced.cli_args, vec!["--model", "sonnet"]);

        let reasoning =
            ModelTierResolver::resolve(CliType::Claude, Some("reasoning"), None).unwrap();
        assert_eq!(reasoning.model_id, "opus");
        assert_eq!(reasoning.cli_args, vec!["--model", "opus"]);

        let latest = ModelTierResolver::resolve(CliType::Claude, Some("latest"), None);
        assert!(latest.is_none());
    }

    #[test]
    fn test_resolve_codex_tiers() {
        let fast = ModelTierResolver::resolve(CliType::Codex, Some("fast"), None).unwrap();
        assert_eq!(fast.model_id, "gpt-4o-mini");
        assert_eq!(fast.cli_args, vec!["-m", "gpt-4o-mini"]);

        let reasoning =
            ModelTierResolver::resolve(CliType::Codex, Some("reasoning"), None).unwrap();
        assert_eq!(reasoning.model_id, "o3-mini");
        assert_eq!(reasoning.cli_args, vec!["-m", "o3-mini"]);
    }

    #[test]
    fn test_resolve_agy_tiers() {
        let balanced = ModelTierResolver::resolve(CliType::Agy, Some("balanced"), None).unwrap();
        assert_eq!(balanced.model_id, "gemini-3.8-flash-high");
        assert_eq!(
            balanced.cli_args,
            vec!["--model", "gemini-3.8-flash-high", "--effort", "medium"]
        );
    }

    #[test]
    fn test_resolve_explicit_model_override() {
        let explicit =
            ModelTierResolver::resolve(CliType::Codex, Some("fast"), Some("o3-mini")).unwrap();
        assert_eq!(explicit.model_id, "o3-mini");
        assert_eq!(explicit.cli_args, vec!["-m", "o3-mini"]);

        let explicit_claude =
            ModelTierResolver::resolve(CliType::Claude, None, Some("claude-3-7-sonnet")).unwrap();
        assert_eq!(explicit_claude.model_id, "claude-3-7-sonnet");
        assert_eq!(
            explicit_claude.cli_args,
            vec!["--model", "claude-3-7-sonnet"]
        );
    }
}
