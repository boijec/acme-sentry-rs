use crate::challenge::error::ChallengeServiceError;
use crate::error::SharedResourceError;
use sentry_db::error::DatabaseError;
use sentry_io::fs::FileSystemError;
use sentry_jose::error::KeyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderServiceError {
    #[error("File system error in order service :: {0}")]
    FileSystemError(#[from] FileSystemError),
    #[error("Database error in order service :: {0}")]
    OrderDatabaseError(#[from] DatabaseError),
    #[error("Key error in order service :: {0}")]
    OrderKeyError(#[from] KeyError),
    #[error("Internal error in order service :: {0}")]
    InternalError(String),
    #[error("Error while extracting shared resources :: {0}")]
    ResourceError(#[from] SharedResourceError),
    #[error("Nested service error :: {0}")]
    ChallengeServiceError(#[from] ChallengeServiceError),
}