use crate::error::ServerError;
use crate::orders;
use actix_web::{App, HttpServer, web};
use sentry_core::config::ApplicationConfig;
use sentry_core::services::Services;
use sentry_job_engine::dependencies::JobDependencies;
use sentry_job_engine::scheduling::SchedulerHandle;
use std::sync::Arc;
use tracing::{Instrument, info, info_span};

pub async fn start_http_server(
    application_config: Arc<ApplicationConfig>,
    handle: SchedulerHandle<JobDependencies>,
    services: Arc<Services>,
) -> Result<(), ServerError> {
    let bind_addr = "0.0.0.0:8080";
    let scheduler_data = web::Data::new(handle.clone());
    let config = web::Data::new(application_config.clone());
    let service_list = web::Data::new(services.clone());
    let server = HttpServer::new(move || {
        App::new()
            .app_data(scheduler_data.clone())
            .app_data(config.clone())
            .app_data(service_list.clone())
            .route("/order/new", web::post().to(orders::post_new_order))
            .route("/orders", web::get().to(orders::get_all_orders))
    })
    .workers(2)
    .bind(bind_addr)
    .map_err(|e| {
        ServerError::SentryApiError(format!(
            "Failed to bind HTTP server to {}: {}",
            bind_addr, e
        ))
    })?
    .run();
    let server_handle = server.handle();
    let server_span = info_span!("webserver");
    let srv_task = tokio::spawn(server.instrument(server_span));
    info!("Server available at: {}", bind_addr);
    info!("Order new certificate at: `/order/new`");
    info!("Check open orders and status at: `/orders`");
    let h = handle.clone();
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
