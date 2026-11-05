use crate::client::build_client;
use crate::default::DefaultOperations;
use crate::error::{AcmeAccountError, DefaultOperationsError};
use crate::idempotency::fetch_nonce;
use crate::jws::{
    build_jws_body_jwk,
    try_validate_jws_response,
};
use sentry_jose::jwk::GenericJwk;
use sentry_jose::keys::PrivateKey;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{debug, info};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountResponse {
    pub key: GenericJwk,
    pub orders: String,
    pub status: String,
}

pub struct AccountOperations {
    private_key: PrivateKey,
    account_url: String,
    client: reqwest::Client,
}

impl DefaultOperations for AccountOperations {
    fn get_account_url(&self) -> String {
        self.account_url.clone()
    }

    fn get_private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }
}

impl AccountOperations {
    pub fn new(
        private_key: PrivateKey,
        account_url: String,
        accept_invalid_certs: bool,
    ) -> Result<AccountOperations, AcmeAccountError> {
        let client = build_client(accept_invalid_certs)?;
        Ok(AccountOperations {
            client,
            private_key,
            account_url,
        })
    }

    pub async fn get_account(
        &self,
        new_nonce_url: &str,
    ) -> Result<AccountResponse, AcmeAccountError> {
        let nonce = fetch_nonce(new_nonce_url, &self.client).await?;
        let protected = self.protected_kid(nonce, self.account_url.as_str());
        let jws_body = self.build_jws_body_kid_post_as_get(protected)?;

        info!("Sending account request to {}", &self.account_url);
        let resp = self
            .client
            .post(self.account_url.clone())
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| AcmeAccountError::AccountInformationError(e))?;

        let status = resp.status();

        let body: Value = resp
            .json()
            .await
            .map_err(|e| DefaultOperationsError::ReadResponseBodyError(e))?;

        try_validate_jws_response(&body);
        match status {
            reqwest::StatusCode::OK => {
                debug!("Account information retrieved");
            }
            _ => {
                return Err(DefaultOperationsError::UnexpectedStatus(
                    status,
                    body
                ).into());
            }
        }
        let acc = serde_json::from_value(body)
            .map_err(|e| DefaultOperationsError::DeserializationError(e))?;
        Ok(acc)
    }
}

pub async fn login_or_register(
    private_key: &PrivateKey,
    new_nonce_url: &str,
    new_account_url: &str,
    accept_invalid_certs: bool,
) -> Result<String, AcmeAccountError> {
    let http = build_client(accept_invalid_certs)?;

    let nonce = fetch_nonce(new_nonce_url, &http).await?;
    let alg = private_key.kt.get_key_alg();
    let jwk = private_key
        .get_jwk()
        .map_err(|e| AcmeAccountError::KeyDerivationError(e))?;

    let protected = json!({
        "alg":   alg.to_string(),
        "jwk":   jwk,
        "nonce": nonce,
        "url":   new_account_url,
    });
    let payload = json!({ "termsOfServiceAgreed": true });
    let jws_body = build_jws_body_jwk(protected, &payload, private_key)?;

    info!("Sending newAccount request to {}", new_account_url);
    let resp = http
        .post(new_account_url)
        .header("Content-Type", "application/jose+json")
        .json(&jws_body)
        .send()
        .await
        .map_err(|e| AcmeAccountError::NewAccountError(e))?;

    let status = resp.status();
    let account_url = resp
        .headers()
        .get("Location")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    let body: Value = resp
        .json()
        .await
        .map_err(|e| DefaultOperationsError::ReadResponseBodyError(e))?;

    try_validate_jws_response(&body);
    debug!("newAccount response body: {}", body);

    match status {
        reqwest::StatusCode::OK => {
            info!(
                "Existing account found. Account URL: {}",
                account_url.as_deref().unwrap_or("<none>")
            );
        }
        reqwest::StatusCode::CREATED => {
            info!(
                "New account created. Account URL: {}",
                account_url.as_deref().unwrap_or("<none>")
            );
        }
        _ => {
            return Err(DefaultOperationsError::UnexpectedStatus(
                status,
                body
            ).into());
        }
    }

    account_url.ok_or_else(|| AcmeAccountError::MissingLocationHeader)
}
