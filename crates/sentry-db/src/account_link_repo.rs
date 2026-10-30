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

#[async_trait]
pub trait AccountLinkRepo {
    async fn get_link_for_user(&self, user_id: i64) -> Result<Option<AccountLink>, DatabaseError>;
    async fn insert_linked_user(&self, user_link: AccountLink) -> Result<AccountLink, DatabaseError>;
    async fn update_linked_user(&self, user_link: AccountLink) -> Result<AccountLink, DatabaseError>;
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
        let account_link = sqlx::query_as::<_, AccountLink>("SELECT * FROM acme_account_links WHERE user_id = $1 ORDER BY id DESC LIMIT 1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(Some(account_link))
    }

    async fn insert_linked_user(&self, user_link: AccountLink) -> Result<AccountLink, DatabaseError> {
        let new_account_link = sqlx::query_as::<_, AccountLink>(
            r#"
                    INSERT INTO acme_account_links (user_id, account_url)
                    VALUES ($1, $2) RETURNING *;
                "#
        )
            .bind(user_link.user_id)
            .bind(user_link.account_url)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(new_account_link)
    }

    async fn update_linked_user(&self, user_link: AccountLink) -> Result<AccountLink, DatabaseError> {
        let new_account_link = sqlx::query_as::<_, AccountLink>(
            r#"
                    UPDATE acme_account_links SET account_url = $1 WHERE user_id = $2 RETURNING *;
                "#
        )
            .bind(user_link.account_url)
            .bind(user_link.user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(new_account_link)
    }
}