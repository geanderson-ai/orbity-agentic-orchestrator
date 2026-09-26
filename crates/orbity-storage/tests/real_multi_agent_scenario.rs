//! Real multi-agent orchestration test scenario.
//!
//! Validates:
//! 1. Declarative team parsing (forester.yaml).
//! 2. Concurrent workers (Codex, Claude, Hermes, Pi) coordinated by Topcoat orchestrator.
//! 3. Concurrent SQLite transactions in WAL mode writing tasks, token ledgers and audit records.
//! 4. FinOps token and cost accounting.
//! 5. Cryptographic hash-chain integrity verification.
//! 6. Active detection of simulated tamper / corruption.

use chrono::Utc;
use orbity_core::contracts::load_team_file;
use orbity_core::events::RuntimeEvent;
use orbity_core::finops::{BudgetPolicy, BudgetStatus};
use orbity_core::security::SecretMasker;
use orbity_storage::audit_chain::AuditStore;
use orbity_storage::audit_verifier::{AuditVerificationResult, AuditVerifier};
use orbity_storage::dao::{
    RunDao, RunRecord, TaskDao, TaskRecord, TeamDao, TeamRecord, TokenLedgerDao, TokenLedgerRecord,
};
use orbity_storage::pool::SqliteStoragePool;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_real_multi_agent_concurrent_orchestration_and_audit() {
    // 1. Load team contract YAML
    let team_path = std::path::Path::new("../../examples/teams/forester.yaml");
    let fallback_path = std::path::Path::new("examples/teams/forester.yaml");
    let target_path = if team_path.exists() {
        team_path
    } else {
        fallback_path
    };
    let team_def = load_team_file(target_path).expect("Must parse forester.yaml");
    assert_eq!(team_def.team.name, "forester");
    assert_eq!(team_def.team.workers.len(), 5);

    // 2. Setup real SQLite WAL database file
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("orbity_e2e_multi_agent_{}.db", Uuid::new_v4()));
    let pool = SqliteStoragePool::connect_file(&db_path)
        .await
        .expect("Connect to file-backed SQLite database");

    // Verify WAL mode is active
    let pragma_status = pool.verify_pragmas().await.expect("Pragmas status");
    assert_eq!(pragma_status.journal_mode, "WAL");
    assert!(pragma_status.foreign_keys_enabled);

    // 3. Persist Team in SQLite
    let team_dao = TeamDao::new(pool.clone());
    let team_record = TeamRecord {
        name: team_def.team.name.clone(),
        description: team_def.team.description.clone(),
        config_yaml: serde_yaml::to_string(&team_def).unwrap(),
        config_hash: "hash_forester_v1".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    team_dao.upsert(&team_record).await.expect("Upsert team");

    // 4. Topcoat orchestrator initiates execution run
    let run_id = format!("run-multi-{}", Uuid::new_v4());
    let run_dao = RunDao::new(pool.clone());
    let audit_store = AuditStore::new(pool.clone());
    let task_dao = TaskDao::new(pool.clone());
    let ledger_dao = TokenLedgerDao::new(pool.clone());
    let audit_verifier = AuditVerifier::new(pool.clone());

    let run_record = RunRecord {
        id: run_id.clone(),
        status: "Running".to_string(),
        initiated_at: Utc::now(),
        completed_at: None,
        total_tokens: 0,
        total_cost_usd: 0.0,
        metadata: Some(
            r#"{"objective": "Implement SQLite WAL & Cryptographic Audit"}"#.to_string(),
        ),
    };
    run_dao.create(&run_record).await.expect("Create run");

    // Masker to sanitize events
    let mut masker = SecretMasker::new();
    masker.register_secret("anthropic_key", "sk-ant-secret-key-1234567890abcdef");

    // Genesis Block in Audit Store
    let mut init_event = RuntimeEvent::RunInitiated {
        run_id: run_id.clone(),
        prompt:
            "Refactor SQLite storage and audit chain with key sk-ant-secret-key-1234567890abcdef"
                .to_string(),
        team_name: Some("forester".to_string()),
    };
    masker.mask_event(&mut init_event);
    let genesis_record = audit_store
        .append_event(&run_id, None, &init_event)
        .await
        .expect("Append genesis event");
    assert_eq!(genesis_record.sequence_num, 0);
    assert!(!genesis_record.payload_json.contains("sk-ant-secret-key"));

    // 5. Concurrently dispatch 4 multi-agent workers
    let shared_pool = Arc::new(pool.clone());
    let shared_run_id = Arc::new(run_id.clone());

    // Worker 1: Codex Dev (Code Implementation)
    let p1 = Arc::clone(&shared_pool);
    let r1 = Arc::clone(&shared_run_id);
    let handle_codex = tokio::spawn(async move {
        let t_dao = TaskDao::new((*p1).clone());
        let a_store = AuditStore::new((*p1).clone());
        let l_dao = TokenLedgerDao::new((*p1).clone());
        let task_id = format!("task-codex-{}", Uuid::new_v4());

        t_dao
            .create(&TaskRecord {
                id: task_id.clone(),
                run_id: (*r1).clone(),
                parent_task_id: None,
                agent_name: "codex-worker".to_string(),
                status: "Running".to_string(),
                input_prompt: Some("Implement SQLite WAL and migrations in Rust".to_string()),
                output_result: None,
                duration_ms: None,
                created_at: Utc::now(),
            })
            .await
            .unwrap();

        a_store
            .append_event(
                &r1,
                Some(&task_id),
                &RuntimeEvent::AgentStarted {
                    run_id: (*r1).clone(),
                    task_id: Some(task_id.clone()),
                    agent_id: "codex-worker".to_string(),
                    agent_name: "Codex CLI Worker".to_string(),
                },
            )
            .await
            .unwrap();

        // Simulate code generation and compilation
        a_store
            .append_event(
                &r1,
                Some(&task_id),
                &RuntimeEvent::FileWritten {
                    run_id: (*r1).clone(),
                    task_id: Some(task_id.clone()),
                    agent_name: "codex-worker".to_string(),
                    file_path: "crates/orbity-storage/src/migrations.rs".to_string(),
                    bytes_written: 2048,
                    content_hash: "hash_migrations_v1".to_string(),
                },
            )
            .await
            .unwrap();

        a_store
            .append_event(
                &r1,
                Some(&task_id),
                &RuntimeEvent::CommandExecuted {
                    run_id: (*r1).clone(),
                    task_id: Some(task_id.clone()),
                    agent_name: "codex-worker".to_string(),
                    command: "cargo".to_string(),
                    args: vec!["test".to_string(), "--workspace".to_string()],
                    exit_code: 0,
                    duration_ms: 1200,
                    stdout_preview: Some("test result: ok".to_string()),
                    stderr_preview: None,
                    sandbox_id: Some("sbx-01".to_string()),
                },
            )
            .await
            .unwrap();

        l_dao
            .record_usage(&TokenLedgerRecord {
                id: Uuid::new_v4().to_string(),
                run_id: (*r1).clone(),
                task_id: Some(task_id.clone()),
                agent_name: "codex-worker".to_string(),
                input_tokens: 3500,
                output_tokens: 1200,
                cached_tokens: 800,
                reasoning_tokens: 400,
                cost_usd: 0.045,
                recorded_at: Utc::now(),
            })
            .await
            .unwrap();

        t_dao
            .update_status(
                &task_id,
                "Completed",
                Some("Rust code and tests generated successfully"),
                Some(1250),
            )
            .await
            .unwrap();

        a_store
            .append_event(
                &r1,
                Some(&task_id),
                &RuntimeEvent::AgentFinished {
                    run_id: (*r1).clone(),
                    task_id: Some(task_id.clone()),
                    agent_id: "codex-worker".to_string(),
                    agent_name: "Codex CLI Worker".to_string(),
                    summary: Some("Code implemented and passed test suite".to_string()),
                },
            )
            .await
            .unwrap();
    });

    // Worker 2: Claude Code Sentinel (Security & Review)
    let p2 = Arc::clone(&shared_pool);
    let r2 = Arc::clone(&shared_run_id);
    let handle_claude = tokio::spawn(async move {
        let t_dao = TaskDao::new((*p2).clone());
        let a_store = AuditStore::new((*p2).clone());
        let l_dao = TokenLedgerDao::new((*p2).clone());
        let task_id = format!("task-claude-{}", Uuid::new_v4());

        t_dao
            .create(&TaskRecord {
                id: task_id.clone(),
                run_id: (*r2).clone(),
                parent_task_id: None,
                agent_name: "claude-code-worker".to_string(),
                status: "Running".to_string(),
                input_prompt: Some(
                    "Review cryptographic hash chain invariants and secret masking".to_string(),
                ),
                output_result: None,
                duration_ms: None,
                created_at: Utc::now(),
            })
            .await
            .unwrap();

        a_store
            .append_event(
                &r2,
                Some(&task_id),
                &RuntimeEvent::AgentStarted {
                    run_id: (*r2).clone(),
                    task_id: Some(task_id.clone()),
                    agent_id: "claude-code-worker".to_string(),
                    agent_name: "Claude Code Sentinel".to_string(),
                },
            )
            .await
            .unwrap();

        a_store
            .append_event(
                &r2,
                Some(&task_id),
                &RuntimeEvent::FileRead {
                    run_id: (*r2).clone(),
                    task_id: Some(task_id.clone()),
                    agent_name: "claude-code-worker".to_string(),
                    file_path: "crates/orbity-storage/src/audit_chain.rs".to_string(),
                    bytes_read: 4096,
                },
            )
            .await
            .unwrap();

        a_store
            .append_event(
                &r2,
                Some(&task_id),
                &RuntimeEvent::PolicyAllowed {
                    run_id: (*r2).clone(),
                    agent_name: "claude-code-worker".to_string(),
                    action: "merge_diff".to_string(),
                    resource: "crates/orbity-storage".to_string(),
                },
            )
            .await
            .unwrap();

        l_dao
            .record_usage(&TokenLedgerRecord {
                id: Uuid::new_v4().to_string(),
                run_id: (*r2).clone(),
                task_id: Some(task_id.clone()),
                agent_name: "claude-code-worker".to_string(),
                input_tokens: 4200,
                output_tokens: 850,
                cached_tokens: 1500,
                reasoning_tokens: 600,
                cost_usd: 0.052,
                recorded_at: Utc::now(),
            })
            .await
            .unwrap();

        t_dao
            .update_status(
                &task_id,
                "Completed",
                Some("Zero vulnerabilities found. Approved."),
                Some(800),
            )
            .await
            .unwrap();

        a_store
            .append_event(
                &r2,
                Some(&task_id),
                &RuntimeEvent::AgentFinished {
                    run_id: (*r2).clone(),
                    task_id: Some(task_id.clone()),
                    agent_id: "claude-code-worker".to_string(),
                    agent_name: "Claude Code Sentinel".to_string(),
                    summary: Some("Invariants verified and approved".to_string()),
                },
            )
            .await
            .unwrap();
    });

    // Worker 3: Hermes Researcher (Tool & Search Execution)
    let p3 = Arc::clone(&shared_pool);
    let r3 = Arc::clone(&shared_run_id);
    let handle_hermes = tokio::spawn(async move {
        let t_dao = TaskDao::new((*p3).clone());
        let a_store = AuditStore::new((*p3).clone());
        let l_dao = TokenLedgerDao::new((*p3).clone());
        let task_id = format!("task-hermes-{}", Uuid::new_v4());

        t_dao
            .create(&TaskRecord {
                id: task_id.clone(),
                run_id: (*r3).clone(),
                parent_task_id: None,
                agent_name: "hermes-researcher".to_string(),
                status: "Running".to_string(),
                input_prompt: Some("Search best practices on SQLite WAL busy timeout".to_string()),
                output_result: None,
                duration_ms: None,
                created_at: Utc::now(),
            })
            .await
            .unwrap();

        a_store
            .append_event(
                &r3,
                Some(&task_id),
                &RuntimeEvent::ToolCalled {
                    run_id: (*r3).clone(),
                    task_id: Some(task_id.clone()),
                    agent_name: "hermes-researcher".to_string(),
                    tool_name: "web.search".to_string(),
                    input: serde_json::json!({"query": "sqlite wal busy_timeout rust sqlx"}),
                    output: Some(serde_json::json!({"status": "found", "count": 3})),
                    duration_ms: 320,
                },
            )
            .await
            .unwrap();

        l_dao
            .record_usage(&TokenLedgerRecord {
                id: Uuid::new_v4().to_string(),
                run_id: (*r3).clone(),
                task_id: Some(task_id.clone()),
                agent_name: "hermes-researcher".to_string(),
                input_tokens: 1800,
                output_tokens: 400,
                cached_tokens: 0,
                reasoning_tokens: 0,
                cost_usd: 0.015,
                recorded_at: Utc::now(),
            })
            .await
            .unwrap();

        t_dao
            .update_status(&task_id, "Completed", Some("Research completed"), Some(350))
            .await
            .unwrap();
    });

    // Worker 4: Pi Assistant (Quick Fix Refactoring)
    let p4 = Arc::clone(&shared_pool);
    let r4 = Arc::clone(&shared_run_id);
    let handle_pi = tokio::spawn(async move {
        let t_dao = TaskDao::new((*p4).clone());
        let a_store = AuditStore::new((*p4).clone());
        let l_dao = TokenLedgerDao::new((*p4).clone());
        let task_id = format!("task-pi-{}", Uuid::new_v4());

        t_dao
            .create(&TaskRecord {
                id: task_id.clone(),
                run_id: (*r4).clone(),
                parent_task_id: None,
                agent_name: "pi-coder".to_string(),
                status: "Running".to_string(),
                input_prompt: Some("Fast lint and format cleanup".to_string()),
                output_result: None,
                duration_ms: None,
                created_at: Utc::now(),
            })
            .await
            .unwrap();

        a_store
            .append_event(
                &r4,
                Some(&task_id),
                &RuntimeEvent::CommandExecuted {
                    run_id: (*r4).clone(),
                    task_id: Some(task_id.clone()),
                    agent_name: "pi-coder".to_string(),
                    command: "cargo".to_string(),
                    args: vec!["fmt".to_string(), "--check".to_string()],
                    exit_code: 0,
                    duration_ms: 180,
                    stdout_preview: Some("formatted".to_string()),
                    stderr_preview: None,
                    sandbox_id: None,
                },
            )
            .await
            .unwrap();

        l_dao
            .record_usage(&TokenLedgerRecord {
                id: Uuid::new_v4().to_string(),
                run_id: (*r4).clone(),
                task_id: Some(task_id.clone()),
                agent_name: "pi-coder".to_string(),
                input_tokens: 800,
                output_tokens: 200,
                cached_tokens: 0,
                reasoning_tokens: 0,
                cost_usd: 0.005,
                recorded_at: Utc::now(),
            })
            .await
            .unwrap();

        t_dao
            .update_status(
                &task_id,
                "Completed",
                Some("Clean formatting verified"),
                Some(200),
            )
            .await
            .unwrap();
    });

    // Await all concurrent agent tasks
    let (res_codex, res_claude, res_hermes, res_pi) =
        tokio::join!(handle_codex, handle_claude, handle_hermes, handle_pi);
    res_codex.expect("Codex task succeeded");
    res_claude.expect("Claude task succeeded");
    res_hermes.expect("Hermes task succeeded");
    res_pi.expect("Pi task succeeded");

    // 6. Supervisor FinOps consolidation
    let summary = ledger_dao
        .get_run_summary(&run_id)
        .await
        .expect("Get run summary");
    assert_eq!(summary.records_count, 4);
    assert_eq!(
        summary.total_tokens,
        3500 + 1200 + 800 + 400 + 4200 + 850 + 1500 + 600 + 1800 + 400 + 800 + 200
    );
    assert!((summary.total_cost_usd - (0.045 + 0.052 + 0.015 + 0.005)).abs() < 1e-6);

    // Verify against BudgetPolicy from forester.yaml ($2.00 max budget)
    let budget_policy = BudgetPolicy {
        max_cost_usd: team_def
            .team
            .finops
            .as_ref()
            .and_then(|f| f.max_budget_usd)
            .unwrap_or(2.0),
        max_tokens: team_def
            .team
            .finops
            .as_ref()
            .and_then(|f| f.max_total_tokens)
            .unwrap_or(300_000),
        max_agent_calls: 50,
        expensive_model_approval_threshold: Some(0.80),
        auto_approve_expensive_models: false,
        alert_at_budget_percentage: Some(75.0),
    };

    let check_status = budget_policy.check_limits(
        summary.total_cost_usd,
        summary.total_tokens,
        summary.records_count,
    );
    assert_eq!(check_status, BudgetStatus::Ok);

    // Append RunCompleted event to audit chain
    let completion_event = RuntimeEvent::RunCompleted {
        run_id: run_id.clone(),
        total_tokens: summary.total_tokens,
        total_cost_usd: summary.total_cost_usd,
        duration_ms: 2500,
    };
    audit_store
        .append_event(&run_id, None, &completion_event)
        .await
        .expect("Append completion event");

    // Update Run record in SQLite
    run_dao
        .update_tokens_and_cost(&run_id, summary.total_tokens, summary.total_cost_usd)
        .await
        .expect("Update tokens and cost");
    run_dao
        .update_status(&run_id, "Completed", Some(Utc::now()))
        .await
        .expect("Update run status");

    let final_run = run_dao
        .get(&run_id)
        .await
        .unwrap()
        .expect("Final run found");
    assert_eq!(final_run.status, "Completed");
    assert_eq!(final_run.total_tokens, summary.total_tokens);

    // 7. Verify all tasks recorded
    let tasks = task_dao.list_by_run(&run_id).await.expect("List tasks");
    assert_eq!(tasks.len(), 4);
    for t in &tasks {
        assert_eq!(t.status, "Completed");
    }

    // 8. Cryptographic Audit Verification
    let audit_result = audit_verifier
        .verify_run(&run_id)
        .await
        .expect("Run audit verifier");
    match &audit_result {
        AuditVerificationResult::Valid {
            total_events,
            final_hash,
            ..
        } => {
            println!(
                "Cryptographic audit trail verified successfully! Total events: {}, Final hash: {}",
                total_events, final_hash
            );
            assert!(*total_events >= 10);
            assert_eq!(final_hash.len(), 64);
        }
        other => panic!("Expected valid audit verification, got: {:?}", other),
    }

    // 9. Tamper Simulation Test on the real SQLite database
    // Deliberately tamper with one event in the middle of the chain
    sqlx::query(
        "UPDATE audit_events SET payload_json = json_set(payload_json, '$.data.exit_code', 99) WHERE run_id = ? AND sequence_num = 3"
    )
    .bind(&run_id)
    .execute(pool.inner())
    .await
    .expect("Simulate malicious tamper");

    // Verify that the audit verifier immediately detects the fraud
    let tampered_result = audit_verifier
        .verify_run(&run_id)
        .await
        .expect("Run audit verifier after tamper");
    match tampered_result {
        AuditVerificationResult::Tampered {
            sequence_num,
            reason,
            ..
        } => {
            assert_eq!(sequence_num, 3);
            println!(
                "Malicious tampering successfully detected at sequence 3: {}",
                reason
            );
            assert!(reason.contains("hash mismatch"));
        }
        other => panic!(
            "Audit verifier failed to detect tampering! Got: {:?}",
            other
        ),
    }

    // 10. Clean up test database file
    let _ = std::fs::remove_file(&db_path);
}
