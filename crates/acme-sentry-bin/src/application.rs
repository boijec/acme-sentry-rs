use std::sync::Arc;
use anyhow::anyhow;
use tracing::{info_span, Span, info, error, Instrument};
use sentry_api::server::start_http_server;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::services::initialize_services;
use sentry_db::pool::{connect, DB_FILE_NAME};
use sentry_db::repo::initialize_repos;
use sentry_io::fs::BasicUnixFileSystem;
use sentry_job_engine::dependencies::JobDependencies;
use sentry_job_engine::jobs::preflight_start_sequence;
use sentry_job_engine::scheduling::Scheduler;
use crate::clap_config::Operation;
use crate::EndState;

pub(crate) fn run_application(application_config: ApplicationConfig, op_mode: Operation) -> anyhow::Result<EndState> {
    let level = application_config.get_logging_level()
        .ok_or_else(|| anyhow!("No logging level"))?;
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    let span = info_span!("main");
    tokio::runtime::Runtime::new()?
        .block_on(async_main(application_config, op_mode).instrument(span))
        .map_err(|e| e)
}

pub(crate) async fn async_main(application_config: ApplicationConfig, op_mode: Operation) -> anyhow::Result<EndState> {
    let fs = BasicUnixFileSystem::new(application_config.base_dir.as_str())?;
    let application_config = Arc::new(application_config);
    let file_system_operations = Arc::new(fs);
    let db_location = format!("{}/{}", application_config.base_dir.clone(), DB_FILE_NAME);
    let pool = connect(&db_location).await?;
    let repos = initialize_repos(&pool);
    let services = Arc::new(initialize_services(
        application_config.clone(),
        repos,
        file_system_operations.clone(),
    ));

    let (mut scheduler, handle) = Scheduler::new(32);
    scheduler.with_dependencies(JobDependencies::new(
        application_config.clone(),
        services.clone(),
    ));
    let scheduler_span = info_span!("scheduler");
    scheduler_span.follows_from(Span::current());
    let _ = tokio::spawn(scheduler.run(handle.clone()).instrument(scheduler_span));

    let _ = preflight_start_sequence(&handle).await?;
    let mut preflight_handler = handle.clone();
    let pre_flight_result = preflight_handler.wait_for_healthy().await;
    if let Ok(_) = pre_flight_result {
        info!("Preflight steps ran successfully");
    } else if let Err(error) = pre_flight_result {
        error!("Preflight steps raised error :: {}", error.to_string());
        handle.shutdown().await;
        preflight_handler.wait_for_shutdown().await;
        return Ok(EndState::Failure);
    }

    match op_mode {
        Operation::Watch => {
            info!("Application mode has been enabled, monitoring input signals.");
            let _ = start_http_server(application_config.clone(), handle.clone(), services.clone())
                .await
                .map_err(|err| anyhow!(err))?;
            Ok(EndState::Success)
        },
        Operation::Run => {
            info!("Single shot mode enabled waiting for engine 'done' flag");
            let mut shutdown_handle = handle.clone();
            shutdown_handle.wait_for_completion().await;
            Ok(EndState::Success)
        },
        _ => {
            Err(anyhow::anyhow!("Invalid operation mode for main"))
        }
    }
}