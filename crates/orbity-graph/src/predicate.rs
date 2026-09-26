//! Dynamic conditional edge predicates and evaluation against Blackboard.

use crate::blackboard::Blackboard;
use crate::types::NodeOutput;

/// Evaluates conditional expressions for edge transitions.
pub struct ConditionalEvaluator;

impl ConditionalEvaluator {
    /// Evaluates a predicate string against the node's output, the blackboard, and optional loop key.
    pub async fn evaluate(
        predicate: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
        loop_key: Option<&str>,
    ) -> bool {
        let p = predicate.trim();
        if p.is_empty() || p == "true" || p == "always" {
            return true;
        }
        if p == "false" || p == "never" {
            return false;
        }

        Self::eval_or_expr(p, node_output, blackboard, loop_key).await
    }

    async fn eval_or_expr(
        expr: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
        loop_key: Option<&str>,
    ) -> bool {
        let terms = split_top_level(expr, &["||", " or "]);
        if terms.len() > 1 {
            for term in terms {
                if Box::pin(Self::eval_and_expr(term.trim(), node_output, blackboard, loop_key)).await {
                    return true;
                }
            }
            return false;
        }
        Self::eval_and_expr(expr, node_output, blackboard, loop_key).await
    }

    async fn eval_and_expr(
        expr: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
        loop_key: Option<&str>,
    ) -> bool {
        let terms = split_top_level(expr, &["&&", " and "]);
        if terms.len() > 1 {
            for term in terms {
                if !Box::pin(Self::eval_unary_or_atomic(term.trim(), node_output, blackboard, loop_key)).await {
                    return false;
                }
            }
            return true;
        }
        Self::eval_unary_or_atomic(expr, node_output, blackboard, loop_key).await
    }

    async fn eval_unary_or_atomic(
        expr: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
        loop_key: Option<&str>,
    ) -> bool {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return true;
        }

        // Parentheses
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let inner = &trimmed[1..trimmed.len() - 1];
            if is_balanced(inner) {
                return Box::pin(Self::eval_or_expr(inner, node_output, blackboard, loop_key)).await;
            }
        }

        // Negation: ! or not
        if let Some(rest) = trimmed.strip_prefix('!') {
            return !Box::pin(Self::eval_unary_or_atomic(rest.trim(), node_output, blackboard, loop_key)).await;
        }
        if let Some(rest) = trimmed.strip_prefix("not ") {
            return !Box::pin(Self::eval_unary_or_atomic(rest.trim(), node_output, blackboard, loop_key)).await;
        }

        // Method calls: .contains(...)
        if let Some(pos) = trimmed.find(".contains(") {
            let target_prop = trimmed[..pos].trim();
            let args_part = &trimmed[pos + 10..];
            if let Some(closing) = args_part.rfind(')') {
                let pattern = args_part[..closing].trim().trim_matches(|c| c == '\'' || c == '"');
                let text = Self::resolve_str_value(target_prop, node_output, blackboard).await;
                return text.contains(pattern);
            }
        }

        // Comparison operators: ==, !=, <=, >=, <, >
        let ops = ["==", "!=", "<=", ">=", "<", ">"];
        for op in &ops {
            if let Some((lhs, rhs)) = split_comparison(trimmed, op) {
                return Self::compare_values(lhs.trim(), *op, rhs.trim(), node_output, blackboard, loop_key).await;
            }
        }

        // Atomic truthy checks
        match trimmed {
            "true" | "always" => true,
            "false" | "never" => false,
            "success" | "outcome == 'success'" => node_output.success,
            "failed" | "outcome == 'failed'" => !node_output.success,
            _ => {
                // If it resolves to a boolean string or non-empty string
                let resolved = Self::resolve_str_value(trimmed, node_output, blackboard).await;
                resolved == "true" || (!resolved.is_empty() && resolved != "false" && resolved != "0")
            }
        }
    }

    async fn compare_values(
        lhs_raw: &str,
        op: &str,
        rhs_raw: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
        loop_key: Option<&str>,
    ) -> bool {
        let rhs_clean = rhs_raw.trim_matches(|c| c == '\'' || c == '"');

        // Numeric comparison
        let lhs_num = Self::resolve_num_value(lhs_raw, node_output, blackboard, loop_key).await;
        let rhs_num = rhs_raw.trim().parse::<f64>().ok();

        if let (Some(l), Some(r)) = (lhs_num, rhs_num) {
            return match op {
                "==" => (l - r).abs() < f64::EPSILON,
                "!=" => (l - r).abs() >= f64::EPSILON,
                "<" => l < r,
                "<=" => l <= r,
                ">" => l > r,
                ">=" => l >= r,
                _ => false,
            };
        }

        // String / enum comparison
        let lhs_str = Self::resolve_str_value(lhs_raw, node_output, blackboard).await;
        let rhs_str = if rhs_raw.starts_with('\'') || rhs_raw.starts_with('"') {
            rhs_clean.to_string()
        } else {
            Self::resolve_str_value(rhs_raw, node_output, blackboard).await
        };

        match op {
            "==" => lhs_str == rhs_str,
            "!=" => lhs_str != rhs_str,
            "<" => lhs_str < rhs_str,
            "<=" => lhs_str <= rhs_str,
            ">" => lhs_str > rhs_str,
            ">=" => lhs_str >= rhs_str,
            _ => false,
        }
    }

    async fn resolve_num_value(
        prop: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
        loop_key: Option<&str>,
    ) -> Option<f64> {
        let trimmed = prop.trim();
        if let Ok(n) = trimmed.parse::<f64>() {
            return Some(n);
        }

        if trimmed == "exit_code" || trimmed == "outcome.exit_code" {
            return node_output.exit_code.map(|c| c as f64);
        }

        if trimmed == "loop_count" || trimmed == "loop_iterations" {
            if let Some(key) = loop_key {
                let count = blackboard.get_loop_count(key).await;
                return Some(count as f64);
            }
        }

        if let Some(node_id) = trimmed.strip_suffix(".exit_code") {
            if let Some(val) = blackboard.get_node_data(node_id).await {
                if let Some(c) = val.get("exit_code").and_then(|v| v.as_i64()) {
                    return Some(c as f64);
                }
            }
        }

        if let Some(ctx_key) = trimmed.strip_prefix("context.") {
            if let Some(val) = blackboard.get_context(ctx_key).await {
                if let Some(n) = val.as_f64() {
                    return Some(n);
                }
            }
        }

        None
    }

    async fn resolve_str_value(
        prop: &str,
        node_output: &NodeOutput,
        blackboard: &Blackboard,
    ) -> String {
        let trimmed = prop.trim();
        if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('"') && trimmed.ends_with('"'))
        {
            return trimmed[1..trimmed.len() - 1].to_string();
        }

        match trimmed {
            "stdout" | "outcome.stdout" => node_output.stdout.clone(),
            "stderr" | "outcome.stderr" => node_output.stderr.clone(),
            "status" | "outcome.status" => if node_output.success { "success".to_string() } else { "failed".to_string() },
            "outcome" => if node_output.success { "success".to_string() } else { "failed".to_string() },
            "approval.status" | "approval" => {
                if let Some(val) = blackboard.get_context("approval_status").await {
                    val.as_str().unwrap_or("").to_string()
                } else {
                    "approved".to_string()
                }
            }
            _ => {
                if let Some(ctx_key) = trimmed.strip_prefix("context.") {
                    if let Some(val) = blackboard.get_context(ctx_key).await {
                        if let Some(s) = val.as_str() {
                            return s.to_string();
                        }
                        return val.to_string();
                    }
                }

                if let Some(node_id) = trimmed.strip_suffix(".stdout") {
                    if let Some(out) = blackboard.get_node_output(node_id).await {
                        return out;
                    }
                }

                if let Some(node_id) = trimmed.strip_suffix(".status") {
                    if let Some(out) = blackboard.get_node_output(node_id).await {
                        return if out.is_empty() { "unknown".to_string() } else { "passed".to_string() };
                    }
                }

                // Node output fallback
                if let Some(out) = blackboard.get_node_output(trimmed).await {
                    return out;
                }

                trimmed.to_string()
            }
        }
    }
}

/// Splits string at top-level delimiter, ignoring delimiters inside quotes or parentheses.
fn split_top_level<'a>(input: &'a str, delimiters: &[&str]) -> Vec<&'a str> {
    let mut results = Vec::new();
    let mut paren_depth = 0;
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut last_idx = 0;

    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut i = 0;

    while i < chars.len() {
        let (idx, c) = chars[i];

        if (c == '\'' || c == '"') && paren_depth >= 0 {
            if !in_quote {
                in_quote = true;
                quote_char = c;
            } else if c == quote_char {
                in_quote = false;
            }
        } else if !in_quote {
            if c == '(' {
                paren_depth += 1;
            } else if c == ')' && paren_depth > 0 {
                paren_depth -= 1;
            } else if paren_depth == 0 {
                for delim in delimiters {
                    if input[idx..].starts_with(delim) {
                        results.push(&input[last_idx..idx]);
                        last_idx = idx + delim.len();
                        i += delim.chars().count().saturating_sub(1);
                        break;
                    }
                }
            }
        }
        i += 1;
    }

    results.push(&input[last_idx..]);
    results
}

fn split_comparison<'a>(input: &'a str, op: &str) -> Option<(&'a str, &'a str)> {
    let mut paren_depth = 0;
    let mut in_quote = false;
    let mut quote_char = ' ';

    let chars: Vec<(usize, char)> = input.char_indices().collect();
    for (idx, c) in chars {
        if c == '\'' || c == '"' {
            if !in_quote {
                in_quote = true;
                quote_char = c;
            } else if c == quote_char {
                in_quote = false;
            }
        } else if !in_quote {
            if c == '(' {
                paren_depth += 1;
            } else if c == ')' && paren_depth > 0 {
                paren_depth -= 1;
            } else if paren_depth == 0 && input[idx..].starts_with(op) {
                // Ensure not part of longer operator like == when checking <
                if op == "<" && input[idx..].starts_with("<=") {
                    continue;
                }
                if op == ">" && input[idx..].starts_with(">=") {
                    continue;
                }
                if op == "=" && input[idx..].starts_with("==") {
                    continue;
                }
                let lhs = &input[..idx];
                let rhs = &input[idx + op.len()..];
                return Some((lhs, rhs));
            }
        }
    }
    None
}

fn is_balanced(s: &str) -> bool {
    let mut depth = 0;
    let mut in_quote = false;
    let mut quote_char = ' ';

    for c in s.chars() {
        if c == '\'' || c == '"' {
            if !in_quote {
                in_quote = true;
                quote_char = c;
            } else if c == quote_char {
                in_quote = false;
            }
        } else if !in_quote {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_composite_predicates() {
        let bb = Blackboard::new();
        bb.set_context("env", serde_json::Value::String("production".to_string())).await;
        let out = NodeOutput {
            success: true,
            exit_code: Some(0),
            stdout: "ALL TESTS PASSED: 42 ok".to_string(),
            stderr: String::new(),
            ..Default::default()
        };

        // 1. Simple exit_code
        assert!(ConditionalEvaluator::evaluate("exit_code == 0", &out, &bb, None).await);
        assert!(!ConditionalEvaluator::evaluate("exit_code != 0", &out, &bb, None).await);

        // 2. Compound && and ||
        assert!(ConditionalEvaluator::evaluate("exit_code == 0 && stdout.contains('PASSED')", &out, &bb, None).await);
        assert!(ConditionalEvaluator::evaluate("exit_code == 1 || stdout.contains('42 ok')", &out, &bb, None).await);
        assert!(!ConditionalEvaluator::evaluate("exit_code == 1 && stdout.contains('PASSED')", &out, &bb, None).await);

        // 3. Parentheses & context
        assert!(ConditionalEvaluator::evaluate("(exit_code == 0 || exit_code == 2) && context.env == 'production'", &out, &bb, None).await);

        // 4. Negation
        assert!(ConditionalEvaluator::evaluate("!stdout.contains('ERROR')", &out, &bb, None).await);
    }
}
