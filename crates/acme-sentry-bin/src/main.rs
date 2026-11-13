use crate::application::run_application;
use crate::clap_config::{Operation, argument_man};
use crate::config_parsing::build_application_config;
use crate::initialization::init_application;
use clap::{crate_authors, crate_version};
use sentry_io::fs::BasicUnixFileSystem;
use std::process::exit;
use tracing::{Level, warn};

mod config_parsing;
mod clap_config;
mod initialization;
mod application;

enum EndState {
    Success,
    Failure,
}

const UNIVERSAL_BASE_DIR: &str = ".acmes";

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
    let (matches, operation) = argument_man()
        .map_err(|e| format!("Argument error: {}", e))?;

    print!("{}", splash());

    let level: &Level = matches.get_one("log-level")
        .unwrap_or(&Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(*level)
        .init();

    match operation {
        Operation::Run | Operation::Watch => {
            let dest = BasicUnixFileSystem::scan_for_home_folder(UNIVERSAL_BASE_DIR)
                .map_err(|e| format!("Sanity check failed: {}", e))?;
            let conf = build_application_config(dest)
                .map_err(|e| format!("Application configuration failed: {}", e))?;
            let _ = run_application(conf, operation)
                .map_err(|e| format!("Application failed: {}", e))?;
        },
        Operation::Init => {
            let dest = BasicUnixFileSystem::scan_for_home_folder(UNIVERSAL_BASE_DIR);
            if let Ok(dir) = dest {
                warn!("Acme Sentry might already be initialized, check directory: {}", dir);
                return Ok(());
            }
            init_application(matches)
                .map_err(|e| format!("Application init failed: {}", e))?
        },
        Operation::Invalid => {
            eprintln!("Invalid mode");
            exit(1);
        },
    }
    Ok(())
}
