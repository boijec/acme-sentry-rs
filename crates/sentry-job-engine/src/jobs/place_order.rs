use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::JobErrorCode::ACMES133;
use crate::job::{Job, JobExecutionFlag};
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::directory::service::DirectoryService;
use sentry_core::order::service::OrderService;
use sentry_core::user::service::UserService;
use std::sync::Arc;

#[derive(Default)]
pub struct PlaceOrderJob {
    config: Option<Arc<ApplicationConfig>>,
    user_service: Option<Arc<UserService>>,
    directory_service: Option<Arc<DirectoryService>>,
    order_service: Option<Arc<OrderService>>,

    domain: String,
    // company_name: String,
    // business_unit: String
    // ...etc
}

impl PlaceOrderJob {
    pub fn instantiate(domain: String) -> PlaceOrderJob {
        PlaceOrderJob {
            domain,
            ..Default::default()
        }
    }
}

#[async_trait]
impl<'a> Job<'a> for PlaceOrderJob {
    fn job_type(&self) -> &'a str {
        "place-order-job"
    }

    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        self.config = Some(dependencies.configuration());
        self.user_service = Some(dependencies.services().user_service());
        self.directory_service = Some(dependencies.services().directory_service());
        self.order_service = Some(dependencies.services().order_service());
    }

    async fn execute(&self, handle: SchedulerHandle<'a>) -> Result<JobExecutionFlag, JobEngineError> {
        let services = (&self.user_service, &self.config, &self.directory_service, &self.order_service);
        if let (
            Some(user_service),
            Some(config),
            Some(directory_service),
            Some(order_service)
        ) = services {
            let config = config.clone();
            let user_service = user_service.clone();
            let directory_service = directory_service.clone();
            let order_service = order_service.clone();

            let user_id = config.get_user_id();
            let user = user_service.get_user(&user_id)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?
                .ok_or_else(|| JobEngineError::JobExecutionError("No user found matching configured user_id".into()))?;

            let directory = directory_service.fetch().await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            let domain = self.domain.clone();

            let order = order_service.place_order_for_domain(domain, &user, &directory).await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

            // handle
            //     .submit(AcceptDnsChallengeJob::new(order.id, self.user_id.clone()))
            //     .await
            //     .map_err(|_| anyhow::anyhow!("Failed to submit AcceptDnsChallengeJob"))?;

            return Ok(JobExecutionFlag::Stop)
        }
        return Ok(JobExecutionFlag::Fatal(ACMES133));
    }
}

