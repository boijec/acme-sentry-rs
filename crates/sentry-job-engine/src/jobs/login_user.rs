use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;
use std::sync::Arc;
use tracing::info;

pub struct LoginUserJob {
    config: Option<Arc<dyn ConfigurationMap + Send + Sync>>,
    // user_id: String,
}

#[derive(Default)]
pub struct LoginUserJobBuilder;

impl LoginUserJob {
    pub fn new(user_id: String) -> Self {
        LoginUserJob { user_id }
    }
    pub fn builder() -> LoginUserJobBuilder {
        LoginUserJobBuilder::default()
    }
}

impl LoginUserJobBuilder {
    pub fn build(self) -> LoginUserJob {
        LoginUserJob {
            config: None
        }
    }
}

#[async_trait]
impl<'a> Job<'a> for LoginUserJob {
    fn job_type(&self) -> &'a str {
        "login-user-job"
    }

    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        let conf = dependencies.configuration();
        self.config = Some(conf);
    }

    async fn execute(&self, _handle: SchedulerHandle<'a>) -> Result<(), JobEngineError> {
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
