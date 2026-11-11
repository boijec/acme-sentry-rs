use crate::about;
use crate::config_parsing::log_level_parse;
use clap::{Arg, ArgAction, ArgMatches, Command, crate_version};
use thiserror::Error;
use crate::clap_config::ConfigurationError::OperationError;

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
        .subcommand(init())
        .subcommand(watch())
        .subcommand(run())
        .get_matches();
    let operation: Operation = matches
        .subcommand_name()
        .ok_or_else(|| OperationError("operation does not exist".into()))?
        .into();
    Ok((matches, operation))
}

fn init() -> Command {
    Command::new("init")
        .about("Initialize all basic directories for acme-sentry in: '~/.acmes'")
        .arg(
            Arg::new("dry-run")
                .short('d')
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Run initialization sequence but don't create directories or files")
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
