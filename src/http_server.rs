use crate::acme_jobs::place_order::PlaceOrderJob;
use crate::job_execution::job_base::SchedulerHandle;
use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
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
