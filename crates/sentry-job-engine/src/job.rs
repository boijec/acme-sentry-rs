use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;

#[async_trait]
pub trait Job<'a>: Send + 'a {
    fn job_type(&self) -> &'a str;
    fn inject_dependencies(&mut self, dependencies: &JobDependencies);
    async fn execute(
        &self,
        handle: SchedulerHandle<'a>,
    ) -> Result<JobExecutionFlag, JobEngineError>;
}

#[non_exhaustive]
#[derive(Debug)]
pub enum JobExecutionFlag {
    PreflightNext,
    PreflightCompleted,
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
                132 => "(acmes123): job pipeline broken".to_uppercase(),
                133 => "(acmes133): inadequate dependencies during job execution".to_uppercase(),
                _ => format!("unmapped error code -> '{}'", err).to_uppercase(),
            },
            code => format!("unmapped error code -> '{:?}'", code).to_uppercase(),
        }
    }
}
