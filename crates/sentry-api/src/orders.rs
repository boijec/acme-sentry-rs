use actix_web::dev::AppConfig;
use actix_web::{HttpResponse, Responder, web};
use sentry_core::config::ApplicationConfig;
use sentry_job_engine::jobs::place_order::PlaceOrderJob;
use sentry_job_engine::scheduling::SchedulerHandle;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewOrderRequest {
    pub domain: String,
}

pub async fn post_new_order<'a>(
    req: web::Json<NewOrderRequest>,
    handle: web::Data<SchedulerHandle<'a>>,
    application_config: web::Data<ApplicationConfig>,
) -> impl Responder {
    info!("Received new order request: domain='{}'", req.domain);
    let user_id = application_config.user_id;
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