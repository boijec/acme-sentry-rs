use std::sync::Arc;
use sqlx::{Pool, Sqlite};
use crate::repo::account_link_repo::{AccountLinkRepo, AccountLinkRepository};
use crate::repo::certificate_repo::{CertificateRepo, CertificateRepository};
use crate::repo::challenge_repo::{ChallengeRepo, ChallengeRepository};
use crate::repo::directory_repo::{DirectoryRepo, DirectoryRepository};
use crate::repo::order_repo::{OrderRepo, OrderRepository};
use crate::repo::user_repo::{UserRepo, UserRepository};

pub mod user_repo;
pub mod directory_repo;
pub mod account_link_repo;
pub mod certificate_repo;
pub mod challenge_repo;
pub mod order_repo;

pub struct DatabaseRepositories {
    user_repo: Arc<dyn UserRepo + Send + Sync>,
    directory_repo: Arc<dyn DirectoryRepo + Send + Sync>,
    account_link_repo: Arc<dyn AccountLinkRepo + Send + Sync>,
    certificate_repo: Arc<dyn CertificateRepo + Send + Sync>,
    challenge_repo: Arc<dyn ChallengeRepo + Send + Sync>,
    order_repo: Arc<dyn OrderRepo + Send + Sync>,
}

impl DatabaseRepositories {
    pub fn user_repo(&self) -> Arc<dyn UserRepo + Send + Sync> {
        self.user_repo.clone()
    }
    pub fn directory_repo(&self) -> Arc<dyn DirectoryRepo + Send + Sync> {
        self.directory_repo.clone()
    }
    pub fn account_link_repo(&self) -> Arc<dyn AccountLinkRepo + Send + Sync> {
        self.account_link_repo.clone()
    }
    pub fn certificate_repo(&self) -> Arc<dyn CertificateRepo + Send + Sync> {
        self.certificate_repo.clone()
    }
    pub fn challenge_repo(&self) -> Arc<dyn ChallengeRepo + Send + Sync> {
        self.challenge_repo.clone()
    }
    pub fn order_repo(&self) -> Arc<dyn OrderRepo + Send + Sync> {
        self.order_repo.clone()
    }
}

pub fn setup_repos(pool: &Pool<Sqlite>) -> DatabaseRepositories {
    DatabaseRepositories {
        user_repo: Arc::new(UserRepository::new(pool.clone())),
        directory_repo: Arc::new(DirectoryRepository::new(pool.clone())),
        account_link_repo: Arc::new(AccountLinkRepository::new(pool.clone())),
        certificate_repo: Arc::new(CertificateRepository::new(pool.clone())),
        challenge_repo: Arc::new(ChallengeRepository::new(pool.clone())),
        order_repo: Arc::new(OrderRepository::new(pool.clone())),
    }
}