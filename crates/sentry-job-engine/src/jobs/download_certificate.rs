use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::{Job, JobContext, JobExecutionFlag};
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;
use sentry_core::order::model::DownloadStatus;
use std::time::{Duration, SystemTime};
use tracing::{error, info};

/// Polling retries before giving up on a processing order.
const MAX_POLL_RETRIES: u32 = 20;

#[derive(Default)]
pub struct DownloadCertificateJob {
    order_id: i64,
    attempt: u32,
}

impl DownloadCertificateJob {
    pub fn initialize(order_id: i64) -> Self {
        DownloadCertificateJob {
            order_id,
            attempt: 1,
        }
    }

    fn next_attempt(&self) -> Self {
        DownloadCertificateJob {
            order_id: self.order_id,
            attempt: self.attempt + 1,
        }
    }
}

#[async_trait]
impl Job<JobDependencies> for DownloadCertificateJob {
    fn job_type(&self) -> &str {
        "download-certificate-job"
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
                JobEngineError::JobExecutionError(format!("user {} not found", user_id))
            })?;
        let order_id = self.order_id.clone();
        let order = order_service
            .get_order(order_id, &user)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        let result = order_service
            .try_download_pem(&order, &user)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        let status = match result {
            DownloadStatus::Result(bytes) => Some(bytes),
            DownloadStatus::ContinuePolling => None,
        };
        if let Some(certificate_bytes) = status {
            user_service
                .create_pem_from_bytes(&user, order_id, certificate_bytes)
                .await
                .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
            Ok(JobExecutionFlag::Stop)
        } else {
            if self.attempt < MAX_POLL_RETRIES {
                info!(
                    "Order {} still processing (attempt {}/{}). Retrying in 10s.",
                    self.order_id, self.attempt, MAX_POLL_RETRIES
                );
                context
                    .handle
                    .submit(
                        self.next_attempt(),
                        Some(SystemTime::now() + Duration::from_secs(10)),
                    )
                    .await?;
                Ok(JobExecutionFlag::Next)
            } else {
                error!(
                    "Order {} did not become valid after {} poll attempts.",
                    self.order_id, MAX_POLL_RETRIES
                );
                Ok(JobExecutionFlag::Stop)
            }
        }
    }
}
