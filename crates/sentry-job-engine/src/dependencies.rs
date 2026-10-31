use sentry_core::config::ConfigurationMap;
use sentry_core::services::Services;
use sentry_io::fs::FileSystemOperations;
use std::sync::Arc;

pub struct JobDependencies {
    file_system: Arc<dyn FileSystemOperations + Send + Sync>,
    configuration: Arc<dyn ConfigurationMap + Send + Sync>,
    services: Arc<Services>
}

impl JobDependencies {
    pub fn new(
        file_system: Arc<dyn FileSystemOperations + Send + Sync>,
        configuration: Arc<dyn ConfigurationMap + Send + Sync>,
        services: Arc<Services>
    ) -> Self {
        Self {
            file_system,
            configuration,
            services,
        }
    }
    pub fn file_system(&self) -> Arc<dyn FileSystemOperations + Send + Sync> {
        Arc::clone(&self.file_system)
    }
    pub fn configuration(&self) -> Arc<dyn ConfigurationMap + Send + Sync> {
        Arc::clone(&self.configuration)
    }
    pub fn services(&self) -> Arc<Services> {
        Arc::clone(&self.services)
    }
}
