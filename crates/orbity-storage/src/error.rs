//! Storage errors.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Audit verification failed at sequence {sequence_num} (event {event_id}): expected hash {expected_hash}, found {actual_hash}")]
    AuditTampered {
        event_id: String,
        sequence_num: i64,
        expected_hash: String,
        actual_hash: String,
    },

    #[error("Invalid sequence number: expected {expected}, got {actual}")]
    InvalidSequence { expected: i64, actual: i64 },

    #[error("Invalid previous hash: expected {expected}, got {actual}")]
    InvalidPreviousHash { expected: String, actual: String },

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}
