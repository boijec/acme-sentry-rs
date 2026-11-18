use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::{Job, JobContext, JobExecutionFlag};
use crate::jobs::download_certificate::DownloadCertificateJob;
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;

#[derive(Default)]
pub struct GenerateCsrJob {
    order_id: i64,
}

impl GenerateCsrJob {
    pub fn initialize(order_id: i64) -> Self {
        GenerateCsrJob { order_id }
    }
}

#[async_trait]
impl Job<JobDependencies> for GenerateCsrJob {
    fn job_type(&self) -> &str {
        "generate-csr-job"
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
            .ok_or_else(|| JobEngineError::JobExecutionError("User not found".into()))?;
        let pkey = user_service
            .create_private_key_for_cert(&user)
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        let order_id = self.order_id.clone();
        let order = order_service
            .get_order(order_id, &user)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        order_service
            .finalize(&order, &user, &pkey)
            .await
            .map_err(|e| JobEngineError::JobExecutionError(e.to_string()))?;
        context
            .handle
            .submit(DownloadCertificateJob::initialize(order_id), None)
            .await?;
        return Ok(JobExecutionFlag::Next);
    }
}
