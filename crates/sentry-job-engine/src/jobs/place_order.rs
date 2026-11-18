use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::{Job, JobContext, JobExecutionFlag};
use crate::jobs::accept_dns_challenge::AcceptDnsChallengeJob;
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;

#[derive(Default)]
pub struct PlaceOrderJob {
    domain: String,
    // company_name: String,
    // business_unit: String
    // ...etc
}

impl PlaceOrderJob {
    pub fn instantiate(domain: String) -> PlaceOrderJob {
        PlaceOrderJob { domain }
    }
}

#[async_trait]
impl Job<JobDependencies> for PlaceOrderJob {
    fn job_type(&self) -> &str {
        "place-order-job"
    }

    async fn execute(
        self: Box<Self>,
        context: JobContext<JobDependencies>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let (user_service, config, order_service) = (
            context.deps.services().user_service(),
            context.deps.configuration(),
            context.deps.services().order_service(),
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

        let domain = self.domain.clone();

        let order = order_service
            .place_order_for_domain(domain, &user)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

        context
            .handle
            .submit(AcceptDnsChallengeJob::initialize(order.id), None)
            .await?;

        Ok(JobExecutionFlag::Next)
    }
}
