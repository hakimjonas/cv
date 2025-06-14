/*!
 * Custom error types for database operations
 */

use thiserror::Error;

/// Database error types
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Connection error
    #[error("Database connection error: {0}")]
    Connection(String),

    /// Query error
    #[error("Database query error: {0}")]
    Query(String),

    /// Transaction error
    #[error("Database transaction error: {0}")]
    Transaction(String),

    /// Migration error
    #[error("Database migration error: {0}")]
    Migration(String),

    /// Data error
    #[error("Database data error: {0}")]
    Data(String),

    /// Locking error
    #[error("Database locking error: {0}")]
    Locking(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(error, _) => {
                if error.code == rusqlite::ffi::ErrorCode::DatabaseBusy
                    || error.code == rusqlite::ffi::ErrorCode::DatabaseLocked
                {
                    DatabaseError::Locking(error.to_string())
                } else {
                    DatabaseError::Query(error.to_string())
                }
            }
            rusqlite::Error::QueryReturnedNoRows => {
                DatabaseError::NotFound("Query returned no rows".to_string())
            }
            _ => DatabaseError::Query(error.to_string()),
        }
    }
}

impl From<r2d2::Error> for DatabaseError {
    fn from(error: r2d2::Error) -> Self {
        DatabaseError::Connection(error.to_string())
    }
}

// This implementation is not needed as anyhow::Error already implements From<E> for any E that implements std::error::Error
// impl From<DatabaseError> for anyhow::Error {
//     fn from(error: DatabaseError) -> Self {
//         anyhow::anyhow!("{}", error)
//     }
// }
