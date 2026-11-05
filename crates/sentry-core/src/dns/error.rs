use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum DnsServiceError {
    #[error("Internal error in dns service :: {0}")]
    InternalError(String),
    #[error("Command panic in dns service :: {0}")]
    DigPanic(JoinError),
}
