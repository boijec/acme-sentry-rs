use base64::DecodeError;
use openssl::error::ErrorStack;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("{0}")]
    DecodingError(#[from] DecodeError),
}

#[derive(Error, Debug)]
pub enum CommonsSslError {
    #[error("Error while getting big number: {0}")]
    BigNumError(ErrorStack),
    #[error("Error while getting big number context: {0}")]
    BigNumContextError(ErrorStack),
    #[error("Unable to get coordinate bytes from slice to big num: {1} - {0}")]
    BigNumFromSliceError(ErrorStack, String),
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Unsupported hashing operation: {0}")]
    UnsupportedHashError(String),
    #[error("Unknown curve: {0}")]
    UnknownCurveError(String),
    #[error("Unknown kty: {0}")]
    UnknownKtyError(String),
    #[error("Unknown key type: {0}")]
    UnknownKeyTypeError(String),
    #[error("Coordinate size is not available for key type: {0}")]
    CoordinateSizeError(String),
    #[error("Algorithm {0} is not supported")]
    AlgorithmSupportError(String),
}