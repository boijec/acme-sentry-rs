use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::JobErrorCode::ACMES133;
use crate::job::{Job, JobExecutionFlag};
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::directory::service::DirectoryService;
use sentry_core::user::service::UserService;
use std::sync::Arc;

#[derive(Default)]
pub struct LoginUserJob {
    config: Option<Arc<ApplicationConfig>>,
    user_service: Option<Arc<UserService>>,
    directory_service: Option<Arc<DirectoryService>>
}

impl LoginUserJob {
    pub fn instantiate() -> LoginUserJob {
        LoginUserJob::default()
    }
}

#[async_trait]
impl<'a> Job<'a> for LoginUserJob {
    fn job_type(&self) -> &'a str {
        "login-user-job"
    }

    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        self.config = Some(dependencies.configuration());
        self.user_service = Some(dependencies.services().user_service());
        self.directory_service = Some(dependencies.services().directory_service());
    }

    async fn execute(&self, _handle: SchedulerHandle<'a>) -> Result<JobExecutionFlag, JobEngineError> {
        let services = (&self.user_service, &self.config, &self.directory_service);
        if let (
            Some(user_service),
            Some(config),
            Some(directory_service)
        ) = services {
            let config = config.clone();
            let user_service = user_service.clone();
            let directory_service = directory_service.clone();

            let user_id = config.get_user_id();
            let user = user_service.get_user(&user_id)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?
                .ok_or_else(|| JobEngineError::JobExecutionError("No user found matching configured user_id".into()))?;

            let directory = directory_service.fetch().await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

            let _ = user_service.login(&user, &directory).await;

            return Ok(JobExecutionFlag::PreflightCompleted)
        }
        Ok(JobExecutionFlag::Fatal(ACMES133))
    }
}
