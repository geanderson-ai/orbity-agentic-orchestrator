//! Sandbox errors.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("I/O error in sandbox: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Sandbox runtime not found: {0}")]
    RuntimeNotFound(String),

    #[error("Command execution timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Policy denied: {action} on {resource}: {reason}")]
    PolicyDenied {
        action: String,
        resource: String,
        reason: String,
    },

    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    #[error("Rollback error: {0}")]
    RollbackError(String),

    #[error("Promotion error: {0}")]
    PromoteError(String),

    #[error("Initialization error: {0}")]
    InitError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
