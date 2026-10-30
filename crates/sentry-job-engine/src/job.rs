use async_trait::async_trait;
use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::scheduling::SchedulerHandle;

#[async_trait]
pub trait Job<'a>: Send + 'a {
    fn job_type(&self) -> &'a str;
    fn inject_dependencies(&mut self, dependencies: &JobDependencies);
    async fn execute(&self, handle: SchedulerHandle<'a>) -> Result<(), JobEngineError>;
}