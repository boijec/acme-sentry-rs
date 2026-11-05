use crate::config::ApplicationConfig;
use crate::directory::error::DirectoryServiceError;
use crate::directory::model::ServiceUserDirectory;
use sentry_rfc8555::directory::operations::DirectoryOperations;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DirectoryService {
    configuration: Arc<ApplicationConfig>,

    in_mem_directory: Arc<Mutex<Option<ServiceUserDirectory>>>,
}

impl DirectoryService {
    pub fn new(configuration: Arc<ApplicationConfig>) -> Self {
        DirectoryService {
            configuration,
            in_mem_directory: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn fetch(&self) -> Result<ServiceUserDirectory, DirectoryServiceError> {
        let mut in_mem_directory = self.in_mem_directory.lock().await;
        if let Some(ref mut directory) = *in_mem_directory {
            return Ok(directory.clone());
        }
        let dir_ops = DirectoryOperations::new_from_url(&self.configuration.base_url, true)
            .map_err(|e| DirectoryServiceError::InternalError(e.to_string()))?;
        let dir = dir_ops
            .query_directory()
            .await
            .map_err(|e| DirectoryServiceError::InternalError(e.to_string()))?;
        let user_dir = ServiceUserDirectory::builder()
            .revoke_cert(dir.revoke_cert.as_str())
            .key_change(dir.key_change.as_str())
            .new_nonce(dir.new_nonce.as_str())
            .new_account(dir.new_account.as_str())
            .new_order(dir.new_order.as_str())
            .new_authz_option(dir.new_authz)
            .build();
        *in_mem_directory = Some(user_dir.clone());
        Ok(user_dir)
    }
}
