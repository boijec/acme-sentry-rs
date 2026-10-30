use crate::config_parsing::{Args, write_application_config};
use clap::{Parser, crate_version};
use sentry_core::config::ApplicationConfig;
use sentry_io::fs::BasicUnixFileSystem;
use sentry_job_engine::dependencies::JobDependencies;
use std::sync::Arc;
use tracing::{info, info_span, Instrument, Span};
use sentry_api::server::start_http_server;
use sentry_db::pool::connect;
use sentry_job_engine::scheduling::Scheduler;

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
    let conf = write_application_config(args.clone()).unwrap();
    tracing_subscriber::fmt()
        .with_max_level(conf.logging_level.unwrap())
        .init();
    let user_id = conf.user_id.clone();
    let span = info_span!("main", user_id = user_id);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main(conf).instrument(span))
        .expect("Tokio runtime panicked with error:");
}

async fn async_main(application_config: ApplicationConfig) -> anyhow::Result<()> {
    let fs = BasicUnixFileSystem::new(application_config.output_dir.as_str())?;
    let db_location = format!("{}/acme-sentry.db", application_config.base_dir.clone());
    let _pool = connect(&db_location).await?;
    
    let (mut scheduler, handle) = Scheduler::new(32);
    scheduler.with_dependencies(
        JobDependencies::new(
            Arc::new(fs),
        )
    );
    DirectoryUpdateJob::validate_url(Some(config.base_url.clone()))?;
    let scheduler_span = info_span!("scheduler", user_id = config.user_id);
    scheduler_span.follows_from(Span::current());
    let _ = tokio::spawn(scheduler.run(handle.clone()).instrument(scheduler_span));
    handle.submit(DbInitializationJob::new()).await?;
    handle
        .submit(InitializeLocalUserJob::new(
            config.output_dir.to_string(),
            config.key_type.to_string(),
            config.user_id.clone(),
        ))
        .await?;
    handle
        .submit(DirectoryUpdateJob::new(
            Some(config.base_url.to_string()),
            config.user_id.clone(),
        )?)
        .await?;
    handle
        .submit(LoginUserJob::new(config.user_id.clone()))
        .await?;
    handle
        .submit(DirectoryUpdateJob::new(None, config.user_id.clone())?)
        .await?;
    if application_config.application_mode {
        info!("Application mode has been enabled, monitoring input signals.");
        start_http_server(application_config, handle.clone())
    } else {
        info!("Single shot mode enabled - application will shut down right now!");
        handle.shutdown().await;
        Ok(())
    }
}