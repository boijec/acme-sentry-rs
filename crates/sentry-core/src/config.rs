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
    fn application_mode(&mut self, value: bool) -> &mut Self;
    fn base_url(&mut self, value: String) -> &mut Self;
    fn base_dir(&mut self, value: String) -> &mut Self;
    fn output_dir(&mut self, value: String) -> &mut Self;
    fn user_id(&mut self, value: String) -> &mut Self;
    fn user_email(&mut self, value: String) -> &mut Self;
    fn key_type(&mut self, value: String) -> &mut Self;
    fn logging_level(&mut self, value: Option<Level>) -> &mut Self;
    fn acme_challenge_plugin(&mut self, value: String) -> &mut Self;
    fn discovery_type(&mut self, value: String) -> &mut Self;
    fn build(self) -> ApplicationConfig;
}

impl ApplicationConfig {
    pub fn builder() -> ApplicationConfigBuilder {
        ApplicationConfigBuilder::default()
    }
}

impl ConfigBuilder for ApplicationConfigBuilder {
    fn application_mode(&mut self, value: bool) -> &mut Self {
        self.application_mode = value;
        self
    }

    fn base_url(&mut self, value: String) -> &mut Self {
        self.base_url = value;
        self
    }

    fn base_dir(&mut self, value: String) -> &mut Self {
        self.base_dir = value;
        self
    }

    fn output_dir(&mut self, value: String) -> &mut Self {
        self.output_dir = value;
        self
    }

    fn user_id(&mut self, value: String) -> &mut Self {
        self.user_id = value;
        self
    }

    fn user_email(&mut self, value: String) -> &mut Self {
        self.user_email = value;
        self
    }

    fn key_type(&mut self, value: String) -> &mut Self {
        self.key_type = value;
        self
    }

    fn logging_level(&mut self, value: Option<Level>) -> &mut Self {
        self.logging_level = value;
        self
    }

    fn acme_challenge_plugin(&mut self, value: String) -> &mut Self {
        self.acme_challenge_plugin = value;
        self
    }

    fn discovery_type(&mut self, value: String) -> &mut Self {
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