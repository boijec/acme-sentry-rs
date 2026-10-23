use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::directory::AcmeDirectoryApi;
use acme_client::comms::request::build_client;
use async_trait::async_trait;
use persistence::database::DatabaseConnection;
use persistence::directory::AcmeDirectory;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_value};
use std::error::Error;
use tracing::{info, instrument};

#[derive(Serialize, Deserialize)]
pub struct DirectoryUpdateJob {
    pub base_url: Option<String>,
    pub user_id: String,
}
impl DirectoryUpdateJob {
    pub fn new(base_url: Option<String>, user_id: String) -> Result<Self, Box<dyn Error>> {
        if base_url.is_none() {
            return Ok(DirectoryUpdateJob {
                user_id,
                base_url: None,
            })
        }
        let url = Self::validate_url(base_url.clone())?;
        Ok(DirectoryUpdateJob {
            user_id,
            base_url: Some(url.to_string()),
        })
    }
    pub fn validate_url(base_url: Option<String>) -> Result<Url, Box<dyn Error>> {
        if let Some(base_url) = base_url {
            let x = base_url.to_owned() + "/dir";
            let url = Url::parse(x.as_str())?;
            return Ok(url);
        }
        Err("Acme CA base url could not be parsed!".into())
    }
    async fn call_directory(&self, base_url: String) -> anyhow::Result<Value> {
        let client = build_client(true)?;
        info!("Calling ACME directory at: {}", base_url);
        let response = client
            .get(base_url.as_str())
            .send()
            .await?;
        info!("ACME directory returned response: {}", response.status());
        let bytes = response.bytes().await?;
        let slice = bytes.iter().as_slice();
        let value = serde_json::from_slice::<Value>(slice)?;
        Ok(value)
    }
}

#[async_trait]
impl Job for DirectoryUpdateJob {
    fn job_type(&self) -> &'static str {
        "directory-update-job"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    #[instrument(level = "trace", name = "directory_update_job", fields(job_name = %self.job_type()), skip_all)]
    async fn execute(&self, _scheduler: SchedulerHandle) -> anyhow::Result<()> {
        if self.base_url.is_none() {
            info!("Acme base uri not configured for job, will look in user state...");
        }
        let dir: AcmeDirectoryApi;
        let connection = DatabaseConnection::get_connection().unwrap();
        let result = AcmeDirectory::get_dir_from_user(self.user_id.clone(), &connection)?;
        let directory_result: Option<AcmeDirectory>;
        if result.is_none() {
            let url = self.base_url.clone().unwrap();
            let value = self.call_directory(url.clone()).await?;
            dir = from_value(value.clone())?;
            directory_result = AcmeDirectory::refresh_if_diff(dir, self.user_id.clone(), url, &connection)?;
        } else {
            let result = result.unwrap();
            let url = result.directory_base_url;
            let value = self.call_directory(url.clone()).await?;
            dir = from_value(value.clone())?;
            directory_result = AcmeDirectory::refresh_if_diff(dir, self.user_id.clone(), url, &connection)?;
        }
        if let Some(dir) = directory_result {
            info!("Directory refresh returned directory with id: {}", dir.directory_id);
        }
        Ok(())
    }
}
