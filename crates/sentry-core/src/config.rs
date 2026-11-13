use tracing::Level;

#[derive(Debug)]
pub struct ApplicationConfig {
    pub base_url: String,
    pub base_dir: String,
    pub user_id: String,
    pub user_email: String,
    pub key_type: String,
    pub logging_level: Option<Level>,
}

#[derive(Default)]
pub struct ApplicationConfigBuilder {
    pub base_url: String,
    pub base_dir: String,
    pub user_id: String,
    pub user_email: String,
    pub key_type: String,
    pub logging_level: Option<Level>,
}

pub trait ConfigBuilder {
    fn base_url(self, value: String) -> Self;
    fn base_dir(self, value: String) -> Self;
    fn user_id(self, value: String) -> Self;
    fn user_email(self, value: String) -> Self;
    fn key_type(self, value: String) -> Self;
    fn logging_level(self, value: Option<Level>) -> Self;
    fn build(self) -> ApplicationConfig;
}

pub trait ConfigurationMap {
    fn get_base_url(&self) -> String;
    fn get_base_dir(&self) -> String;
    fn get_user_id(&self) -> String;
    fn get_user_email(&self) -> String;
    fn get_key_type(&self) -> String;
    fn get_logging_level(&self) -> Option<Level>;
}

impl ApplicationConfig {
    pub fn builder() -> ApplicationConfigBuilder {
        ApplicationConfigBuilder::default()
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
    fn get_logging_level(&self) -> Option<Level> {
        self.logging_level
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

    fn logging_level(mut self, value: Option<Level>) -> Self {
        self.logging_level = value;
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
        }
    }
}
