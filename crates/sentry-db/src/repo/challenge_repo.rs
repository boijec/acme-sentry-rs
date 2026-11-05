use crate::error::DatabaseError;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Serialize, FromRow)]
pub struct DnsChallenge {
    id: i64,
    order_id: i64,
    token: String,
    key_authorization: String,
    dns_record_name: String,
    txt_record_value: String,
    status: String,
    challenge_ready_url: String,
}

impl DnsChallenge {
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn order_id(&self) -> i64 {
        self.order_id
    }
    pub fn token(&self) -> &str {
        &self.token
    }
    pub fn key_authorization(&self) -> &str {
        &self.key_authorization
    }
    pub fn dns_record_name(&self) -> &str {
        &self.dns_record_name
    }
    pub fn txt_record_value(&self) -> &str {
        &self.txt_record_value
    }
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn challenge_ready_url(&self) -> &str {
        &self.challenge_ready_url
    }
}

#[async_trait]
pub trait ChallengeRepo {
    async fn create_challenge(
        &self,
        order_id: i64,
        token: String,
        key_authorization: String,
        dns_record_name: String,
        txt_record_value: String,
        challenge_url: String,
    ) -> Result<DnsChallenge, DatabaseError>;
    async fn get_for_order(&self, order_id: i64) -> Result<Option<DnsChallenge>, DatabaseError>;
    async fn set_challenge_status(
        &self,
        challenge_id: i64,
        status: String,
    ) -> Result<DnsChallenge, DatabaseError>;
}

pub struct ChallengeRepository {
    pool: Pool<Sqlite>,
}

impl ChallengeRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        ChallengeRepository { pool }
    }
}

#[async_trait]
impl ChallengeRepo for ChallengeRepository {
    async fn create_challenge(
        &self,
        order_id: i64,
        token: String,
        key_authorization: String,
        dns_record_name: String,
        txt_record_value: String,
        challenge_url: String,
    ) -> Result<DnsChallenge, DatabaseError> {
        let challenge = sqlx::query_as::<_, DnsChallenge>(
            r#"
                    INSERT INTO acme_dns_challenges (order_id, token, key_authorization, dns_record_name, status, txt_record_value, challenge_ready_url)
                    VALUES ($1, $2, $3, $4, 'pending', $5, $6) RETURNING *;
                "#
        )
            .bind(order_id)
            .bind(token)
            .bind(key_authorization)
            .bind(dns_record_name)
            .bind(txt_record_value)
            .bind(challenge_url)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(challenge)
    }

    async fn get_for_order(&self, order_id: i64) -> Result<Option<DnsChallenge>, DatabaseError> {
        let challenge = sqlx::query_as::<_, DnsChallenge>(
            r#"
                    SELECT * FROM acme_dns_challenges WHERE order_id = $1 ORDER BY id DESC LIMIT 1;
                "#,
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(Some(challenge))
    }

    async fn set_challenge_status(
        &self,
        challenge_id: i64,
        status: String,
    ) -> Result<DnsChallenge, DatabaseError> {
        let challenge = sqlx::query_as::<_, DnsChallenge>(
            r#"
                    UPDATE acme_dns_challenges SET status = $1 WHERE id = $2 RETURNING *;
                "#,
        )
        .bind(status)
        .bind(challenge_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDatabaseError(err.to_string()))?;
        Ok(challenge)
    }
}
