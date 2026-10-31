use reqwest::Client;
use crate::error::HttpClientError;

pub fn build_client(
    accept_invalid_certs: bool,
) -> Result<Client, HttpClientError> {
    let client = Client::builder()
        .tls_danger_accept_invalid_certs(accept_invalid_certs)
        .user_agent("acme-sentry-rs/0.1.0 (reqwest/0.13.5)")
        .build()?;
    Ok(client)
}