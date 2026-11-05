use crate::error::AcmeIdempotencyError;
use tracing::debug;

pub async fn fetch_nonce(
    nonce_url: &str,
    http: &reqwest::Client,
) -> Result<String, AcmeIdempotencyError> {
    debug!("Fetching nonce at: {}", nonce_url);
    let resp = http
        .head(nonce_url)
        .send()
        .await
        .map_err(|e| AcmeIdempotencyError::FailedToFetchNonce(e))?;
    resp.headers()
        .get("Replay-Nonce")
        .ok_or_else(|| AcmeIdempotencyError::FailedToLocateNonceInResponse)?
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|e| AcmeIdempotencyError::InvalidNonceValue(e))
}
