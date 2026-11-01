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

#[derive(Default)]
pub struct UserBuilder {
    id: u32,
    user_id: String,
    key_type: String,
    key_path: String,
    user_dump_path: String,
}

impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    pub fn key_type(&self) -> &str {
        &self.key_type
    }
    pub fn key_path(&self) -> &str {
        &self.key_path
    }
    pub fn user_dump_path(&self) -> &str {
        &self.user_dump_path
    }
}

impl UserBuilder {
    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = user_id.to_string();
        self
    }
    pub fn key_type(mut self, key_type: &str) -> Self {
        self.key_type = key_type.to_string();
        self
    }
    pub fn key_path(mut self, key_path: String) -> Self {
        self.key_path = key_path.to_string();
        self
    }
    pub fn user_dump_path(mut self, user_dump_path: String) -> Self {
        self.user_dump_path = user_dump_path.to_string();
        self
    }
    pub fn build(self) -> User {
        User {
            id: self.id,
            user_id: self.user_id,
            key_type: self.key_type,
            key_path: self.key_path,
            user_dump_path: self.user_dump_path,
        }
    }
}

#[async_trait]
pub trait UserRepo {
    async fn get_user(&self, user_id: &str) -> Result<Option<User>, DatabaseError>;
    async fn save_user(
        &self,
        user_id: &str,
        key_type: &str,
        key_path: &str,
        user_dump_path: &str,
    ) -> Result<User, DatabaseError>;
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
    async fn save_user(
        &self,
        user_id: &str,
        key_type: &str,
        key_path: &str,
        user_dump_path: &str,
    ) -> Result<User, DatabaseError> {
        let new_user = sqlx::query_as::<_, User>(
            "INSERT INTO acme_users (user_id, key_type, key_path, user_dump_path) VALUES ($1, $2, $3, $4) RETURNING *"
        )
            .bind(user_id)
            .bind(key_type)
            .bind(key_path)
            .bind(user_dump_path)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(new_user)
    }
}
