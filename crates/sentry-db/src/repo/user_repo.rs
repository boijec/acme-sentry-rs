use crate::error::DatabaseError;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::{Pool, Sqlite};

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    id: i64,
    user_id: String,
    key_type: String,
    key_path: String,
    key_name: String,
    user_dump_path: String,
}

#[derive(Default)]
pub struct UserBuilder {
    id: i64,
    user_id: String,
    key_type: String,
    key_path: String,
    key_name: String,
    user_dump_path: String,
}

impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
    pub fn id(&self) -> i64 {
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
    pub fn key_name(&self) -> &str {
        &self.key_name
    }
    pub fn user_dump_path(&self) -> &str {
        &self.user_dump_path
    }
}

impl UserBuilder {
    pub fn id(mut self, id: i64) -> Self {
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
    pub fn key_name(mut self, key_name: String) -> Self {
        self.key_name = key_name.to_string();
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
            key_name: self.key_name,
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
        key_name: &str,
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
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(user)
    }
    async fn save_user(
        &self,
        user_id: &str,
        key_type: &str,
        key_path: &str,
        key_name: &str,
        user_dump_path: &str,
    ) -> Result<User, DatabaseError> {
        let new_user = sqlx::query_as::<_, User>(
            "INSERT INTO acme_users (user_id, key_type, key_path, key_name, user_dump_path) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
            .bind(user_id)
            .bind(key_type)
            .bind(key_path)
            .bind(key_name)
            .bind(user_dump_path)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(new_user)
    }
}
