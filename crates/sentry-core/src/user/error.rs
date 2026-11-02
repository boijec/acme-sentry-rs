use thiserror::Error;
use sentry_crypto::error::CryptoError;
use sentry_db::error::DatabaseError;
use sentry_io::fs::FileSystemError;
use sentry_jose::error::KeyError;

#[derive(Error, Debug)]
pub enum InternalUserServiceError {
    #[error("File system error in user service: {0}")]
    FileSystemError(#[from] FileSystemError),
    #[error("Database error in user service: {0}")]
    UserDatabaseError(#[from] DatabaseError),
    #[error("Key error in user service: {0}")]
    UserKeyError(#[from] KeyError),
    #[error("Key crypto error in user service: {0}")]
    UserKeyCryptoError(#[from] CryptoError),
}