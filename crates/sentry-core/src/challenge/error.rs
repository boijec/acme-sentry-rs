use crate::error::SharedResourceError;
use sentry_db::error::DatabaseError;
use sentry_io::fs::FileSystemError;
use sentry_jose::error::KeyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChallengeServiceError {
    #[error("File system error in challenge service :: {0}")]
    FileSystemError(#[from] FileSystemError),
    #[error("Database error in challenge service :: {0}")]
    ChallengeDatabaseError(#[from] DatabaseError),
    #[error("Key error in challenge service :: {0}")]
    ChallengeKeyError(#[from] KeyError),
    #[error("Internal error in challenge service :: {0}")]
    InternalError(String),
    #[error("Error while extracting shared resources :: {0}")]
    ResourceError(#[from] SharedResourceError),
}