//! Server-Push and WebSockets real-time broadcaster (Topcoat 0.9 native push model).

use crate::context::Cx;
use orbity_core::events::EventEnvelope;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Manages long-lived server-push channels delivering real-time agent updates to browsers.
pub struct ServerPushManager {
    cx: Cx,
    pushed_messages: Arc<AtomicU64>,
}

impl ServerPushManager {
    pub fn new(cx: Cx) -> Self {
        Self {
            cx,
            pushed_messages: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Subscribes a client browser session to server-push deltas.
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.cx.bus().subscribe()
    }

    /// Formats an event envelope as a Server-Sent Event (SSE) or WebSocket text payload.
    pub fn format_sse(event: &EventEnvelope) -> String {
        let json = serde_json::to_string(event).unwrap_or_default();
        format!(
            "event: {}\ndata: {}\n\n",
            event.event.event_type_name(),
            json
        )
    }

    /// Spawns a background loop broadcasting from the EventBus to an active connection sink.
    pub fn spawn_push_loop<F>(&self, mut sender_fn: F) -> tokio::task::JoinHandle<()>
    where
        F: FnMut(String) -> bool + Send + 'static,
    {
        let mut rx = self.cx.bus().subscribe();
        let counter = Arc::clone(&self.pushed_messages);

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let payload = Self::format_sse(&event);
                counter.fetch_add(1, Ordering::Relaxed);
                if !sender_fn(payload) {
                    // Client disconnected
                    break;
                }
            }
        })
    }

    pub fn total_pushed_count(&self) -> u64 {
        self.pushed_messages.load(Ordering::Relaxed)
    }
}
