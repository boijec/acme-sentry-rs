use crate::job_execution::job_base::{Job, JobDependencies, SchedulerHandle};
use acme_client::crypto::SupportedKey;
use acme_client::keys::PrivateKey;
use async_trait::async_trait;
use common_utils::fs::FileSystemOperations;
use persistence::database::DatabaseConnection;
use persistence::user::AcmeUser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct InitializeLocalUserJob {
    #[serde(skip)]
    fs: Option<Arc<dyn FileSystemOperations + Send + Sync>>,
    path: String,
    key_type: String,
    user_id: String,
}
impl InitializeLocalUserJob {
    pub fn new(path: String, key_type: String, user_id: String) -> Self {
        InitializeLocalUserJob { fs: None, path, key_type, user_id }
    }
    fn create_from_incoming_type(&self, supported_key: SupportedKey) -> Result<PrivateKey, Box<dyn Error>> {
        let result = PrivateKey::from_supported_type(supported_key)?;
        Ok(result)
    }
    fn check_for_required_files(&self, user: AcmeUser) -> Result<PrivateKey, Box<dyn Error>> {
        let system = self.fs.as_ref().unwrap();
        let result = system.ensure_sub_dir(user.key_path.as_str())?;
        let s = user.user_id.as_str().to_owned() + ".pem";
        let fp = result.clone();
        let supported_key = SupportedKey::from_str(user.key_type.as_str())?;
        info!("Configured key type for user: {}", supported_key.to_string());
        let key: PrivateKey;
        if system.file_exists(fp, s.as_str()) {
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
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        let fs = dependencies.file_system();
        self.fs = Some(fs);
    }
    async fn execute(&self, _: SchedulerHandle<'a>) -> anyhow::Result<()>{
        let connection = DatabaseConnection::get_connection().unwrap();
        let mut user = AcmeUser::get_user(self.user_id.as_str(), &connection);
        if user.is_err() {
            info!("User not found, creating user entry in database..");
            let system = self.fs.as_ref().unwrap();
            let path = system.ensure_sub_dir(format!("{}/login-keys/{}", self.user_id, self.key_type).as_str()).unwrap();
            let dump_path = system.ensure_sub_dir(format!("{}", self.user_id).as_str()).unwrap();
            let result = AcmeUser::new_user(path, dump_path, self.user_id.clone(), self.key_type.clone(), &connection).unwrap();
            if let Some(acme_user) = result {
                info!("User created: User [ id: {}, user_id: {} ] with key type: {}", acme_user.id, acme_user.user_id, acme_user.key_type);
                user = Ok(Some(acme_user));
            } else {
                return Err(anyhow::anyhow!("User could not be picked back up!"));
            }
        }
        let user = user.unwrap().unwrap();
        info!("User found in database: User [ id: \"{}\", user_id: \"{}\" ]", user.id, user.user_id);
        let _key = self.check_for_required_files(user).unwrap();
        Ok(())
    }
}