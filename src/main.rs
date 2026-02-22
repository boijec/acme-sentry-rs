mod acme_jobs;
mod job_execution;
mod statics;

use crate::acme_jobs::db_initialization::DbInitializationJob;
use crate::acme_jobs::directory_query::DirectoryUpdateJob;
use crate::acme_jobs::initialize_keys_for_user::InitializeLocalUserJob;
use crate::job_execution::job_base::Scheduler;
use crate::statics::{Args, YamlConfig};
use clap::{Parser, crate_version};
use common_utils::{APPLICATION_CONFIG, ApplicationConfig, InternalIdTooling};
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs};
use tracing::{Instrument, Span, error, info, info_span};

async fn async_main() -> Result<(), Box<dyn Error>> {
    let (scheduler, handle) = Scheduler::new(32);
    let config = APPLICATION_CONFIG.get().unwrap();
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
            config.base_url.to_string(),
            config.user_id.clone(),
        )?)
        .await?;
    if config.application_mode {
        info!("Application mode has been enabled, monitoring input signals.");
        let mut h = handle.clone();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => { h.shutdown().await; }
            _ = h.wait_for_shutdown() => {}
        }
        Ok(())
    } else {
        info!("Single shot mode enabled - application will shut down right now!");
        handle.shutdown().await;
        Ok(())
    }
}

fn main() {
    let args = Args::parse();
    splash(args.clone().version);
    if args.version {
        return;
    }
    let _ = write_application_config(args.clone()).unwrap();
    let conf = APPLICATION_CONFIG.get().unwrap();
    tracing_subscriber::fmt()
        .with_max_level(conf.logging_level.unwrap())
        .init();
    let user_id = conf.user_id.clone();
    let span = info_span!("main", user_id = user_id);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main().instrument(span))
        .expect("Tokio runtime panicked with error:");
}

fn handle_mode(args: Args) -> Result<(bool, bool), Box<dyn Error>> {
    if args.application_mode {
        if args.yaml_config.is_none() {
            error!("YAML configuration option is required for this mode");
            return Err("Yaml config parameter, not present in input parameter".into());
        }
        return Ok((true, true));
    }
    if args.yaml {
        if args.yaml_config.is_none() {
            error!("YAML configuration option is required for this mode");
            return Err("Yaml config parameter, not present in input parameter".into());
        }
        return Ok((true, false));
    }
    Ok((false, false))
}

fn write_application_config(args: Args) -> Result<(), Box<dyn Error>> {
    let (yaml_mode, application_mode) = handle_mode(args.clone())?;
    if yaml_mode {
        let yaml_file = fs::read_to_string(PathBuf::from_str(args.yaml_config.unwrap().as_str())?)?;
        let yaml_config: YamlConfig = serde_yaml::from_str(&yaml_file)?;
        let config = ApplicationConfig {
            application_mode,
            user_id: yaml_config
                .acme_sentry_configuration
                .user
                .id
                .unwrap_or(InternalIdTooling::new_compact_id()),
            user_email: yaml_config.acme_sentry_configuration.user.email,
            key_type: yaml_config.acme_sentry_configuration.user.key_type,
            logging_level: yaml_config.acme_sentry_configuration.logging.logging_level,
            base_dir: yaml_config.acme_sentry_configuration.fs.base_dir.clone(),
            output_dir: PathBuf::from_str(
                yaml_config.acme_sentry_configuration.fs.base_dir.as_str(),
            )?
            .join(yaml_config.acme_sentry_configuration.fs.output_dir.as_str())
            .to_str()
            .unwrap()
            .to_string(),
            base_url: yaml_config.acme_sentry_configuration.base_url,
        };
        APPLICATION_CONFIG.set(config).unwrap();
    } else {
        let email = args.with_email;
        let config = ApplicationConfig {
            application_mode,
            user_id: args
                .with_user_id
                .unwrap_or(InternalIdTooling::new_compact_id()),
            user_email: email.unwrap(),
            key_type: args.requested_login_key_type,
            logging_level: Some(args.logging_level),
            base_dir: args.base_dir.clone(),
            output_dir: PathBuf::from_str(args.base_dir.as_str())?
                .join(args.output_dir.as_str())
                .to_str()
                .unwrap()
                .to_string(),
            base_url: args.acme_base_url.unwrap(),
        };
        APPLICATION_CONFIG.set(config).unwrap();
    }
    Ok(())
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
