use sentry_common::iter::{CompareFields, FieldDiff};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AcmeDirectoryApi {
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

impl CompareFields<AcmeDirectoryApi> for AcmeDirectoryApi {
    fn compare_fields(&self, other: &AcmeDirectoryApi) -> Vec<FieldDiff> {
        let auths = self.new_authz.clone().unwrap_or("".to_string()).to_string();
        let other_auths = other
            .new_authz
            .clone()
            .unwrap_or("".to_string())
            .to_string();
        vec![
            FieldDiff {
                field: "key_change",
                are_equal: self.key_change == other.key_change,
            },
            FieldDiff {
                field: "new_authz",
                are_equal: auths == other_auths,
            },
            FieldDiff {
                field: "new_nonce",
                are_equal: self.new_nonce == other.new_nonce,
            },
            FieldDiff {
                field: "new_account",
                are_equal: self.new_account == other.new_account,
            },
            FieldDiff {
                field: "new_order",
                are_equal: self.new_order == other.new_order,
            },
            FieldDiff {
                field: "revoke_cert",
                are_equal: self.revoke_cert == other.revoke_cert,
            },
        ]
    }
}
