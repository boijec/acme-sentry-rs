use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Sentry API error: {0}")]
    SentryApiError(String)
}