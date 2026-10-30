use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};
use crate::directory_repo::UserDirectory;
use crate::error::DatabaseError;
use crate::util::unix_now;

#[derive(Serialize, FromRow)]
pub struct Order {
    id: i64,
    user_id: i64,
    domain: String,
    order_url: Option<String>,
    authorization_url: Option<String>,
    finalize_url: Option<String>,
    certificate_url: Option<String>,
    status: String,
    created_at: String,
}

#[async_trait]
pub trait OrderRepo {
    async fn create_order(&self, user_id: i64, domain: String) -> Result<Order, DatabaseError>;
    async fn get_by_id(&self, order_id: i64) -> Result<Option<Order>, DatabaseError>;
    async fn get_pending_for_user(&self, user_id: i64) -> Result<Option<Order>, DatabaseError>;
    async fn update_order_urls(
        &self,
        order_id: i64,
        order_url: String,
        authorization_url: String,
        finalize_url: Option<String>,
    ) -> Result<Order, DatabaseError>;
    async fn set_certificate_url(&self, order_id: i64, certificate_url: String) -> Result<Order, DatabaseError>;
    async fn set_status(&self, order_id: i64, status: String) -> Result<Order, DatabaseError>;
}

pub struct OrderRepository {
    pool: Pool<Sqlite>,
}

impl OrderRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        OrderRepository { pool }
    }
}

#[async_trait]
impl OrderRepo for OrderRepository {
    async fn create_order(&self, user_id: i64, domain: String) -> Result<Order, DatabaseError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
                    INSERT INTO acme_orders (user_id, domain, status, created_at)
                    VALUES ($1, $2, 'pending', $3) RETURNING *;
                 "#
        )
            .bind(user_id)
            .bind(domain)
            .bind(unix_now().as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(order)
    }

    async fn get_by_id(&self, order_id: i64) -> Result<Option<Order>, DatabaseError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
                    SELECT * FROM acme_orders WHERE id = $1;
                 "#
        )
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(Some(order))
    }

    async fn get_pending_for_user(&self, user_id: i64) -> Result<Option<Order>, DatabaseError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
                    SELECT * FROM acme_orders
                    WHERE user_id = $1 AND status = 'pending'
                    ORDER BY id DESC LIMIT 1;
                 "#
        )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(Some(order))
    }

    async fn update_order_urls(&self,
                               order_id: i64,
                               order_url: String,
                               authorization_url: String,
                               finalize_url: Option<String>
    ) -> Result<Order, DatabaseError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
                    UPDATE acme_orders
                    SET order_url = $1, authorization_url = $2, finalize_url = $3, status = 'ready'
                    WHERE id = $4 RETURNING *;
                 "#
        )
            .bind(order_url)
            .bind(authorization_url)
            .bind(finalize_url)
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(order)
    }

    async fn set_certificate_url(&self, order_id: i64, certificate_url: String) -> Result<Order, DatabaseError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
                    UPDATE acme_orders
                    SET certificate_url = $1, status = 'valid'
                    WHERE id = $2 RETURNING *;
                 "#
        )
            .bind(certificate_url)
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(order)
    }

    async fn set_status(&self, order_id: i64, status: String) -> Result<Order, DatabaseError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
                    UPDATE acme_orders SET status = $1 WHERE id = $2 RETURNING *;
                 "#
        )
            .bind(status)
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(order)
    }
}