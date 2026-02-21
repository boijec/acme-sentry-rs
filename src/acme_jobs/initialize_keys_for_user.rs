use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::crypto::SupportedKey;
use acme_client::keys::PrivateKey;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use tracing::instrument;

#[derive(Serialize, Deserialize)]
pub struct InitializeLocalUserJob {
    path: String,
    key_type: String,
}
impl InitializeLocalUserJob {
    pub fn new(path: String, key_type: String) -> Self {
        InitializeLocalUserJob { path, key_type }
    }
    fn create_from_incoming_type(&self) -> Result<PrivateKey, Box<dyn Error>> {
        let supported_key = SupportedKey::from_str(self.key_type.as_str())?;
        PrivateKey::from_supported_type(supported_key)
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
        let result = self.create_from_incoming_type();
        Ok(())
    }
}