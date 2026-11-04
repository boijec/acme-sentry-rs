use sentry_db::error::DatabaseError;
use sentry_io::fs::FileSystemError;
use sentry_jose::error::KeyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirectoryServiceError {
    #[error("File system error in directory service: {0}")]
    FileSystemError(#[from] FileSystemError),
    #[error("Database error in directory service: {0}")]
    DirectoryDatabaseError(#[from] DatabaseError),
    #[error("Key error in directory service: {0}")]
    DirectoryKeyError(#[from] KeyError),
    #[error("Internal error in directory service: {0}")]
    InternalError(String),
}