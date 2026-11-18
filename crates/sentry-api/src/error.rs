use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Sentry web-server error: {0}")]
    SentryApiError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub error: &'static str,
    pub message: String,
}

#[allow(unused)]
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Validation failed: {0}")]
    BadRequest(String),
    #[error("Internal server error occurred")]
    Internal(String),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        let error_payload = ErrorResponse {
            code: status_code.as_u16(),
            error: match self {
                ApiError::NotFound(_) => "NOT_FOUND",
                ApiError::BadRequest(_) => "BAD_REQUEST",
                ApiError::Internal(_) => "INTERNAL_SERVER_ERROR",
            },
            message: self.to_string(),
        };

        HttpResponse::build(status_code).json(error_payload)
    }
}