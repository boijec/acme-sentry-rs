use sentry_core::config::{ApplicationConfig, ConfigBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::option::Option;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::Level;

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    #[serde(rename = "acme-sentry")]
    pub acme_sentry_configuration: AcmeSentryConfiguration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcmeSentryConfiguration {
    #[serde(rename = "base-url")]
    pub base_url: String,
    pub user: UserConfig,
    pub logging: LoggingConfig,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default, rename = "id")]
    pub id: String,
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

    pub fn serialize<S: Serializer>(
        level: &Option<Level>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match level {
            Some(l) => serializer.serialize_some(&l.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Level>, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Some(Level::from_str(&s).unwrap_or(Level::INFO)))
    }
}

pub(crate) fn log_level_parse(s: &str) -> Result<Level, String> {
    match s.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Ok(Level::INFO),
    }
}

#[allow(unused)]
const CONFIG_TEMPLATE: &str = r#""
acme-sentry:
    base-url: "{}"
    user:
        id: "{}"
        email: "{}"
        login-key-type: "{}"
    logging:
        logging-level: "{}"
"#;

pub(crate) fn build_application_config(base_dir: String) -> anyhow::Result<ApplicationConfig> {
    let yaml_file = fs::read_to_string(PathBuf::from_str(
        format!("{}/config.yaml", base_dir).as_str(),
    )?)?;
    let yaml_config: YamlConfig = serde_yaml::from_str(&yaml_file)?;
    Ok(ApplicationConfig::builder()
        .base_dir(base_dir)
        .user_id(yaml_config.acme_sentry_configuration.user.id)
        .user_email(yaml_config.acme_sentry_configuration.user.email)
        .key_type(yaml_config.acme_sentry_configuration.user.key_type)
        .logging_level(yaml_config.acme_sentry_configuration.logging.logging_level)
        .base_url(yaml_config.acme_sentry_configuration.base_url)
        .build())
}
