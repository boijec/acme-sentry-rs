use crate::csr::build_csr_der;
use crate::encoding::encode_b64;
use crate::jws::{JWS, KeyFetcher};
use crate::keys::PrivateKey;
use openssl::pkey::{PKey, Public};
use serde_json::{json, Value};
use std::error::Error;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct PlaceOrderResult {
    pub order_url: String,
    pub authorization_url: String,
    pub finalize_url: String,
}

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

struct EmbeddedJwkOnly;

impl KeyFetcher for EmbeddedJwkOnly {
    fn fetch_key(&self, kid: String) -> Result<PKey<Public>, Box<dyn Error>> {
        Err(format!(
            "CA JWS response used 'kid' ({}), but only embedded 'jwk' is supported for response validation",
            kid
        )
        .into())
    }
}


pub struct AcmeAccountClient {
    private_key: PrivateKey,
    account_url: String,
    http: reqwest::Client,
}

impl AcmeAccountClient {
    pub fn new(
        private_key: PrivateKey,
        account_url: String,
        accept_invalid_certs: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .build()?;
        Ok(AcmeAccountClient {
            private_key,
            account_url,
            http,
        })
    }

    pub async fn login_or_register(
        private_key: &PrivateKey,
        new_nonce_url: &str,
        new_account_url: &str,
        accept_invalid_certs: bool,
    ) -> anyhow::Result<String> {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .build()?;

        let nonce = fetch_nonce(new_nonce_url, &http).await?;
        let alg = private_key.kt.get_key_alg();
        let jwk = private_key
            .get_jwk()
            .map_err(|e| anyhow::anyhow!("Failed to derive JWK: {}", e))?;

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
            .map_err(|e| anyhow::anyhow!("newAccount POST failed: {}", e))?;

        let status = resp.status();
        let account_url = resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        let body: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read newAccount response body: {}", e))?;

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
                return Err(anyhow::anyhow!(
                    "newAccount returned unexpected status {}: {}",
                    status,
                    body
                ));
            }
        }

        account_url
            .ok_or_else(|| anyhow::anyhow!("newAccount response missing Location header"))
    }

    pub async fn place_order(
        &self,
        new_nonce_url: &str,
        new_order_url: &str,
        domain: &str,
    ) -> anyhow::Result<PlaceOrderResult> {
        let nonce = fetch_nonce(new_nonce_url, &self.http).await?;
        let protected = self.protected_kid(nonce, new_order_url);
        let payload = json!({
            "identifiers": [{ "type": "dns", "value": domain }]
        });
        let jws_body = self.build_jws_body_kid(protected, &payload)?;

        info!("Sending newOrder request for domain '{}' to {}", domain, new_order_url);
        let resp = self
            .http
            .post(new_order_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("newOrder POST failed: {}", e))?;

        let status = resp.status();
        let order_url = resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .ok_or_else(|| anyhow::anyhow!("newOrder response missing Location header"))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read newOrder response: {}", e))?;

        try_validate_jws_response(&body);
        debug!("newOrder response: {}", body);

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "newOrder returned {} for domain '{}': {}",
                status,
                domain,
                body
            ));
        }

        let authorization_url = body["authorizations"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("newOrder response missing authorizations array"))?
            .to_owned();

        let finalize_url = body["finalize"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("newOrder response missing finalize URL"))?
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

    pub async fn fetch_dns01_challenge(
        &self,
        new_nonce_url: &str,
        authorization_url: &str,
    ) -> anyhow::Result<Dns01ChallengeInfo> {
        let nonce = fetch_nonce(new_nonce_url, &self.http).await?;
        // POST-as-GET: empty string payload
        let protected = self.protected_kid(nonce, authorization_url);
        let jws_body = self.build_jws_body_kid_post_as_get(protected)?;

        let resp = self
            .http
            .post(authorization_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Authorization fetch failed: {}", e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse authorization response: {}", e))?;

        try_validate_jws_response(&body);
        debug!("Authorization response: {}", body);

        let challenges = body["challenges"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Authorization response missing challenges array"))?;

        let dns_challenge = challenges
            .iter()
            .find(|c| c["type"].as_str() == Some("dns-01"))
            .ok_or_else(|| anyhow::anyhow!("No dns-01 challenge found in authorization"))?;

        let token = dns_challenge["token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("dns-01 challenge missing token"))?
            .to_owned();

        let challenge_url = dns_challenge["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("dns-01 challenge missing url"))?
            .to_owned();

        let thumbprint = self
            .private_key
            .jwk_thumbprint()
            .map_err(|e| anyhow::anyhow!("Failed to compute JWK thumbprint: {}", e))?;
        let key_authorization = format!("{}.{}", token, thumbprint);

        // TXT record value: base64url(sha256(key_authorization))
        let txt_record_value = crate::crypto::sha256_b64(key_authorization.as_bytes());

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
    ) -> anyhow::Result<()> {
        let nonce = fetch_nonce(new_nonce_url, &self.http).await?;
        let protected = self.protected_kid(nonce, challenge_url);
        // RFC 8555: payload must be `{}` (empty JSON object) when responding to a challenge
        let payload = json!({});
        let jws_body = self.build_jws_body_kid(protected, &payload)?;

        info!("Notifying CA that challenge is ready: {}", challenge_url);
        let resp = self
            .http
            .post(challenge_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Challenge notify POST failed: {}", e))?;

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
    ) -> anyhow::Result<FinalizeResult> {
        let csr_der = build_csr_der(domain, &self.private_key)
            .map_err(|e| anyhow::anyhow!("CSR generation failed for '{}': {}", domain, e))?;
        let csr_b64 = encode_b64(&csr_der);

        let nonce = fetch_nonce(new_nonce_url, &self.http).await?;
        let protected = self.protected_kid(nonce, finalize_url);
        let payload = json!({ "csr": csr_b64 });
        let jws_body = self.build_jws_body_kid(protected, &payload)?;

        info!("Sending finalize request to {} for domain '{}'", finalize_url, domain);
        let resp = self
            .http
            .post(finalize_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Finalize POST failed: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Finalize returned error: {}", body));
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse finalize response: {}", e))?;

        try_validate_jws_response(&body);
        debug!("Finalize response: {}", body);

        let certificate_url = body["certificate"].as_str().map(|s| s.to_owned());
        Ok(FinalizeResult { certificate_url })
    }

    pub async fn poll_order_certificate_url(
        &self,
        new_nonce_url: &str,
        order_url: &str,
    ) -> anyhow::Result<Option<String>> {
        let nonce = fetch_nonce(new_nonce_url, &self.http).await?;
        let protected = self.protected_kid(nonce, order_url);
        let jws_body = self.build_jws_body_kid_post_as_get(protected)?;

        let resp = self
            .http
            .post(order_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Order poll POST failed: {}", e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse order poll response: {}", e))?;

        try_validate_jws_response(&body);

        match body["status"].as_str() {
            Some("valid") => {
                let cert_url = body["certificate"]
                    .as_str()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Order is valid but has no certificate URL in body")
                    })?
                    .to_owned();
                info!("Order is valid. Certificate URL: {}", cert_url);
                Ok(Some(cert_url))
            }
            Some("processing") => {
                info!("Order is still processing.");
                Ok(None)
            }
            Some(other) => Err(anyhow::anyhow!(
                "Order poll returned unexpected status '{}'",
                other
            )),
            None => Err(anyhow::anyhow!("Order poll response missing 'status' field")),
        }
    }

    pub async fn download_certificate(
        &self,
        new_nonce_url: &str,
        cert_url: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let nonce = fetch_nonce(new_nonce_url, &self.http).await?;
        let protected = self.protected_kid(nonce, cert_url);
        let jws_body = self.build_jws_body_kid_post_as_get(protected)?;

        info!("Downloading certificate from {}", cert_url);
        let resp = self
            .http
            .post(cert_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Certificate download POST failed: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Certificate download returned error: {}", body));
        }

        Ok(resp
            .bytes()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read certificate bytes: {}", e))?
            .to_vec())
    }

    fn protected_kid(&self, nonce: String, url: &str) -> Value {
        let alg = self.private_key.kt.get_key_alg();
        json!({
            "alg":   alg.to_string(),
            "kid":   self.account_url,
            "nonce": nonce,
            "url":   url,
        })
    }

    fn build_jws_body_kid(
        &self,
        protected: Value,
        payload: &Value,
    ) -> anyhow::Result<Value> {
        build_jws_body_kid(protected, payload, &self.private_key)
    }

    /// POST-as-GET variant: payload is the empty string (base64url of empty bytes).
    fn build_jws_body_kid_post_as_get(&self, protected: Value) -> anyhow::Result<Value> {
        build_jws_body_kid_post_as_get(protected, &self.private_key)
    }
}

fn build_jws_body_jwk(
    protected: Value,
    payload: &Value,
    key: &PrivateKey,
) -> anyhow::Result<Value> {
    let header = crate::jws::JWSHeader::with_alg(key.kt.get_key_alg());
    let protected_b64 = encode_b64(protected.to_string().as_bytes());
    let payload_b64 = encode_b64(payload.to_string().as_bytes());
    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature = key
        .sign(&header, &signing_input)
        .map_err(|e| anyhow::anyhow!("JWS signing failed: {}", e))?;
    Ok(json!({
        "protected": protected_b64,
        "payload":   payload_b64,
        "signature": encode_b64(&signature),
    }))
}

fn build_jws_body_kid(
    protected: Value,
    payload: &Value,
    key: &PrivateKey,
) -> anyhow::Result<Value> {
    let header = crate::jws::JWSHeader::with_alg(key.kt.get_key_alg());
    let protected_b64 = encode_b64(protected.to_string().as_bytes());
    let payload_b64 = encode_b64(payload.to_string().as_bytes());
    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature = key
        .sign(&header, &signing_input)
        .map_err(|e| anyhow::anyhow!("JWS signing failed: {}", e))?;
    Ok(json!({
        "protected": protected_b64,
        "payload":   payload_b64,
        "signature": encode_b64(&signature),
    }))
}

fn build_jws_body_kid_post_as_get(protected: Value, key: &PrivateKey) -> anyhow::Result<Value> {
    let header = crate::jws::JWSHeader::with_alg(key.kt.get_key_alg());
    let protected_b64 = encode_b64(protected.to_string().as_bytes());
    // POST-as-GET: payload is the empty string, not even a JSON null
    let payload_b64 = encode_b64(b"");
    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature = key
        .sign(&header, &signing_input)
        .map_err(|e| anyhow::anyhow!("JWS signing (POST-as-GET) failed: {}", e))?;
    Ok(json!({
        "protected": protected_b64,
        "payload":   payload_b64,
        "signature": encode_b64(&signature),
    }))
}

async fn fetch_nonce(nonce_url: &str, http: &reqwest::Client) -> anyhow::Result<String> {
    let resp = http
        .head(nonce_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch nonce from '{}': {}", nonce_url, e))?;
    resp.headers()
        .get("Replay-Nonce")
        .ok_or_else(|| anyhow::anyhow!("Server did not return a Replay-Nonce header"))?
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|e| anyhow::anyhow!("Invalid Replay-Nonce header value: {}", e))
}

fn try_validate_jws_response(body: &Value) {
    let is_flat_jws = body.get("protected").is_some()
        && body.get("payload").is_some()
        && body.get("signature").is_some();

    if is_flat_jws {
        match JWS::parse_flat(body, Box::new(EmbeddedJwkOnly)) {
            Ok(Some(_)) => {
                info!("CA response validated as a signed flat JWS successfully.");
            }
            Ok(None) => {}
            Err(e) => {
                warn!("CA response looks like a flat JWS but validation failed: {}", e);
            }
        }
    }
}
