use crate::error::JobEngineError;
use crate::scheduling::JobHandle;
use async_trait::async_trait;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait Job<D>: Send + 'static {
    fn job_type(&self) -> &str;
    async fn execute(
        self: Box<Self>,
        ctx: JobContext<D>,
    ) -> Result<JobExecutionFlag, JobEngineError>;
}

pub trait JobKind<D>: Job<D> + Sized {}
impl<D, T: Job<D>> JobKind<D> for T {}

pub struct JobContext<D> {
    pub handle: JobHandle<D>,
    pub deps: Arc<D>,
    pub token: CancellationToken,
}

#[non_exhaustive]
#[derive(Debug, Default)]
pub enum JobExecutionFlag {
    PreflightNext,
    PreflightCompleted,
    #[default]
    Next,
    Fatal(JobErrorCode),
    Stop,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum JobErrorCode {
    Code(i32),
}

impl JobErrorCode {
    #[allow(unreachable_patterns)]
    pub fn explain(&self) -> String {
        match self {
            JobErrorCode::Code(err) => match err {
                132 => "(acmes132): job pipeline broken".to_uppercase(),
                133 => "(acmes133): inadequate dependencies during job execution".to_uppercase(),
                _ => format!("unmapped error code -> '{}'", err).to_uppercase(),
            },
            code => format!("unmapped error code -> '{:?}'", code).to_uppercase(),
        }
    }
}
