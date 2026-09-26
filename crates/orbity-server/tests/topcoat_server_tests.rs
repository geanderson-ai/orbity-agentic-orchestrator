//! Unit and integration tests for Gate 5: Tokio Topcoat server application,
//! reactive views, client signals, server-push, and HITL governance console.

use orbity_core::bus::{EventBus, EventBusConfig};
use orbity_core::events::RuntimeEvent;
use orbity_server::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_topcoat_context_and_signals() {
    let sig = signal("initial".to_string());
    assert_eq!(sig.get(), "initial");
    assert_eq!(sig.version(), 0);

    sig.set("updated".to_string());
    assert_eq!(sig.get(), "updated");
    assert_eq!(sig.version(), 1);
}

#[tokio::test]
async fn test_topcoat_shard_component_rendering() {
    let shard = ShardComponent::new("agent-metric", "shard-101", || {
        Views::agent_card("Codex", "codex", "Executing", 12_500, 0.35)
    });

    let rendered = shard.render_shard();
    assert!(rendered.contains("data-topcoat-shard=\"shard-101\""));
    assert!(rendered.contains("data-shard-name=\"agent-metric\""));
    assert!(rendered.contains("Codex"));
    assert!(rendered.contains("Executing"));
    assert!(rendered.contains("$0.3500"));
}

#[tokio::test]
async fn test_topcoat_server_dashboard_rendering() {
    let bus = EventBus::new(EventBusConfig::default());
    let config = ServerConfig::default();
    let cx = Cx::new(bus, None, config);
    let server = TopcoatServer::new(cx);

    let html = server.handle_index().await;
    assert!(html.contains("Orbity Agentic Platform - Topcoat Console"));
    assert!(html.contains("FinOps Real-Time Budget"));
    assert!(html.contains("SHA-256 Chain Verified"));
    assert!(html.contains("Codex Worker"));
    assert!(html.contains("Claude Reviewer"));
    assert!(html.contains("Agy Researcher"));
    assert!(html.contains("Hermes Tool"));
    assert!(html.contains("Pi Refactor"));
}

#[tokio::test]
async fn test_topcoat_server_push_websocket_stream() {
    let bus = EventBus::new(EventBusConfig::default());
    let config = ServerConfig::default();
    let cx = Cx::new(bus.clone(), None, config);
    let server = TopcoatServer::new(cx);

    let received = Arc::new(AtomicBool::new(false));
    let received_clone = Arc::clone(&received);

    let _handle = server.push_manager().spawn_push_loop(move |msg| {
        if msg.contains("AgentStarted") {
            received_clone.store(true, Ordering::SeqCst);
        }
        true
    });

    // Emit event through the EventBus
    bus.emit(
        "run-101",
        RuntimeEvent::AgentStarted {
            run_id: "run-101".into(),
            task_id: Some("task-1".into()),
            agent_id: "agent-codex".into(),
            agent_name: "Codex Worker".into(),
        },
    )
    .await
    .expect("Publish should succeed");

    // Wait briefly for server-push dispatch
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert!(
        received.load(Ordering::SeqCst),
        "Server-push should deliver real-time event"
    );
    assert!(server.push_manager().total_pushed_count() > 0);
}

#[tokio::test]
async fn test_topcoat_hitl_approval_interaction() {
    let bus = EventBus::new(EventBusConfig::default());
    let cx = Cx::new(bus, None, ServerConfig::default());
    let server = TopcoatServer::new(cx);

    let req_approve = HitlApprovalRequest {
        execution_id: "exec-99".into(),
        node_id: "deploy_node".into(),
        prompt: "Deploy to production".into(),
        approved: true,
        reason: None,
    };

    let resp = server.handle_hitl_approval(req_approve).await;
    assert!(resp.success);
    assert_eq!(resp.decision, "approved");
    assert!(resp.message.contains("Approved step 'deploy_node'"));

    let req_reject = HitlApprovalRequest {
        execution_id: "exec-99".into(),
        node_id: "delete_db".into(),
        prompt: "Delete database table".into(),
        approved: false,
        reason: Some("High security risk".into()),
    };

    let resp_rej = server.handle_hitl_approval(req_reject).await;
    assert!(resp_rej.success);
    assert_eq!(resp_rej.decision, "rejected");
    assert!(resp_rej.message.contains("High security risk"));
}
