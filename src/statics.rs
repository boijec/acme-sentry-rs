use clap::Parser;
use serde::{Deserialize, Serialize};
use std::option::Option;
use tracing::Level;

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