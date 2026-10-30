use crate::error::DatabaseError;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Serialize, FromRow)]
pub struct UserDirectory {
    directory_id: i64,
    user_id: String,
    directory_base_url: String,
    key_change: String,
    new_authz: Option<String>,
    new_nonce: String,
    new_account: String,
    new_order: String,
    revoke_cert: String,
}

#[async_trait]
pub trait DirectoryRepo {
    async fn get_directory_from_user(&self, user_id: &str) -> Result<Option<UserDirectory>, DatabaseError>;
    async fn insert_directory(&self, directory: UserDirectory) -> Result<UserDirectory, DatabaseError>;
    async fn update_directory(&self, directory: UserDirectory) -> Result<UserDirectory, DatabaseError>;
}

pub struct DirectoryRepository {
    pool: Pool<Sqlite>,
}

impl DirectoryRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        DirectoryRepository { pool }
    }
}

#[async_trait]
impl DirectoryRepo for DirectoryRepository {
    async fn get_directory_from_user(&self, user_id: &str) -> Result<Option<UserDirectory>, DatabaseError> {
        let user = sqlx::query_as::<_, UserDirectory>("SELECT * FROM acme_users_directory WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(Some(user))
    }
    async fn insert_directory(&self, directory: UserDirectory) -> Result<UserDirectory, DatabaseError> {
        let user = sqlx::query_as::<_, UserDirectory>(r#"
            INSERT INTO acme_users_directory(
                user_id,
                directory_base_url,
                new_nonce,
                new_account,
                new_order,
                new_authz,
                revoke_cert,
                key_change
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *;
        "#)
            .bind(directory.user_id.as_str())
            .bind(directory.directory_base_url.as_str())
            .bind(directory.new_nonce.as_str())
            .bind(directory.new_account.as_str())
            .bind(directory.new_order.as_str())
            .bind(directory.new_authz.unwrap_or("".to_string()).as_str())
            .bind(directory.revoke_cert.as_str())
            .bind(directory.key_change.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(user)
    }
    async fn update_directory(&self, directory: UserDirectory) -> Result<UserDirectory, DatabaseError> {
        let user = sqlx::query_as::<_, UserDirectory>(r#"
            UPDATE acme_users_directory SET
                directory_base_url = $1,
                new_nonce = $2,
                new_account = $3,
                new_order = $4,
                new_authz = $5,
                revoke_cert = $6,
                key_change = $7,
            WHERE user_id = $8 RETURNING *;
        "#)
            .bind(directory.directory_base_url.as_str())
            .bind(directory.new_nonce.as_str())
            .bind(directory.new_account.as_str())
            .bind(directory.new_order.as_str())
            .bind(directory.new_authz.unwrap_or("".to_string()).as_str())
            .bind(directory.revoke_cert.as_str())
            .bind(directory.key_change.as_str())
            .bind(directory.user_id.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(user)
    }
}