use sentry_core::config::ApplicationConfig;
use sentry_core::services::Services;
use std::sync::Arc;

pub struct JobDependencies {
    configuration: Arc<ApplicationConfig>,
    services: Arc<Services>,
}

impl JobDependencies {
    pub fn new(configuration: Arc<ApplicationConfig>, services: Arc<Services>) -> Self {
        Self {
            configuration,
            services,
        }
    }
    pub fn configuration(&self) -> Arc<ApplicationConfig> {
        Arc::clone(&self.configuration)
    }
    pub fn services(&self) -> Arc<Services> {
        Arc::clone(&self.services)
    }
}
