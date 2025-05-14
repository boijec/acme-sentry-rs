use serde::Serialize;

#[derive(Serialize)]
pub struct AcmeDirectory {
    new_nonce: String,
    new_account: String,
    new_order: String,
    new_authz: Option<String>,
    revoke_cert: String,
    key_change: String,
}