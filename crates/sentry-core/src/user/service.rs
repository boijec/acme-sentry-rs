use crate::config::{ApplicationConfig, ConfigurationMap};
use crate::directory::service::DirectoryService;
use crate::user::error::UserServiceError;
use crate::user::model::ServiceUser;
use sentry_crypto::crypto::SupportedKey;
use sentry_db::repo::account_link_repo::{AccountLink, AccountLinkRepo};
use sentry_db::repo::certificate_repo::CertificateRepo;
use sentry_db::repo::user_repo::{User, UserRepo};
use sentry_io::fs::FileSystemOperations;
use sentry_jose::keys::PrivateKey;
use sentry_rfc8555::account::login_or_register;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

pub struct UserService {
    configuration: Arc<ApplicationConfig>,
    user_repo: Arc<dyn UserRepo + Send + Sync>,
    account_link_repo: Arc<dyn AccountLinkRepo + Send + Sync>,
    certificate_repo: Arc<dyn CertificateRepo + Send + Sync>,
    file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,

    directory_service: Arc<DirectoryService>,

    account_resource: Arc<Mutex<Option<String>>>,
    private_key: Arc<Mutex<Option<PrivateKey>>>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepo + Send + Sync>,
        account_link_repo: Arc<dyn AccountLinkRepo + Send + Sync>,
        certificate_repo: Arc<dyn CertificateRepo + Send + Sync>,
        configuration: Arc<ApplicationConfig>,
        file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,
        directory_service: Arc<DirectoryService>,
    ) -> Self {
        Self {
            configuration,
            user_repo,
            certificate_repo,
            account_link_repo,
            file_system_operations,
            directory_service,
            account_resource: Arc::new(Mutex::new(None)),
            private_key: Arc::new(Mutex::new(None)),
        }
    }

    #[instrument(skip(self, user_id))]
    pub async fn get_user(&self, user_id: &str) -> Result<Option<ServiceUser>, UserServiceError> {
        if let Some(user) = self
            .user_repo
            .get_user(user_id)
            .await
            .map_err(|e| UserServiceError::UserDatabaseError(e))?
        {
            return Ok(Some(self.build_service_user_from_service_user(user).await?));
        }
        Ok(None)
    }

    async fn create_new_user(
        &self,
        new_user: ServiceUser,
    ) -> Result<ServiceUser, UserServiceError> {
        let user = self
            .user_repo
            .save_user(
                new_user.user_id.as_str(),
                &new_user.key_type.as_str(),
                &new_user.key_path.as_str(),
                &new_user.key_name.as_str(),
                &new_user.user_dump_path.as_str(),
            )
            .await
            .map_err(|e| UserServiceError::UserDatabaseError(e))?;
        Ok(self.build_service_user_from_service_user(user).await?)
    }
    #[instrument(skip(self, user_id))]
    pub async fn lazy_load_user(&self, user_id: &str) -> Result<ServiceUser, UserServiceError> {
        let optional_user = self.get_user(user_id).await?;
        if let Some(user) = optional_user {
            info!(
                "User found in database: User [ id: \"{}\", user_id: \"{}\" ]",
                user.id, user.user_id
            );
            return Ok(user);
        }
        warn!("User not found in database, creating new user..");
        let key_type = self.configuration.get_key_type();
        let (dump_path, key_path) = self.compute_paths_for_input(user_id, key_type.as_str());
        let key_name = format!("{}-key.pem", key_type);
        let _ = self
            .file_system_operations
            .ensure_sub_dir(key_path.as_str())?;
        let _ = self
            .file_system_operations
            .ensure_sub_dir(dump_path.as_str())?;
        let new_user = self
            .create_new_user(
                ServiceUser::builder()
                    .user_id(user_id)
                    .key_path(key_path.as_str())
                    .key_type(key_type.as_str())
                    .key_name(key_name.as_str())
                    .user_dump_path(dump_path.as_str())
                    .build(),
            )
            .await?;
        info!(
            "New service user created: User [ id: \"{}\", user_id: \"{}\" ]",
            new_user.id, new_user.user_id
        );
        Ok(new_user)
    }
    #[allow(unused)]
    fn compute_paths_for_system_user(&self, user: &ServiceUser) -> (String, String) {
        self.compute_paths_for_input(user.user_id.as_str(), user.key_path.as_str())
    }
    fn compute_paths_for_input(&self, user_id: &str, key_type: &str) -> (String, String) {
        let dump_path = format!("{}/output_files", user_id);
        let key_path = format!("{}/login-keys/{}", user_id, key_type);
        (dump_path, key_path)
    }

    async fn build_service_user_from_service_user(
        &self,
        user: User,
    ) -> Result<ServiceUser, UserServiceError> {
        let mut serv_user = ServiceUser::builder()
            .id(user.id())
            .user_id(user.user_id())
            .key_type(user.key_type())
            .key_path(user.key_path())
            .key_name(user.key_name())
            .user_dump_path(user.user_dump_path())
            .build();
        let private_key = self.private_key.lock().await;
        if let Some(ref pkey) = *private_key {
            serv_user = serv_user.with_private_key(pkey);
        }
        let account_resource = self.account_resource.lock().await;
        if let Some(ref account_resource) = *account_resource {
            serv_user = serv_user.with_account_resource(&account_resource);
        }
        Ok(serv_user)
    }

    #[instrument(skip(self, user))]
    pub async fn user_sanity_check(&self, user: &ServiceUser) -> Result<(), UserServiceError> {
        let supported_key = SupportedKey::from_str(user.key_type.as_str())?;
        info!(
            "Current user: {} is configured for key-type: {}",
            user.user_id.as_str(),
            supported_key.to_string(),
        );
        debug!("Locating file: {}", user.key_name.as_str());
        let key_path_buf = Path::new(user.key_path.as_str()).to_path_buf();
        if self
            .file_system_operations
            .file_exists(key_path_buf.clone(), user.key_name.as_str())
        {
            info!(
                "Key file located - {}. Instantiating key.",
                user.key_name.as_str()
            );
            let key = self
                .file_system_operations
                .read_from_file(user.key_path.as_str(), user.key_name.as_str())?;
            let pkey = PrivateKey::load_private_bytes(&key, supported_key)?;
            let mut private_key = self.private_key.lock().await;
            *private_key = Some(pkey);
        } else {
            warn!(
                "Unable to locate user key file - {}. Generating key file for key-type: {}.",
                user.key_name.as_str(),
                supported_key.to_string(),
            );
            let pkey = PrivateKey::from_supported_type(supported_key)?;
            let _ = self.file_system_operations.write_to_file(
                user.key_path.as_str(),
                user.key_name.as_str(),
                pkey.get_pem_bytes()?.as_slice(),
            )?;
            let mut private_key = self.private_key.lock().await;
            *private_key = Some(pkey);
        }
        Ok(())
    }
    #[instrument(skip(self, user))]
    pub async fn login(&self, user: &ServiceUser) -> Result<(), UserServiceError> {
        let directory = self
            .directory_service
            .fetch()
            .await
            .map_err(|e| UserServiceError::InternalError(e.to_string()))?;
        let private_key = self.private_key.lock().await.clone().ok_or_else(|| {
            UserServiceError::InternalError("No private key resource to use for login".into())
        })?;
        let acc_resource = login_or_register(
            &private_key,
            &directory.new_nonce,
            &directory.new_account,
            true,
        )
        .await
        .map_err(|e| UserServiceError::InternalError(format!("Unable to register user: {}", e)))?;
        self.upsert_account_link(user, acc_resource.clone()).await?;
        let mut account_resource = self.account_resource.lock().await;
        *account_resource = Some(acc_resource.clone());
        Ok(())
    }

    pub fn create_private_key_for_cert(
        &self,
        user: &ServiceUser,
    ) -> Result<PrivateKey, UserServiceError> {
        let supported_key = SupportedKey::from_str(user.key_type.as_str())?;
        let pkey = PrivateKey::from_supported_type(supported_key)?;
        let bytes = pkey.get_pem_bytes()?;
        let _ = self.file_system_operations.write_to_file(
            &user.user_dump_path,
            "new_key.pem",
            bytes.as_slice(),
        )?;
        Ok(pkey)
    }

    async fn upsert_account_link(
        &self,
        user: &ServiceUser,
        acc_resource: String,
    ) -> Result<(), UserServiceError> {
        let account_link = self
            .account_link_repo
            .get_link_for_user(user.id)
            .await
            .map_err(|e| UserServiceError::InternalError(e.to_string()))?;
        if let Some(link) = account_link {
            debug!(
                "Existing account link found for user: '{}' - updating",
                user.user_id
            );
            let _ = self
                .account_link_repo
                .update_linked_user(
                    AccountLink::builder()
                        .id(link.id())
                        .user_id(link.user_id())
                        .account_url(acc_resource)
                        .build(),
                )
                .await
                .map_err(|e| UserServiceError::InternalError(e.to_string()))?;
        } else {
            warn!(
                "No account link found for user: '{}' - creating",
                user.user_id
            );
            let _ = self
                .account_link_repo
                .insert_linked_user(
                    AccountLink::builder()
                        .user_id(user.id)
                        .account_url(acc_resource)
                        .build(),
                )
                .await
                .map_err(|e| UserServiceError::InternalError(e.to_string()))?;
        }
        debug!("Account link saved for user: '{}'.", user.user_id);
        Ok(())
    }

    pub async fn create_pem_from_bytes(
        &self,
        user: &ServiceUser,
        order_id: i64,
        bytes: Vec<u8>,
    ) -> Result<(), UserServiceError> {
        let path = self.file_system_operations.write_to_file(
            user.user_dump_path.as_str(),
            "sample_cert.crt",
            bytes.as_slice(),
        )?;
        let path_str = path.to_str().ok_or_else(|| {
            UserServiceError::InternalError("Unable to convert path to string".into())
        })?;
        info!("Certificate for domain saved at '{}'", path_str);
        self.certificate_repo
            .log_downloaded(order_id, path_str.to_string())
            .await
            .map_err(|e| UserServiceError::InternalError(e.to_string()))?;
        Ok(())
    }
}
