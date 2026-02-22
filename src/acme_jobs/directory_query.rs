use crate::acme_jobs::initialize_keys_for_user::InitializeLocalUserJob;
use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::directory::AcmeDirectoryApi;
use async_trait::async_trait;
use persistence::data_model::AcmeDirectory;
use persistence::database::DatabaseConnection;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::error::Error;
use tracing::{info, instrument};

#[derive(Serialize, Deserialize)]
pub struct DirectoryQueryJob {
    pub base_url: String,
    pub user_id: String,
    pub db_loc: String,
}
impl DirectoryQueryJob {
    pub fn new(base_url: Option<String>, user_id: String, db_loc: String) -> Result<Self, Box<dyn Error>> {
        if let Some(base_url) = base_url {
            let x = base_url.to_owned() + "/dir";
            let _ = Url::parse(x.as_str())?;
            return Ok(DirectoryQueryJob { base_url: x.to_string(), user_id, db_loc })
        }
        Err("Acme CA base url could not be parsed!".into())
    }
    async fn call_directory(&self) -> anyhow::Result<Value> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        info!("Calling the requested ACME directory");
        let response = client.get(self.base_url.as_str()).send().await?;
        info!("Response returned: {}", response.status());
        let bytes = response.bytes().await?;
        let slice = bytes.iter().as_slice();
        let value = serde_json::from_slice::<Value>(slice)?;
        info!("{}", value);
        Ok(value)
    }
    fn insert_dir_in_db(&self, acme_directory: AcmeDirectoryApi) -> anyhow::Result<Option<AcmeDirectory>> {
        let connection = DatabaseConnection::get_connection(self.db_loc.as_str()).unwrap();
        let user = InitializeLocalUserJob::get_user(self.user_id.as_str(), &connection).unwrap();
        if user.is_none() {
            return Err(anyhow::anyhow!("Could not complete directory job since queried user could not be found!"))?;
        }
        let user = user.unwrap();
        let sql = r#"
            INSERT INTO acme_users_directory(
                user_id,
                new_nonce,
                new_account,
                new_order,
                new_authz,
                revoke_cert,
                key_change
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING *;
            "#;
        let mut statement = connection.prepare(sql).unwrap();
        statement.bind((1, user.id))?;
        statement.bind((2, acme_directory.new_nonce.as_str()))?;
        statement.bind((3, acme_directory.new_account.as_str()))?;
        statement.bind((4, acme_directory.new_order.as_str()))?;
        statement.bind((5, ""))?;
        statement.bind((6, acme_directory.revoke_cert.as_str()))?;
        statement.bind((7, acme_directory.key_change.as_str()))?;
        Ok(Some(AcmeDirectory::scan_statement(statement).unwrap()))
    }
}

#[async_trait]
impl Job for DirectoryQueryJob {
    fn job_type(&self) -> &'static str {
        "directory-query-job"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    #[instrument(level = "trace", name = "directory_query_job", fields(job_name = %self.job_type()), skip_all)]
    async fn execute(&self, _scheduler: SchedulerHandle) -> anyhow::Result<()> {
        let value = self.call_directory().await?;
        let dir: AcmeDirectoryApi = from_value(value.clone())?;
        let t = self.insert_dir_in_db(dir)?;
        if let Some(dir) = t {
            info!("Directory successfully inserted! {}", dir.directory_id);
        }
        Ok(())
    }
}
