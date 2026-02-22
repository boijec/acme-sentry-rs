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
use tracing::{info, info_span, Instrument, Level, Span};
use common_utils::InternalIdTooling;

async fn async_main(args: Args, user_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let (scheduler, handle) = Scheduler::new(32);
    let directory_job = DirectoryQueryJob::new(
        args.acme_base_url,
        user_id.clone(),
    )?;
    let scheduler_span = info_span!("scheduler", user_id = user_id);
    scheduler_span.follows_from(Span::current());
    let _ = tokio::spawn(scheduler.run(handle.clone()).instrument(scheduler_span));
    handle.submit(DbInitializationJob::new()).await?;
    handle
        .submit(InitializeLocalUserJob::new(
            args.dir_out.to_string(),
            args.requested_login_key_type.to_string(),
            user_id.clone(),
        ))
        .await?;
    handle
        .submit(directory_job)
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
    let level: Level = args.logging_level;
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    splash(args.version);
    if args.version {
        return;
    }
    let arg_clone = args.clone();
    let user_id = arg_clone.with_user_id.unwrap_or(InternalIdTooling::new_compact_id());
    let span = info_span!("main", user_id = user_id);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main(args, user_id).instrument(span))
        .expect("Tokio runtime panicked with error:");
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
