//! Mock sandbox implementation for isolated unit testing.

use crate::error::SandboxError;
use crate::traits::Sandbox;
use crate::types::{ExecutionResult, FileChangeSummary, FileChangeType, SandboxId, SnapshotId};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

type FileMap = HashMap<PathBuf, Vec<u8>>;
type SnapshotMap = HashMap<SnapshotId, FileMap>;

#[derive(Clone, Debug)]
pub struct MockSandbox {
    id: SandboxId,
    files: Arc<Mutex<FileMap>>,
    snapshots: Arc<Mutex<SnapshotMap>>,
    command_responses: Arc<Mutex<HashMap<String, ExecutionResult>>>,
    cleaned_up: Arc<Mutex<bool>>,
}

impl Default for MockSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSandbox {
    pub fn new() -> Self {
        Self {
            id: SandboxId(format!("mock-sbx-{}", Uuid::new_v4())),
            files: Arc::new(Mutex::new(HashMap::new())),
            snapshots: Arc::new(Mutex::new(HashMap::new())),
            command_responses: Arc::new(Mutex::new(HashMap::new())),
            cleaned_up: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_command_response(&self, cmd: &str, result: ExecutionResult) {
        let mut map = self.command_responses.lock().unwrap();
        map.insert(cmd.to_string(), result);
    }

    pub fn is_cleaned_up(&self) -> bool {
        *self.cleaned_up.lock().unwrap()
    }
}

#[async_trait]
impl Sandbox for MockSandbox {
    async fn initialize(&mut self) -> Result<SandboxId, SandboxError> {
        Ok(self.id.clone())
    }

    async fn run_command(
        &self,
        cmd: &str,
        _args: &[String],
        _env: &HashMap<String, String>,
        _timeout: Duration,
    ) -> Result<ExecutionResult, SandboxError> {
        let map = self.command_responses.lock().unwrap();
        if let Some(res) = map.get(cmd) {
            Ok(res.clone())
        } else {
            Ok(ExecutionResult {
                exit_code: 0,
                stdout: format!("Mock stdout for {}", cmd),
                stderr: String::new(),
                duration_ms: 5,
                timed_out: false,
            })
        }
    }

    async fn write_file(&self, relative_path: &Path, content: &[u8]) -> Result<(), SandboxError> {
        let mut files = self.files.lock().unwrap();
        files.insert(relative_path.to_path_buf(), content.to_vec());
        Ok(())
    }

    async fn read_file(&self, relative_path: &Path) -> Result<Vec<u8>, SandboxError> {
        let files = self.files.lock().unwrap();
        files.get(relative_path).cloned().ok_or_else(|| {
            SandboxError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found in mock sandbox",
            ))
        })
    }

    async fn snapshot(&self) -> Result<SnapshotId, SandboxError> {
        let snap_id = SnapshotId(format!("snap-{}", Uuid::new_v4()));
        let files = self.files.lock().unwrap().clone();
        let mut snaps = self.snapshots.lock().unwrap();
        snaps.insert(snap_id.clone(), files);
        Ok(snap_id)
    }

    async fn rollback(&self, snapshot: SnapshotId) -> Result<(), SandboxError> {
        let snaps = self.snapshots.lock().unwrap();
        if let Some(saved) = snaps.get(&snapshot) {
            let mut files = self.files.lock().unwrap();
            *files = saved.clone();
            Ok(())
        } else {
            Err(SandboxError::RollbackError(format!(
                "Snapshot {} not found",
                snapshot
            )))
        }
    }

    async fn promote_changes(
        &self,
        _target_host_path: &Path,
    ) -> Result<Vec<FileChangeSummary>, SandboxError> {
        let files = self.files.lock().unwrap();
        let mut summaries = Vec::new();
        for (path, content) in files.iter() {
            let mut hasher = Sha256::new();
            hasher.update(content);
            let hash = hex::encode(hasher.finalize());
            summaries.push(FileChangeSummary {
                relative_path: path.clone(),
                change_type: FileChangeType::Created,
                bytes: content.len(),
                content_hash: hash,
            });
        }
        Ok(summaries)
    }

    async fn cleanup(&mut self) -> Result<(), SandboxError> {
        let mut cleaned = self.cleaned_up.lock().unwrap();
        *cleaned = true;
        let mut files = self.files.lock().unwrap();
        files.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_sandbox_lifecycle() {
        let mut sandbox = MockSandbox::new();
        let id = sandbox.initialize().await.unwrap();
        assert!(!id.0.is_empty());

        // Write file and snapshot
        let file_path = Path::new("src/main.rs");
        sandbox
            .write_file(file_path, b"fn main() {}")
            .await
            .unwrap();
        let snap1 = sandbox.snapshot().await.unwrap();

        // Mutate file
        sandbox
            .write_file(file_path, b"fn main() { panic!(); }")
            .await
            .unwrap();
        let read1 = sandbox.read_file(file_path).await.unwrap();
        assert_eq!(read1, b"fn main() { panic!(); }");

        // Rollback
        sandbox.rollback(snap1).await.unwrap();
        let read_rollback = sandbox.read_file(file_path).await.unwrap();
        assert_eq!(read_rollback, b"fn main() {}");

        // Promote
        let changes = sandbox.promote_changes(Path::new("/tmp")).await.unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].relative_path, PathBuf::from("src/main.rs"));

        // Cleanup
        assert!(!sandbox.is_cleaned_up());
        sandbox.cleanup().await.unwrap();
        assert!(sandbox.is_cleaned_up());
    }
}
