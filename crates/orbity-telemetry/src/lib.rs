//! Orbity Telemetry (v0.1.0-beta) - 4-layer tracing, spans, subscribers and OpenTelemetry metrics.

pub mod metrics;
pub mod setup;
pub mod sink;
pub mod spans;

pub use metrics::TelemetryMetrics;
pub use setup::init_subscriber;
pub use sink::TelemetrySink;
pub use spans::{SpanRecord, SpanStatus, SpanTree};
