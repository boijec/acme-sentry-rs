use crate::clap_config::{Operation, argument_man};
use crate::initialization::init_application;
use crate::toml_config::scan_and_build_application_config;
use anyhow::anyhow;
use clap::{crate_authors, crate_version};
use sentry_core::config::ConfigurationMap;
use sentry_io::fs::BasicUnixFileSystem;
use std::process::ExitCode;
use std::sync::Arc;
use tracing::{Level, warn};

mod application;
mod clap_config;
mod initialization;
mod plugins;
mod toml_config;

const UNIVERSAL_BASE_DIR: &str = ".acmes";

fn splash() -> String {
    format!(
        "{}\n",
        String::from_utf8_lossy(include_bytes!("../../../assets/ico.bin"))
    )
}

fn version() -> String {
    format!("Version: {}\n", crate_version!())
}

fn about() -> String {
    let authors = format!("Created by:\n\t{}\n", crate_authors!());
    format!("{}{}{}", splash(), version(), authors)
}

fn main() -> anyhow::Result<ExitCode> {
    let (mut matches, operation) =
        argument_man().map_err(|e| anyhow!("Argument error :: {}", e))?;

    print!("{}\n", splash());

    let temp: Option<Level> = matches.remove_one("log-level");
    let level: Level = if let Some(l) = temp { l } else { Level::INFO };

    if operation == Operation::Init {
        tracing_subscriber::fmt().with_max_level(level).init();
        init_application(&mut matches).map_err(|e| anyhow!("Application init failed :: {}", e))?;
        return Ok(ExitCode::SUCCESS);
    }

    let base_dir_path =
        BasicUnixFileSystem::scan_for_home_folder(UNIVERSAL_BASE_DIR).map_err(|e| {
            anyhow!(
                "Acme Sentry could not start :: error during home directory scan :: {}",
                e
            )
        })?;
    let fs = BasicUnixFileSystem::start(base_dir_path)?;
    let mut conf = scan_and_build_application_config(&fs)
        .map_err(|e| anyhow!("Error while reading configuration :: {}", e))?;
    if !level.eq(&conf.get_logging_level()) {
        conf = conf.with_logging_level(level);
    }
    let file_system = Arc::new(fs);
    tracing_subscriber::fmt()
        .with_max_level(conf.logging_level)
        .init();

    let application_configuration = Arc::new(conf);
    match operation {
        Operation::OneShot | Operation::Watch => {
            let ready = plugins::check_plugin_ready(file_system.clone())
                .map_err(|e| anyhow!("Error while checking plugin ready: {}", e))?;
            if ready {
                let state = application::run(application_configuration, file_system, operation)
                    .map_err(|e| anyhow!("Critical Failure :: {}", e))?;
                return Ok(state);
            } else {
                warn!("No acme plugins installed - quitting..");
            }
        }
        Operation::Plugin => {
            plugins::list_or_install_plugin(&mut matches, file_system, application_configuration)?;
        }
        Operation::Invalid => {
            eprintln!("Invalid mode");
            return Ok(ExitCode::FAILURE);
        }
        _ => {
            eprintln!("Operation not allowed here!");
        }
    }
    Ok(ExitCode::SUCCESS)
}
