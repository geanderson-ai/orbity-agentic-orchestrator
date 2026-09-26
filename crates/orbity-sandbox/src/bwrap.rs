//! Native Linux Bubblewrap (`bwrap`) sandbox provider with namespace isolation.

use crate::error::SandboxError;
use crate::traits::Sandbox;
use crate::types::{
    ExecutionResult, FileChangeSummary, FileChangeType, NetworkMode, SandboxConfig, SandboxId,
    SnapshotId, WorkspaceMode,
};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug)]
pub struct BwrapSandbox {
    id: SandboxId,
    config: SandboxConfig,
    bwrap_path: PathBuf,
    base_dir: PathBuf,
    workspace_dir: PathBuf,
    snapshots_dir: PathBuf,
    initialized: bool,
}

impl BwrapSandbox {
    /// Creates a new BwrapSandbox instance with the provided configuration.
    pub fn new(config: SandboxConfig) -> Self {
        let id_str = format!("sbx-{}", Uuid::new_v4());
        let base_dir = std::env::temp_dir().join(&id_str);
        let workspace_dir = base_dir.join("workspace");
        let snapshots_dir = base_dir.join("snapshots");
        let bwrap_path = Self::detect_bwrap().unwrap_or_else(|_| PathBuf::from("/usr/bin/bwrap"));

        Self {
            id: SandboxId(id_str),
            config,
            bwrap_path,
            base_dir,
            workspace_dir,
            snapshots_dir,
            initialized: false,
        }
    }

    /// Detects the location of the `bwrap` binary on the host system.
    pub fn detect_bwrap() -> Result<PathBuf, SandboxError> {
        let standard_paths = [
            PathBuf::from("/usr/bin/bwrap"),
            PathBuf::from("/usr/local/bin/bwrap"),
            PathBuf::from("/bin/bwrap"),
        ];

        for path in &standard_paths {
            if path.exists() && path.is_file() {
                return Ok(path.clone());
            }
        }

        if let Ok(output) = std::process::Command::new("which").arg("bwrap").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    return Ok(PathBuf::from(path_str));
                }
            }
        }

        Err(SandboxError::RuntimeNotFound(
            "Bubblewrap ('bwrap') binary not found in standard system paths".to_string(),
        ))
    }

    /// Returns the workspace directory path on the host.
    pub fn workspace_path(&self) -> &Path {
        &self.workspace_dir
    }

    /// Validates that a relative path stays inside the workspace without escaping.
    fn resolve_path(&self, relative_path: &Path) -> Result<PathBuf, SandboxError> {
        let path_str = relative_path.to_string_lossy();
        if path_str.contains("..") || relative_path.is_absolute() {
            return Err(SandboxError::PolicyDenied {
                action: "access_path".to_string(),
                resource: path_str.to_string(),
                reason: "Directory traversal or absolute path outside sandbox workspace detected"
                    .to_string(),
            });
        }
        Ok(self.workspace_dir.join(relative_path))
    }
}

#[async_trait]
impl Sandbox for BwrapSandbox {
    async fn initialize(&mut self) -> Result<SandboxId, SandboxError> {
        if !self.bwrap_path.exists() {
            return Err(SandboxError::RuntimeNotFound(format!(
                "bwrap binary not found at {}",
                self.bwrap_path.display()
            )));
        }

        // Create ephemeral filesystem layout
        tokio::fs::create_dir_all(&self.workspace_dir).await?;
        tokio::fs::create_dir_all(&self.snapshots_dir).await?;

        // If configured with CopyOnWrite from existing source, copy files over
        if let WorkspaceMode::EphemeralCopyOnWrite(ref source) = self.config.workspace {
            if source.exists() {
                copy_dir_recursive(source, &self.workspace_dir).await?;
            }
        }

        self.initialized = true;
        Ok(self.id.clone())
    }

    async fn run_command(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<ExecutionResult, SandboxError> {
        if !self.initialized {
            return Err(SandboxError::InitError(
                "Sandbox must be initialized before running commands".to_string(),
            ));
        }

        let start_time = Instant::now();

        // Build bwrap containment arguments
        let mut bwrap_cmd = Command::new(&self.bwrap_path);

        // 1. Filesystem isolation
        if self.config.root_readonly {
            bwrap_cmd.arg("--ro-bind").arg("/").arg("/");
        } else {
            bwrap_cmd.arg("--bind").arg("/").arg("/");
        }

        // Ephemeral mounts: tmpfs on /tmp, create /tmp/workspace and bind isolated workspace
        bwrap_cmd.arg("--tmpfs").arg("/tmp");
        bwrap_cmd.arg("--dir").arg("/tmp/workspace");
        bwrap_cmd
            .arg("--bind")
            .arg(&self.workspace_dir)
            .arg("/tmp/workspace");

        bwrap_cmd.arg("--proc").arg("/proc");
        bwrap_cmd.arg("--dev").arg("/dev");

        // Working directory inside sandbox
        bwrap_cmd.arg("--chdir").arg("/tmp/workspace");

        // 2. Namespace isolation (PID, IPC, UTS)
        bwrap_cmd.arg("--unshare-pid");
        bwrap_cmd.arg("--unshare-ipc");
        bwrap_cmd.arg("--unshare-uts");

        // 3. Network isolation
        if self.config.network == NetworkMode::Isolated {
            bwrap_cmd.arg("--unshare-net");
        }

        // 4. Environment variable filtering
        let real_home = std::env::var("HOME").unwrap_or_else(|_| "/tmp/workspace".to_string());
        bwrap_cmd.arg("--clearenv");
        bwrap_cmd.arg("--setenv").arg("HOME").arg(&real_home);
        bwrap_cmd.arg("--setenv").arg("PATH").arg(
            std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".to_string()),
        );
        bwrap_cmd.arg("--setenv").arg("LANG").arg("C.UTF-8");
        bwrap_cmd.arg("--setenv").arg("TERM").arg("xterm-256color");

        // Forward host environment variables needed for LLM CLI tools and auth
        for (k, v) in std::env::vars() {
            let k_upper = k.to_uppercase();
            if k_upper.starts_with("ANTHROPIC_")
                || k_upper.starts_with("OPENAI_")
                || k_upper.starts_with("GEMINI_")
                || k_upper.starts_with("CLAUDE_")
                || k_upper.starts_with("CODEX_")
                || k_upper.starts_with("ORBITY_")
                || k_upper.starts_with("XDG_")
                || k_upper == "USER"
                || k_upper == "SHELL"
                || self.config.env_passthrough.contains(&k)
            {
                bwrap_cmd.arg("--setenv").arg(k).arg(v);
            }
        }

        // Inject node-specific environment variables
        for (k, v) in env {
            bwrap_cmd.arg("--setenv").arg(k).arg(v);
        }

        // Command and its arguments
        bwrap_cmd.arg(cmd);
        for arg in args {
            bwrap_cmd.arg(arg);
        }

        bwrap_cmd.stdout(Stdio::piped());
        bwrap_cmd.stderr(Stdio::piped());

        // Spawn child
        let mut child = bwrap_cmd.spawn()?;

        // Wait with strict timeout
        let wait_result = tokio::time::timeout(timeout, child.wait()).await;

        match wait_result {
            Ok(Ok(status)) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;

                let mut stdout = String::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_string(&mut stdout).await;
                }

                let mut stderr = String::new();
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_string(&mut stderr).await;
                }

                let exit_code = status.code().unwrap_or(-1);

                Ok(ExecutionResult {
                    exit_code,
                    stdout,
                    stderr,
                    duration_ms,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(SandboxError::IoError(e)),
            Err(_) => {
                // Timeout occurred: kill child forcefully with SIGKILL
                let _ = child.kill().await;
                let duration_ms = start_time.elapsed().as_millis() as u64;

                Ok(ExecutionResult {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!(
                        "Process killed: execution timed out after {}ms",
                        timeout.as_millis()
                    ),
                    duration_ms,
                    timed_out: true,
                })
            }
        }
    }

    async fn write_file(&self, relative_path: &Path, content: &[u8]) -> Result<(), SandboxError> {
        let dest = self.resolve_path(relative_path)?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&dest, content).await?;
        Ok(())
    }

    async fn read_file(&self, relative_path: &Path) -> Result<Vec<u8>, SandboxError> {
        let target = self.resolve_path(relative_path)?;
        let content = tokio::fs::read(&target).await?;
        Ok(content)
    }

    async fn snapshot(&self) -> Result<SnapshotId, SandboxError> {
        let snap_id = SnapshotId(format!("snap-{}", Uuid::new_v4()));
        let snap_dir = self.snapshots_dir.join(snap_id.to_string());
        tokio::fs::create_dir_all(&snap_dir).await?;

        copy_dir_recursive(&self.workspace_dir, &snap_dir).await?;
        Ok(snap_id)
    }

    async fn rollback(&self, snapshot: SnapshotId) -> Result<(), SandboxError> {
        let snap_dir = self.snapshots_dir.join(snapshot.to_string());
        if !snap_dir.exists() {
            return Err(SandboxError::RollbackError(format!(
                "Snapshot {} does not exist",
                snapshot
            )));
        }

        // Clean current workspace
        let _ = tokio::fs::remove_dir_all(&self.workspace_dir).await;
        tokio::fs::create_dir_all(&self.workspace_dir).await?;

        // Restore from snapshot
        copy_dir_recursive(&snap_dir, &self.workspace_dir).await?;
        Ok(())
    }

    async fn promote_changes(
        &self,
        target_host_path: &Path,
    ) -> Result<Vec<FileChangeSummary>, SandboxError> {
        tokio::fs::create_dir_all(target_host_path).await?;
        let mut summaries = Vec::new();

        collect_and_promote(
            &self.workspace_dir,
            &self.workspace_dir,
            target_host_path,
            &mut summaries,
        )
        .await?;

        Ok(summaries)
    }

    async fn cleanup(&mut self) -> Result<(), SandboxError> {
        if self.base_dir.exists() {
            tokio::fs::remove_dir_all(&self.base_dir).await?;
        }
        self.initialized = false;
        Ok(())
    }
}

impl Drop for BwrapSandbox {
    fn drop(&mut self) {
        if self.base_dir.exists() {
            let _ = std::fs::remove_dir_all(&self.base_dir);
        }
    }
}

/// Recursively copies a directory to another directory asynchronously.
async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), SandboxError> {
    tokio::fs::create_dir_all(dst).await?;
    let mut entries = tokio::fs::read_dir(src).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else if file_type.is_file() {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }

    Ok(())
}

/// Recursively traverses workspace, computes SHA-256 for modified files and promotes them to target.
async fn collect_and_promote(
    current_dir: &Path,
    workspace_root: &Path,
    target_host_root: &Path,
    summaries: &mut Vec<FileChangeSummary>,
) -> Result<(), SandboxError> {
    let mut entries = tokio::fs::read_dir(current_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let path = entry.path();

        if file_type.is_dir() {
            Box::pin(collect_and_promote(
                &path,
                workspace_root,
                target_host_root,
                summaries,
            ))
            .await?;
        } else if file_type.is_file() {
            let rel_path = path.strip_prefix(workspace_root).map_err(|e| {
                SandboxError::PromoteError(format!("Failed to calculate relative path: {}", e))
            })?;

            let content = tokio::fs::read(&path).await?;
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let content_hash = hex::encode(hasher.finalize());

            let target_dest = target_host_root.join(rel_path);
            if let Some(parent) = target_dest.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let change_type = if target_dest.exists() {
                FileChangeType::Modified
            } else {
                FileChangeType::Created
            };

            tokio::fs::write(&target_dest, &content).await?;

            summaries.push(FileChangeSummary {
                relative_path: rel_path.to_path_buf(),
                change_type,
                bytes: content.len(),
                content_hash,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bwrap_detection() {
        let bwrap = BwrapSandbox::detect_bwrap();
        assert!(
            bwrap.is_ok(),
            "Bubblewrap should be detected on Linux: {:?}",
            bwrap
        );
    }

    #[tokio::test]
    async fn test_bwrap_basic_command_execution() {
        let config = SandboxConfig::default();
        let mut sandbox = BwrapSandbox::new(config);
        sandbox
            .initialize()
            .await
            .expect("initialize bwrap sandbox");

        let res = sandbox
            .run_command(
                "echo",
                &["Hello Sandbox".to_string()],
                &HashMap::new(),
                Duration::from_secs(5),
            )
            .await
            .expect("run echo");

        assert!(res.success());
        assert!(res.stdout.contains("Hello Sandbox"));
    }

    #[tokio::test]
    async fn test_bwrap_read_only_root_protection() {
        let config = SandboxConfig::default();
        let mut sandbox = BwrapSandbox::new(config);
        sandbox.initialize().await.unwrap();

        // Attempting to write to /etc/ must fail with Read-only file system
        let res = sandbox
            .run_command(
                "touch",
                &["/etc/malicious_probe".to_string()],
                &HashMap::new(),
                Duration::from_secs(5),
            )
            .await
            .unwrap();

        assert_ne!(res.exit_code, 0, "Writing to /etc/ must fail");
        assert!(res.stderr.contains("Read-only file system"));
    }

    #[tokio::test]
    async fn test_bwrap_network_isolation() {
        let config = SandboxConfig {
            network: NetworkMode::Isolated,
            ..Default::default()
        };
        let mut sandbox = BwrapSandbox::new(config);
        sandbox.initialize().await.unwrap();

        // Check network interfaces with ip link: only loopback lo must exist
        let res = sandbox
            .run_command(
                "ip",
                &["link".to_string()],
                &HashMap::new(),
                Duration::from_secs(5),
            )
            .await
            .unwrap();

        assert!(res.success());
        assert!(res.stdout.contains("lo:"));
        assert!(!res.stdout.contains("eth0"));
        assert!(!res.stdout.contains("wlan0"));
    }

    #[tokio::test]
    async fn test_bwrap_timeout_and_sigkill() {
        let config = SandboxConfig::default();
        let mut sandbox = BwrapSandbox::new(config);
        sandbox.initialize().await.unwrap();

        // Run a command that takes 5 seconds, but set timeout to 100 milliseconds
        let res = sandbox
            .run_command(
                "sleep",
                &["5".to_string()],
                &HashMap::new(),
                Duration::from_millis(150),
            )
            .await
            .unwrap();

        assert!(res.timed_out);
        assert_eq!(res.exit_code, -1);
        assert!(res.stderr.contains("timed out"));
    }

    #[tokio::test]
    async fn test_bwrap_files_snapshot_and_rollback() {
        let config = SandboxConfig::default();
        let mut sandbox = BwrapSandbox::new(config);
        sandbox.initialize().await.unwrap();

        let file_path = Path::new("test_doc.txt");
        sandbox.write_file(file_path, b"Version 1").await.unwrap();

        let snap1 = sandbox.snapshot().await.unwrap();

        // Overwrite file
        sandbox
            .write_file(file_path, b"Version 2 Corrupted")
            .await
            .unwrap();
        let v2 = sandbox.read_file(file_path).await.unwrap();
        assert_eq!(v2, b"Version 2 Corrupted");

        // Rollback
        sandbox.rollback(snap1).await.unwrap();
        let v1 = sandbox.read_file(file_path).await.unwrap();
        assert_eq!(v1, b"Version 1");
    }

    #[tokio::test]
    async fn test_bwrap_promote_changes() {
        let config = SandboxConfig::default();
        let mut sandbox = BwrapSandbox::new(config);
        sandbox.initialize().await.unwrap();

        sandbox
            .write_file(
                Path::new("code/lib.rs"),
                b"pub fn add(a: i32, b: i32) -> i32 { a + b }",
            )
            .await
            .unwrap();

        let temp_target = std::env::temp_dir().join(format!("target_repo_{}", Uuid::new_v4()));
        let changes = sandbox.promote_changes(&temp_target).await.unwrap();

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].relative_path, PathBuf::from("code/lib.rs"));
        assert_eq!(changes[0].change_type, FileChangeType::Created);

        // Verify file on target host
        let target_file = temp_target.join("code/lib.rs");
        assert!(target_file.exists());
        let content = std::fs::read(&target_file).unwrap();
        assert_eq!(content, b"pub fn add(a: i32, b: i32) -> i32 { a + b }");

        let _ = std::fs::remove_dir_all(&temp_target);
    }
}
