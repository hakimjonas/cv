/*!
 * Custom error types for blog operations
 */

use crate::db::error::DatabaseError;
use rusqlite::Connection;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error;
use tokio::task::JoinError;

/// Blog error types
#[derive(Debug, Error)]
pub enum BlogError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Mutex lock error
    #[error("Mutex lock error: {0}")]
    MutexLock(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for blog operations
pub type Result<T> = std::result::Result<T, BlogError>;

impl From<serde_json::Error> for BlogError {
    fn from(error: serde_json::Error) -> Self {
        if error.is_data() {
            BlogError::Deserialization(error.to_string())
        } else {
            BlogError::Serialization(error.to_string())
        }
    }
}

impl From<rusqlite::Error> for BlogError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::QueryReturnedNoRows => {
                BlogError::NotFound("Query returned no rows".to_string())
            }
            _ => BlogError::Database(DatabaseError::from(error)),
        }
    }
}

impl From<r2d2::Error> for BlogError {
    fn from(error: r2d2::Error) -> Self {
        BlogError::Database(DatabaseError::from(error))
    }
}

impl From<String> for BlogError {
    fn from(error: String) -> Self {
        BlogError::Internal(error)
    }
}

impl From<&str> for BlogError {
    fn from(error: &str) -> Self {
        BlogError::Internal(error.to_string())
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Connection>>> for BlogError {
    fn from(error: PoisonError<MutexGuard<'a, Connection>>) -> Self {
        BlogError::MutexLock(format!("Mutex lock error: {error}"))
    }
}

impl From<anyhow::Error> for BlogError {
    fn from(error: anyhow::Error) -> Self {
        BlogError::Internal(error.to_string())
    }
}

impl From<JoinError> for BlogError {
    fn from(error: JoinError) -> Self {
        BlogError::Internal(format!("Task join error: {error}"))
    }
}
