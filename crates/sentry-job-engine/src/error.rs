use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobEngineError {
    #[error("Error during execution of job: {0}")]
    JobExecutionError(String),
    #[error("Error during scheduling of job: {0}")]
    SchedulingError(String),
}

#[derive(Error, Debug)]
pub enum PreFlightError {
    #[error("{0}")]
    PreFlightScheduling(#[from] JobEngineError),
}