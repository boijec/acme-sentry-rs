use crate::error::ServerError;
use crate::orders;
use actix_web::{App, HttpServer, web};
use sentry_core::config::ApplicationConfig;
use sentry_core::services::Services;
use sentry_job_engine::scheduling::SchedulerHandle;
use std::sync::Arc;
use tracing::info;

pub async fn start_http_server(application_config: Arc<ApplicationConfig>, handle: SchedulerHandle<'_>, services: Arc<Services>) -> Result<(), ServerError> {
    // TODO: add to yaml config / args params
    let bind_addr = "0.0.0.0:8080";
    let scheduler_data = web::Data::new(handle.clone());
    let server = HttpServer::new(move || {
        App::new()
            .app_data(scheduler_data.clone())
            .app_data(application_config)
            .app_data(services)
            .route("/orders", web::post().to(orders::post_new_order))
    })
        .workers(2)
        .bind(bind_addr)
        .map_err(|e| ServerError::SentryApiError(format!("Failed to bind HTTP server to {}: {}", bind_addr, e)))?
        .run();
    let server_handle = server.handle();
    let srv_task = tokio::spawn(server);
    info!("HTTP server listening on {}", bind_addr);
    let mut h = handle.clone();
    tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                server_handle.stop(true).await;
                h.shutdown().await;
            }
            _ = h.wait_for_shutdown() => {
                server_handle.stop(true).await;
            }
        }
    let _ = srv_task.await;
    Ok(())
}