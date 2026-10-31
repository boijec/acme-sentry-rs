use thiserror::Error;
use sentry_crypto::error::CryptoError;
use sentry_io::fs::FileSystemError;
use sentry_jose::error::KeyError;

#[derive(Error, Debug)]
pub enum LoadKeyError {
    #[error("Could not determine path of key: {0}")]
    ErrorDeterminingPath(String),
    #[error("Unable to read PEM bytes: {1} - {0}")]
    UnableToReadFromKeyFile(FileSystemError, String),
    #[error("Unsupported key type: {1} - {0}")]
    UnsupportedKeyType(CryptoError, String),
    #[error("Failed to instantiate key bytes: {1} - {0}")]
    FailedToInstantiateKey(KeyError, String),
}

#[derive(Error, Debug)]
pub enum JobBuilderError {
    #[error("Could not create job: {0}")]
    JobCreationError(String),
}

#[derive(Error, Debug)]
pub enum GenericJobError {
    #[error("{0}")]
    GenericError(String),
}

#[derive(Error, Debug)]
pub enum HttpClientError {
    #[error("{0}")]
    UnableToBuildClient(#[from] reqwest::Error),
}