use reqwest::Client;

pub fn build_client(
    accept_invalid_certs: bool,
) -> anyhow::Result<Client> {
    let client = Client::builder()
        .tls_danger_accept_invalid_certs(accept_invalid_certs)
        .user_agent("acme-sentry-rs/1.0.0 (reqwest/0.13.1)")
        .build()?;
    anyhow::Ok(client)
}