use orbity_sandbox::{
    BwrapSandbox, NetworkMode, Sandbox, SandboxConfig, SandboxError, SandboxEventEmitter,
};
use orbity_storage::audit_chain::AuditStore;
use orbity_storage::audit_verifier::{AuditVerificationResult, AuditVerifier};
use orbity_storage::pool::SqliteStoragePool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_sandbox_confinement_rollback_and_audit_integration() {
    let run_id = format!("run-sbx-{}", Uuid::new_v4());

    // 1. Initialize SQLite storage for audit tracking
    let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
    let audit_store = AuditStore::new(pool.clone());
    let audit_verifier = AuditVerifier::new(pool);

    // 2. Initialize native bwrap sandbox
    let config = SandboxConfig {
        network: NetworkMode::Isolated,
        root_readonly: true,
        ..Default::default()
    };
    let mut sandbox = BwrapSandbox::new(config);
    let sandbox_id = sandbox.initialize().await.expect("initialize sandbox");
    let emitter = SandboxEventEmitter::new(&run_id, sandbox_id.to_string());

    // Audit event 1: SandboxCreated
    let ev_created = emitter.sandbox_created("ephemeral_workspace", "bwrap");
    audit_store.append_event(&run_id, None, &ev_created).await.unwrap();

    // 3. Security Confinement Test: Attempting to write to /etc/ must fail with Read-only file system
    let probe_res = sandbox
        .run_command("touch", &["/etc/exploit_probe".to_string()], &HashMap::new(), Duration::from_secs(5))
        .await
        .unwrap();
    assert_ne!(probe_res.exit_code, 0);
    assert!(probe_res.stderr.contains("Read-only file system"));

    // Audit event 2: PolicyDenied
    let ev_denied = emitter.policy_denied("codex-agent", "write_filesystem", "/etc/exploit_probe", "Read-only host root");
    audit_store.append_event(&run_id, None, &ev_denied).await.unwrap();

    // Audit event 3: CommandExecuted
    let ev_cmd = emitter.command_executed(None, "codex-agent", "touch", &["/etc/exploit_probe".to_string()], &probe_res);
    audit_store.append_event(&run_id, None, &ev_cmd).await.unwrap();

    // 4. Directory Traversal Security Test: Attempting to write via relative traversal must be blocked
    let traversal_err = sandbox
        .write_file(Path::new("../../etc/passwd"), b"malicious")
        .await;
    assert!(matches!(traversal_err, Err(SandboxError::PolicyDenied { .. })));

    // 5. Network Isolation Test: Offline mode must prevent external socket connection
    let net_res = sandbox
        .run_command("ip", &["link".to_string()], &HashMap::new(), Duration::from_secs(5))
        .await
        .unwrap();
    assert!(net_res.success());
    assert!(net_res.stdout.contains("lo:"));
    assert!(!net_res.stdout.contains("eth0"));

    // 6. Strict Timeout Enforcement: Long command must be forcefully terminated
    let timeout_res = sandbox
        .run_command("sleep", &["10".to_string()], &HashMap::new(), Duration::from_millis(200))
        .await
        .unwrap();
    assert!(timeout_res.timed_out);
    assert_eq!(timeout_res.exit_code, -1);

    // 7. Ephemeral Workspace, Snapshot and Rollback Test
    // Step A: Agent writes initial code
    let code_file = Path::new("src/lib.rs");
    sandbox
        .write_file(code_file, b"pub fn calculate() -> i32 { 42 }")
        .await
        .unwrap();

    let snap_initial = sandbox.snapshot().await.expect("take snapshot");

    // Step B: Agent introduces broken code / regression
    sandbox
        .write_file(code_file, b"pub fn calculate() -> i32 { broken_syntax_error }")
        .await
        .unwrap();
    let broken_content = sandbox.read_file(code_file).await.unwrap();
    assert_eq!(broken_content, b"pub fn calculate() -> i32 { broken_syntax_error }");

    // Step C: Rollback to pristine snapshot
    sandbox.rollback(snap_initial).await.expect("perform rollback");
    let restored_content = sandbox.read_file(code_file).await.unwrap();
    assert_eq!(restored_content, b"pub fn calculate() -> i32 { 42 }");

    // 8. Promote Verified Changes to Target Host Path
    let temp_target = std::env::temp_dir().join(format!("promoted_host_repo_{}", Uuid::new_v4()));
    let changes = sandbox.promote_changes(&temp_target).await.expect("promote changes");

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].relative_path, PathBuf::from("src/lib.rs"));

    // Verify promoted content on target host
    let promoted_file = temp_target.join("src/lib.rs");
    assert!(promoted_file.exists());
    let host_content = std::fs::read(&promoted_file).unwrap();
    assert_eq!(host_content, b"pub fn calculate() -> i32 { 42 }");

    // Record FileWritten in Audit Store
    let ev_written = emitter.file_written(
        None,
        "codex-agent",
        code_file,
        changes[0].bytes,
        &changes[0].content_hash,
    );
    audit_store.append_event(&run_id, None, &ev_written).await.unwrap();

    // 9. Teardown and Cleanup
    sandbox.cleanup().await.expect("cleanup sandbox");

    let ev_destroyed = emitter.sandbox_destroyed(450);
    audit_store.append_event(&run_id, None, &ev_destroyed).await.unwrap();

    // 10. Verify Complete Cryptographic Audit Trail
    let audit_verification = audit_verifier.verify_run(&run_id).await.unwrap();
    match audit_verification {
        AuditVerificationResult::Valid { total_events, final_hash, .. } => {
            assert_eq!(total_events, 5);
            assert_eq!(final_hash.len(), 64);
            println!("Sandbox audit trail verified! Total events: {}, Final hash: {}", total_events, final_hash);
        }
        other => panic!("Expected valid audit trail, got: {:?}", other),
    }

    // Clean up temporary host directory
    let _ = std::fs::remove_dir_all(&temp_target);
}
