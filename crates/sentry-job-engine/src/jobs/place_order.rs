use crate::dependencies::JobDependencies;
use crate::error::{ACMES133, JobEngineError};
use crate::job::{Job, JobExecutionFlag};
use crate::jobs::accept_dns_challenge::AcceptDnsChallengeJob;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::order::service::OrderService;
use sentry_core::user::service::UserService;
use std::sync::Arc;

#[derive(Default)]
pub struct PlaceOrderJob {
    config: Option<Arc<ApplicationConfig>>,
    user_service: Option<Arc<UserService>>,
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
        self.order_service = Some(dependencies.services().order_service());
    }

    async fn execute(
        &self,
        handle: SchedulerHandle<'a>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let services = (&self.user_service, &self.config, &self.order_service);
        if let (Some(user_service), Some(config), Some(order_service)) = services {
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

            let domain = self.domain.clone();

            let order = order_service
                .place_order_for_domain(domain, &user)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

            handle
                .submit(AcceptDnsChallengeJob::initialize(order.id))
                .await?;

            return Ok(JobExecutionFlag::Next);
        }
        Err(JobEngineError::FlagError(ACMES133))
    }
}
