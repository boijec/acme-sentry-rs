use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AcmeDirectoryApi {
    #[serde(skip)]
    pub directory_id: i64,
    #[serde(skip)]
    pub user_id: String,
    #[serde(rename = "keyChange")]
    pub key_change: String,
    #[serde(rename = "newAuthz")]
    pub new_authz: Option<String>,
    #[serde(rename = "newNonce")]
    pub new_nonce: String,
    #[serde(rename = "newAccount")]
    pub new_account: String,
    #[serde(rename = "newOrder")]
    pub new_order: String,
    #[serde(rename = "revokeCert")]
    pub revoke_cert: String,
}