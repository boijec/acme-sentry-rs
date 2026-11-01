use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;
use sentry_core::user::model::ServiceUser;
use sentry_core::user::service::UserService;
use sentry_crypto::crypto::SupportedKey;
use sentry_io::fs::FileSystemOperations;
use sentry_jose::keys::PrivateKey;
use std::error::Error;
use std::sync::Arc;
use tracing::info;

pub struct InitializeLocalUserJob {
    fs: Option<Arc<dyn FileSystemOperations + Send + Sync>>,
    config: Option<Arc<dyn ConfigurationMap + Send + Sync>>,
    user_service: Option<Arc<UserService>>
}

#[derive(Default)]
pub struct InitializeLocalUserJobBuilder {
    // path: String,
    // key_type: String,
    // user_id: String,
}


impl InitializeLocalUserJobBuilder {
    pub fn build(self) -> InitializeLocalUserJob {
        InitializeLocalUserJob {
            fs: None,
            config: None,
            user_service: None
        }
    }
}

impl InitializeLocalUserJob {
    pub fn builder() -> InitializeLocalUserJobBuilder {
        InitializeLocalUserJobBuilder::default()
    }
    fn create_from_incoming_type(&self, supported_key: SupportedKey) -> Result<PrivateKey, Box<dyn Error>> {
        let result = PrivateKey::from_supported_type(supported_key)?;
        Ok(result)
    }
    fn check_for_required_files(&self, user: ServiceUser) -> Result<PrivateKey, Box<dyn Error>> {
        let system = self.fs.as_ref().unwrap();
        let result = system.ensure_sub_dir(user.key_path.as_str())?;
        let s = user.user_id.as_str().to_owned() + ".pem";
        let fp = result.clone();
        let supported_key = SupportedKey::from_str(user.key_type.as_str())?;
        info!("Configured key type for user: {}", supported_key.to_string());
        let key: PrivateKey;
        if system.file_exists_adaptive(fp, s.as_str()) {
            info!("Key file for user {}, found! Instantiating required keys!", user.user_id);
            let t = system.read_from_file(result.as_path().to_str().unwrap(), s.as_str())?;
            key = PrivateKey::load_private_bytes(&t, supported_key)?;
        } else {
            info!("Key file for user {}, not found! Generating required keys!", user.user_id);
            key = self.create_from_incoming_type(supported_key)?;
            let _ = system.write_to_file(result.as_path().to_str().unwrap(), s.as_str(), key.get_pem_bytes()?.as_slice())?;
        }
        Ok(key)
    }
}
#[async_trait]
impl <'a> Job<'a> for InitializeLocalUserJob {
    fn job_type(&self) -> &'a str {
        "initialize-local-user-job"
    }
    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        let fs = dependencies.file_system();
        self.fs = Some(fs);
        let conf = dependencies.configuration();
        self.config = Some(conf);
        let usr_service = dependencies.services().user_service();
        self.user_service = Some(usr_service);
    }
    async fn execute(&self, _: SchedulerHandle<'a>) -> Result<(), JobEngineError> {
        if let (Some(user_service), Some(config)) = (&self.user_service, &self.config) {
            let config = config.clone();
            let user_service = user_service.clone();

            info!("user id: {}", config.get_user_id());
            let mut user = user_service.get_user(config.get_user_id().as_str()).await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()));
            if user.is_err() {
                info!("User not found, creating user entry in database..");
                let system = self.fs.as_ref().unwrap();
                let temp_key_path = format!("{}/login-keys/{}", config.get_user_id(), config.get_key_type());
                let _path = system.ensure_sub_dir(temp_key_path.as_str()).unwrap();
                let temp_dump_path = format!("{}", config.get_user_id());
                let _dump_path = system.ensure_sub_dir(temp_dump_path.as_str()).unwrap();
                let result = user_service.create_new_user(
                    ServiceUser::builder()
                        .user_id(config.get_user_id().as_str())
                        .key_path(temp_key_path.as_str())
                        .key_type(config.get_key_type().as_str())
                        .user_dump_path(temp_dump_path.as_str())
                        .build()
                ).await
                    .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
                user = Ok(Some(result));
            }
            let user = user.ok().unwrap().unwrap();
            info!("User found in database: User [ id: \"{}\", user_id: \"{}\" ]", user.id, user.user_id);
            let _key = self.check_for_required_files(user).unwrap();
            return Ok(())
        }
        Err(JobEngineError::JobExecutionError(String::from("No files were provided")))
    }
}