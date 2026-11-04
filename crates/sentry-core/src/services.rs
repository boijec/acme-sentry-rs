use crate::config::ApplicationConfig;
use crate::user::service::UserService;
use sentry_db::repo::DatabaseRepositories;
use sentry_io::fs::FileSystemOperations;
use std::sync::Arc;
use crate::directory::service::DirectoryService;
use crate::order::service::OrderService;

pub struct Services {
    user_service: Arc<UserService>,
    directory_service: Arc<DirectoryService>,
    order_service: Arc<OrderService>,
}

impl Services {
    pub fn user_service(&self) -> Arc<UserService> {
        self.user_service.clone()
    }
    pub fn directory_service(&self) -> Arc<DirectoryService> {
        self.directory_service.clone()
    }
    pub fn order_service(&self) -> Arc<OrderService> {
        self.order_service.clone()
    }
    
}

pub fn initialize_services(
    configuration: Arc<ApplicationConfig>,
    database_repositories: DatabaseRepositories,
    file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,
) -> Services {
    Services {
        user_service: Arc::new(UserService::new(
            database_repositories.user_repo(),
            database_repositories.account_link_repo(),
            configuration.clone(),
            file_system_operations.clone(),
        )),
        directory_service: Arc::new(DirectoryService::new(configuration.clone())),
        order_service: Arc::new(OrderService::new(configuration.clone(), database_repositories.order_repo())),
    }
}
