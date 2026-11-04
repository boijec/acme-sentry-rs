use thiserror::Error;
use sentry_db::error::DatabaseError;
use sentry_io::fs::FileSystemError;
use sentry_jose::error::KeyError;

#[derive(Error, Debug)]
pub enum OrderServiceError {
    #[error("File system error in order service: {0}")]
    FileSystemError(#[from] FileSystemError),
    #[error("Database error in order service: {0}")]
    OrderDatabaseError(#[from] DatabaseError),
    #[error("Key error in order service: {0}")]
    OrderKeyError(#[from] KeyError),
    #[error("Internal error in order service: {0}")]
    InternalError(String),
}