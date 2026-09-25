//! Secret masking and sanitization mechanism to prevent credential leaks.

use crate::events::RuntimeEvent;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Metadata of a secret tracked by the system without revealing its cleartext value.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SecretMetadata {
    pub secret_id: String,
    pub value_logged: bool,
    pub sha256_fingerprint: String,
}

impl SecretMetadata {
    pub fn new(secret_id: impl Into<String>, secret_value: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(secret_value.as_bytes());
        let hash = hex::encode(hasher.finalize());
        Self {
            secret_id: secret_id.into(),
            value_logged: false,
            sha256_fingerprint: hash,
        }
    }
}

/// SecretMasker finds and redacts sensitive patterns and registered credentials.
#[derive(Debug, Clone, Default)]
pub struct SecretMasker {
    registered_secrets: HashMap<String, String>, // secret_id -> cleartext
}

impl SecretMasker {
    pub fn new() -> Self {
        Self {
            registered_secrets: HashMap::new(),
        }
    }

    /// Registers a sensitive key/value pair to be scrubbed.
    pub fn register_secret(&mut self, secret_id: impl Into<String>, secret_value: impl Into<String>) {
        let val = secret_value.into();
        if !val.trim().is_empty() {
            self.registered_secrets.insert(secret_id.into(), val);
        }
    }

    /// Sanitizes a string by replacing registered secrets and well-known token patterns.
    pub fn mask_str(&self, input: &str) -> String {
        let mut output = input.to_string();

        // 1. Mask explicitly registered secrets
        for (id, val) in &self.registered_secrets {
            if output.contains(val) {
                output = output.replace(val, &format!("[REDACTED:{}]", id));
            }
        }

        // 2. Mask OpenAI-style keys (sk-...)
        output = mask_pattern(&output, "sk-", 20);

        // 3. Mask GitHub tokens (ghp_..., gho_...)
        output = mask_pattern(&output, "ghp_", 20);
        output = mask_pattern(&output, "gho_", 20);

        // 4. Mask GitLab tokens (glpat-...)
        output = mask_pattern(&output, "glpat-", 20);

        output
    }

    /// Sanitizes all string values within a serde_json::Value in-place.
    pub fn mask_json(&self, val: &mut serde_json::Value) {
        match val {
            serde_json::Value::String(s) => {
                *s = self.mask_str(s);
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.mask_json(item);
                }
            }
            serde_json::Value::Object(map) => {
                for (_, v) in map {
                    self.mask_json(v);
                }
            }
            _ => {}
        }
    }

    /// Sanitizes an event before it is persisted or broadcasted.
    pub fn mask_event(&self, event: &mut RuntimeEvent) {
        match event {
            RuntimeEvent::RunInitiated { prompt, .. } => {
                *prompt = self.mask_str(prompt);
            }
            RuntimeEvent::RunFailed { error, .. } => {
                *error = self.mask_str(error);
            }
            RuntimeEvent::AgentFailed { error, .. } => {
                *error = self.mask_str(error);
            }
            RuntimeEvent::AgentFinished { summary, .. } => {
                if let Some(s) = summary {
                    *s = self.mask_str(s);
                }
            }
            RuntimeEvent::CommandExecuted {
                command,
                args,
                stdout_preview,
                stderr_preview,
                ..
            } => {
                *command = self.mask_str(command);
                for arg in args {
                    *arg = self.mask_str(arg);
                }
                if let Some(s) = stdout_preview {
                    *s = self.mask_str(s);
                }
                if let Some(s) = stderr_preview {
                    *s = self.mask_str(s);
                }
            }
            RuntimeEvent::ToolCalled { input, output, .. } => {
                self.mask_json(input);
                if let Some(out) = output {
                    self.mask_json(out);
                }
            }
            RuntimeEvent::ApprovalRequired { prompt, .. } => {
                *prompt = self.mask_str(prompt);
            }
            _ => {}
        }
    }
}

fn mask_pattern(text: &str, prefix: &str, min_len: usize) -> String {
    let mut result = String::new();
    let mut rest = text;

    while let Some(start_idx) = rest.find(prefix) {
        result.push_str(&rest[..start_idx]);
        let token_slice = &rest[start_idx..];
        // find token boundary (whitespace, quotes, punctuation)
        let end_idx = token_slice
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == ';' || c == ')' || c == '}')
            .unwrap_or(token_slice.len());

        let candidate = &token_slice[..end_idx];
        if candidate.len() >= prefix.len() + min_len {
            result.push_str("[REDACTED_API_KEY]");
        } else {
            result.push_str(candidate);
        }

        rest = &token_slice[end_idx..];
    }

    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_metadata_hashing() {
        let meta = SecretMetadata::new("github_token", "ghp_1234567890abcdef1234567890abcdef");
        assert_eq!(meta.secret_id, "github_token");
        assert!(!meta.value_logged);
        assert!(!meta.sha256_fingerprint.is_empty());
        assert_ne!(meta.sha256_fingerprint, "ghp_1234567890abcdef1234567890abcdef");
    }

    #[test]
    fn test_mask_openai_key() {
        let masker = SecretMasker::new();
        let raw = "Connecting to API with sk-proj-1234567890abcdefghijklmnopqrstuv and finished.";
        let masked = masker.mask_str(raw);
        assert!(!masked.contains("sk-proj-1234567890abcdefghijklmnopqrstuv"));
        assert!(masked.contains("[REDACTED_API_KEY]"));
    }

    #[test]
    fn test_mask_registered_secrets() {
        let mut masker = SecretMasker::new();
        masker.register_secret("db_pass", "SuperSecretPassword123!");

        let raw = "sqlite://user:SuperSecretPassword123!@localhost/db";
        let masked = masker.mask_str(raw);
        assert_eq!(masked, "sqlite://user:[REDACTED:db_pass]@localhost/db");
    }

    #[test]
    fn test_mask_event_command_executed() {
        let mut masker = SecretMasker::new();
        masker.register_secret("hf_token", "hf_xyz9876543210token");

        let mut event = RuntimeEvent::CommandExecuted {
            run_id: "run-001".to_string(),
            task_id: Some("t-1".to_string()),
            agent_name: "codex".to_string(),
            command: "curl".to_string(),
            args: vec!["-H".to_string(), "Authorization: Bearer hf_xyz9876543210token".to_string()],
            exit_code: 0,
            duration_ms: 120,
            stdout_preview: Some("Using token sk-proj-abcdef1234567890987654321".to_string()),
            stderr_preview: None,
            sandbox_id: None,
        };

        masker.mask_event(&mut event);

        if let RuntimeEvent::CommandExecuted { args, stdout_preview, .. } = event {
            assert!(args[1].contains("[REDACTED:hf_token]"));
            assert!(!stdout_preview.unwrap().contains("sk-proj-"));
        } else {
            panic!("Unexpected event type");
        }
    }
}
