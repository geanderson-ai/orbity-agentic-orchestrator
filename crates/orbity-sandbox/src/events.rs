//! Sandbox event emission and instrumentation for canonical RuntimeEvents.

use crate::error::SandboxError;
use crate::traits::Sandbox;
use crate::types::{ExecutionResult, FileChangeSummary, SandboxId, SnapshotId};
use async_trait::async_trait;
use orbity_core::events::RuntimeEvent;
use sha2::Digest;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Helper that crafts canonical RuntimeEvents for sandbox lifecycle actions.
#[derive(Debug, Clone)]
pub struct SandboxEventEmitter {
    pub run_id: String,
    pub sandbox_id: String,
}

impl SandboxEventEmitter {
    pub fn new(run_id: impl Into<String>, sandbox_id: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            sandbox_id: sandbox_id.into(),
        }
    }

    pub fn sandbox_created(&self, path: &str, provider: &str) -> RuntimeEvent {
        RuntimeEvent::SandboxCreated {
            sandbox_id: self.sandbox_id.clone(),
            run_id: self.run_id.clone(),
            path: path.to_string(),
            provider: provider.to_string(),
        }
    }

    pub fn sandbox_destroyed(&self, duration_ms: u64) -> RuntimeEvent {
        RuntimeEvent::SandboxDestroyed {
            sandbox_id: self.sandbox_id.clone(),
            run_id: self.run_id.clone(),
            duration_ms,
        }
    }

    pub fn command_executed(
        &self,
        task_id: Option<&str>,
        agent_name: &str,
        cmd: &str,
        args: &[String],
        res: &ExecutionResult,
    ) -> RuntimeEvent {
        let stdout_preview = if res.stdout.trim().is_empty() {
            None
        } else {
            Some(truncate_string(&res.stdout, 500))
        };

        let stderr_preview = if res.stderr.trim().is_empty() {
            None
        } else {
            Some(truncate_string(&res.stderr, 500))
        };

        RuntimeEvent::CommandExecuted {
            run_id: self.run_id.clone(),
            task_id: task_id.map(|s| s.to_string()),
            agent_name: agent_name.to_string(),
            command: cmd.to_string(),
            args: args.to_vec(),
            exit_code: res.exit_code,
            duration_ms: res.duration_ms,
            stdout_preview,
            stderr_preview,
            sandbox_id: Some(self.sandbox_id.clone()),
        }
    }

    pub fn policy_denied(
        &self,
        agent_name: &str,
        action: &str,
        resource: &str,
        reason: &str,
    ) -> RuntimeEvent {
        RuntimeEvent::PolicyDenied {
            run_id: self.run_id.clone(),
            agent_name: agent_name.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn file_written(
        &self,
        task_id: Option<&str>,
        agent_name: &str,
        file_path: &Path,
        bytes_written: usize,
        content_hash: &str,
    ) -> RuntimeEvent {
        RuntimeEvent::FileWritten {
            run_id: self.run_id.clone(),
            task_id: task_id.map(|s| s.to_string()),
            agent_name: agent_name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            bytes_written,
            content_hash: content_hash.to_string(),
        }
    }

    pub fn file_read(
        &self,
        task_id: Option<&str>,
        agent_name: &str,
        file_path: &Path,
        bytes_read: usize,
    ) -> RuntimeEvent {
        RuntimeEvent::FileRead {
            run_id: self.run_id.clone(),
            task_id: task_id.map(|s| s.to_string()),
            agent_name: agent_name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            bytes_read,
        }
    }
}

/// An instrumented sandbox decorator that records all lifecycle and execution events.
pub struct InstrumentedSandbox<S: Sandbox> {
    inner: S,
    emitter: SandboxEventEmitter,
    recorded_events: Arc<Mutex<Vec<RuntimeEvent>>>,
    start_instant: std::time::Instant,
}

impl<S: Sandbox> InstrumentedSandbox<S> {
    pub fn new(inner: S, run_id: impl Into<String>, sandbox_id: impl Into<String>) -> Self {
        let emitter = SandboxEventEmitter::new(run_id, sandbox_id);
        Self {
            inner,
            emitter,
            recorded_events: Arc::new(Mutex::new(Vec::new())),
            start_instant: std::time::Instant::now(),
        }
    }

    pub async fn get_recorded_events(&self) -> Vec<RuntimeEvent> {
        let events = self.recorded_events.lock().await;
        events.clone()
    }
}

#[async_trait]
impl<S: Sandbox> Sandbox for InstrumentedSandbox<S> {
    async fn initialize(&mut self) -> Result<SandboxId, SandboxError> {
        let id = self.inner.initialize().await?;
        let event = self.emitter.sandbox_created("ephemeral_workspace", "bwrap");
        self.recorded_events.lock().await.push(event);
        Ok(id)
    }

    async fn run_command(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<ExecutionResult, SandboxError> {
        let result = self.inner.run_command(cmd, args, env, timeout).await?;
        let event = self
            .emitter
            .command_executed(None, "sandbox_agent", cmd, args, &result);
        self.recorded_events.lock().await.push(event);
        Ok(result)
    }

    async fn write_file(&self, relative_path: &Path, content: &[u8]) -> Result<(), SandboxError> {
        self.inner.write_file(relative_path, content).await?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(content);
        let hash = hex::encode(hasher.finalize());
        let event =
            self.emitter
                .file_written(None, "sandbox_agent", relative_path, content.len(), &hash);
        self.recorded_events.lock().await.push(event);
        Ok(())
    }

    async fn read_file(&self, relative_path: &Path) -> Result<Vec<u8>, SandboxError> {
        let bytes = self.inner.read_file(relative_path).await?;
        let event = self
            .emitter
            .file_read(None, "sandbox_agent", relative_path, bytes.len());
        self.recorded_events.lock().await.push(event);
        Ok(bytes)
    }

    async fn snapshot(&self) -> Result<SnapshotId, SandboxError> {
        self.inner.snapshot().await
    }

    async fn rollback(&self, snapshot: SnapshotId) -> Result<(), SandboxError> {
        self.inner.rollback(snapshot).await
    }

    async fn promote_changes(
        &self,
        target_host_path: &Path,
    ) -> Result<Vec<FileChangeSummary>, SandboxError> {
        self.inner.promote_changes(target_host_path).await
    }

    async fn cleanup(&mut self) -> Result<(), SandboxError> {
        self.inner.cleanup().await?;
        let duration_ms = self.start_instant.elapsed().as_millis() as u64;
        let event = self.emitter.sandbox_destroyed(duration_ms);
        self.recorded_events.lock().await.push(event);
        Ok(())
    }
}

fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}... [truncated]", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockSandbox;

    #[tokio::test]
    async fn test_instrumented_sandbox_events() {
        let mock = MockSandbox::new();
        let mut sandbox = InstrumentedSandbox::new(mock, "run-ev-01", "sbx-01");

        sandbox.initialize().await.unwrap();
        sandbox
            .write_file(Path::new("test.txt"), b"test content")
            .await
            .unwrap();
        sandbox
            .run_command(
                "cargo",
                &["test".to_string()],
                &HashMap::new(),
                Duration::from_secs(5),
            )
            .await
            .unwrap();
        sandbox.cleanup().await.unwrap();

        let events = sandbox.get_recorded_events().await;
        assert_eq!(events.len(), 4);

        assert_eq!(events[0].event_type_name(), "SandboxCreated");
        assert_eq!(events[1].event_type_name(), "FileWritten");
        assert_eq!(events[2].event_type_name(), "CommandExecuted");
        assert_eq!(events[3].event_type_name(), "SandboxDestroyed");
    }
}
