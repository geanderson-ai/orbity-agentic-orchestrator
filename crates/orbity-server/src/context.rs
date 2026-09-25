//! Topcoat Application Context (`Cx`), configuration, and server runtime state.

use orbity_core::bus::EventBus;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Server configuration settings for the Tokio Topcoat web application.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub auth_token: Option<String>,
    pub max_connections: usize,
    pub enable_live_streaming: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:3000".parse().unwrap(),
            auth_token: None,
            max_connections: 10_000,
            enable_live_streaming: true,
        }
    }
}

/// The Topcoat Application Context (`Cx`) passed to pages, shards, and live streams.
#[derive(Clone)]
pub struct Cx {
    pub event_bus: EventBus,
    pub pool: Option<SqlitePool>,
    pub config: ServerConfig,
    state: Arc<RwLock<AppState>>,
}

#[derive(Default)]
struct AppState {
    active_connections: usize,
}

impl Cx {
    pub fn new(event_bus: EventBus, pool: Option<SqlitePool>, config: ServerConfig) -> Self {
        Self {
            event_bus,
            pool,
            config,
            state: Arc::new(RwLock::new(AppState::default())),
        }
    }

    /// Accessor for global event bus in page/component functions.
    pub fn bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Accessor for SQLite pool.
    pub fn pool(&self) -> Option<&SqlitePool> {
        self.pool.as_ref()
    }

    pub async fn increment_connection(&self) -> usize {
        let mut s = self.state.write().await;
        s.active_connections += 1;
        s.active_connections
    }

    pub async fn decrement_connection(&self) -> usize {
        let mut s = self.state.write().await;
        if s.active_connections > 0 {
            s.active_connections -= 1;
        }
        s.active_connections
    }

    pub async fn active_connections(&self) -> usize {
        self.state.read().await.active_connections
    }
}
