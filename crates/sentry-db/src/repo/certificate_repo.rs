use crate::error::DatabaseError;
use crate::util::unix_now;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Serialize, FromRow)]
pub struct Certificate {
    id: i64,
    order_id: i64,
    certificate_path: String,
    downloaded_at: String,
}

#[async_trait]
pub trait CertificateRepo {
    async fn log_downloaded(&self, order_id: i64, certificate_path: String) -> Result<Certificate, DatabaseError>;
    async fn get_for_order(&self, order_id: i64) -> Result<Option<Certificate>, DatabaseError>;
}

pub struct CertificateRepository {
    pool: Pool<Sqlite>,
}

impl CertificateRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        CertificateRepository { pool }
    }
}

#[async_trait]
impl CertificateRepo for CertificateRepository {
    async fn log_downloaded(&self, order_id: i64, certificate_path: String) -> Result<Certificate, DatabaseError> {
        let certificate_log = sqlx::query_as::<_, Certificate>(
            r#"
                    INSERT INTO acme_certificates (order_id, certificate_path, downloaded_at)
                    VALUES ($1, $2, $3) RETURNING *;
                "#
        )
            .bind(order_id)
            .bind(certificate_path)
            .bind(unix_now().as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(certificate_log)
    }

    async fn get_for_order(&self, order_id: i64) -> Result<Option<Certificate>, DatabaseError> {
        let certificate_log = sqlx::query_as::<_, Certificate>(
            r#"
                    SELECT * FROM acme_certificates WHERE order_id = $1 ORDER BY id DESC LIMIT 1;
                "#
        )
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(Some(certificate_log))
    }
}