use crate::config::{ApplicationConfig, ConfigurationMap};
use crate::user::error::InternalUserServiceError;
use crate::user::model::ServiceUser;
use sentry_crypto::crypto::SupportedKey;
use sentry_db::repo::user_repo::{User, UserRepo};
use sentry_io::fs::FileSystemOperations;
use sentry_jose::keys::PrivateKey;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

pub struct UserService {
    configuration: Arc<ApplicationConfig>,
    user_repo: Arc<dyn UserRepo + Send + Sync>,
    file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepo + Send + Sync>,
        configuration: Arc<ApplicationConfig>,
        file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,
    ) -> Self {
        Self { configuration, user_repo, file_system_operations }
    }

    #[instrument(skip(self, user_id))]
    pub async fn get_user(&self, user_id: &str) -> Result<Option<ServiceUser>, InternalUserServiceError> {
        if let Some(user) = self
            .user_repo
            .get_user(user_id)
            .await
            .map_err(|e| InternalUserServiceError::UserDatabaseError(e))?
        {
            return Ok(Some(Self::build_service_user_from_service_user(user)));
        }
        Ok(None)
    }

    async fn create_new_user(
        &self,
        new_user: ServiceUser
    ) -> Result<ServiceUser, InternalUserServiceError> {
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
            .map_err(|e| InternalUserServiceError::UserDatabaseError(e))?;
        Ok(
            Self::build_service_user_from_service_user(user)
        )
    }
    #[instrument(skip(self, user_id))]
    pub async fn lazy_load_user(&self, user_id: &str) -> Result<ServiceUser, InternalUserServiceError> {
        let optional_user = self.get_user(user_id).await?;
        if let Some(user) = optional_user {
            info!(
                "User found in database: User [ id: \"{}\", user_id: \"{}\" ]",
                user.id, user.user_id
            );
            return Ok(user)
        }
        warn!("User not found in database, creating new user..");
        let key_type = self.configuration.get_key_type();
        let (dump_path, key_path) = self.compute_paths_for_input(user_id, key_type.as_str());
        let key_name = format!("{}-key.pem", key_type);
        let _ = self.file_system_operations.ensure_sub_dir(key_path.as_str())?;
        let _ = self.file_system_operations.ensure_sub_dir(dump_path.as_str())?;
        let new_user = self.create_new_user(
            ServiceUser::builder()
                .user_id(user_id)
                .key_path(key_path.as_str())
                .key_type(key_type.as_str())
                .key_name(key_name.as_str())
                .user_dump_path(dump_path.as_str())
                .build()
        ).await?;
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

    fn build_service_user_from_service_user(user: User) -> ServiceUser {
        ServiceUser::builder()
            .id(user.id())
            .user_id(user.user_id())
            .key_type(user.key_type())
            .key_path(user.key_path())
            .key_name(user.key_name())
            .user_dump_path(user.user_dump_path())
            .build()
    }


    #[instrument(skip(self, user))]
    pub async fn user_sanity_check(&self, user: &ServiceUser) -> Result<(), InternalUserServiceError> {
        let supported_key = SupportedKey::from_str(user.key_type.as_str())?;
        info!(
            "Current user: {} is configured for key-type: {}",
            user.user_id.as_str(),
            supported_key.to_string(),
        );
        debug!(
            "Locating file: {}",
            user.key_name.as_str()
        );
        let key_path_buf = Path::new(user.key_path.as_str()).to_path_buf();
        if self.file_system_operations.file_exists(key_path_buf.clone(), user.key_name.as_str()) {
            info!(
                "Key file located - {}. Instantiating key.",
                user.key_name.as_str()
            );
            let key = self.file_system_operations.read_from_file(
                user.key_path.as_str(),
                user.key_name.as_str(),
            )?;
            let _ = PrivateKey::load_private_bytes(&key, supported_key);
        } else {
            warn!(
                "Unable to locate user key file - {}. Generating key file for key-type: {}.",
                user.key_name.as_str(),
                supported_key.to_string(),
            );
            let key = PrivateKey::from_supported_type(supported_key)?;
            let _ = self.file_system_operations.write_to_file(
                user.key_path.as_str(),
                user.key_name.as_str(),
                key.get_pem_bytes()?.as_slice(),
            )?;
        }
        Ok(())
    }
}
