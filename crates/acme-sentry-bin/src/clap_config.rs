use crate::about;
use crate::clap_config::ConfigurationError::OperationError;
use crate::config_parsing::log_level_parse;
use clap::{Arg, ArgMatches, Command, crate_version, value_parser};
use serde::Serialize;
use thiserror::Error;

#[non_exhaustive]
pub enum Operation {
    Run,
    Watch,
    Init,
    Invalid,
}
impl Into<Operation> for &str {
    fn into(self) -> Operation {
        match self {
            "watch" => Operation::Watch,
            "init" => Operation::Init,
            "run" => Operation::Run,
            _ => Operation::Invalid,
        }
    }
}
impl Into<String> for Operation {
    fn into(self) -> String {
        match self {
            Operation::Watch => "watch".into(),
            Operation::Init => "init".into(),
            Operation::Run => "run".into(),
            Operation::Invalid => "invalid".into(),
        }
    }
}

#[derive(
    clap::ValueEnum, Clone, Default, Debug, Serialize
)]
#[serde(rename_all = "kebab-case")]
pub enum InitializationMode {
    #[default]
    Headless,
    Server,
    User
}

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Clap error :: {0}")]
    ClapError(#[from] clap::Error),
    #[error("Operation error :: {0}")]
    OperationError(String),
}

pub(crate) fn argument_man() -> Result<(ArgMatches, Operation), ConfigurationError> {
    let matches = Command::new("acme-sentry")
        .version(crate_version!())
        .long_version(crate_version!())
        .about(about())
        .long_about(about())
        .arg(
            Arg::new("log-level")
                .num_args(1)
                .short('l')
                .value_parser(log_level_parse)
                .long("level")
                .help("Set log level"),
        )
        .subcommand(init().args_conflicts_with_subcommands(true))
        .subcommand(watch().args_conflicts_with_subcommands(true))
        .subcommand(run().args_conflicts_with_subcommands(true))
        .get_matches();
    let operation: Operation = matches
        .subcommand_name()
        .ok_or_else(|| OperationError("operation does not exist".into()))?
        .into();
    Ok((matches, operation))
}

fn init() -> Command {
    Command::new("init")
        .about("Initialize all basic directories for acme-sentry")
        .arg(
            Arg::new("init-mode")
                .long("mode")
                .num_args(1)
                .value_parser(value_parser!(InitializationMode))
                .help("Set the initialization mode")
        )
}
fn watch() -> Command {
    Command::new("watch")
        .about("Start acme-sentry in watch mode")
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("CONFIGURATION_FILE")
                .help("Specify path to the yaml configuration file")
        )
}
fn run() -> Command {
    Command::new("run")
        .about("Start acme-sentry in 'one-shot' mode")
        .arg(
            Arg::new("domain")
                .short('d')
                .long("domain")
                .value_name("DOMAIN")
                .help("Name of the domain that should be requested")
        )
}
