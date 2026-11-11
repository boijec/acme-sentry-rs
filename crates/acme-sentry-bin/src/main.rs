use crate::clap_config::{Operation, argument_man};
use crate::config_parsing::build_application_config;
use anyhow::anyhow;
use clap::{crate_authors, crate_version};
use sentry_api::server::start_http_server;
use sentry_core::config::{ApplicationConfig, ConfigurationMap};
use sentry_core::services::initialize_services;
use sentry_db::pool::connect;
use sentry_db::repo::initialize_repos;
use sentry_io::fs::BasicUnixFileSystem;
use sentry_job_engine::dependencies::JobDependencies;
use sentry_job_engine::jobs::preflight_start_sequence;
use sentry_job_engine::scheduling::Scheduler;
use std::process::exit;
use std::sync::Arc;
use tracing::{Instrument, Span, error, info, info_span};

mod config_parsing;
mod clap_config;
mod sanity_checker;

enum EndState {
    Success,
    Failure,
}

fn splash() -> String {
    format!(
        "{}\n",
        String::from_utf8_lossy(include_bytes!("../../../assets/ico.bin"))
    )
}

fn version() -> &'static str {
    format!("Version: {}\n", crate_version!()).leak()
}

fn about() -> &'static str {
    let authors = format!("By: {}\n", crate_authors!());
    format!("{}{}{}", splash(), version(), authors).leak()
}

fn main() -> Result<(), String> {
    // TODO: use matches
    let (_matches, operation) = argument_man()
        .map_err(|e| format!("Argument error: {}", e))?;
    print!("{}", splash());
    match operation {
        Operation::Run | Operation::Watch => {
            let dest = sanity_checker::check_surroundings()
                .map_err(|e| format!("Sanity check failed: {}", e))?;
            let conf = build_application_config(dest)
                .map_err(|e| format!("Application configuration failed: {}", e))?;
            let _ = run_application(conf, operation)
                .map_err(|e| format!("Application failed: {}", e))?;
        },
        Operation::Init => println!("init mode"),
        Operation::Invalid => {
            eprintln!("Invalid mode");
            exit(1);
        },
    }
    Ok(())
}

fn run_application(application_config: ApplicationConfig, op_mode: Operation) -> anyhow::Result<EndState> {
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

async fn async_main(application_config: ApplicationConfig, op_mode: Operation) -> anyhow::Result<EndState> {
    let fs = BasicUnixFileSystem::new(application_config.base_dir.as_str())?;
    let application_config = Arc::new(application_config);
    let file_system_operations = Arc::new(fs);
    let db_location = format!("{}/acme-sentry.db", application_config.base_dir.clone());
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
