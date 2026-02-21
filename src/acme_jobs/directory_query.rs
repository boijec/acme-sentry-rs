use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::directory::AcmeDirectory;
use async_trait::async_trait;
use persistence::database::DatabaseConnection;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::error::Error;
use tracing::{info, instrument, trace};

#[derive(Serialize, Deserialize)]
pub struct DirectoryQueryJob {
    pub base_url: String,
}
impl DirectoryQueryJob {
    pub fn new(base_url: String) -> Result<DirectoryQueryJob, Box<dyn Error>> {
        let x = base_url.to_owned() + "/dir";
        let _ = Url::parse(x.as_str())?;
        Ok(DirectoryQueryJob { base_url: x.to_string() })
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
    fn insert_dir_in_db(&self, acme_directory: AcmeDirectory) -> anyhow::Result<()> {
        let connection = DatabaseConnection::get_connection().unwrap();
        let sql = r#"
            INSERT INTO acme_users_directory(
                directory_id,
                user_id,
                new_nonce,
                new_account,
                new_order,
                new_authz,
                revoke_cert,
                key_change
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#;
        let mut statement = connection.prepare(sql).unwrap();
        statement.bind((1, 1))?;
        statement.bind((2, 1))?;
        statement.bind((3, acme_directory.new_nonce.as_str()))?;
        statement.bind((4, acme_directory.new_account.as_str()))?;
        statement.bind((5, acme_directory.new_order.as_str()))?;
        statement.bind((6, ""))?;
        statement.bind((7, acme_directory.revoke_cert.as_str()))?;
        statement.bind((8, acme_directory.key_change.as_str()))?;
        statement.next()?;
        Ok(())
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
        let dir: AcmeDirectory = from_value(value.clone())?;
        self.insert_dir_in_db(dir)?;
        Ok(())
    }
}
