use tracing::Level;

#[derive(Debug)]
pub struct ApplicationConfig {
    pub application_mode: bool,
    pub base_url: String,
    pub base_dir: String,
    pub output_dir: String,
    pub user_id: String,
    pub user_email: String,
    pub key_type: String,
    pub logging_level: Option<Level>,
    pub acme_challenge_plugin: String,
    pub discovery_type: String,
}

#[derive(Default)]
pub struct ApplicationConfigBuilder {
    pub application_mode: bool,
    pub base_url: String,
    pub base_dir: String,
    pub output_dir: String,
    pub user_id: String,
    pub user_email: String,
    pub key_type: String,
    pub logging_level: Option<Level>,
    pub acme_challenge_plugin: String,
    pub discovery_type: String,
}

pub trait ConfigBuilder {
    fn application_mode(self, value: bool) -> Self;
    fn base_url(self, value: String) -> Self;
    fn base_dir(self, value: String) -> Self;
    fn output_dir(self, value: String) -> Self;
    fn user_id(self, value: String) -> Self;
    fn user_email(self, value: String) -> Self;
    fn key_type(self, value: String) -> Self;
    fn logging_level(self, value: Option<Level>) -> Self;
    fn acme_challenge_plugin(self, value: String) -> Self;
    fn discovery_type(self, value: String) -> Self;
    fn build(self) -> ApplicationConfig;
}

pub trait ConfigurationMap {
    fn get_application_mode(&self) -> bool;
    fn get_base_url(&self) -> String;
    fn get_base_dir(&self) -> String;
    fn get_output_dir(&self) -> String;
    fn get_user_id(&self) -> String;
    fn get_user_email(&self) -> String;
    fn get_key_type(&self) -> String;
    fn get_logging_level(&self) -> Option<Level>;
    fn get_acme_challenge_plugin(&self) -> String;
    fn get_discovery_type(&self) -> String;
}

impl ApplicationConfig {
    pub fn builder() -> ApplicationConfigBuilder {
        ApplicationConfigBuilder::default()
    }
}

impl ConfigurationMap for ApplicationConfig {
    fn get_application_mode(&self) -> bool {
        self.application_mode
    }
    fn get_base_url(&self) -> String {
        self.base_url.clone()
    }
    fn get_base_dir(&self) -> String {
        self.base_dir.clone()
    }
    fn get_output_dir(&self) -> String {
        self.output_dir.clone()
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
    fn get_logging_level(&self) -> Option<Level> {
        self.logging_level
    }
    fn get_acme_challenge_plugin(&self) -> String {
        self.acme_challenge_plugin.clone()
    }
    fn get_discovery_type(&self) -> String {
        self.discovery_type.clone()
    }
}

impl ConfigBuilder for ApplicationConfigBuilder {
    fn application_mode(mut self, value: bool) -> Self {
        self.application_mode = value;
        self
    }

    fn base_url(mut self, value: String) -> Self {
        self.base_url = value;
        self
    }

    fn base_dir(mut self, value: String) -> Self {
        self.base_dir = value;
        self
    }

    fn output_dir(mut self, value: String) -> Self {
        self.output_dir = value;
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

    fn logging_level(mut self, value: Option<Level>) -> Self {
        self.logging_level = value;
        self
    }

    fn acme_challenge_plugin(mut self, value: String) -> Self {
        self.acme_challenge_plugin = value;
        self
    }

    fn discovery_type(mut self, value: String) -> Self {
        self.discovery_type = value;
        self
    }

    fn build(self) -> ApplicationConfig {
        ApplicationConfig {
            application_mode: self.application_mode,
            base_url: self.base_url,
            base_dir: self.base_dir,
            output_dir: self.output_dir,
            key_type: self.key_type,
            user_email: self.user_email,
            user_id: self.user_id,
            logging_level: self.logging_level,
            discovery_type: self.discovery_type,
            acme_challenge_plugin: self.acme_challenge_plugin,
        }
    }
}