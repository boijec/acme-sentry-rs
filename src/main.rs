mod statics;
mod acme_jobs;
mod job_execution;

use crate::acme_jobs::db_initialization::DbInitializationJob;
use crate::acme_jobs::directory_query::DirectoryQueryJob;
use crate::job_execution::job_base::Scheduler;
use crate::statics::Args;
use clap::{crate_version, Parser};
use common_utils::logging::{Logger, LoggingLevel};
use std::env;
use crate::acme_jobs::initialize_keys_for_user::InitializeLocalUserJob;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    splash(args.version());
    let verbose = args.verbose();
    let _logger_instance_handler = Logger::new();
    let level;
    if verbose {
        level = LoggingLevel::TRACE;
    } else {
        level = LoggingLevel::INFO;
    }
    Logger::initialize_with(level);
    let (scheduler, handle) = Scheduler::new(32);
    tokio::spawn(scheduler.run(handle.clone(), level));
    handle.submit(DbInitializationJob::new()).await?;
    handle.submit(InitializeLocalUserJob::new(args.dir_out().to_string(), args.requested_login_key_type().to_string())).await?;
    handle.submit(DirectoryQueryJob::new(args.acme_base_url().to_string())?).await?;
    if args.wait_for_shutdown() {
        handle.clone().wait_for_shutdown().await;
    } else {
        handle.shutdown().await;
    }
    Ok(())
}

fn splash(print_version: bool) {
    print!("{}\n", String::from_utf8_lossy(include_bytes!("assets/ico.bin")));
    if(print_version) {
        println!(crate_version!());
        print!("{}\n", String::from_utf8_lossy(include_bytes!("assets/creators.bin")));
    }
}