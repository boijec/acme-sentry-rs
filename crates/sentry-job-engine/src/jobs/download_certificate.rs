use crate::dependencies::JobDependencies;
use crate::error::{ACMES133, JobEngineError};
use crate::job::{Job, JobExecutionFlag};
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::order::model::DownloadStatus;
use sentry_core::order::service::OrderService;
use sentry_core::user::service::UserService;
use std::sync::Arc;
use tracing::{error, info};

/// Polling retries before giving up on a processing order.
const MAX_POLL_RETRIES: u32 = 20;

#[derive(Default)]
pub struct DownloadCertificateJob {
    application_config: Option<Arc<ApplicationConfig>>,
    user_service: Option<Arc<UserService>>,
    order_service: Option<Arc<OrderService>>,

    order_id: i64,
    attempt: u32,
}

impl DownloadCertificateJob {
    pub fn initialize(order_id: i64) -> Self {
        DownloadCertificateJob {
            order_id,
            attempt: 1,
            ..Default::default()
        }
    }

    fn next_attempt(&self) -> Self {
        DownloadCertificateJob {
            order_id: self.order_id,
            attempt: self.attempt + 1,
            ..Default::default()
        }
    }
}

#[async_trait]
impl<'a> Job<'a> for DownloadCertificateJob {
    fn job_type(&self) -> &'a str {
        "download-certificate-job"
    }

    fn inject_dependencies(&mut self, dependencies: &JobDependencies) {
        self.application_config = Some(dependencies.configuration());
        self.user_service = Some(dependencies.services().user_service());
        self.order_service = Some(dependencies.services().order_service());
    }

    async fn execute(
        &self,
        handle: SchedulerHandle<'a>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let services = (
            &self.application_config,
            &self.user_service,
            &self.order_service,
        );
        if let (Some(config), Some(user_service), Some(order_service)) = services {
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
            return if let Some(certificate_bytes) = status {
                user_service
                    .create_pem_from_bytes(&user, order_id, certificate_bytes)
                    .await
                    .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
                Ok(JobExecutionFlag::Stop)
            } else {
                return if self.attempt < MAX_POLL_RETRIES {
                    info!(
                        "Order {} still processing (attempt {}/{}). Retrying in 10s.",
                        self.order_id, self.attempt, MAX_POLL_RETRIES
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    handle.submit(self.next_attempt()).await?;
                    Ok(JobExecutionFlag::Next)
                } else {
                    error!(
                        "Order {} did not become valid after {} poll attempts.",
                        self.order_id, MAX_POLL_RETRIES
                    );
                    Ok(JobExecutionFlag::Stop)
                };
            };
        }
        Err(JobEngineError::FlagError(ACMES133))
    }
}
