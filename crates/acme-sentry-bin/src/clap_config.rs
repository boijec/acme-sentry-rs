use crate::about;
use crate::clap_config::ConfigurationError::OperationError;
use crate::toml_config::log_level_parse;
use clap::{Arg, ArgMatches, Command, ValueHint, crate_version};
use thiserror::Error;

#[non_exhaustive]
#[derive(PartialEq)]
pub enum Operation {
    OneShot,
    Watch,
    Init,
    Plugin,
    Invalid,
}
impl Into<Operation> for &str {
    fn into(self) -> Operation {
        match self {
            "watch" => Operation::Watch,
            "init" => Operation::Init,
            "oneshot" => Operation::OneShot,
            "plugin" => Operation::Plugin,
            _ => Operation::Invalid,
        }
    }
}
impl Into<String> for Operation {
    fn into(self) -> String {
        match self {
            Operation::Watch => "watch".into(),
            Operation::Init => "init".into(),
            Operation::OneShot => "run".into(),
            Operation::Plugin => "plugin".into(),
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
        .arg_required_else_help(true)
        .arg(
            Arg::new("log-level")
                .num_args(1)
                .short('l')
                .value_parser(log_level_parse)
                .long("level")
                .help(
                    "Optionally set log level at runtime - overrides the log level in base config",
                ),
        )
        .subcommand(init().args_conflicts_with_subcommands(true))
        .subcommand(watch().args_conflicts_with_subcommands(true))
        .subcommand(run().args_conflicts_with_subcommands(true))
        .subcommand(plugin().args_conflicts_with_subcommands(true))
        .get_matches();
    let operation: Operation = matches
        .subcommand_name()
        .ok_or_else(|| OperationError("operation does not exist".into()))?
        .into();
    Ok((matches, operation))
}

fn init() -> Command {
    let init_value_hints = ["headless", "server", "user"];

    Command::new("init")
        .about("Initialize all basic directories for acme-sentry")
        .arg_required_else_help(true)
        .arg(
            Arg::new("init-mode")
                .long("mode")
                .num_args(1)
                .value_parser(init_value_hints)
                .help("Set the initialization mode")
        )
        .arg(
            Arg::new("base-url")
                .long("acme-ca-dir-url")
                .short('B')
                .num_args(1)
                .help("Specify the directory url to the ACME Certificate Authority eg. (https://acme-staging-v02.api.letsencrypt.org/directory)")
        )
        .arg(
            Arg::new("email")
                .long("email")
                .short('e')
                .num_args(1)
                .help("Specify the email of the acme user account to use / register")
        )
        .arg(
            Arg::new("key-type")
                .long("key-type")
                .short('k')
                .num_args(1)
                .help("Specify the key type that will be used for your account")
        )
}
fn watch() -> Command {
    Command::new("watch").about("Start acme-sentry in watch mode (args not required)")
}
fn run() -> Command {
    Command::new("oneshot")
        .about("Start acme-sentry in 'one-shot' mode (args not required)")
        .arg(
            Arg::new("domain")
                .short('d')
                .long("domain")
                .value_name("DOMAIN")
                .help("Name of the domain that should be requested"),
        )
}

fn plugin() -> Command {
    Command::new("plugin")
        .about("Check / Validate / Install Acme Sentry plugins")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("install")
                .arg_required_else_help(true)
                .about("Install a plugin in Acme Sentry")
                .arg(
                    Arg::new("path")
                        .required(true)
                        .long("path")
                        .num_args(1)
                        .value_name("SO_PLUGIN_PATH")
                        .value_hint(ValueHint::FilePath)
                        .help("Install plugin in Acme Sentry"),
                )
                .args_conflicts_with_subcommands(true),
        )
        .subcommand(
            Command::new("ls")
                .about("List installed plugins")
                .args_conflicts_with_subcommands(true),
        )
}
