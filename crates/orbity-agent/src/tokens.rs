//! Token metrics extractors for CLI agents (Codex, Claude, Agy, Hermes, Pi).

use orbity_core::finops::TokenUsage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliTokenReport {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cached_tokens: u32,
    pub reasoning_tokens: u32,
    pub cost_usd: f64,
}

impl From<CliTokenReport> for TokenUsage {
    fn from(r: CliTokenReport) -> Self {
        TokenUsage::new(
            r.input_tokens as u64,
            r.output_tokens as u64,
            r.cached_tokens as u64,
            r.reasoning_tokens as u64,
            Some(r.cost_usd),
        )
    }
}


pub struct TokenExtractor;

impl TokenExtractor {
    /// Extracts token metrics from JSON output or raw text for the 5 supported CLIs.
    pub fn extract(cli: orbity_graph::CliType, output_text: &str) -> CliTokenReport {
        // Attempt to parse JSON envelope if available
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(output_text) {
            if let Some(usage) = val.get("usage") {
                let input = usage.get("input_tokens").or_else(|| usage.get("prompt_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let output = usage.get("output_tokens").or_else(|| usage.get("completion_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let cached = usage.get("cached_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let reasoning = usage.get("reasoning_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let cost = val.get("cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

                let calc_cost = if cost > 0.0 {
                    cost
                } else {
                    Self::estimate_cost(cli, input, output)
                };

                return CliTokenReport {
                    input_tokens: input,
                    output_tokens: output,
                    cached_tokens: cached,
                    reasoning_tokens: reasoning,
                    cost_usd: calc_cost,
                };
            }
        }

        // Fallback: estimate from text length (heuristics: 4 chars ~ 1 token)
        let chars = output_text.len();
        let estimated_output = (chars / 4).max(1) as u32;
        let estimated_input = 200; // baseline input context
        let cost = Self::estimate_cost(cli, estimated_input, estimated_output);

        CliTokenReport {
            input_tokens: estimated_input,
            output_tokens: estimated_output,
            cached_tokens: 0,
            reasoning_tokens: 0,
            cost_usd: cost,
        }
    }

    /// Estimates cost based on model/CLI tier pricing per 1M tokens.
    pub fn estimate_cost(cli: orbity_graph::CliType, input: u32, output: u32) -> f64 {
        let (in_rate_per_1m, out_rate_per_1m) = match cli {
            orbity_graph::CliType::Codex => (2.50, 10.00),     // Codex / GPT-4o style
            orbity_graph::CliType::Claude => (3.00, 15.00),    // Claude 3.5 Sonnet
            orbity_graph::CliType::Agy => (1.25, 5.00),        // Antigravity Flash / Pro
            orbity_graph::CliType::Hermes => (0.50, 1.50),     // Hermes open weights
            orbity_graph::CliType::Pi => (0.15, 0.60),         // Pi / lightweight assistant
            orbity_graph::CliType::Custom => (1.00, 3.00),
        };

        let in_cost = (input as f64 / 1_000_000.0) * in_rate_per_1m;
        let out_cost = (output as f64 / 1_000_000.0) * out_rate_per_1m;
        in_cost + out_cost
    }
}
