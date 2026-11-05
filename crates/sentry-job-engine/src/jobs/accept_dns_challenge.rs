use crate::dependencies::JobDependencies;
use crate::error::{ACMES133, JobEngineError};
use crate::job::{Job, JobExecutionFlag};
use crate::jobs::dns_dig::DnsDigJob;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::order::service::OrderService;
use sentry_core::user::service::UserService;
use std::sync::Arc;

#[derive(Default)]
pub struct AcceptDnsChallengeJob {
    config: Option<Arc<ApplicationConfig>>,
    user_service: Option<Arc<UserService>>,
    order_service: Option<Arc<OrderService>>,

    order_id: i64,
}

impl AcceptDnsChallengeJob {
    pub fn initialize(order_id: i64) -> AcceptDnsChallengeJob {
        AcceptDnsChallengeJob {
            order_id,
            ..Default::default()
        }
    }
}

#[async_trait]
impl<'a> Job<'a> for AcceptDnsChallengeJob {
    fn job_type(&self) -> &'a str {
        "accept-dns-challenge-job"
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
        let services = (&self.config, &self.user_service, &self.order_service);
        if let (Some(config), Some(user_service), Some(order_service)) = services {
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

            let order_id = self.order_id;
            let order = order_service
                .get_order(order_id, &user)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

            let _ = order_service
                .prepare(&user, order)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;

            handle.submit(DnsDigJob::initialize(order_id)).await?;

            return Ok(JobExecutionFlag::Next);
        }
        Err(JobEngineError::FlagError(ACMES133))
    }
}
