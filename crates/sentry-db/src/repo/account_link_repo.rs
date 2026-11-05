use crate::error::DatabaseError;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Serialize, FromRow)]
pub struct AccountLink {
    id: i64,
    user_id: i64,
    account_url: String,
}

#[derive(Default)]
pub struct AccountLinkBuilder {
    id: i64,
    user_id: i64,
    account_url: String,
}

impl AccountLink {
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn user_id(&self) -> i64 {
        self.user_id
    }
    pub fn account_url(&self) -> &str {
        &self.account_url
    }
    pub fn builder() -> AccountLinkBuilder {
        AccountLinkBuilder::default()
    }
}
impl AccountLinkBuilder {
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }
    pub fn user_id(mut self, user_id: i64) -> Self {
        self.user_id = user_id;
        self
    }
    pub fn account_url(mut self, account_url: String) -> Self {
        self.account_url = account_url;
        self
    }
    pub fn build(self) -> AccountLink {
        AccountLink {
            id: self.id,
            user_id: self.user_id,
            account_url: self.account_url,
        }
    }
}

#[async_trait]
pub trait AccountLinkRepo {
    async fn get_link_for_user(&self, user_id: i64) -> Result<Option<AccountLink>, DatabaseError>;
    async fn insert_linked_user(
        &self,
        user_link: AccountLink,
    ) -> Result<AccountLink, DatabaseError>;
    async fn update_linked_user(
        &self,
        user_link: AccountLink,
    ) -> Result<AccountLink, DatabaseError>;
}

pub struct AccountLinkRepository {
    pool: Pool<Sqlite>,
}

impl AccountLinkRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        AccountLinkRepository { pool }
    }
}

#[async_trait]
impl AccountLinkRepo for AccountLinkRepository {
    async fn get_link_for_user(&self, user_id: i64) -> Result<Option<AccountLink>, DatabaseError> {
        let account_link = sqlx::query_as::<_, AccountLink>(
            "SELECT * FROM acme_account_links WHERE user_id = $1 ORDER BY id DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(account_link)
    }

    async fn insert_linked_user(
        &self,
        user_link: AccountLink,
    ) -> Result<AccountLink, DatabaseError> {
        let new_account_link = sqlx::query_as::<_, AccountLink>(
            r#"
                    INSERT INTO acme_account_links (user_id, account_url)
                    VALUES ($1, $2) RETURNING *;
                "#,
        )
        .bind(user_link.user_id)
        .bind(user_link.account_url)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(new_account_link)
    }

    async fn update_linked_user(
        &self,
        user_link: AccountLink,
    ) -> Result<AccountLink, DatabaseError> {
        let new_account_link = sqlx::query_as::<_, AccountLink>(
            r#"
                    UPDATE acme_account_links SET account_url = $1 WHERE user_id = $2 RETURNING *;
                "#,
        )
        .bind(user_link.account_url)
        .bind(user_link.user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(new_account_link)
    }
}
