use async_trait::async_trait;
use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::scheduling::SchedulerHandle;

#[async_trait]
pub trait Job<'a>: Send + 'a {
    fn job_type(&self) -> &'a str;
    fn inject_dependencies(&mut self, dependencies: &JobDependencies);
    async fn execute(&self, handle: SchedulerHandle<'a>) -> Result<JobExecutionFlag, JobEngineError>;
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
    ACMES132,
    ACMES133,
}

impl JobErrorCode {
    #[allow(unreachable_patterns)]
    pub fn explain(&self) -> String {
        match self {
            JobErrorCode::ACMES132 => "(acmes123): job pipeline broken".to_uppercase(),
            JobErrorCode::ACMES133 => "(acmes133): inadequate dependencies during job execution".to_uppercase(),
            code => format!("unmapped error code -> '{:?}'", code).to_uppercase(),
        }
    }
}