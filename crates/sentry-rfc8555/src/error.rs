use sentry_jose::error::KeyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpClientError {
    #[error("{0}")]
    UnableToBuildClient(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum DefaultOperationsError {
    #[error("Jws Signing failed: {0}")]
    JswSignatureFailed(KeyError),
    #[error("Error reading response body: {0}")]
    ReadResponseBodyError(reqwest::Error),
    #[error("Request returned unexpected state - status: {0} - body: {1}")]
    UnexpectedStatus(reqwest::StatusCode, serde_json::Value),
    #[error("Error deserializing response body: {0}")]
    DeserializationError(serde_json::Error),
}

#[derive(Error, Debug)]
pub enum AcmeIdempotencyError {
    #[error("Unable to fetch nonce: {0}")]
    FailedToFetchNonce(reqwest::Error),
    #[error("Nonce header not present in response!")]
    FailedToLocateNonceInResponse,
    #[error("Nonce header value could not be parsed: {0}")]
    InvalidNonceValue(reqwest::header::ToStrError),
    #[error("Call failed due to invalid nonce: {0}")]
    InvalidNonce(String),
}

#[derive(Error, Debug)]
pub enum AcmeAccountError {
    #[error("Idempotency error in acme account operations: {0}")]
    IdempotencyError(#[from] AcmeIdempotencyError),
    #[error("Error building http client in acme account operations: {0}")]
    RequestClientError(#[from] HttpClientError),
    #[error("Error during a default operation: {0}")]
    DefaultMethodError(#[from] DefaultOperationsError),

    #[error("account information POST-AS-GET failed: {0}")]
    AccountInformationError(reqwest::Error),
    #[error("Failed to derive Jwk: {0}")]
    KeyDerivationError(KeyError),
    #[error("new account request POST failed: {0}")]
    NewAccountError(reqwest::Error),
    #[error("new account response is missing Location header")]
    MissingLocationHeader,
}

#[derive(Error, Debug)]
pub enum OrderOperationsError {
    #[error("Idempotency error in acme order operations: {0}")]
    IdempotencyError(#[from] AcmeIdempotencyError),
    #[error("Error building http client in acme order operations: {0}")]
    RequestClientError(#[from] HttpClientError),
    #[error("Error during a default operation: {0}")]
    DefaultMethodError(#[from] DefaultOperationsError),

    #[error("new order request POST failed: {0}")]
    NewOrderError(reqwest::Error),
    #[error("new order response is missing Location header")]
    MissingLocationHeader,
    #[error("new order returned {0} for domain '{1}': {2}")]
    OrderStatusError(reqwest::StatusCode, String, serde_json::Value),
    #[error("new order response is missing authorizations array")]
    MissingAuthorizationResponseError,
    #[error("new order response is missing finalize url")]
    MissingFinalizeUrl,
}

#[derive(Error, Debug)]
pub enum ChallengeOperationsError {
    #[error("Idempotency error in acme challenge operations: {0}")]
    IdempotencyError(#[from] AcmeIdempotencyError),
    #[error("Error building http client in acme challenge operations: {0}")]
    RequestClientError(#[from] HttpClientError),
    #[error("Error during a default operation: {0}")]
    DefaultMethodError(#[from] DefaultOperationsError),

    #[error("authorization fetch failed: {0}")]
    AuthorizationFetchError(reqwest::Error),
    #[error("authorization response missing challenges array")]
    MissingChallengesArrayError,
    #[error("no {0} challenge found in authorization")]
    ChallengeTypeNotFound(String),
    #[error("{0} challenge missing token")]
    MissingChallengeTokenError(String),
    #[error("{0} challenge missing url")]
    MissingChallengeUrlError(String),
    #[error("Failed to compute Jwk thumbprint: {0}")]
    JwkThumbprintError(KeyError),

    #[error("challenge notify POST failed: {0}")]
    ChallengeNotificationError(reqwest::Error),
    #[error("could not build CSR for domain: '{0}'")]
    CrsGenerationError(String),

    #[error("finalize POST failed: {0}")]
    FinalizationRequestError(reqwest::Error),
    #[error("finalization returned error: {0}")]
    FinalizeResponseError(String),
}

#[derive(Error, Debug)]
pub enum CertificateOperationsError {
    #[error("Idempotency error in acme certificate operations: {0}")]
    IdempotencyError(#[from] AcmeIdempotencyError),
    #[error("Error building http client in acme certificate operations: {0}")]
    RequestClientError(#[from] HttpClientError),
    #[error("Error during a default operation: {0}")]
    DefaultMethodError(#[from] DefaultOperationsError),

    #[error("order poll POST failed: {0}")]
    OrderPollRequestError(reqwest::Error),
    #[error("order is valid but has no certificate url in body")]
    OrderMissingCertificateUrl,
    #[error("order poll returned unexpected status: '{0}'")]
    UnexpectedOrderStatusError(String),
    #[error("order poll response missing 'status' field")]
    MissingOrderStatusField,

    #[error("certificate download POST failed: {0}")]
    CertificateDownloadFailed(reqwest::Error),
    #[error("certificate download returned error: {0}")]
    CertificateDownloadResponseError(String),
    #[error("failed to read certificate bytes from response: {0}")]
    CertificateBytesError(reqwest::Error),
}

#[allow(unused)]
#[derive(Error, Debug)]
pub enum DirectoryOperationsError {
    #[error("Error building http client in acme certificate operations: {0}")]
    RequestClientError(#[from] HttpClientError),
    #[error("Error during a default operation: {0}")]
    DefaultMethodError(#[from] DefaultOperationsError),

    #[error("error parsing url: {1} - {0}")]
    UrlParsingError(String, String),
    #[error("error calling directory path: {0}")]
    DirectoryCallError(String),
    #[error("error parsing directory response: {0}")]
    DirectoryResponseParseError(reqwest::Error),
    #[error("missing {0} endpoint in response")]
    MissingResourceError(String),
}
