use crate::client::build_client;
use crate::directory::model::AcmeDirectoryApi;
use crate::error::DirectoryOperationsError;
use reqwest::{Client, Url};
use serde_json::Value;
use tracing::info;

pub struct DirectoryOperations {
    url: Url,
    client: Client,
}

impl DirectoryOperations {
    pub fn new_from_url(
        url_string: &str,
        accept_invalid_certs: bool,
    ) -> Result<Self, DirectoryOperationsError> {
        let parsed_directory_base = format!("{}/dir", url_string);
        let url = Url::parse(parsed_directory_base.as_str()).map_err(|e| {
            DirectoryOperationsError::UrlParsingError(e.to_string(), url_string.to_string())
        })?;
        let client = build_client(accept_invalid_certs)?;
        Ok(DirectoryOperations { url, client })
    }

    pub async fn query_directory(&self) -> Result<AcmeDirectoryApi, DirectoryOperationsError> {
        info!("Calling ACME directory at: {}", self.url.as_str());
        let response = self
            .client
            .get(self.url.clone())
            .send()
            .await
            .map_err(|e| DirectoryOperationsError::DirectoryCallError(e.to_string()))?;
        info!("ACME directory returned response: {}", response.status());
        let body: Value = response
            .json()
            .await
            .map_err(|e| DirectoryOperationsError::DirectoryResponseParseError(e))?;
        let key_change = body["keyChange"]
            .as_str()
            .ok_or_else(|| DirectoryOperationsError::MissingResourceError("keyChange".into()))?
            .to_owned();
        let new_auths = body["newAuthz"].as_str().unwrap_or_else(|| "").to_owned();
        let new_nonce = body["newNonce"]
            .as_str()
            .ok_or_else(|| DirectoryOperationsError::MissingResourceError("newNonce".into()))?
            .to_owned();
        let new_account = body["newAccount"]
            .as_str()
            .ok_or_else(|| DirectoryOperationsError::MissingResourceError("newAccount".into()))?
            .to_owned();
        let new_order = body["newOrder"]
            .as_str()
            .ok_or_else(|| DirectoryOperationsError::MissingResourceError("newOrder".into()))?
            .to_owned();
        let revoke_cert = body["revokeCert"]
            .as_str()
            .ok_or_else(|| DirectoryOperationsError::MissingResourceError("revokeCert".into()))?
            .to_owned();
        Ok(AcmeDirectoryApi {
            key_change,
            new_authz: Some(new_auths),
            new_nonce,
            new_account,
            new_order,
            revoke_cert,
        })
    }
}
