//! Comprehensive unit and integration tests for Gate 4:
//! DAG algorithms, Kahn's topological sort, fan-out/fan-in, feedback loops,
//! Blackboard context passing, FinOps budget tripwires, and YAML topology loading.

use orbity_core::contracts::{ApprovalMode, ApprovalPolicy, AutoApprovalRule};
use orbity_graph::*;
use std::sync::Arc;


#[tokio::test]
async fn test_kahns_topological_sort_diamond() {
    // Diamond DAG:
    //      Start
    //     /     \
    //   NodeA   NodeB
    //     \     /
    //      Join
    let start = GraphNode::new("start", NodeKind::Orchestrator { engine: "topcoat".into() });
    let a = GraphNode::new("node_a", NodeKind::Tool { command: "echo A".into(), timeout_secs: 10 });
    let b = GraphNode::new("node_b", NodeKind::Tool { command: "echo B".into(), timeout_secs: 10 });
    let join = GraphNode::new("join", NodeKind::JoinBarrier { quorum: Some(2) });

    let graph = GraphDefinition::builder("diamond", "Diamond Graph", "start")
        .add_node(start)
        .add_node(a)
        .add_node(b)
        .add_node(join)
        .add_edge(GraphEdge::fan_out("start", "node_a"))
        .add_edge(GraphEdge::fan_out("start", "node_b"))
        .add_edge(GraphEdge::fan_in("node_a", "join"))
        .add_edge(GraphEdge::fan_in("node_b", "join"))
        .add_terminal_node("join")
        .build();

    let plan = TopologyValidator::validate_and_plan(&graph).expect("Should validate successfully");

    assert_eq!(plan[0], NodeId::new("start"));
    assert_eq!(plan[3], NodeId::new("join"));
    assert!(plan.contains(&NodeId::new("node_a")));
    assert!(plan.contains(&NodeId::new("node_b")));
}

#[tokio::test]
async fn test_uncontrolled_cycle_detection() {
    // Illegal cycle without feedback_loop edge
    let a = GraphNode::new("a", NodeKind::Tool { command: "echo A".into(), timeout_secs: 5 });
    let b = GraphNode::new("b", NodeKind::Tool { command: "echo B".into(), timeout_secs: 5 });

    let graph = GraphDefinition::builder("cyclic", "Cyclic Graph", "a")
        .add_node(a)
        .add_node(b)
        .add_edge(GraphEdge::direct("a", "b"))
        .add_edge(GraphEdge::direct("b", "a")) // Illegal direct cycle
        .build();

    let res = TopologyValidator::validate_and_plan(&graph);
    assert!(matches!(res, Err(TopologyError::IllegalCycle(_))));
}

#[tokio::test]
async fn test_blackboard_context_injection() {
    let bb = Blackboard::new();
    bb.set_node_output("step_1", "Analysis: Code looks mostly clean.").await;
    bb.set_node_output("step_2", "Tests: 2 unit tests failed.").await;

    let injected = bb.inject_context(&[NodeId::new("step_1"), NodeId::new("step_2")]).await;

    assert!(injected.contains("Analysis: Code looks mostly clean."));
    assert!(injected.contains("Tests: 2 unit tests failed."));
}

#[tokio::test]
async fn test_graph_executor_fan_out_fan_in() {
    let start = GraphNode::new("start", NodeKind::Orchestrator { engine: "topcoat".into() });
    let worker1 = GraphNode::new("worker1", NodeKind::Agent {
        cli: CliType::Codex,
        config: AgentNodeSpec { name: "codex_worker".into(), ..Default::default() },
    });
    let worker2 = GraphNode::new("worker2", NodeKind::Agent {
        cli: CliType::Claude,
        config: AgentNodeSpec { name: "claude_worker".into(), ..Default::default() },
    });
    let aggregator = GraphNode::new("aggregator", NodeKind::JoinBarrier { quorum: Some(2) });

    let graph = GraphDefinition::builder("fan_out_in", "Fan-out Fan-in", "start")
        .add_node(start)
        .add_node(worker1)
        .add_node(worker2)
        .add_node(aggregator)
        .add_edge(GraphEdge::fan_out("start", "worker1"))
        .add_edge(GraphEdge::fan_out("start", "worker2"))
        .add_edge(GraphEdge::fan_in("worker1", "aggregator"))
        .add_edge(GraphEdge::fan_in("worker2", "aggregator"))
        .add_terminal_node("aggregator")
        .build();

    let bb = Blackboard::new();
    let finops = GraphFinOpsTracker::new(Some(10.0), None);
    let runner = Arc::new(DefaultNodeRunner);

    let executor = GraphExecutor::new(graph, bb.clone(), finops.clone(), runner);
    let result = executor.execute().await;

    assert!(result.is_ok(), "Graph execution should succeed: {:?}", result.err());
    assert!(bb.get_node_output("start").await.is_some());
    assert!(bb.get_node_output("worker1").await.is_some());
    assert!(bb.get_node_output("worker2").await.is_some());
    assert!(bb.get_node_output("aggregator").await.is_some());
    assert!(finops.total_cost().await > 0.0);
    assert!(finops.total_tokens().await > 0);
}

#[tokio::test]
async fn test_feedback_loop_retry_and_limit() {
    let coder = GraphNode::new("coder", NodeKind::Tool { command: "write_code".into(), timeout_secs: 5 });
    let tester = GraphNode::new("tester", NodeKind::Tool { command: "run_tests".into(), timeout_secs: 5 });

    let graph = GraphDefinition::builder("feedback", "Self-healing Loop", "coder")
        .add_node(coder)
        .add_node(tester)
        .add_edge(GraphEdge::direct("coder", "tester"))
        .add_edge(GraphEdge::feedback_loop("tester", "coder", 2))
        .add_terminal_node("tester")
        .build();

    let bb = Blackboard::new();
    let finops = GraphFinOpsTracker::new(None, None);
    let runner = Arc::new(DefaultNodeRunner);

    let executor = GraphExecutor::new(graph, bb.clone(), finops, runner);
    let res = executor.execute().await;
    println!("DEBUG: res = {:?}", res);

    // After 2 loop iterations, it will hit LoopBudgetExceeded
    assert!(matches!(res, Err(ExecutionError::LoopBudgetExceeded { limit: 2, .. })));

    assert_eq!(bb.get_loop_count("tester_to_coder").await, 3);
}

#[tokio::test]
async fn test_finops_tripwire_limit() {
    let node1 = GraphNode::new("node1", NodeKind::Tool { command: "echo 1".into(), timeout_secs: 5 });

    let graph = GraphDefinition::builder("budget_test", "Budget Test", "node1")
        .add_node(node1)
        .build();

    let bb = Blackboard::new();
    // Budget set to $0.0001, but each DefaultNodeRunner run costs $0.001
    let finops = GraphFinOpsTracker::new(Some(0.0001), None);
    // Pre-record spend exceeding the limit
    finops.record_spend(&NodeId::new("pre"), 0.0002, 100).await;

    let runner = Arc::new(DefaultNodeRunner);
    let executor = GraphExecutor::new(graph, bb, finops, runner);
    let res = executor.execute().await;

    assert!(matches!(res, Err(ExecutionError::BudgetExceeded(_))));
}

#[tokio::test]
async fn test_yaml_loader_and_declarative_approval() {
    let yaml = r#"
name: "forester_pipeline"
description: "Forester autonomous multi-agent pipeline"
start_node: "planner"
terminal_nodes: ["reviewer"]
nodes:
  - id: "planner"
    type: "orchestrator"
    engine: "topcoat"
  - id: "coder"
    type: "agent"
    cli: "codex"
    prompt: "Implement required feature"
  - id: "reviewer"
    type: "human_gate"
    prompt: "auto_approve_test"
edges:
  - from: "planner"
    to: "coder"
    type: "direct"
  - from: "coder"
    to: "reviewer"
    type: "direct"
"#;

    let graph = GraphYamlLoader::parse_yaml(yaml).expect("YAML parse should succeed");
    assert_eq!(graph.name, "forester_pipeline");
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);

    let policy = ApprovalPolicy {
        mode: ApprovalMode::Automatic,
        auto_approve: vec![AutoApprovalRule {
            rule: "test_auto".into(),
            condition: None,
            tools: vec![],
            patterns: vec!["auto_approve*".into()],
        }],
        ..Default::default()
    };


    let bb = Blackboard::new();
    let finops = GraphFinOpsTracker::new(Some(5.0), Some(policy));
    let runner = Arc::new(DefaultNodeRunner);

    let executor = GraphExecutor::new(graph, bb.clone(), finops, runner);
    let res = executor.execute().await;

    assert!(res.is_ok(), "Declarative auto-approval should allow reviewer node without pausing: {:?}", res.err());
}
