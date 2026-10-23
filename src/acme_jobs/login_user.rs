use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::account::AcmeAccountClient;
use acme_client::keys::load_private_key;
use async_trait::async_trait;
use persistence::account_link::AcmeAccountLink;
use persistence::database::DatabaseConnection;
use persistence::directory::AcmeDirectory;
use persistence::user::AcmeUser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct LoginUserJob {
    pub user_id: String,
}

impl LoginUserJob {
    pub fn new(user_id: String) -> Self {
        LoginUserJob { user_id }
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

    async fn execute(&self, _handle: SchedulerHandle) -> anyhow::Result<()> {
        let connection = DatabaseConnection::get_connection()
            .map_err(|e| anyhow::anyhow!("DB connection failed: {}", e))?;

        let user = AcmeUser::get_user(self.user_id.as_str(), &connection)
            .map_err(|e| anyhow::anyhow!("Failed to look up user: {}", e))?
            .ok_or_else(|| anyhow::anyhow!(
                "Could not complete login job since queried user could not be found!"
            ))?;

        let dir = AcmeDirectory::get_existing(user.id, &connection)?
            .ok_or_else(|| anyhow::anyhow!("No ACME directory found for user '{}'", self.user_id))?;

        // TODO: export to separate function and use where key needs to be fetched, for posteriors sake
        let private_key = load_private_key(&user.user_id, &user.key_path, &user.key_type)?;

        let account_url = AcmeAccountClient::login_or_register(
            &private_key,
            &dir.new_nonce,
            &dir.new_account,
            true, // TODO: configure via application config
        )
        .await?;

        AcmeAccountLink::upsert_for_user_id(self.user_id.as_str(), &account_url, &connection)
            .map_err(|e| anyhow::anyhow!("Failed to persist account link: {}", e))?;

        info!("Account link persisted for user '{}'.", self.user_id);
        Ok(())
    }
}
