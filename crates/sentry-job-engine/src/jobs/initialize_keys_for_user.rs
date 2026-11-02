use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::user::service::UserService;
use std::sync::Arc;

pub struct InitializeLocalUserJob {
    config: Option<Arc<ApplicationConfig>>,
    user_service: Option<Arc<UserService>>,
}

#[derive(Default)]
pub struct InitializeLocalUserJobBuilder {}

impl InitializeLocalUserJobBuilder {
    pub fn build(self) -> InitializeLocalUserJob {
        InitializeLocalUserJob {
            config: None,
            user_service: None,
        }
    }
}

impl InitializeLocalUserJob {
    pub fn builder() -> InitializeLocalUserJobBuilder {
        InitializeLocalUserJobBuilder::default()
    }
}

#[async_trait]
impl<'a> Job<'a> for InitializeLocalUserJob {
    fn job_type(&self) -> &'a str {
        "initialize-local-user-job"
    }
    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        let conf = dependencies.configuration();
        self.config = Some(conf);
        let usr_service = dependencies.services().user_service();
        self.user_service = Some(usr_service);
    }
    async fn execute(&self, _: SchedulerHandle<'a>) -> Result<(), JobEngineError> {
        if let (Some(user_service), Some(config)) = (&self.user_service, &self.config) {
            let config = config.clone();
            let user_service = user_service.clone();

            let user_id = config.get_user_id();
            let user = user_service.lazy_load_user(&user_id).await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            let _ = user_service.user_sanity_check(&user).await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

            return Ok(());
        }
        Err(JobEngineError::JobExecutionError(String::from(
            "No dependencies were provided",
        )))
    }
}
