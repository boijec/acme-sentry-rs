use std::io::Read;
use serde_json::{json, Value};
use tracing::{debug, info};
use sentry_jose::keys::PrivateKey;
use crate::challenge::ChallengeOperations;
use crate::client::build_client;
use crate::default::{ConvertOperation, DefaultOperations};
use crate::error::{DefaultOperationsError, OrderOperationsError};
use crate::idempotency::fetch_nonce;
use crate::jws::try_validate_jws_response;

#[derive(Debug)]
pub struct PlaceOrderResult {
    pub order_url: String,
    pub authorization_url: String,
    pub finalize_url: String,
}

pub struct OrderOperations {
    private_key: PrivateKey,
    account_url: String,
    client: reqwest::Client,
}

impl DefaultOperations for OrderOperations {
    fn get_account_url(&self) -> String {
        self.account_url.clone()
    }

    fn get_private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }
}

impl OrderOperations {
    pub fn new(
        private_key: PrivateKey,
        account_url: String,
        accept_invalid_certs: bool,
    ) -> Result<OrderOperations, OrderOperationsError> {
        let client = build_client(accept_invalid_certs)?;
        Ok(OrderOperations {
            client,
            private_key,
            account_url,
        })
    }

    pub async fn place_order(
        &self,
        new_nonce_url: &str,
        new_order_url: &str,
        domain: &str,
    ) -> Result<PlaceOrderResult, OrderOperationsError> {
        let nonce = fetch_nonce(new_nonce_url, &self.client).await?;
        let protected = self.protected_kid(nonce, new_order_url);
        let payload = json!({
            "identifiers": [{ "type": "dns", "value": domain }]
        });
        let jws_body = self.build_jws_body_kid(protected, &payload)?;

        info!("Sending newOrder request for domain '{}' to {}", domain, new_order_url);
        let resp = self
            .client
            .post(new_order_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| OrderOperationsError::NewOrderError(e))?;

        let status = resp.status();
        let order_url = resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .ok_or_else(|| OrderOperationsError::MissingLocationHeader)?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| DefaultOperationsError::ReadResponseBodyError(e))?;

        try_validate_jws_response(&body);
        debug!("newOrder response: {}", body);

        if !status.is_success() {
            return Err(OrderOperationsError::OrderStatusError(
                status,
                domain.to_string(),
                body
            ));
        }

        let authorization_url = body["authorizations"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .ok_or_else(|| OrderOperationsError::MissingAuthorizationResponseError)?
            .to_owned();

        let finalize_url = body["finalize"]
            .as_str()
            .ok_or_else(|| OrderOperationsError::MissingFinalizeUrl)?
            .to_owned();

        info!(
            "Order placed for '{}'. order_url={}, finalize_url={}",
            domain, order_url, finalize_url
        );

        Ok(PlaceOrderResult {
            order_url,
            authorization_url,
            finalize_url,
        })
    }
}

impl ConvertOperation<ChallengeOperations> for OrderOperations {
    fn convert(self) -> ChallengeOperations {
        ChallengeOperations::__new(
            self.private_key,
            self.account_url,
            self.client
        )
    }
}