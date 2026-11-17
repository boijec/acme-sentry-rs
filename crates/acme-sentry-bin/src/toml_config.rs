use std::sync::Arc;
use anyhow::anyhow;
use sentry_core::config::{ApplicationConfig, ConfigBuilder};
use sentry_core::initialization::CONFIG_FILE_NAME;
use sentry_io::fs::FileSystemOperations;
use serde::{Deserialize, Serialize};
use tracing::Level;
use sentry_common::uuid::InternalIdTooling;

#[derive(Serialize, Deserialize)]
pub struct ConfigFileFormat {
    #[serde(rename = "acme-ca-base-url")]
    pub acme_ca_base_url: String,
    #[serde(rename = "acme-default-plugin")]
    pub acme_plugin: Option<String>,
    #[serde(rename = "logging-level", with = "level_serde")]
    pub logging_level: Level,
    pub user: UserConf,
}

#[derive(Serialize, Deserialize)]
pub struct UserConf {
    pub id: String,
    pub email: String,
    #[serde(rename = "key-type")]
    pub key_type: String,
}

impl ConfigFileFormat {
    pub fn with_default_plugin(self, plugin_name: String) -> Self {
        ConfigFileFormat {
            acme_plugin: Some(plugin_name),
            user: self.user,
            acme_ca_base_url: self.acme_ca_base_url,
            logging_level: self.logging_level,
        }
    }
}

mod level_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;
    use tracing::Level;

    pub fn serialize<S: Serializer>(
        level: &Level,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&level.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Level, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Level::from_str(&s).unwrap_or(Level::INFO))
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


pub(crate) fn scan_and_build_application_config(fs: &impl FileSystemOperations) -> anyhow::Result<ApplicationConfig> {
    let content = fs.read_from_file_to_string("", CONFIG_FILE_NAME)
        .map_err(|e| anyhow!("Error while reading configuration :: {}", e))?;
    let base_dir = fs.get_base_dir().into_string()
        .map_err(|e| anyhow!("Error while parsing base-dir string :: {:?}", e))?;
    let toml: ConfigFileFormat = toml::from_str(&content)?;
    Ok(ApplicationConfig::builder()
        .base_dir(base_dir)
        .base_url(toml.acme_ca_base_url)
        .user_id(toml.user.id)
        .user_email(toml.user.email)
        .key_type(toml.user.key_type)
        .logging_level(toml.logging_level)
        .default_plugin_option(toml.acme_plugin)
        .build())
}

pub(crate) fn read_current_toml_config(fs: Arc<dyn FileSystemOperations>) -> anyhow::Result<ConfigFileFormat> {
    let content = fs.read_from_file_to_string("", CONFIG_FILE_NAME)
        .map_err(|e| anyhow!("Error while reading configuration :: {}", e))?;
    let toml: ConfigFileFormat = toml::from_str(&content)?;
    Ok(toml)
}

pub(crate) fn build_toml_config(base_url: String, key_type: String, email: String) -> anyhow::Result<String> {
    let conf = ConfigFileFormat {
        logging_level: Level::INFO,
        acme_ca_base_url: base_url,
        acme_plugin: None,
        user: UserConf {
            id: InternalIdTooling::new_compact_id(),
            key_type,
            email,
        }
    };
    let conf_content = toml::to_string(&conf)
        .map_err(|e| anyhow!("Failed to serialize `{}`: {}", CONFIG_FILE_NAME, e))?;
    Ok(conf_content)
}

pub(crate) fn save_serialized_config(config: ConfigFileFormat, fs: Arc<dyn FileSystemOperations>) -> anyhow::Result<()> {
    let content = toml::to_string(&config).map_err(|e| anyhow!("Failed to serialize `{}`: {}", CONFIG_FILE_NAME, e))?;
    fs.write_to_file("", CONFIG_FILE_NAME, content.as_bytes())
        .map_err(|e| anyhow!("Failed to save `{}`: {}", CONFIG_FILE_NAME, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML_CONFIG: &str = r#"
    acme-ca-base-url = 'https://acme.demorith.com'
    logging-level = 'info'

    [user]
    id = '1ca1ea78-2da8-496b-a406-fd6448c836f4'
    email = 'test@test.com'
    key-type = 'ec-p512'
    "#;

    #[test]
    fn test_toml_config() {
        let toml: ConfigFileFormat = toml::from_str(TOML_CONFIG).unwrap();
        assert_eq!(toml.acme_ca_base_url, "https://acme.demorith.com");
        assert_eq!(toml.logging_level, Level::INFO);
        assert_eq!(toml.user.id, "1ca1ea78-2da8-496b-a406-fd6448c836f4");
        assert_eq!(toml.user.email, "test@test.com");
        assert_eq!(toml.user.key_type, "ec-p512");
    }
}