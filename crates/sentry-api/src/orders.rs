use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use chrono::DateTime;
use sentry_core::services::Services;
use sentry_job_engine::dependencies::JobDependencies;
use sentry_job_engine::jobs::place_order::PlaceOrderJob;
use sentry_job_engine::scheduling::SchedulerHandle;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewOrderRequest {
    pub domain: String,
}

pub async fn post_new_order(
    req: web::Json<NewOrderRequest>,
    handle: web::Data<SchedulerHandle<JobDependencies>>,
) -> Result<HttpResponse, ApiError> {
    info!("Received new order request: domain = '{}'", req.domain);
    match handle
        .submit(PlaceOrderJob::instantiate(req.domain.clone()), None)
        .await
    {
        Ok(_) => Ok(HttpResponse::Accepted().json(json!({
            "status": "accepted",
            "message": format!("Order for domain '{}' has been queued", req.domain)
        }))),
        Err(err) => Err(ApiError::Internal(format!("Failed to queue order: {}", err))),
    }
}

pub async fn get_all_orders(
    services: web::Data<Arc<Services>>,
) -> Result<HttpResponse, ApiError> {
    let order_service = services.order_service();
    let result = order_service.get_all_orders().await;
    match result {
        Ok(orders) => {
            let orders_json: Vec<serde_json::Value> = orders
                .into_iter()
                .map(|order| {
                    let epoc: i64 = order.created_at.parse().unwrap_or(0);
                    let t = DateTime::from_timestamp_secs(epoc).unwrap_or(DateTime::from(SystemTime::UNIX_EPOCH));
                    json!({
                        "id": order.id,
                        "domain": order.domain,
                        "status": order.status.to_string(),
                        "created_at": t.to_rfc3339(),
                    })
                })
                .collect();
            Ok(HttpResponse::Ok().json(json!({
                "orders": orders_json
            })))
        },
        Err(error) => Err(ApiError::Internal(format!("Error while fetching orders: {}", error))),
    }
}
