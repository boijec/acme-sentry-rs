use crate::config_parsing::{Args, build_application_config};
use clap::{Parser, crate_version};
use sentry_api::server::start_http_server;
use sentry_core::config::ApplicationConfig;
use sentry_core::services::start_services;
use sentry_db::pool::connect;
use sentry_db::repo::setup_repos;
use sentry_io::fs::BasicUnixFileSystem;
use sentry_job_engine::dependencies::JobDependencies;
use sentry_job_engine::jobs::directory_query::DirectoryUpdateJob;
use sentry_job_engine::jobs::initialize_keys_for_user::InitializeLocalUserJob;
use sentry_job_engine::jobs::login_user::LoginUserJob;
use sentry_job_engine::jobs::preflight_start_sequence;
use sentry_job_engine::scheduling::{Scheduler, SchedulerHandle};
use std::sync::Arc;
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
    let user_id = conf.user_id.clone();
    let span = info_span!("main", user_id = user_id);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main(conf).instrument(span))
        .expect("Tokio runtime panicked with error:");
}

async fn async_main(application_config: ApplicationConfig) -> anyhow::Result<()> {
    let fs = BasicUnixFileSystem::new(application_config.output_dir.as_str())?;
    let application_config = Arc::new(application_config);
    let db_location = format!("{}/acme-sentry.db", application_config.base_dir.clone());
    let pool = connect(&db_location).await?;
    let repos = setup_repos(&pool);
    let services = Arc::new(start_services(repos));

    let (mut scheduler, handle) = Scheduler::new(32);
    scheduler.with_dependencies(
        JobDependencies::new(
            Arc::new(fs),
            application_config.clone(),
            services.clone(),
        )
    );
    let scheduler_span = info_span!("scheduler", user_id = application_config.user_id);
    scheduler_span.follows_from(Span::current());
    let _ = tokio::spawn(scheduler.run(handle.clone()).instrument(scheduler_span));

    preflight_start_sequence(&handle).await?;

    if application_config.application_mode {
        info!("Application mode has been enabled, monitoring input signals.");
        start_http_server(application_config, handle.clone(), services.clone())
    } else {
        info!("Single shot mode enabled - application will shut down right now!");
        handle.shutdown().await;
        Ok(())
    }
}