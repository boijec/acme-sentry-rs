use std::sync::Arc;
use sentry_db::repo::DatabaseRepositories;
use crate::user::service::UserService;

pub struct Services {
    user_service: Arc<UserService>,
}

impl Services {
    pub fn user_service(&self) -> Arc<UserService> {
        self.user_service.clone()
    }
}

pub fn start_services(database_repositories: DatabaseRepositories) -> Services {
    Services {
        user_service: Arc::new(UserService::new(database_repositories.user_repo()))
    }
}