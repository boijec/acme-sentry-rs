use crate::client::build_client;
use crate::default::DefaultOperations;
use crate::error::{ChallengeOperationsError, DefaultOperationsError};
use crate::idempotency::fetch_nonce;
use crate::jws::try_validate_jws_response;
use sentry_crypto::crypto::sha256_b64;
use sentry_crypto::encoding::encode_b64;
use sentry_jose::csr::build_csr_der;
use sentry_jose::keys::PrivateKey;
use serde_json::{Value, json};
use tracing::{debug, info};

#[derive(Debug)]
pub struct Dns01ChallengeInfo {
    /// The raw token returned by the CA in the challenge object.
    pub token: String,
    /// `token + "." + base64url(sha256(canonical_jwk))` — the full key authorisation string.
    pub key_authorization: String,
    /// The DNS name that must have the TXT record (`_acme-challenge.<domain>.`).
    pub dns_record_name: String,
    /// `base64url(sha256(key_authorization))`.
    pub txt_record_value: String,
    pub challenge_url: String,
}

#[derive(Debug)]
pub struct FinalizeResult {
    /// Some(url) if the CA returned the certificate URL immediately.
    /// None if the order is still in the "processing" state and must be polled.
    pub certificate_url: Option<String>,
}

pub struct ChallengeOperations {
    private_key: PrivateKey,
    account_url: String,
    client: reqwest::Client,
}

impl DefaultOperations for ChallengeOperations {
    fn get_account_url(&self) -> String {
        self.account_url.clone()
    }

    fn get_private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }
}

impl ChallengeOperations {
    pub fn new(
        private_key: PrivateKey,
        account_url: String,
        accept_invalid_certs: bool,
    ) -> Result<ChallengeOperations, ChallengeOperationsError> {
        let client = build_client(accept_invalid_certs)?;
        Ok(ChallengeOperations {
            client,
            private_key,
            account_url,
        })
    }
    pub fn __new(
        private_key: PrivateKey,
        account_url: String,
        client: reqwest::Client,
    ) -> ChallengeOperations {
        ChallengeOperations {
            private_key,
            account_url,
            client,
        }
    }

    pub async fn fetch_dns01_challenge(
        &self,
        new_nonce_url: &str,
        authorization_url: &str,
    ) -> Result<Dns01ChallengeInfo, ChallengeOperationsError> {
        let nonce = fetch_nonce(new_nonce_url, &self.client).await?;
        // POST-as-GET: empty string payload
        let protected = self.protected_kid(nonce, authorization_url);
        let jws_body = self.build_jws_body_kid_post_as_get(protected)?;

        let resp = self
            .client
            .post(authorization_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| ChallengeOperationsError::AuthorizationFetchError(e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| DefaultOperationsError::ReadResponseBodyError(e))?;

        try_validate_jws_response(&body);
        debug!("Authorization response: {}", body);

        let challenges = body["challenges"]
            .as_array()
            .ok_or_else(|| ChallengeOperationsError::MissingChallengesArrayError)?;

        let dns_challenge = challenges
            .iter()
            .find(|c| c["type"].as_str() == Some("dns-01"))
            .ok_or_else(|| ChallengeOperationsError::ChallengeTypeNotFound("dns-01".into()))?;

        let token = dns_challenge["token"]
            .as_str()
            .ok_or_else(|| ChallengeOperationsError::MissingChallengeTokenError("dns-01".into()))?
            .to_owned();

        let challenge_url = dns_challenge["url"]
            .as_str()
            .ok_or_else(|| ChallengeOperationsError::MissingChallengeUrlError("dns-01".into()))?
            .to_owned();

        let thumbprint = self
            .private_key
            .jwk_thumbprint()
            .map_err(|e| ChallengeOperationsError::JwkThumbprintError(e))?;
        let key_authorization = format!("{}.{}", token, thumbprint);

        // TXT record value: base64url(sha256(key_authorization))
        let txt_record_value = sha256_b64(key_authorization.as_bytes());

        let domain = body["identifier"]["value"]
            .as_str()
            .unwrap_or("unknown");
        let dns_record_name = format!("_acme-challenge.{}.", domain);

        Ok(Dns01ChallengeInfo {
            token,
            key_authorization,
            dns_record_name,
            txt_record_value,
            challenge_url,
        })
    }

    pub async fn notify_challenge_ready(
        &self,
        new_nonce_url: &str,
        challenge_url: &str,
    ) -> Result<(), ChallengeOperationsError> {
        let nonce = fetch_nonce(new_nonce_url, &self.client).await?;
        let protected = self.protected_kid(nonce, challenge_url);
        // RFC 8555: payload must be `{}` (empty JSON object) when responding to a challenge
        let payload = json!({});
        let jws_body = self.build_jws_body_kid(protected, &payload)?;

        info!("Notifying CA that challenge is ready: {}", challenge_url);
        let resp = self
            .client
            .post(challenge_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| ChallengeOperationsError::ChallengeNotificationError(e))?;

        let body: Value = resp
            .json()
            .await
            .unwrap_or(Value::Null);

        try_validate_jws_response(&body);
        debug!("Challenge notify response: {}", body);
        Ok(())
    }

    pub async fn finalize(
        &self,
        new_nonce_url: &str,
        finalize_url: &str,
        domain: &str,
        certificate_private_key: &PrivateKey,
    ) -> Result<FinalizeResult, ChallengeOperationsError> {
        let csr_der = build_csr_der(domain, certificate_private_key)
            .map_err(|_| ChallengeOperationsError::CrsGenerationError(domain.to_string()))?;
        let csr_b64 = encode_b64(&csr_der);

        let nonce = fetch_nonce(new_nonce_url, &self.client).await?;
        let protected = self.protected_kid(nonce, finalize_url);
        let payload = json!({ "csr": csr_b64 });
        let jws_body = self.build_jws_body_kid(protected, &payload)?;

        info!("Sending finalize request to {} for domain '{}'", finalize_url, domain);
        let resp = self
            .client
            .post(finalize_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| ChallengeOperationsError::FinalizationRequestError(e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ChallengeOperationsError::FinalizeResponseError(body));
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|e| DefaultOperationsError::ReadResponseBodyError(e))?;

        try_validate_jws_response(&body);
        debug!("Finalize response: {}", body);

        let certificate_url = body["certificate"].as_str().map(|s| s.to_owned());
        Ok(FinalizeResult { certificate_url })
    }
}