use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::crypto::SupportedKey;
use acme_client::keys::PrivateKey;
use async_trait::async_trait;
use common_utils::fs;
use fs::FileSystem;
use persistence::data_model::AcmeUser;
use persistence::database::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::path::PathBuf;
use tracing::{info, instrument};

#[derive(Serialize, Deserialize)]
pub struct InitializeLocalUserJob {
    path: String,
    key_type: String,
    user_id: String,
    db_loc: String,
}
impl InitializeLocalUserJob {
    pub fn new(path: String, key_type: String, user_id: String, db_loc: String) -> Self {
        InitializeLocalUserJob { path, key_type, user_id, db_loc }
    }
    fn create_from_incoming_type(&self, supported_key: SupportedKey) -> Result<PrivateKey, Box<dyn Error>> {
        let result = PrivateKey::from_supported_type(supported_key)?;
        Ok(result)
    }
    fn check_for_required_files(&self, user: AcmeUser) -> Result<PrivateKey, Box<dyn Error>> {
        let system = FileSystem::new(self.path.as_str())?;
        let result = system.ensure_sub_dir(user.key_path.as_str())?;
        let s = user.user_id.as_str().to_owned() + ".pem";
        let fp = result.clone();
        let supported_key = SupportedKey::from_str(user.key_type.as_str())?;
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
    pub fn get_user(user_id: &str, conn: &DatabaseConnection) -> anyhow::Result<Option<AcmeUser>, Box<dyn Error>> {
        let sql = r#"
            SELECT * FROM acme_users WHERE user_id = ?1
            "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, user_id))?;
        Ok(Some(AcmeUser::scan_statement(statement)?))
    }
    fn new_user(&self, key_path: PathBuf, dump_path: PathBuf, conn: &DatabaseConnection) -> Result<Option<AcmeUser>, Box<dyn Error>> {
        let sql = r#"
            INSERT INTO acme_users (user_id, key_type, key_path, user_dump_path)
            VALUES (?1, ?2, ?3, ?4) RETURNING *;
            "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, self.user_id.as_str()))?;
        statement.bind((2, self.key_type.as_str()))?;
        statement.bind((3, key_path.as_path().to_str().unwrap()))?;
        statement.bind((4, dump_path.as_path().to_str().unwrap()))?;
        Ok(Some(AcmeUser::scan_statement(statement)?))
    }
}
#[async_trait]
impl Job for InitializeLocalUserJob {
    fn job_type(&self) -> &'static str {
        "initialize-local-user-job"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    #[instrument(level = "trace", name = "initialize_local_user_job", fields(job_name = %self.job_type()), skip_all)]
    async fn execute(&self, handle: SchedulerHandle) -> anyhow::Result<()>{
        let connection = DatabaseConnection::get_connection(self.db_loc.as_str()).unwrap();
        let mut user = Self::get_user(self.user_id.as_str(), &connection);
        if user.is_err() {
            info!("User not found, creating user entry in database..");
            let system = FileSystem::new(self.path.as_str()).unwrap();
            let path = system.ensure_sub_dir(format!("{}/login-keys/{}", self.user_id, self.key_type).as_str()).unwrap();
            let dump_path = system.ensure_sub_dir(format!("{}", self.user_id).as_str()).unwrap();
            let result = self.new_user(path, dump_path, &connection).unwrap();
            if let Some(acme_user) = result {
                info!("User created: {:?}", acme_user);
                user = Ok(Some(acme_user));
            } else {
                return Err(anyhow::anyhow!("User could not be picked back up!"));
            }
        }
        let user = user.unwrap().unwrap();
        info!("User found in database: {:?}", user);
        let key = self.check_for_required_files(user).unwrap();
        Ok(())
    }
}