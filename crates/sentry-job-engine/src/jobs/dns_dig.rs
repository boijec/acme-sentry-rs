use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::{Job, JobContext, JobExecutionFlag};
use crate::jobs::generate_csr::GenerateCsrJob;
use async_trait::async_trait;
use sentry_core::challenge::model::ChallengeMark;
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

/// Maximum number of polling retries before giving up on DNS propagation.
const MAX_DIG_RETRIES: u32 = 30;

#[derive(Default)]
pub struct DnsDigJob {
    order_id: i64,
    attempt: u32,
}

impl DnsDigJob {
    pub fn initialize(order_id: i64) -> Self {
        DnsDigJob {
            order_id,
            attempt: 1,
        }
    }

    fn next_attempt(&self) -> Self {
        DnsDigJob {
            order_id: self.order_id,
            attempt: self.attempt + 1,
        }
    }
}

#[async_trait]
impl Job<JobDependencies> for DnsDigJob {
    fn job_type(&self) -> &str {
        "dns-dig-job"
    }

    async fn execute(
        self: Box<Self>,
        context: JobContext<JobDependencies>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let (order_service, dns_service) = (
            context.deps.services().order_service(),
            context.deps.services().dns_service(),
        );
        let id = self.order_id.clone();
        let challenge = order_service
            .get_order_challenge(id)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?
            .ok_or_else(|| {
                JobEngineError::JobExecutionError("No challenge found for order".into())
            })?;

        info!("Dig attempt {}/{}", self.attempt, MAX_DIG_RETRIES,);
        let propagated = dns_service
            .compute_and_dig(&challenge)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        return if propagated {
            let order = order_service
                .__get_order_any(id)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            order_service
                .mark(order, ChallengeMark::Verified)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            context
                .handle
                .submit(GenerateCsrJob::initialize(id), None)
                .await?;
            Ok(JobExecutionFlag::Next)
        } else if self.attempt >= MAX_DIG_RETRIES {
            warn!(
                "DNS TXT record for '{}' not found after {} attempts — marking order as invalid.",
                challenge.dns_record_name, MAX_DIG_RETRIES
            );
            let order = order_service
                .__get_order_any(id)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            order_service
                .mark(order, ChallengeMark::Failed)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            Ok(JobExecutionFlag::Stop)
        } else {
            warn!(
                "DNS TXT record for '{}' not yet visible (attempt {}/{}). Retrying in 30s.",
                challenge.dns_record_name, self.attempt, MAX_DIG_RETRIES
            );
            context
                .handle
                .submit(
                    self.next_attempt(),
                    Some(SystemTime::now() + Duration::from_secs(30)),
                )
                .await?;
            Ok(JobExecutionFlag::Next)
        };
    }
}
