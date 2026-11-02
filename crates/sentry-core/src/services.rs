use std::sync::Arc;
use sentry_db::repo::DatabaseRepositories;
use sentry_io::fs::FileSystemOperations;
use crate::config::ApplicationConfig;
use crate::user::service::UserService;

pub struct Services {
    user_service: Arc<UserService>,
}

impl Services {
    pub fn user_service(&self) -> Arc<UserService> {
        self.user_service.clone()
    }
}

pub fn initialize_services(
    configuration: Arc<ApplicationConfig>,
    database_repositories: DatabaseRepositories,
    file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,
) -> Services {
    Services {
        user_service: Arc::new(UserService::new(database_repositories.user_repo(), configuration.clone(), file_system_operations.clone())),
    }
}