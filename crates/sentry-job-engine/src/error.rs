use crate::job::JobErrorCode::Code;
use crate::job::JobExecutionFlag;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobEngineError {
    #[error("Error during execution of job :: {0}")]
    JobExecutionError(String),
    #[error("Error during scheduling of job :: {0}")]
    SchedulingError(String),
    #[error("Job crashed with job execution flag")]
    FlagError(JobExecutionFlag),
}

#[derive(Error, Debug)]
pub enum PreFlightError {
    #[error("{0}")]
    PreFlightScheduling(#[from] JobEngineError),
}

pub const ACMES132: JobExecutionFlag = JobExecutionFlag::Fatal(Code(132));
pub const ACMES133: JobExecutionFlag = JobExecutionFlag::Fatal(Code(133));
