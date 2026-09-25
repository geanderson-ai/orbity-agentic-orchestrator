//! Tracing subscriber setup and initialization for Orbity.

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

/// Error types for telemetry initialization.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Failed to initialize tracing subscriber: {0}")]
    InitFailed(String),
}

/// Initializes the global tracing subscriber with standard env filter and formatting.
pub fn init_subscriber(json_format: bool) -> Result<(), TelemetryError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,orbity=debug"));

    if json_format {
        let fmt_layer = fmt::layer().json().flatten_event(true);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| TelemetryError::InitFailed(e.to_string()))?;
    } else {
        let fmt_layer = fmt::layer().with_target(true).with_thread_ids(false);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| TelemetryError::InitFailed(e.to_string()))?;
    }

    Ok(())
}
