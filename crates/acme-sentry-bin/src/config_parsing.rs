use std::fs;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::option::Option;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::anyhow;
use tracing::{error, Level};
use sentry_core::config::{ApplicationConfig, ConfigBuilder};
use sentry_plugin_engine::checkers::check_plugins_directory;
use sentry_tools::uuid::InternalIdTooling;

#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub version: bool,
    #[arg(short, long, default_value = "info", value_parser = log_level_parse, help = "Enable verbose logging")]
    pub logging_level: Level,
    #[arg(long, default_value_t = false, help = "Enable application-mode (input is required from user to terminate application)")]
    pub application_mode: bool,
    #[arg(long, default_value_t = false, help = "Enable yaml config (enabled by default if application mode is on)")]
    pub yaml: bool,
    #[arg(long, help = "Location of the acme-sentry config file")]
    pub yaml_config: Option<String>,
    #[arg(long, default_value = "ec-p256", help = "Specify what key type, that acme-sentry should use to log in to the CA with")]
    pub requested_login_key_type: String,
    #[arg(long, help = "ACME system base url")]
    pub acme_base_url: Option<String>,
    #[arg(long, default_value = "/opt/acme-sentry", help = "Application base directory")]
    pub base_dir: String,
    #[arg(long, default_value = "out", help = "Output directory of generated files")]
    pub output_dir: String,
    #[arg(long, help = "Absolute path to acme challenge plugin file in .so format")]
    pub with_challenge_plugins: Option<String>,
    #[arg(long, help = "User id that acme-sentry shall persist reference to if this is not present, acme-sentry will generate a new user")]
    pub with_user_id: Option<String>,
    #[arg(long, help = "Email that acme-sentry shall try to connect with the user")]
    pub with_email: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    #[serde(rename = "acme-sentry")]
    pub acme_sentry_configuration: AcmeSentryConfiguration
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcmeSentryConfiguration {
    #[serde(rename = "base-url")]
    pub base_url: String,
    pub fs: FsConfig,
    pub user: UserConfig,
    pub logging: LoggingConfig,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FsConfig {
    #[serde(rename = "base-dir")]
    pub base_dir: String,
    #[serde(rename = "acme-challenge-plugin")]
    pub acme_plugin: String,
    #[serde(rename = "output-dir")]
    pub output_dir: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default, rename = "id")]
    pub id: Option<String>,
    pub email: String,
    #[serde(rename = "login-key-type")]
    pub key_type: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(rename = "logging-level", with = "level_serde")]
    pub logging_level: Option<Level>,
}


mod level_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;
    use tracing::Level;

    pub fn serialize<S: Serializer>(level: &Option<Level>, serializer: S) -> Result<S::Ok, S::Error> {
        match level {
            Some(l) => serializer.serialize_some(&l.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Level>, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Some(Level::from_str(&s).unwrap_or(Level::INFO)))
    }
}

fn log_level_parse(s: &str) -> Result<Level, String> {
    match s.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Ok(Level::INFO),
    }
}

pub(crate) fn write_application_config(args: Args) -> anyhow::Result<ApplicationConfig> {
    let (yaml_mode, application_mode) = handle_mode(args.clone())?;
    if yaml_mode {
        let yaml_file = fs::read_to_string(PathBuf::from_str(args.yaml_config.unwrap().as_str())?)?;
        let yaml_config: YamlConfig = serde_yaml::from_str(&yaml_file)?;
        let plugin_dir = yaml_config.acme_sentry_configuration.fs.acme_plugin;
        let discovery_type =
            check_plugins_directory(plugin_dir.as_str()).map_err(|e| anyhow!("{:?}", e))?;
        let mut builder = ApplicationConfig::builder();
        builder
            .application_mode(application_mode)
            .user_id(yaml_config
                .acme_sentry_configuration
                .user
                .id
                .unwrap_or(InternalIdTooling::new_compact_id()))
            .user_email(yaml_config.acme_sentry_configuration.user.email)
            .key_type(yaml_config.acme_sentry_configuration.user.key_type)
            .logging_level(yaml_config.acme_sentry_configuration.logging.logging_level)
            .base_dir(yaml_config.acme_sentry_configuration.fs.base_dir.clone())
            .acme_challenge_plugin(plugin_dir)
            .discovery_type(discovery_type.into())
            .output_dir(
                PathBuf::from_str(
                    yaml_config.acme_sentry_configuration.fs.base_dir.as_str(),
                )?
                    .join(yaml_config.acme_sentry_configuration.fs.output_dir.as_str())
                    .to_str()
                    .unwrap()
                    .to_string()
            )
            .base_url(yaml_config.acme_sentry_configuration.base_url);
        Ok(builder.build())
    } else {
        let email = args.with_email;
        if args.with_challenge_plugins.is_none() {
            return Err(anyhow!("Challenge plugin configuration is required for this mode"));
        }
        let plugin_dir = args.with_challenge_plugins.unwrap();
        let discovery_type =
            check_plugins_directory(plugin_dir.as_str()).map_err(|e| anyhow!("{:?}", e))?;
        let mut builder = ApplicationConfig::builder();
        builder
            .application_mode(application_mode)
            .user_id(args
                .with_user_id
                .unwrap_or(InternalIdTooling::new_compact_id()))
            .user_email(email.unwrap())
            .key_type(args.requested_login_key_type)
            .logging_level(Some(args.logging_level))
            .base_dir(args.base_dir.clone())
            .acme_challenge_plugin(plugin_dir)
            .discovery_type(discovery_type.into())
            .output_dir(PathBuf::from_str(args.base_dir.as_str())?
                .join(args.output_dir.as_str())
                .to_str()
                .unwrap()
                .to_string())
            .base_url(args.acme_base_url.unwrap());
        Ok(builder.build())
    }
}

pub(crate) fn handle_mode(args: Args) -> anyhow::Result<(bool, bool), anyhow::Error> {
    if args.application_mode {
        if args.yaml_config.is_none() {
            error!("YAML configuration option is required for this mode");
            return Err(anyhow!(
                "Yaml config parameter, not present in input parameter"
            ));
        }
        return Ok((true, true));
    }
    if args.yaml {
        if args.yaml_config.is_none() {
            error!("YAML configuration option is required for this mode");
            return Err(anyhow!(
                "Yaml config parameter, not present in input parameter"
            ));
        }
        return Ok((true, false));
    }
    Ok((false, false))
}