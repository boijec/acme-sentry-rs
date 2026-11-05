use crate::dependencies::JobDependencies;
use crate::error::{ACMES133, JobEngineError};
use crate::job::{Job, JobExecutionFlag};
use crate::jobs::generate_csr::GenerateCsrJob;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::challenge::model::ChallengeMark;
use sentry_core::dns::service::DnsService;
use sentry_core::order::service::OrderService;
use std::sync::Arc;
use tracing::{info, warn};

/// Maximum number of polling retries before giving up on DNS propagation.
const MAX_DIG_RETRIES: u32 = 30;

#[derive(Default)]
pub struct DnsDigJob {
    order_service: Option<Arc<OrderService>>,
    dns_service: Option<Arc<DnsService>>,

    order_id: i64,
    attempt: u32,
}

impl DnsDigJob {
    pub fn initialize(order_id: i64) -> Self {
        DnsDigJob {
            order_id,
            attempt: 1,
            ..Default::default()
        }
    }

    fn next_attempt(&self) -> Self {
        DnsDigJob {
            order_id: self.order_id,
            attempt: self.attempt + 1,
            ..Default::default()
        }
    }
}

#[async_trait]
impl<'a> Job<'a> for DnsDigJob {
    fn job_type(&self) -> &'a str {
        "dns-dig-job"
    }

    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        self.order_service = Some(dependencies.services().order_service());
        self.dns_service = Some(dependencies.services().dns_service());
    }

    async fn execute(
        &self,
        handle: SchedulerHandle<'a>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let services = (&self.order_service, &self.dns_service);
        if let (
            Some(order_service),
            Some(dns_service),
        ) = services {
            let id = self.order_id.clone();
            let challenge = order_service
                .get_order_challenge(id)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?
                .ok_or_else(|| {
                    JobEngineError::JobExecutionError("No challenge found for order".into())
                })?;

            info!(
                "Dig attempt {}/{}",
                self.attempt,
                MAX_DIG_RETRIES,
            );
            let propagated = dns_service
                .compute_and_dig(&challenge)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            return if propagated {
                let order = order_service.__get_order_any(id)
                    .await
                    .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
                order_service.mark(order, ChallengeMark::Verified)
                    .await
                    .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
                handle
                    .submit(GenerateCsrJob::initialize(id))
                    .await?;
                Ok(JobExecutionFlag::Next)
            } else if self.attempt >= MAX_DIG_RETRIES {
                warn!(
                    "DNS TXT record for '{}' not found after {} attempts — marking order as invalid.",
                    challenge.dns_record_name, MAX_DIG_RETRIES
                );
                let order = order_service.__get_order_any(id)
                    .await
                    .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
                order_service.mark(order, ChallengeMark::Failed)
                    .await
                    .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
                Ok(JobExecutionFlag::Stop)
            } else {
                warn!(
                    "DNS TXT record for '{}' not yet visible (attempt {}/{}). Retrying in 30s.",
                    challenge.dns_record_name,
                    self.attempt,
                    MAX_DIG_RETRIES
                );
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                handle
                    .submit(self.next_attempt())
                    .await?;
                Ok(JobExecutionFlag::Next)
            }
        }
        Err(JobEngineError::FlagError(ACMES133))
    }
}
