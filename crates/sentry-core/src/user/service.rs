use crate::user::model::ServiceUser;
use sentry_db::repo::user_repo::UserRepo;
use std::sync::Arc;

pub struct UserService {
    user_repo: Arc<dyn UserRepo + Send + Sync>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepo + Send + Sync>) -> Self {
        Self { user_repo }
    }
}

impl UserService {
    async fn get_user(&self, user_id: &str) -> Result<Option<ServiceUser>, String> {
        if let Some(user) = self.user_repo.get_user(user_id).await.map_err(|e| e.to_string())? {
            return Ok(Some(ServiceUser {
                user_id: user.user_id().to_string(),
            }))
        }
        Ok(None)
    }
}