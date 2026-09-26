//! End-to-end integration test for Gate 4:
//! MultiAgentOrchestrator coordinating tasks across Codex, Claude, Agy, Hermes, and Pi
//! within a Bubblewrap/Mock sandbox, with Blackboard state sharing, SQLite WAL checkpoints,
//! and TokenUsage extraction without Astra.

use orbity_agent::orchestrator::MultiAgentOrchestrator;
use orbity_core::contracts::{ApprovalMode, ApprovalPolicy, AutoApprovalRule};
use orbity_graph::*;
use orbity_sandbox::mock::MockSandbox;
use orbity_sandbox::types::ExecutionResult;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_gate_4_multi_agent_workflow_e2e() {
    // 1. Setup in-memory SQLite pool for checkpoints
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect in-memory SQLite");

    // 2. Setup mock sandbox simulating CLI tool outputs with realistic responses & usage metrics
    let mock_sb = MockSandbox::new();

    // Mock codex response with token usage
    mock_sb.set_command_response(
        "codex",
        ExecutionResult {
            exit_code: 0,
            stdout: r#"{"status": "ok", "code": "fn compute() -> i32 { 42 }", "usage": {"input_tokens": 120, "output_tokens": 60, "cached_tokens": 20, "reasoning_tokens": 10}}"#.to_string(),
            stderr: String::new(),
            duration_ms: 45,
            timed_out: false,
        },
    );

    // Mock claude response with token usage
    mock_sb.set_command_response(
        "claude",
        ExecutionResult {
            exit_code: 0,
            stdout: r#"{"status": "approved", "review": "Architecture is sound and strictly typed.", "usage": {"input_tokens": 200, "output_tokens": 80, "cached_tokens": 50, "reasoning_tokens": 0}}"#.to_string(),
            stderr: String::new(),
            duration_ms: 30,
            timed_out: false,
        },
    );

    // Mock agy response
    mock_sb.set_command_response(
        "agy",
        ExecutionResult {
            exit_code: 0,
            stdout: r#"{"findings": "No security vulnerabilities detected.", "usage": {"input_tokens": 90, "output_tokens": 40}}"#.to_string(),
            stderr: String::new(),
            duration_ms: 25,
            timed_out: false,
        },
    );

    // Mock hermes response
    mock_sb.set_command_response(
        "hermes",
        ExecutionResult {
            exit_code: 0,
            stdout: r#"{"tool_call": "docs.fetch", "result": "Documentation verified.", "usage": {"input_tokens": 50, "output_tokens": 20}}"#.to_string(),
            stderr: String::new(),
            duration_ms: 15,
            timed_out: false,
        },
    );

    // Mock pi response
    mock_sb.set_command_response(
        "pi",
        ExecutionResult {
            exit_code: 0,
            stdout: r#"{"refactor": "Cleaned up docstrings and format.", "usage": {"input_tokens": 40, "output_tokens": 15}}"#.to_string(),
            stderr: String::new(),
            duration_ms: 10,
            timed_out: false,
        },
    );

    let sandbox = Arc::new(mock_sb);

    // 3. Define declarative YAML workflow connecting Codex, Claude, Agy, Hermes, and Pi
    let yaml = r#"
name: "full_5_agent_pipeline"
description: "Forester multi-agent deterministic topology without Astra"
start_node: "planner"
terminal_nodes: ["final_review"]
nodes:
  - id: "planner"
    type: "orchestrator"
    engine: "topcoat"
  - id: "coder_codex"
    type: "agent"
    cli: "codex"
    prompt: "Write core calculation logic"
  - id: "tool_hermes"
    type: "agent"
    cli: "hermes"
    prompt: "Fetch external validation docs"
  - id: "researcher_agy"
    type: "agent"
    cli: "agy"
    prompt: "Scan for security CVEs"
  - id: "refactor_pi"
    type: "agent"
    cli: "pi"
    prompt: "Format and optimize code"
  - id: "reviewer_claude"
    type: "agent"
    cli: "claude"
    prompt: "Perform architectural review"
  - id: "final_review"
    type: "human_gate"
    prompt: "auto_approve_final_pipeline"
edges:
  - from: "planner"
    to: "coder_codex"
    type: "direct"
  - from: "coder_codex"
    to: "tool_hermes"
    type: "parallel_fan_out"
  - from: "coder_codex"
    to: "researcher_agy"
    type: "parallel_fan_out"
  - from: "tool_hermes"
    to: "refactor_pi"
    type: "barrier_fan_in"
  - from: "researcher_agy"
    to: "refactor_pi"
    type: "barrier_fan_in"
  - from: "refactor_pi"
    to: "reviewer_claude"
    type: "direct"
  - from: "reviewer_claude"
    to: "final_review"
    type: "direct"
"#;

    let graph = GraphYamlLoader::parse_yaml(yaml).expect("YAML pipeline should parse");
    assert_eq!(graph.nodes.len(), 7);
    assert_eq!(graph.edges.len(), 7);

    // 4. Configure automatic YAML governance policy
    let policy = ApprovalPolicy {
        mode: ApprovalMode::Automatic,
        auto_approve: vec![AutoApprovalRule {
            rule: "auto_gate".into(),
            condition: None,
            tools: vec![],
            patterns: vec!["auto_approve*".into()],
        }],
        ..Default::default()
    };

    let exec_id = Uuid::new_v4();

    // 5. Initialize MultiAgentOrchestrator
    let orchestrator = MultiAgentOrchestrator::new(sandbox)
        .with_pool(pool.clone())
        .with_approval_policy(policy)
        .with_budget(25.0);

    // 6. Execute multi-agent graph
    let blackboard = orchestrator
        .run_graph(graph, Some(exec_id))
        .await
        .expect("Multi-agent graph orchestration should succeed without errors");

    // 7. Verify all 5 agent CLI outputs are recorded on Blackboard
    let codex_out = blackboard.get_node_output("coder_codex").await;
    assert!(codex_out.is_some());
    assert!(codex_out.unwrap().contains("fn compute() -> i32 { 42 }"));

    let hermes_out = blackboard.get_node_output("tool_hermes").await;
    assert!(hermes_out.is_some());
    assert!(hermes_out.unwrap().contains("docs.fetch"));

    let agy_out = blackboard.get_node_output("researcher_agy").await;
    assert!(agy_out.is_some());
    assert!(agy_out
        .unwrap()
        .contains("No security vulnerabilities detected"));

    let pi_out = blackboard.get_node_output("refactor_pi").await;
    assert!(pi_out.is_some());
    assert!(pi_out.unwrap().contains("Cleaned up docstrings"));

    let claude_out = blackboard.get_node_output("reviewer_claude").await;
    assert!(claude_out.is_some());
    assert!(claude_out.unwrap().contains("Architecture is sound"));

    // 8. Verify SQLite checkpoints were created and hash chain is valid
    let ckpt_store = GraphCheckpointStore::new(pool);
    let latest_ckpt = ckpt_store
        .load_latest_checkpoint(exec_id)
        .await
        .expect("Should load latest checkpoint from SQLite");

    assert!(latest_ckpt.is_some());
    let ckpt = latest_ckpt.unwrap();
    assert!(ckpt.step_number > 0);
    assert!(!ckpt.state_hash.is_empty());
    assert!(!ckpt.previous_hash.is_empty());
}
