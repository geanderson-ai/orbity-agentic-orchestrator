//! Gate 3 Integration Test: Four-Layer Observability Pipeline and 5,000+ ev/s High-Throughput Load Test.

use std::sync::Arc;
use std::time::{Duration, Instant};
use orbity_core::bus::{EventBus, EventBusConfig, JsonLinesSink};
use orbity_core::events::RuntimeEvent;
use orbity_core::execution_log::ExecutionLogSink;
use orbity_core::runtime_log::RuntimeLogSink;
use orbity_storage::audit_chain::AuditStore;
use orbity_storage::audit_sink::AuditLogSink;
use orbity_storage::audit_verifier::AuditVerificationResult;
use orbity_storage::pool::SqliteStoragePool;
use orbity_telemetry::sink::TelemetrySink;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_four_layer_observability_simultaneous_pipeline() {
    let bus = EventBus::new(EventBusConfig::default());
    let mut broadcast_rx = bus.subscribe();

    // 1. Setup Layer 1: Runtime Logs
    let runtime_sink = Arc::new(RuntimeLogSink::new());
    bus.register_sink(runtime_sink.clone()).await;

    // 2. Setup Layer 2: Execution Logs
    let execution_sink = Arc::new(ExecutionLogSink::new());
    bus.register_sink(execution_sink.clone()).await;

    // 3. Setup Layer 3: SQLite Audit Logs (SHA-256 Hash Chain)
    let pool = SqliteStoragePool::connect_in_memory()
        .await
        .expect("Connect SQLite");
    let audit_store = AuditStore::new(pool);
    let audit_sink = Arc::new(AuditLogSink::new(audit_store));
    bus.register_sink(audit_sink.clone()).await;

    // 4. Setup Layer 4: OpenTelemetry Telemetry & Metrics
    let telemetry_sink = Arc::new(TelemetrySink::new());
    bus.register_sink(telemetry_sink.clone()).await;

    // 5. Setup JSON Lines Sink (Disk file)
    let temp_dir = std::env::temp_dir().join(format!("orbity-four-layers-{}", Uuid::new_v4()));
    let jsonl_path = temp_dir.join("pipeline.jsonl");
    let jsonl_sink = Arc::new(JsonLinesSink::new(&jsonl_path).await.unwrap());
    bus.register_sink(jsonl_sink).await;

    let run_id = "run-pipeline-e2e";

    // --- STEP 1: Run Initiated (Astra Supervisor) ---
    bus.emit(
        run_id,
        RuntimeEvent::RunInitiated {
            run_id: run_id.to_string(),
            prompt: "Refactor core engine and run sandbox tests".to_string(),
            team_name: Some("forester".to_string()),
        },
    )
    .await
    .unwrap();

    // --- STEP 2: Agent Started (Codex) ---
    bus.emit(
        run_id,
        RuntimeEvent::AgentStarted {
            run_id: run_id.to_string(),
            task_id: Some("task-code".to_string()),
            agent_id: "agt_codex".to_string(),
            agent_name: "codex".to_string(),
        },
    )
    .await
    .unwrap();

    // --- STEP 3: Operational Command Executed in Sandbox (Codex) ---
    bus.emit(
        run_id,
        RuntimeEvent::CommandExecuted {
            run_id: run_id.to_string(),
            task_id: Some("task-code".to_string()),
            agent_name: "codex".to_string(),
            command: "cargo".to_string(),
            args: vec!["test".to_string(), "--workspace".to_string()],
            exit_code: 0,
            duration_ms: 850,
            stdout_preview: Some("test result: ok. 18 passed".to_string()),
            stderr_preview: None,
            sandbox_id: Some("sbx-e2e-01".to_string()),
        },
    )
    .await
    .unwrap();

    // --- STEP 4: Operational File Written in Sandbox (Codex) ---
    bus.emit(
        run_id,
        RuntimeEvent::FileWritten {
            run_id: run_id.to_string(),
            task_id: Some("task-code".to_string()),
            agent_name: "codex".to_string(),
            file_path: "crates/orbity-core/src/bus.rs".to_string(),
            bytes_written: 5120,
            content_hash: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                .to_string(),
        },
    )
    .await
    .unwrap();

    // --- STEP 5: Token Usage Updated (Codex FinOps) ---
    bus.emit(
        run_id,
        RuntimeEvent::TokenUsageUpdated {
            run_id: run_id.to_string(),
            task_id: Some("task-code".to_string()),
            agent_name: "codex".to_string(),
            input_tokens: 3500,
            output_tokens: 1200,
            cached_tokens: 500,
            reasoning_tokens: 250,
            cost_usd: 0.0215,
        },
    )
    .await
    .unwrap();

    // --- STEP 6: Agent Finished (Codex) ---
    bus.emit(
        run_id,
        RuntimeEvent::AgentFinished {
            run_id: run_id.to_string(),
            task_id: Some("task-code".to_string()),
            agent_id: "agt_codex".to_string(),
            agent_name: "codex".to_string(),
            summary: Some("Refactored bus and tests verified".to_string()),
        },
    )
    .await
    .unwrap();

    // --- STEP 7: Agent Started (Claude Reviewer) ---
    bus.emit(
        run_id,
        RuntimeEvent::AgentStarted {
            run_id: run_id.to_string(),
            task_id: Some("task-review".to_string()),
            agent_id: "agt_claude".to_string(),
            agent_name: "claude".to_string(),
        },
    )
    .await
    .unwrap();

    // --- STEP 8: Security Policy Denied in Sandbox (Claude) ---
    bus.emit(
        run_id,
        RuntimeEvent::PolicyDenied {
            run_id: run_id.to_string(),
            agent_name: "claude".to_string(),
            action: "network_connect".to_string(),
            resource: "api.external-review.com:443".to_string(),
            reason: "Sandbox network isolation forbids external egress".to_string(),
        },
    )
    .await
    .unwrap();

    // --- STEP 9: Agent Finished (Claude) ---
    bus.emit(
        run_id,
        RuntimeEvent::AgentFinished {
            run_id: run_id.to_string(),
            task_id: Some("task-review".to_string()),
            agent_id: "agt_claude".to_string(),
            agent_name: "claude".to_string(),
            summary: Some("Architectural invariants verified successfully".to_string()),
        },
    )
    .await
    .unwrap();

    // --- STEP 10: Run Completed (Astra Synthesis) ---
    bus.emit(
        run_id,
        RuntimeEvent::RunCompleted {
            run_id: run_id.to_string(),
            total_tokens: 4700,
            total_cost_usd: 0.0215,
            duration_ms: 3200,
        },
    )
    .await
    .unwrap();

    // Allow asynchronous dispatch worker to process all events across all sinks
    sleep(Duration::from_millis(150)).await;

    // --- ASSERTION 1: Broadcast Channel (Real-time SSE/WebSocket) ---
    let mut broadcast_count = 0;
    while let Ok(_evt) = broadcast_rx.try_recv() {
        broadcast_count += 1;
    }
    assert_eq!(broadcast_count, 10, "Broadcast stream must receive all 10 events");

    // --- ASSERTION 2: Layer 1 Runtime Logs ---
    let runtime_records = runtime_sink.records().await;
    // Lifecycle events: RunInitiated, AgentStarted(codex), AgentFinished(codex), AgentStarted(claude), AgentFinished(claude), RunCompleted = 6
    assert_eq!(runtime_records.len(), 6);
    assert_eq!(runtime_records[0].action, "RunInitiated");
    assert_eq!(runtime_records[1].agent_name.as_deref(), Some("codex"));
    assert_eq!(runtime_records[3].agent_name.as_deref(), Some("claude"));
    assert_eq!(runtime_records[5].action, "RunCompleted");

    // --- ASSERTION 3: Layer 2 Execution Logs ---
    let exec_records = execution_sink.records().await;
    // Operational events: CommandExecuted, FileWritten = 2
    assert_eq!(exec_records.len(), 2);
    let commands = execution_sink.commands().await;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].target, "cargo test --workspace");
    assert_eq!(commands[0].duration_ms, 850);
    assert!(commands[0].success);

    let files = execution_sink.files_written().await;
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].target, "crates/orbity-core/src/bus.rs");

    // --- ASSERTION 4: Layer 3 SQLite Audit Logs (SHA-256 Hash Chain) ---
    assert_eq!(audit_sink.audited_count(), 10);
    let audit_verification = audit_sink
        .verify_run_integrity(run_id)
        .await
        .expect("Audit verification failed");

    match audit_verification {
        AuditVerificationResult::Valid { total_events, final_hash, .. } => {
            assert_eq!(total_events, 10, "All 10 events must be chained into SQLite ledger");
            assert!(!final_hash.is_empty(), "Cryptographic head hash must exist");
        }
        _ => panic!("Expected Valid audit chain, got {:?}", audit_verification),
    }

    // --- ASSERTION 5: Layer 4 Telemetry Spans & OTel Metrics ---
    let metrics = telemetry_sink.metrics().await;
    assert_eq!(metrics.total_events, 10);
    assert_eq!(metrics.total_input_tokens, 3500);
    assert_eq!(metrics.total_output_tokens, 1200);
    assert_eq!(metrics.total_cost_usd, 0.0215);
    assert_eq!(metrics.security_denials_count, 1);
    assert_eq!(metrics.commands_count, 1);
    assert_eq!(metrics.avg_command_duration_ms(), 850.0);

    let otlp_json = telemetry_sink.export_otlp_json("orbity-agentic-runtime").await;
    assert!(otlp_json["resourceSpans"].is_array());
    assert_eq!(otlp_json["metricsSummary"]["tokens"]["input"], 3500);
    assert_eq!(otlp_json["metricsSummary"]["tokens"]["estimated_cost_usd"], 0.0215);

    // --- ASSERTION 6: JSONL File Sink ---
    let file_content = tokio::fs::read_to_string(&jsonl_path).await.unwrap();
    let lines_count = file_content.lines().count();
    assert_eq!(lines_count, 10, "JSONL file on disk must have exactly 10 recorded lines");

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_high_throughput_load_5000_events_per_second() {
    const TOTAL_EVENTS: usize = 5_000;

    let config = EventBusConfig {
        queue_capacity: 32_768,
        broadcast_capacity: 16_384,
    };
    let bus = EventBus::new(config);

    let runtime_sink = Arc::new(RuntimeLogSink::new());
    let execution_sink = Arc::new(ExecutionLogSink::new());
    let telemetry_sink = Arc::new(TelemetrySink::new());

    bus.register_sink(runtime_sink.clone()).await;
    bus.register_sink(execution_sink.clone()).await;
    bus.register_sink(telemetry_sink.clone()).await;

    let start_instant = Instant::now();

    // Spawn 5 concurrent worker producers emitting 1,000 events each
    let bus_arc = Arc::new(bus);
    let mut handles = Vec::new();

    for worker_idx in 0..5 {
        let bus = Arc::clone(&bus_arc);
        let handle = tokio::spawn(async move {
            for i in 0..1_000 {
                let event = RuntimeEvent::CommandExecuted {
                    run_id: format!("run-load-{}", worker_idx),
                    task_id: Some(format!("task-{}", i)),
                    agent_name: format!("worker_{}", worker_idx),
                    command: "ping".to_string(),
                    args: vec!["-c".to_string(), "1".to_string()],
                    exit_code: 0,
                    duration_ms: (i % 50) as u64,
                    stdout_preview: None,
                    stderr_preview: None,
                    sandbox_id: Some("sbx-bench".to_string()),
                };

                bus.emit(format!("run-load-{}", worker_idx), event)
                    .await
                    .expect("Failed to emit load event");
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.expect("Producer task failed");
    }

    // Wait until sinks have processed all events
    let timeout_duration = Duration::from_secs(3);
    let deadline = Instant::now() + timeout_duration;

    while Instant::now() < deadline {
        let processed = execution_sink.commands().await.len();
        if processed >= TOTAL_EVENTS {
            break;
        }
        sleep(Duration::from_millis(10)).await;
    }

    let elapsed = start_instant.elapsed();
    let commands_processed = execution_sink.commands().await.len();
    let events_published = bus_arc.events_published_count();

    assert_eq!(
        events_published, TOTAL_EVENTS as u64,
        "All 5,000 events must be published"
    );
    assert_eq!(
        commands_processed, TOTAL_EVENTS,
        "Execution sink must process all 5,000 events without loss"
    );

    let throughput_ev_sec = (TOTAL_EVENTS as f64) / elapsed.as_secs_f64();
    println!(
        "\n🚀 [LOAD TEST RESULT] Processed {} events in {:.2?} -> Throughput: {:.1} ev/s\n",
        TOTAL_EVENTS, elapsed, throughput_ev_sec
    );

    assert!(
        throughput_ev_sec >= 5_000.0,
        "Throughput ({:.1} ev/s) must be at least 5,000 ev/s according to Gate 3 exit criteria!",
        throughput_ev_sec
    );
}
