use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use reqwest::Url;
use sentry_core::config::ConfigurationMap;
use sentry_core::error::GenericJobError;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

pub struct DirectoryUpdateJob {
    config: Option<Arc<dyn ConfigurationMap + Send + Sync>>,
    fetch_local: bool
    // base_url: Option<String>,
    // user_id: String,
}

pub struct DirectoryUpdateJobBuilder {
    fetch_local: bool,
}

impl Default for DirectoryUpdateJobBuilder {
    fn default() -> Self {
        DirectoryUpdateJobBuilder { fetch_local: false }
    }
}

impl DirectoryUpdateJobBuilder {
    pub fn fetch_local(mut self) -> Self {
        self.fetch_local = true;
        self
    }
    pub fn build(self) -> DirectoryUpdateJob {
        DirectoryUpdateJob {
            config: None,
            fetch_local: self.fetch_local,
        }
    }
}

impl DirectoryUpdateJob {
    pub fn validate_url(base_url: Option<String>) -> Result<Url, GenericJobError> {
        if let Some(base_url) = base_url {
            let x = base_url.to_owned() + "/dir";
            let url = Url::parse(x.as_str())
                .map_err(|e| GenericJobError::GenericError("Unable to parse base URL".to_owned()))?;
            return Ok(url);
        }
        Err(GenericJobError::GenericError("Acme CA base url could not be parsed!".into()))
    }
    pub fn builder() -> DirectoryUpdateJobBuilder {
        DirectoryUpdateJobBuilder::default()
    }

    async fn call_directory(&self, base_url: String) -> Result<Value, GenericJobError> {
        let client = build_client(true)?;
        info!("Calling ACME directory at: {}", base_url);
        let response = client.get(base_url.as_str()).send().await?;
        info!("ACME directory returned response: {}", response.status());
        let bytes = response.bytes().await?;
        let slice = bytes.iter().as_slice();
        let value = serde_json::from_slice::<Value>(slice)?;
        Ok(value)
    }
}

#[async_trait]
impl<'a> Job<'a> for DirectoryUpdateJob {
    fn job_type(&self) -> &'a str {
        "directory-update-job"
    }

    fn inject_dependencies(&mut self, _dependencies: &JobDependencies) {
        return;
    }
    async fn execute(&self, _scheduler: SchedulerHandle<'a>) -> Result<(), JobEngineError> {
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
            directory_result =
                AcmeDirectory::refresh_if_diff(dir, self.user_id.clone(), url, &connection)?;
        } else {
            let result = result.unwrap();
            let url = result.directory_base_url;
            let value = self.call_directory(url.clone()).await?;
            dir = from_value(value.clone())?;
            directory_result =
                AcmeDirectory::refresh_if_diff(dir, self.user_id.clone(), url, &connection)?;
        }
        if let Some(dir) = directory_result {
            info!(
                "Directory refresh returned directory with id: {}",
                dir.directory_id
            );
        }
        Ok(())
    }
}
