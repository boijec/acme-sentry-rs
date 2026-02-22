use crate::job_execution::job_base::{Job, SchedulerHandle};
use async_trait::async_trait;
use persistence::database::DatabaseConnection;
use persistence::directory::AcmeDirectory;
use persistence::user::AcmeUser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info};

#[derive(Serialize, Deserialize)]
pub struct LoginUserJob {
    pub user_id: String,
}
impl LoginUserJob {
    pub fn new(user_id: String) -> Self {
        LoginUserJob { user_id }
    }
    fn log_in(&self, dir: AcmeDirectory, user_id: String, base_path: String) -> anyhow::Result<()> {
        debug!("Will attempt to fetch key from {}/{}.pem", base_path, user_id);

        Ok(())
    }
}
#[async_trait]
impl Job for LoginUserJob {
    fn job_type(&self) -> &'static str {
        "login-user-job"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    async fn execute(&self, handle: SchedulerHandle) -> anyhow::Result<()> {
        let connection = DatabaseConnection::get_connection().unwrap();
        let user = AcmeUser::get_user(self.user_id.as_str(), &connection).unwrap();
        if user.is_none() {
            return Err(anyhow::anyhow!(
                "Could not complete directory job since queried user could not be found!"
            ))?;
        }
        let user = user.unwrap();
        let result = AcmeDirectory::get_existing(user.id, &connection)?;
        let result = result.unwrap();
        self.log_in(result, user.user_id, user.key_path)?;
        Ok(())
    }
}