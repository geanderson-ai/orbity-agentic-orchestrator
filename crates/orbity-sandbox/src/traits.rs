//! Sandbox trait defining process containment, execution, filesystem operations and rollback.

use crate::error::SandboxError;
use crate::types::{ExecutionResult, FileChangeSummary, SandboxId, SnapshotId};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Initializes the sandbox environment and provisions the isolated workspace.
    async fn initialize(&mut self) -> Result<SandboxId, SandboxError>;

    /// Executes an isolated process inside the sandbox with strict timeout and environment sanitization.
    async fn run_command(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<ExecutionResult, SandboxError>;

    /// Writes a file into the isolated workspace.
    async fn write_file(&self, relative_path: &Path, content: &[u8]) -> Result<(), SandboxError>;

    /// Reads a file from the isolated workspace.
    async fn read_file(&self, relative_path: &Path) -> Result<Vec<u8>, SandboxError>;

    /// Takes an atomic snapshot of the current workspace state.
    async fn snapshot(&self) -> Result<SnapshotId, SandboxError>;

    /// Reverts the workspace back to a previously saved snapshot.
    async fn rollback(&self, snapshot: SnapshotId) -> Result<(), SandboxError>;

    /// Promotes verified changes from the isolated workspace to a target host directory.
    async fn promote_changes(
        &self,
        target_host_path: &Path,
    ) -> Result<Vec<FileChangeSummary>, SandboxError>;

    /// Cleans up and destroys all ephemeral resources allocated for this sandbox.
    async fn cleanup(&mut self) -> Result<(), SandboxError>;
}
