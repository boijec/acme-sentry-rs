use std::io::Stderr;
use crate::acme_jobs::place_order::PlaceOrderJob;
use crate::job_execution::job_base::SchedulerHandle;
use actix_web::{web, Error, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use acme_client::comms::request;
use common_utils::APPLICATION_CONFIG;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewOrderRequest {
    pub domain: String,
}

pub async fn post_new_order(
    req: web::Json<NewOrderRequest>,
    handle: web::Data<SchedulerHandle>,
) -> impl Responder {
    info!(
        "Received new order request: domain='{}'",
        req.domain
    );
    let user_id = APPLICATION_CONFIG.get().unwrap().user_id.clone();
    match handle
        .submit(PlaceOrderJob::new(user_id, req.domain.clone()))
        .await
    {
        Ok(_) => HttpResponse::Accepted().json(json!({
            "status": "accepted",
            "message": format!("Order for domain '{}' has been queued.", req.domain)
        })),
        Err(err) => HttpResponse::ServiceUnavailable().json(json!({
            "status": "error",
            "message": format!("Failed to queue order: {}", err)
        })),
    }
}

#[derive(Debug)]
pub enum ApiResponseError {
    ClientBuildError(anyhow::Error),
}

impl std::fmt::Display for ApiResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ApiResponseError {
    pub fn as_str(&self) -> String {
        match self { ApiResponseError::ClientBuildError(e) => e.root_cause().to_string() }
    }
}

impl actix_web::ResponseError for ApiResponseError {}

pub async fn check_account_status() -> Result<impl Responder, Error> {
    let user_id = APPLICATION_CONFIG.get().unwrap().user_id.clone();
    let client = request::build_client(true)
        .map_err(|err| ApiResponseError::ClientBuildError(err))?;
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok"
    })))
}
