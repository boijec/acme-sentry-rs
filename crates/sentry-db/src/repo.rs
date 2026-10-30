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
    user_repo: Arc<dyn UserRepo>,
    directory_repo: Arc<dyn DirectoryRepo>,
    account_link_repo: Arc<dyn AccountLinkRepo>,
    certificate_repo: Arc<dyn CertificateRepo>,
    challenge_repo: Arc<dyn ChallengeRepo>,
    order_repo: Arc<dyn OrderRepo>,
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