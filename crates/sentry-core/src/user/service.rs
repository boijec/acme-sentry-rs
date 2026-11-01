use crate::user::model::ServiceUser;
use sentry_db::repo::user_repo::{User, UserRepo};
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
    pub async fn get_user(&self, user_id: &str) -> Result<Option<ServiceUser>, String> {
        if let Some(user) = self
            .user_repo
            .get_user(user_id)
            .await
            .map_err(|e| e.to_string())?
        {
            return Ok(Some(ServiceUser {
                id: user.id(),
                user_id: user.user_id().to_string(),
                key_type: user.key_type().to_string(),
                key_path: user.key_path().to_string(),
                user_dump_path: user.user_dump_path().to_string(),
            }));
        }
        Ok(None)
    }

    pub async fn create_new_user(
        &self,
        new_user: ServiceUser
    ) -> Result<ServiceUser, String> {
        let user = self
            .user_repo
            .save_user(
                new_user.user_id.as_str(),
                &new_user.key_path.as_str(),
                &new_user.key_type.as_str(),
                &new_user.key_path.as_str(),
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(ServiceUser {
            id: user.id(),
            user_id: user.user_id().to_string(),
            key_type: user.key_type().to_string(),
            key_path: user.key_path().to_string(),
            user_dump_path: user.user_dump_path().to_string(),
        })
    }
}
