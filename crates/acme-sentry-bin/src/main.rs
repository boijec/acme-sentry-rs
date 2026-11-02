use crate::config_parsing::{Args, build_application_config};
use clap::{Parser, crate_version};
use sentry_api::server::start_http_server;
use sentry_core::config::ApplicationConfig;
use sentry_core::services::initialize_services;
use sentry_db::pool::connect;
use sentry_db::repo::initialize_repos;
use sentry_io::fs::BasicUnixFileSystem;
use sentry_job_engine::dependencies::JobDependencies;
use sentry_job_engine::jobs::preflight_start_sequence;
use sentry_job_engine::scheduling::Scheduler;
use std::sync::Arc;
use anyhow::anyhow;
use tracing::{Instrument, Span, info, info_span};

mod config_parsing;

fn splash(print_version: bool) {
    print!(
        "{}\n",
        String::from_utf8_lossy(include_bytes!("../../../assets/ico.bin"))
    );
    if print_version {
        println!("Version: {}", crate_version!());
        print!(
            "{}\n",
            String::from_utf8_lossy(include_bytes!("../../../assets/creators.bin"))
        );
    }
}

fn main() {
    let args = Args::parse();
    splash(args.clone().version);
    if args.version {
        return;
    }
    let conf = build_application_config(args.clone()).unwrap();
    tracing_subscriber::fmt()
        .with_max_level(conf.logging_level.unwrap())
        .init();
    let span = info_span!("main");
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main(conf).instrument(span))
        .expect("Tokio runtime panicked with error:");
}

async fn async_main(application_config: ApplicationConfig) -> anyhow::Result<()> {
    let fs = BasicUnixFileSystem::new(application_config.output_dir.as_str())?;
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

    if application_config.application_mode {
        info!("Application mode has been enabled, monitoring input signals.");
        let _ = start_http_server(application_config.clone(), handle.clone(), services.clone())
            .await
            .map_err(|err| anyhow!(err))?;
        Ok(())
    } else {
        info!("Single shot mode enabled triggering shutdown and waiting for queue to drain!");
        // TODO: something like this instead?
        /*
        let result = handle.wait_for_completion().await;
        match result {
            Ok(()) => {
                info!("Completion result reached, shutting down...");
            },
            Err(e) => error!("Execution error: {}", e.to_string()),
        }
        */
        handle.shutdown().await;
        Ok(())
    }
}
