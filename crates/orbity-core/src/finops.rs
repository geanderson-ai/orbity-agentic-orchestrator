//! FinOps and Tokenomics domain models and budget policy enforcement.

use serde::{Deserialize, Serialize};

/// Detailed token usage metrics for an agent, task, or run.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost_usd: Option<f64>,
}

impl TokenUsage {
    pub fn new(
        input_tokens: u64,
        output_tokens: u64,
        cached_tokens: u64,
        reasoning_tokens: u64,
        estimated_cost_usd: Option<f64>,
    ) -> Self {
        let total_tokens = input_tokens + output_tokens + cached_tokens + reasoning_tokens;
        Self {
            input_tokens,
            output_tokens,
            cached_tokens,
            reasoning_tokens,
            total_tokens,
            estimated_cost_usd,
        }
    }

    /// Accumulates another token usage into this one.
    pub fn accumulate(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cached_tokens += other.cached_tokens;
        self.reasoning_tokens += other.reasoning_tokens;
        self.total_tokens =
            self.input_tokens + self.output_tokens + self.cached_tokens + self.reasoning_tokens;

        match (self.estimated_cost_usd, other.estimated_cost_usd) {
            (Some(c1), Some(c2)) => self.estimated_cost_usd = Some(c1 + c2),
            (None, Some(c2)) => self.estimated_cost_usd = Some(c2),
            (Some(c1), None) => self.estimated_cost_usd = Some(c1),
            (None, None) => self.estimated_cost_usd = None,
        }
    }

    /// Sums two token usages.
    pub fn add(&self, other: &TokenUsage) -> Self {
        let mut copy = *self;
        copy.accumulate(other);
        copy
    }
}

/// Budget policy configuration for teams, runs or agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetPolicy {
    pub max_cost_usd: f64,
    pub max_tokens: u64,
    pub max_agent_calls: usize,
    pub expensive_model_approval_threshold: Option<f64>,
    #[serde(default)]
    pub auto_approve_expensive_models: bool,
    pub alert_at_budget_percentage: Option<f64>,
}

impl Default for BudgetPolicy {
    fn default() -> Self {
        Self {
            max_cost_usd: 5.0,
            max_tokens: 500_000,
            max_agent_calls: 100,
            expensive_model_approval_threshold: Some(1.0),
            auto_approve_expensive_models: false,
            alert_at_budget_percentage: Some(75.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BudgetStatus {
    Ok,
    ThresholdWarning {
        percentage: f64,
        current_usd: f64,
        max_usd: f64,
    },
    ApprovalRequired {
        proposed_cost_usd: f64,
        threshold_usd: f64,
    },
    CostExceeded {
        current_usd: f64,
        max_usd: f64,
    },
    TokensExceeded {
        current_tokens: u64,
        max_tokens: u64,
    },
    CallsExceeded {
        current_calls: usize,
        max_calls: usize,
    },
}

impl BudgetPolicy {
    pub fn new(max_cost_usd: f64, max_tokens: u64, max_agent_calls: usize) -> Self {
        Self {
            max_cost_usd,
            max_tokens,
            max_agent_calls,
            expensive_model_approval_threshold: None,
            auto_approve_expensive_models: false,
            alert_at_budget_percentage: Some(75.0),
        }
    }

    pub fn with_auto_approve_expensive_models(mut self, auto: bool) -> Self {
        self.auto_approve_expensive_models = auto;
        self
    }

    /// Checks if a single expensive model call requires approval.
    pub fn requires_approval(&self, call_cost_usd: f64) -> bool {
        if self.auto_approve_expensive_models {
            return false;
        }
        if let Some(threshold) = self.expensive_model_approval_threshold {
            call_cost_usd >= threshold
        } else {
            false
        }
    }

    /// Evaluates current metrics against the budget policy.
    pub fn check_limits(
        &self,
        current_cost_usd: f64,
        current_tokens: u64,
        current_calls: usize,
    ) -> BudgetStatus {
        if current_cost_usd > self.max_cost_usd {
            return BudgetStatus::CostExceeded {
                current_usd: current_cost_usd,
                max_usd: self.max_cost_usd,
            };
        }

        if current_tokens > self.max_tokens {
            return BudgetStatus::TokensExceeded {
                current_tokens,
                max_tokens: self.max_tokens,
            };
        }

        if current_calls > self.max_agent_calls {
            return BudgetStatus::CallsExceeded {
                current_calls,
                max_calls: self.max_agent_calls,
            };
        }

        if let Some(alert_pct) = self.alert_at_budget_percentage {
            if self.max_cost_usd > 0.0 {
                let pct = (current_cost_usd / self.max_cost_usd) * 100.0;
                if pct >= alert_pct {
                    return BudgetStatus::ThresholdWarning {
                        percentage: pct,
                        current_usd: current_cost_usd,
                        max_usd: self.max_cost_usd,
                    };
                }
            }
        }

        BudgetStatus::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_accumulation() {
        let mut u1 = TokenUsage::new(100, 50, 20, 10, Some(0.005));
        let u2 = TokenUsage::new(200, 100, 0, 5, Some(0.010));

        assert_eq!(u1.total_tokens, 180);
        u1.accumulate(&u2);

        assert_eq!(u1.input_tokens, 300);
        assert_eq!(u1.output_tokens, 150);
        assert_eq!(u1.cached_tokens, 20);
        assert_eq!(u1.reasoning_tokens, 15);
        assert_eq!(u1.total_tokens, 485);
        assert!((u1.estimated_cost_usd.unwrap() - 0.015).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_policy_limits_and_overflow() {
        let policy = BudgetPolicy {
            max_cost_usd: 2.0,
            max_tokens: 10_000,
            max_agent_calls: 5,
            expensive_model_approval_threshold: Some(0.80),
            auto_approve_expensive_models: false,
            alert_at_budget_percentage: Some(75.0),
        };

        // Normal usage
        assert_eq!(policy.check_limits(1.0, 5_000, 3), BudgetStatus::Ok);

        // Threshold warning (75% of $2.0 = $1.50)
        assert_eq!(
            policy.check_limits(1.6, 6_000, 3),
            BudgetStatus::ThresholdWarning {
                percentage: 80.0,
                current_usd: 1.6,
                max_usd: 2.0,
            }
        );

        // Exceeded cost
        assert_eq!(
            policy.check_limits(2.05, 5_000, 3),
            BudgetStatus::CostExceeded {
                current_usd: 2.05,
                max_usd: 2.0,
            }
        );

        // Exceeded tokens
        assert_eq!(
            policy.check_limits(1.0, 12_000, 3),
            BudgetStatus::TokensExceeded {
                current_tokens: 12_000,
                max_tokens: 10_000,
            }
        );

        // Exceeded calls
        assert_eq!(
            policy.check_limits(1.0, 5_000, 6),
            BudgetStatus::CallsExceeded {
                current_calls: 6,
                max_calls: 5,
            }
        );

        // Expensive model approval check
        assert!(!policy.requires_approval(0.50));
        assert!(policy.requires_approval(0.85));

        // When auto_approve_expensive_models is enabled from YAML policy
        let auto_approved_policy = policy.with_auto_approve_expensive_models(true);
        assert!(!auto_approved_policy.requires_approval(0.85));
        assert!(!auto_approved_policy.requires_approval(10.0));
    }
}
