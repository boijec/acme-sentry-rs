use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::{Job, JobContext, JobExecutionFlag};
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;

#[derive(Default)]
pub struct LoginUserJob;

impl LoginUserJob {
    pub fn instantiate() -> LoginUserJob {
        LoginUserJob::default()
    }
}

#[async_trait]
impl Job<JobDependencies> for LoginUserJob {
    fn job_type(&self) -> &str {
        "login-user-job"
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
            .get_user(&user_id)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?
            .ok_or_else(|| {
                JobEngineError::JobExecutionError(
                    "No user found matching configured user_id".into(),
                )
            })?;

        user_service
            .login(&user)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

        Ok(JobExecutionFlag::PreflightCompleted)
    }
}
