use crate::crypto::SupportedKey;
use base64::DecodeError;
use openssl::error::ErrorStack;
use std::ffi::c_int;
use std::string::FromUtf8Error;
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
pub enum KeyError {
    #[error("Could not generate RSA key with size: {1}")]
    RsaGenerationError(ErrorStack, u32),
    #[error("Could not instantiate private key from type {1}")]
    PrivateKeyFromConversionError(ErrorStack, String),
    #[error("Could not generate key with Nid: {1}")]
    EcCurveError(ErrorStack, c_int),
    #[error("Could not generate key from supported type: {1}")]
    EcGenerationError(ErrorStack, SupportedKey),
    #[error("Could not generate key from supported type: {1}")]
    EdGenerationError(ErrorStack, SupportedKey),
    #[error("Error during JWK canonicalization: {0}")]
    JWKFormattingError(String),
    #[error("Error signing data with private key type: {1}")]
    KeySigningError(ErrorStack, String),
    #[error("Error while creating signer private key type: {1}")]
    KeySignerError(ErrorStack, String),
    #[error("Key operations threw crypto error: {0}")]
    KeyCryptoError(#[from] CryptoError),
    #[error("Unable to read key from pem bytes: {0}")]
    UnableToReadKeyFromPemError(ErrorStack),
    #[error("Unable to read pem from utf-8 string: {0}")]
    UnableToReadPemFromUtf8Error(FromUtf8Error),
    #[error("Unable to cast key type to: {1} - {0}")]
    UnableToCast(ErrorStack, String),
    #[error("Error while serializing public key: {1} to pem - {0}")]
    UnableToSerializeToPem(ErrorStack, String),
    #[error("Error during big number operation: {0}")]
    BigNumError(#[from] CommonsSslError),
    #[error("Error while encoding key bytes: {0}")]
    KeyEncodingError(#[from] DecodeError),
    #[error("Error while padding vector: {0}")]
    UnableToPadVector(ErrorStack),
    #[error("Error during computing public key coordinates: {0}")]
    AffineCoordinatesError(ErrorStack),
    #[error("Error while getting Ec Group from Nid: {1} - {0}")]
    EcGroupError(ErrorStack, c_int),
}

#[derive(Error, Debug)]
pub enum JwkError {
    #[error("Serde serialization / deserialization error for JWK: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Unknown kty for JWK: {0}")]
    UnknownKeyTypeError(String),
    #[error("Error during big number operation: {0}")]
    BigNumError(#[from] CommonsSslError),
    #[error("Unable to compute public key from public key components - {0}")]
    PublicKeyComponentsError(ErrorStack),
    #[error("Key operations error during JWK parse / validation: {0}")]
    KeyOperationsError(#[from] KeyError),
    #[error("Encoding error while attempting to decode key parts to bytes: {0}")]
    ComponentEncodingError(#[from] EncodingError),
    #[error("JWK and KID parameter not found in flat JWS protected header")]
    JwkKidNotFoundError,
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

#[derive(Error, Debug)]
pub enum JwsError {
    #[error("{0}")]
    PrivateKeyComponentsError(ErrorStack),
    #[error("{0}")]
    KeyOperationsError(#[from] KeyError),
    #[error("{0}")]
    CommonsSslError(#[from] CommonsSslError),
    #[error("{0}")]
    EncodingError(#[from] EncodingError),
    #[error("{0}")]
    StringUtf8Error(#[from] FromUtf8Error),
    #[error("{0}")]
    JwkOperationError(#[from] JwkError),
    #[error("{0}")]
    JwsElementParseError(String),
    #[error("Serde serialization / deserialization error for JWS: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Invalid raw EdDSA signature length: expected {0} bytes ({1} per coordinate), got {2}")]
    RawEcdsaSignatureLengthError(usize, usize, usize),
    #[error("{0} signature verification failed")]
    JwsSignatureVerificationError(String),
    #[error("{0} signature update failed")]
    JwsSignatureUpdateError(ErrorStack, String),
    #[error("{0} verifier failed")]
    JwsSignatureVerifierError(ErrorStack, String),
}