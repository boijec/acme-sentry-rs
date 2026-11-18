use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::{Job, JobContext, JobExecutionFlag};
use crate::jobs::login_user::LoginUserJob;
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;

#[derive(Default)]
pub struct LoadLocalUserJob;

impl LoadLocalUserJob {
    pub fn instantiate() -> LoadLocalUserJob {
        LoadLocalUserJob::default()
    }
}

#[async_trait]
impl Job<JobDependencies> for LoadLocalUserJob {
    fn job_type(&self) -> &str {
        "load-local-user"
    }

    async fn execute(
        self: Box<Self>,
        context: JobContext<JobDependencies>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let (user_service, config) = (
            context.deps.services().user_service(),
            context.deps.configuration(),
        );
        let user_id = config.get_user_id();
        let user = user_service
            .lazy_load_user(&user_id)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        let _ = user_service
            .user_sanity_check(&user)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        context
            .handle
            .submit(LoginUserJob::instantiate(), None)
            .await?;
        Ok(JobExecutionFlag::PreflightNext)
    }
}
