use crate::dependencies::JobDependencies;
use crate::error::{ACMES133, JobEngineError};
use crate::job::{Job, JobExecutionFlag};
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::user::service::UserService;
use std::sync::Arc;

#[derive(Default)]
pub struct LoadLocalUserJob {
    config: Option<Arc<ApplicationConfig>>,
    user_service: Option<Arc<UserService>>,
}

impl LoadLocalUserJob {
    pub fn instantiate() -> LoadLocalUserJob {
        LoadLocalUserJob::default()
    }
}

#[async_trait]
impl<'a> Job<'a> for LoadLocalUserJob {
    fn job_type(&self) -> &'a str {
        "load-local-user"
    }
    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        self.config = Some(dependencies.configuration());
        self.user_service = Some(dependencies.services().user_service());
    }
    async fn execute(&self, _: SchedulerHandle<'a>) -> Result<JobExecutionFlag, JobEngineError> {
        let services = (&self.user_service, &self.config);
        if let (
            Some(user_service),
            Some(config)
        ) = services {
            let user_id = config.get_user_id();
            let user = user_service.lazy_load_user(&user_id).await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            let _ = user_service.user_sanity_check(&user).await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

            return Ok(JobExecutionFlag::PreflightNext);
        }
        Err(JobEngineError::FlagError(ACMES133))
    }
}
