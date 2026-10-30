use actix_web::{web, HttpServer, App};
use tracing::info;
use sentry_core::config::ApplicationConfig;
use sentry_job_engine::scheduling::SchedulerHandle;
use crate::error::ServerError;

pub async fn start_http_server(application_config: &ApplicationConfig, handle: SchedulerHandle<'_>) -> Result<(), ServerError> {
    // TODO: add to yaml config / args params
    let bind_addr = "0.0.0.0:8080";
    let scheduler_data = web::Data::new(handle.clone());
    let server = HttpServer::new(move || {
        App::new()
            .app_data(scheduler_data.clone())
            .route("/orders", web::post().to(http_server::post_new_order))
            .route("/account", web::get().to(http_server::check_account_status))
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