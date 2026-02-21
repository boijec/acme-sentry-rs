mod acme_jobs;
mod job_execution;
mod statics;

use crate::acme_jobs::db_initialization::DbInitializationJob;
use crate::acme_jobs::directory_query::DirectoryQueryJob;
use crate::acme_jobs::initialize_keys_for_user::InitializeLocalUserJob;
use crate::job_execution::job_base::Scheduler;
use crate::statics::Args;
use clap::{crate_version, Parser};
use std::env;
use tracing::{info, info_span, Instrument, Level};

async fn async_main(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let (scheduler, handle) = Scheduler::new(32);
    let scheduler_span = info_span!("scheduler");
    let _ = tokio::spawn(scheduler.run(handle.clone()).instrument(scheduler_span));
    let span = info_span!("main");
    let _ = span.enter();
    handle.submit(DbInitializationJob::new()).await?;
    handle
        .submit(InitializeLocalUserJob::new(
            args.dir_out.to_string(),
            args.requested_login_key_type.to_string(),
        ))
        .await?;
    handle
        .submit(DirectoryQueryJob::new(
            args.acme_base_url.unwrap().to_string(),
        )?)
        .await?;
    if args.application_mode {
        info!("Application mode has been enabled, monitoring input signals.");
        let mut h = handle.clone();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => { h.shutdown().await; }
            _ = h.wait_for_shutdown() => {}
        }
    } else {
        info!("Single shot mode enabled - application will shut down right now!");
        handle.shutdown().await;
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    let level: Level = args.logging_level.unwrap_or(Level::INFO);
    tracing_subscriber::fmt().with_max_level(level).init();
    splash(args.version);
    if args.version {
        return;
    }
    let span = info_span!("main");
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main(args).instrument(span))
        .expect("TODO: panic message");
}

fn splash(print_version: bool) {
    print!(
        "{}\n",
        String::from_utf8_lossy(include_bytes!("assets/ico.bin"))
    );
    if print_version {
        println!("Version: {}", crate_version!());
        print!(
            "{}\n",
            String::from_utf8_lossy(include_bytes!("assets/creators.bin"))
        );
    }
}
