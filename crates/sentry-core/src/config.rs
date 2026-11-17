use tracing::Level;

#[derive(Debug)]
pub struct ApplicationConfig {
    pub base_url: String,
    pub base_dir: String,
    pub user_id: String,
    pub user_email: String,
    pub key_type: String,
    pub logging_level: Level,
    pub default_plugin: Option<String>,
}

pub struct ApplicationConfigBuilder {
    pub base_url: String,
    pub base_dir: String,
    pub user_id: String,
    pub user_email: String,
    pub key_type: String,
    pub logging_level: Level,
    pub default_plugin: Option<String>,
}

pub trait ConfigBuilder {
    fn base_url(self, value: String) -> Self;
    fn base_dir(self, value: String) -> Self;
    fn user_id(self, value: String) -> Self;
    fn user_email(self, value: String) -> Self;
    fn key_type(self, value: String) -> Self;
    fn logging_level(self, value: Level) -> Self;
    fn default_plugin(self, value: String) -> Self;
    fn default_plugin_option(self, value: Option<String>) -> Self;
    fn build(self) -> ApplicationConfig;
}

pub trait ConfigurationMap {
    fn get_base_url(&self) -> String;
    fn get_base_dir(&self) -> String;
    fn get_user_id(&self) -> String;
    fn get_user_email(&self) -> String;
    fn get_key_type(&self) -> String;
    fn get_logging_level(&self) -> Level;
    fn get_default_plugin(&self) -> Option<String>;
}

impl ApplicationConfig {
    pub fn builder() -> ApplicationConfigBuilder {
        ApplicationConfigBuilder::default()
    }
    pub fn with_logging_level(self, level: Level) -> Self {
        ApplicationConfig {
            base_url: self.base_url,
            base_dir: self.base_dir,
            key_type: self.key_type,
            user_email: self.user_email,
            user_id: self.user_id,
            logging_level: level,
            default_plugin: self.default_plugin,
        }
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            logging_level: Level::INFO,
            base_url: String::default(),
            base_dir: String::default(),
            key_type: String::default(),
            user_email: String::default(),
            user_id: String::default(),
            default_plugin: Option::default(),
        }
    }
}

impl Default for ApplicationConfigBuilder {
    fn default() -> Self {
        ApplicationConfigBuilder {
            logging_level: Level::INFO,
            base_url: String::default(),
            base_dir: String::default(),
            key_type: String::default(),
            user_email: String::default(),
            user_id: String::default(),
            default_plugin: Option::default(),
        }
    }
}

impl ConfigurationMap for ApplicationConfig {
    fn get_base_url(&self) -> String {
        self.base_url.clone()
    }
    fn get_base_dir(&self) -> String {
        self.base_dir.clone()
    }
    fn get_user_id(&self) -> String {
        self.user_id.clone()
    }
    fn get_user_email(&self) -> String {
        self.user_email.clone()
    }
    fn get_key_type(&self) -> String {
        self.key_type.clone()
    }
    fn get_logging_level(&self) -> Level {
        self.logging_level
    }
    fn get_default_plugin(&self) -> Option<String> {
        self.default_plugin.clone()
    }
}

impl ConfigBuilder for ApplicationConfigBuilder {

    fn base_url(mut self, value: String) -> Self {
        self.base_url = value;
        self
    }

    fn base_dir(mut self, value: String) -> Self {
        self.base_dir = value;
        self
    }

    fn user_id(mut self, value: String) -> Self {
        self.user_id = value;
        self
    }

    fn user_email(mut self, value: String) -> Self {
        self.user_email = value;
        self
    }

    fn key_type(mut self, value: String) -> Self {
        self.key_type = value;
        self
    }

    fn logging_level(mut self, value: Level) -> Self {
        self.logging_level = value;
        self
    }
    fn default_plugin(mut self, value: String) -> Self {
        self.default_plugin = Some(value);
        self
    }
    fn default_plugin_option(mut self, value: Option<String>) -> Self {
        self.default_plugin = value;
        self
    }

    fn build(self) -> ApplicationConfig {
        ApplicationConfig {
            base_url: self.base_url,
            base_dir: self.base_dir,
            key_type: self.key_type,
            user_email: self.user_email,
            user_id: self.user_id,
            logging_level: self.logging_level,
            default_plugin: self.default_plugin,
        }
    }
}
