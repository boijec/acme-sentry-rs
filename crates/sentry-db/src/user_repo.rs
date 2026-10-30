use crate::error::DatabaseError;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::{Pool, Sqlite};

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    id: u32,
    user_id: String,
    key_type: String,
    key_path: String,
    user_dump_path: String,
}

#[async_trait]
pub trait UserRepo {
    async fn get_user(&self, user_id: &str) -> Result<Option<User>, DatabaseError>;
    async fn save_user(&self, user: &User) -> Result<User, DatabaseError>;
}

pub struct UserRepository {
    pool: Pool<Sqlite>,
}

impl UserRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        UserRepository { pool }
    }
}

#[async_trait]
impl UserRepo for UserRepository {
    async fn get_user(&self, user_id: &str) -> Result<Option<User>, DatabaseError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM acme_users WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(Some(user))
    }
    async fn save_user(&self, user: &User) -> Result<User, DatabaseError> {
        let new_user = sqlx::query_as::<_, User>(
            "INSERT INTO acme_users (user_id, key_type, key_path, user_dump_path) VALUES ($1, $2, $3, $4) RETURNING *"
        )
            .bind(user.user_id.as_str())
            .bind(user.key_type.as_str())
            .bind(user.key_path.as_str())
            .bind(user.user_dump_path.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(new_user)
    }
}