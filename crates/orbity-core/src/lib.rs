//! Orbity Core (v0.1.0-beta) - Foundation types, structured events, FinOps domain and contracts.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const STAGE: &str = "beta";

pub mod bus;
pub mod contracts;
pub mod events;
pub mod execution_log;
pub mod finops;
pub mod runtime_log;
pub mod security;
