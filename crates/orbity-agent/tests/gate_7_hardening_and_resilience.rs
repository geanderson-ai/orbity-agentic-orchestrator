//! Gate 7 Hardening, Security, Concurrency, and E2E Resilience Tests.
//!
//! Validates:
//! 1. TASK-701: Audit tampering detection & forensic error reporting on modified SQLite records.
//! 2. TASK-702: Strict sandbox confinement against escape attempts, path traversal, and malicious file writes.
//! 3. TASK-703: FinOps tripwire strict enforcement, budget cap trip, and error state transitions.
//! 4. TASK-704: Concurrent multi-instance stress test across simultaneous graph executions without SQLite lock failures.

use orbity_core::events::RuntimeEvent;
use orbity_graph::*;
use orbity_sandbox::bwrap::BwrapSandbox;
use orbity_sandbox::traits::Sandbox;
use orbity_sandbox::types::{NetworkMode, SandboxConfig, WorkspaceMode};
use orbity_storage::audit_chain::AuditStore;
use orbity_storage::audit_verifier::{AuditVerificationResult, AuditVerifier};
use orbity_storage::pool::SqliteStoragePool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// TASK-701: Teste de Resistência e Adulteração de Auditoria
#[tokio::test]
async fn test_task_701_audit_tampering_detection() {
    let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
    pool.run_migrations().await.unwrap();

    let store = AuditStore::new(pool.clone());
    let run_id = format!("run-audit-tamper-{}", Uuid::new_v4());

    // Insert 5 valid cryptographically chained audit events
    for i in 1..=5 {
        let event = RuntimeEvent::AgentStarted {
            run_id: run_id.clone(),
            task_id: Some(format!("task-{}", i)),
            agent_id: format!("agent-{}", i),
            agent_name: format!("Worker {}", i),
        };
        store.append_event(&run_id, None, &event).await.unwrap();
    }

    // Initial check: must be valid
    let verifier = AuditVerifier::new(pool.clone());
    let valid_res = verifier.verify_run(&run_id).await.unwrap();
    assert!(matches!(
        valid_res,
        AuditVerificationResult::Valid {
            total_events: 5,
            ..
        }
    ));

    // Tamper test: Corrupt 1 byte in sequence_num = 3 payload
    sqlx::query(
        "UPDATE audit_events SET payload_json = '{\"tampered\": true}' WHERE run_id = ? AND sequence_num = 3",
    )
    .bind(&run_id)
    .execute(pool.inner())
    .await
    .unwrap();

    // Verification must detect the tampering at sequence 3
    let tampered_res = verifier.verify_run(&run_id).await.unwrap();
    match tampered_res {
        AuditVerificationResult::Tampered {
            sequence_num,
            expected_hash,
            actual_hash,
            ..
        } => {
            assert_eq!(sequence_num, 3);
            assert_ne!(expected_hash, actual_hash);
        }
        other => panic!("Expected Tampered result, got: {:?}", other),
    }
}

/// TASK-702: Teste de Confinamento Estrito da Sandbox
#[tokio::test]
async fn test_task_702_sandbox_strict_confinement() {
    let temp_base =
        std::env::temp_dir().join(format!("orbity_sandbox_confinement_{}", Uuid::new_v4()));
    let config = SandboxConfig {
        provider: "bwrap".into(),
        network: NetworkMode::Isolated,
        workspace: WorkspaceMode::EphemeralCopyOnWrite(temp_base.clone()),
        root_readonly: true,
        default_timeout: Duration::from_secs(10),
        env_passthrough: vec!["PATH".into()],
        memory_limit_mb: Some(512),
    };

    let mut sbx = BwrapSandbox::new(config);
    if let Err(e) = sbx.initialize().await {
        eprintln!("Bwrap not supported in this container environment: {}. Skipping host bwrap confinement test.", e);
        return;
    }

    let env = HashMap::new();

    // 1. Attempt to write to host root /bin/malicious_exec -> Must fail (Read-only file system)
    let res_write_root = sbx
        .run_command(
            "touch",
            &["/bin/malicious_exec".into()],
            &env,
            Duration::from_secs(5),
        )
        .await
        .unwrap();
    assert_ne!(
        res_write_root.exit_code, 0,
        "Writing to root /bin must fail"
    );
    assert!(
        res_write_root.stderr.to_lowercase().contains("read-only")
            || res_write_root
                .stderr
                .to_lowercase()
                .contains("permission denied")
    );

    // 2. Attempt path traversal escape -> Must fail
    let res_traversal = sbx.read_file(Path::new("../../etc/shadow")).await;
    assert!(res_traversal.is_err(), "Path traversal must be rejected");

    sbx.cleanup().await.unwrap();
}

/// TASK-703: Teste de Tripwire Orçamentário e FinOps
#[tokio::test]
async fn test_task_703_finops_tripwire_budget_cap() {
    let node1 = GraphNode::new(
        "node1",
        NodeKind::Tool {
            command: "echo 1".into(),
            timeout_secs: 5,
        },
    );
    let node2 = GraphNode::new(
        "node2",
        NodeKind::Tool {
            command: "echo 2".into(),
            timeout_secs: 5,
        },
    );

    let graph = GraphDefinition::builder("budget_cap_test", "Budget Cap Test", "node1")
        .add_node(node1)
        .add_node(node2)
        .add_edge(GraphEdge::direct("node1", "node2"))
        .build();

    let bb = Blackboard::new();
    // Budget set strictly to $0.0015
    let finops = GraphFinOpsTracker::new(Some(0.0015), None);
    let runner = Arc::new(DefaultNodeRunner);

    let executor = GraphExecutor::new(graph, bb, finops.clone(), runner);

    // Run execution: Node 1 spends $0.0010 (ok), then Node 2 checks budget before running: $0.0010 spent, ok.
    // If we pre-record $0.0010, the total exceeds $0.0015 when Node 2 is about to run.
    finops
        .record_spend(&NodeId::new("initial"), 0.0010, 100)
        .await;

    let res = executor.execute().await;
    assert!(res.is_err(), "Execution must abort due to budget tripwire");
    assert!(matches!(
        res.err().unwrap(),
        ExecutionError::BudgetExceeded(_)
    ));
}

/// TASK-704: Teste de Stress Multi-Agente Concorrente
#[tokio::test]
async fn test_task_704_concurrent_multi_instance_stress() {
    let pool = SqliteStoragePool::connect_in_memory().await.unwrap();
    pool.run_migrations().await.unwrap();

    let mut handles = Vec::new();

    // Launch 4 concurrent graph orchestrations
    for instance_idx in 0..4 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let start = GraphNode::new(
                "start",
                NodeKind::Orchestrator {
                    engine: "topcoat".into(),
                },
            );
            let worker = GraphNode::new(
                "worker",
                NodeKind::Tool {
                    command: "compute".into(),
                    timeout_secs: 5,
                },
            );
            let end = GraphNode::new("end", NodeKind::JoinBarrier { quorum: Some(1) });

            let graph = GraphDefinition::builder(
                format!("graph_{}", instance_idx),
                "Stress Graph",
                "start",
            )
            .add_node(start)
            .add_node(worker)
            .add_node(end)
            .add_edge(GraphEdge::direct("start", "worker"))
            .add_edge(GraphEdge::direct("worker", "end"))
            .add_terminal_node("end")
            .build();

            let bb = Blackboard::new();
            let finops = GraphFinOpsTracker::new(Some(10.0), None);
            let runner = Arc::new(DefaultNodeRunner);

            let store = GraphCheckpointStore::new(pool_clone.inner().clone());
            store.init_schema().await.unwrap();

            let executor =
                GraphExecutor::new(graph, bb.clone(), finops, runner).with_checkpoints(store);

            executor.execute().await.unwrap();

            bb.get_node_output("end").await.is_some()
        });

        handles.push(handle);
    }

    for h in handles {
        let success = h.await.expect("Task join should not panic");
        assert!(success, "Every concurrent graph execution must succeed");
    }
}
