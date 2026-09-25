//! Sandbox domain types, configurations, network policies and execution results.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SandboxId(pub String);

impl std::fmt::Display for SandboxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<String>> From<T> for SandboxId {
    fn from(s: T) -> Self {
        SandboxId(s.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SnapshotId(pub String);

impl std::fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<String>> From<T> for SnapshotId {
    fn from(s: T) -> Self {
        SnapshotId(s.into())
    }
}

/// Network policy controlling sandbox external connectivity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NetworkMode {
    /// Zero network access (--unshare-net, only loopback interface lo).
    #[default]
    Isolated,
    /// Egress traffic restricted strictly to an allowlist of domains/IPs.
    EgressAllowlist(Vec<String>),
    /// Outbound tools (search, docs) mediated by the supervisor on the host.
    HostMediated,
}

/// Filesystem workspace lifecycle mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkspaceMode {
    /// In-memory ephemeral tmpfs (automatically discarded on teardown).
    #[default]
    EphemeralTmpfs,
    /// Ephemeral copy-on-write workspace in /tmp (snapshot and promote capable).
    EphemeralCopyOnWrite(PathBuf),
    /// Direct host path (use with caution, only for unisolated debugging).
    Direct(PathBuf),
}

/// Configuration settings for instantiating a Sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub provider: String,
    pub network: NetworkMode,
    pub workspace: WorkspaceMode,
    pub root_readonly: bool,
    pub default_timeout: Duration,
    pub env_passthrough: Vec<String>,
    pub memory_limit_mb: Option<u64>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            provider: "bwrap".to_string(),
            network: NetworkMode::Isolated,
            workspace: WorkspaceMode::EphemeralTmpfs,
            root_readonly: true,
            default_timeout: Duration::from_secs(30),
            env_passthrough: vec![
                "PATH".to_string(),
                "LANG".to_string(),
                "LC_ALL".to_string(),
                "TERM".to_string(),
            ],
            memory_limit_mb: Some(4096),
        }
    }
}

/// Result of executing a command inside the sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
}

impl ExecutionResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }
}

/// Type of filesystem modification detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
}

/// Summary of a file modified within the sandbox workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChangeSummary {
    pub relative_path: PathBuf,
    pub change_type: FileChangeType,
    pub bytes: usize,
    pub content_hash: String,
}
