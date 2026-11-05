use crate::challenge::service::ChallengeService;
use crate::config::ApplicationConfig;
use crate::directory::service::DirectoryService;
use crate::dns::service::DnsService;
use crate::order::service::OrderService;
use crate::user::service::UserService;
use sentry_db::repo::DatabaseRepositories;
use sentry_io::fs::FileSystemOperations;
use std::sync::Arc;

pub struct Services {
    user_service: Arc<UserService>,
    order_service: Arc<OrderService>,
    challenge_service: Arc<ChallengeService>,
    dns_service: Arc<DnsService>,
}

impl Services {
    pub fn user_service(&self) -> Arc<UserService> {
        self.user_service.clone()
    }
    pub fn order_service(&self) -> Arc<OrderService> {
        self.order_service.clone()
    }
    pub fn challenge_service(&self) -> Arc<ChallengeService> {
        self.challenge_service.clone()
    }
    pub fn dns_service(&self) -> Arc<DnsService> {
        self.dns_service.clone()
    }
}

pub fn initialize_services(
    configuration: Arc<ApplicationConfig>,
    database_repositories: DatabaseRepositories,
    file_system_operations: Arc<dyn FileSystemOperations + Send + Sync>,
) -> Services {
    let directory_service = Arc::new(DirectoryService::new(configuration.clone()));
    let dns_service = Arc::new(DnsService::new(configuration.clone()));
    let challenge_service = Arc::new(ChallengeService::new(
        configuration.clone(),
        database_repositories.challenge_repo(),
        directory_service.clone(),
    ));
    let user_service = Arc::new(UserService::new(
        database_repositories.user_repo(),
        database_repositories.account_link_repo(),
        database_repositories.certificate_repo(),
        configuration.clone(),
        file_system_operations.clone(),
        directory_service.clone(),
    ));
    let order_service = Arc::new(OrderService::new(
        configuration.clone(),
        database_repositories.order_repo(),
        directory_service.clone(),
        challenge_service.clone(),
    ));
    Services {
        user_service,
        order_service,
        challenge_service,
        dns_service,
    }
}
