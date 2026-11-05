use crate::client::build_client;
use crate::default::DefaultOperations;
use crate::error::{CertificateOperationsError, DefaultOperationsError};
use crate::idempotency::fetch_nonce;
use crate::jws::try_validate_jws_response;
use sentry_jose::keys::PrivateKey;
use serde_json::Value;
use tracing::info;

pub struct CertificateOperations {
    private_key: PrivateKey,
    account_url: String,
    client: reqwest::Client,
}

impl DefaultOperations for CertificateOperations {
    fn get_account_url(&self) -> String {
        self.account_url.clone()
    }

    fn get_private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }
}

impl CertificateOperations {
    pub fn new(
        private_key: PrivateKey,
        account_url: String,
        accept_invalid_certs: bool,
    ) -> Result<CertificateOperations, CertificateOperationsError> {
        let client = build_client(accept_invalid_certs)?;
        Ok(CertificateOperations {
            client,
            private_key,
            account_url,
        })
    }

    pub async fn poll_order_certificate_url(
        &self,
        new_nonce_url: &str,
        order_url: &str,
    ) -> Result<Option<String>, CertificateOperationsError> {
        let nonce = fetch_nonce(new_nonce_url, &self.client).await?;
        let protected = self.protected_kid(nonce, order_url);
        let jws_body = self.build_jws_body_kid_post_as_get(protected)?;

        let resp = self
            .client
            .post(order_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| CertificateOperationsError::OrderPollRequestError(e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| DefaultOperationsError::ReadResponseBodyError(e))?;

        try_validate_jws_response(&body);

        match body["status"].as_str() {
            Some("valid") => {
                let cert_url = body["certificate"]
                    .as_str()
                    .ok_or_else(|| CertificateOperationsError::OrderMissingCertificateUrl)?
                    .to_owned();
                info!("Order is valid. Certificate URL: {}", cert_url);
                Ok(Some(cert_url))
            }
            Some("processing") => {
                info!("Order is still processing.");
                Ok(None)
            }
            Some(other) => Err(CertificateOperationsError::UnexpectedOrderStatusError(
                other.to_string(),
            )),
            None => Err(CertificateOperationsError::MissingOrderStatusField),
        }
    }

    pub async fn download_certificate(
        &self,
        new_nonce_url: &str,
        cert_url: &str,
    ) -> Result<Vec<u8>, CertificateOperationsError> {
        let nonce = fetch_nonce(new_nonce_url, &self.client).await?;
        let protected = self.protected_kid(nonce, cert_url);
        let jws_body = self.build_jws_body_kid_post_as_get(protected)?;

        info!("Downloading certificate from {}", cert_url);
        let resp = self
            .client
            .post(cert_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| CertificateOperationsError::CertificateDownloadFailed(e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CertificateOperationsError::CertificateDownloadResponseError(body));
        }

        Ok(resp
            .bytes()
            .await
            .map_err(|e| CertificateOperationsError::CertificateBytesError(e))?
            .to_vec())
    }
}
