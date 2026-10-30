use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobEngineError {
    #[error("Error during execution of job: {0}")]
    JobExecutionError(String)
}