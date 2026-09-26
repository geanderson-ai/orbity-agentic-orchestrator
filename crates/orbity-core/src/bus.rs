//! Event Bus & Multi-Sink Dispatcher for structured runtime events.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

use crate::events::{EventEnvelope, RuntimeEvent};

/// Error types for Event Sink operations.
#[derive(Debug, thiserror::Error)]
pub enum EventSinkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Sink delivery failed: {0}")]
    Delivery(String),
}

/// Error types for Event Bus operations.
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Bus channel full, event dropped: {0}")]
    ChannelFull(String),

    #[error("Bus closed: {0}")]
    BusClosed(String),

    #[error("Sink error in '{sink}': {source}")]
    SinkFailure {
        sink: String,
        source: EventSinkError,
    },
}

/// Trait implemented by destinations that receive structured runtime events.
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Friendly name of this sink (e.g. "stdout", "file-jsonl", "sqlite-audit").
    fn name(&self) -> &str;

    /// Dispatches a single structured event envelope.
    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError>;

    /// Optional flush hook for buffered sinks.
    async fn flush(&self) -> Result<(), EventSinkError> {
        Ok(())
    }
}

/// In-memory event sink for testing and inspection.
#[derive(Debug, Clone, Default)]
pub struct InMemorySink {
    events: Arc<Mutex<Vec<EventEnvelope>>>,
}

impl InMemorySink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn events(&self) -> Vec<EventEnvelope> {
        self.events.lock().await.clone()
    }

    pub async fn count(&self) -> usize {
        self.events.lock().await.len()
    }

    pub async fn clear(&self) {
        self.events.lock().await.clear();
    }
}

#[async_trait]
impl EventSink for InMemorySink {
    fn name(&self) -> &str {
        "in-memory"
    }

    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError> {
        self.events.lock().await.push(event.clone());
        Ok(())
    }
}

/// File-based JSON Lines (.jsonl) event sink with append mode.
#[derive(Debug)]
pub struct JsonLinesSink {
    name: String,
    path: PathBuf,
    file_lock: Mutex<()>,
}

impl JsonLinesSink {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(Self {
            name: "file-jsonl".to_string(),
            path,
            file_lock: Mutex::new(()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl EventSink for JsonLinesSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError> {
        use tokio::io::AsyncWriteExt;
        let _guard = self.file_lock.lock().await;

        let mut json = serde_json::to_string(event)?;
        json.push('\n');

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        file.write_all(json.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }
}

/// Stdout JSON lines event sink.
#[derive(Debug, Default)]
pub struct StdoutSink {
    name: String,
}

impl StdoutSink {
    pub fn new() -> Self {
        Self {
            name: "stdout".to_string(),
        }
    }
}

#[async_trait]
impl EventSink for StdoutSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: &EventEnvelope) -> Result<(), EventSinkError> {
        let json = serde_json::to_string(event)?;
        println!("{}", json);
        Ok(())
    }
}

/// Configuration options for the asynchronous Event Bus.
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub queue_capacity: usize,
    pub broadcast_capacity: usize,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 16_384,
            broadcast_capacity: 8_192,
        }
    }
}

/// High-throughput Asynchronous Event Bus.
///
/// Distributes `EventEnvelope` instances to:
/// 1. Broadcast subscribers (e.g. SSE/WebSocket streams or TUI receivers)
/// 2. Registered `EventSink` instances (Stdout, JSONL, SQLite Audit, Telemetry)
#[derive(Clone)]
pub struct EventBus {
    config: EventBusConfig,

    tx_queue: mpsc::Sender<EventEnvelope>,
    tx_broadcast: broadcast::Sender<EventEnvelope>,
    sinks: Arc<RwLock<Vec<Arc<dyn EventSink>>>>,
    events_published: Arc<AtomicU64>,
    is_closed: Arc<AtomicBool>,
}

impl EventBus {
    /// Creates and spawns a new Event Bus with background distribution worker.
    pub fn new(config: EventBusConfig) -> Self {
        let (tx_queue, mut rx_queue) = mpsc::channel::<EventEnvelope>(config.queue_capacity);
        let (tx_broadcast, _) = broadcast::channel::<EventEnvelope>(config.broadcast_capacity);
        let sinks: Arc<RwLock<Vec<Arc<dyn EventSink>>>> = Arc::new(RwLock::new(Vec::new()));
        let events_published = Arc::new(AtomicU64::new(0));
        let is_closed = Arc::new(AtomicBool::new(false));

        let sinks_clone = Arc::clone(&sinks);
        let tx_broadcast_clone = tx_broadcast.clone();
        let is_closed_worker = Arc::clone(&is_closed);

        tokio::spawn(async move {
            while let Some(event) = rx_queue.recv().await {
                // Broadcast to real-time channel subscribers (ignoring errors if no active receivers)
                let _ = tx_broadcast_clone.send(event.clone());

                // Dispatch to registered sinks concurrently
                let sinks_snapshot = {
                    let r = sinks_clone.read().await;
                    r.clone()
                };

                for sink in sinks_snapshot {
                    if let Err(err) = sink.send(&event).await {
                        eprintln!(
                            "[orbity-bus] Warning: delivery failure on sink '{}': {}",
                            sink.name(),
                            err
                        );
                    }
                }
            }
            is_closed_worker.store(true, Ordering::SeqCst);
        });

        Self {
            config,
            tx_queue,
            tx_broadcast,
            sinks,
            events_published,
            is_closed,
        }
    }

    /// Subscribes to the broadcast channel for real-time event streaming.
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.tx_broadcast.subscribe()
    }

    /// Registers a new event sink.
    pub async fn register_sink(&self, sink: Arc<dyn EventSink>) {
        let mut w = self.sinks.write().await;
        w.push(sink);
    }

    /// Clears all registered sinks.
    pub async fn clear_sinks(&self) {
        let mut w = self.sinks.write().await;
        w.clear();
    }

    /// Publishes a pre-constructed `EventEnvelope`.
    pub async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventBusError> {
        if self.is_closed.load(Ordering::SeqCst) {
            return Err(EventBusError::BusClosed("Bus is closed".to_string()));
        }

        self.tx_queue
            .send(envelope)
            .await
            .map_err(|e| EventBusError::BusClosed(e.to_string()))?;

        self.events_published.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Convenience helper to create and publish a `RuntimeEvent`.
    pub async fn emit(
        &self,
        run_id: impl Into<String>,
        payload: RuntimeEvent,
    ) -> Result<EventEnvelope, EventBusError> {
        let envelope = EventEnvelope::new(run_id, payload);
        self.publish(envelope.clone()).await?;
        Ok(envelope)
    }

    /// Returns the total number of events published.
    pub fn events_published_count(&self) -> u64 {
        self.events_published.load(Ordering::Relaxed)
    }

    /// Returns configuration details.
    pub fn config(&self) -> &EventBusConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_event_bus_publish_and_in_memory_sink() {
        let bus = EventBus::new(EventBusConfig::default());
        let memory_sink = Arc::new(InMemorySink::new());
        bus.register_sink(memory_sink.clone()).await;

        let run_id = "run-test-123".to_string();
        let event = RuntimeEvent::RunInitiated {
            run_id: run_id.clone(),
            prompt: "Test prompt".to_string(),
            team_name: Some("forester".to_string()),
        };

        bus.emit(run_id.clone(), event)
            .await
            .expect("Failed to emit event");

        // Wait brief moment for worker to dispatch
        sleep(Duration::from_millis(50)).await;

        let events = memory_sink.events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].run_id, run_id);
        assert!(matches!(events[0].event, RuntimeEvent::RunInitiated { .. }));
    }

    #[tokio::test]
    async fn test_event_bus_broadcast_subscription() {
        let bus = EventBus::new(EventBusConfig::default());
        let mut subscriber = bus.subscribe();

        let run_id = "run-broadcast-456".to_string();
        let event = RuntimeEvent::AgentStarted {
            run_id: run_id.clone(),
            task_id: Some("task-01".to_string()),
            agent_id: "agt_01".to_string(),
            agent_name: "codex_01".to_string(),
        };

        bus.emit(run_id.clone(), event)
            .await
            .expect("Failed to emit");

        let received = tokio::time::timeout(Duration::from_millis(200), subscriber.recv())
            .await
            .expect("Timed out waiting for broadcast")
            .expect("Broadcast receive failed");

        assert_eq!(received.run_id, run_id);
        assert!(matches!(received.event, RuntimeEvent::AgentStarted { .. }));
    }

    #[tokio::test]
    async fn test_json_lines_file_sink() {
        let dir = std::env::temp_dir().join(format!("orbity-bus-test-{}", Uuid::new_v4()));
        let file_path = dir.join("events.jsonl");

        let file_sink = Arc::new(JsonLinesSink::new(&file_path).await.unwrap());
        let bus = EventBus::new(EventBusConfig::default());
        bus.register_sink(file_sink).await;

        let run_id = "run-jsonl-789".to_string();
        bus.emit(
            run_id.clone(),
            RuntimeEvent::AgentFinished {
                run_id: run_id.clone(),
                task_id: None,
                agent_id: "hermes_01".to_string(),
                agent_name: "hermes".to_string(),
                summary: Some("Search completed".to_string()),
            },
        )
        .await
        .unwrap();

        sleep(Duration::from_millis(50)).await;

        let contents = tokio::fs::read_to_string(&file_path)
            .await
            .expect("Failed to read jsonl");
        assert!(contents.contains("\"agent_name\":\"hermes\""));
        assert!(contents.contains("\"summary\":\"Search completed\""));

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
