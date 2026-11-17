use crate::clap_config::Operation;
use anyhow::anyhow;
use sentry_api::server::start_http_server;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::services::initialize_services;
use sentry_db::pool::{DB_FILE_NAME, connect};
use sentry_db::repo::initialize_repos;
use sentry_io::fs::FileSystemOperations;
use sentry_job_engine::dependencies::JobDependencies;
use sentry_job_engine::jobs::preflight_start_sequence;
use sentry_job_engine::scheduling::Scheduler;
use std::process::ExitCode;
use std::sync::Arc;
use tracing::{Instrument, Span, error, info, info_span};
use sentry_core::plugin::tooling::PluginTool;
use sentry_plugin_engine::engine::PluginEngine;

pub(crate) fn run(
    application_config: Arc<ApplicationConfig>,
    fs: Arc<dyn FileSystemOperations + Send + Sync>,
    op_mode: Operation,
) -> anyhow::Result<ExitCode> {
    let span = info_span!("main");
    tokio::runtime::Runtime::new()?
        .block_on(tree_start(application_config, fs, op_mode).instrument(span))
        .map_err(|e| e)
}

async fn tree_start(
    application_config: Arc<ApplicationConfig>,
    file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,
    op_mode: Operation,
) -> anyhow::Result<ExitCode> {
    let db_location = format!("{}/{}", application_config.get_base_dir(), DB_FILE_NAME);
    let pool = connect(&db_location).await?;
    let plugin_tool = Arc::new(PluginTool::new(file_system_operations.clone()));
    let plugin_engine = Arc::new(PluginEngine::new());
    let repos = initialize_repos(&pool);
    let services = Arc::new(initialize_services(
        application_config.clone(),
        repos,
        file_system_operations.clone(),
        plugin_tool.clone(),
        plugin_engine.clone(),
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
        return Ok(ExitCode::FAILURE);
    }

    match op_mode {
        Operation::Watch => {
            info!("Application mode has been enabled, monitoring input signals.");
            let _ = start_http_server(application_config.clone(), handle.clone(), services.clone())
                .await
                .map_err(|err| anyhow!(err))?;
            Ok(ExitCode::SUCCESS)
        }
        Operation::OneShot => {
            info!("Single shot mode enabled waiting for engine 'done' flag");
            let mut shutdown_handle = handle.clone();
            shutdown_handle.wait_for_completion().await;
            Ok(ExitCode::SUCCESS)
        }
        _ => Err(anyhow::anyhow!("Invalid operation mode for main")),
    }
}
