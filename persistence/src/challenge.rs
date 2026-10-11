use crate::database::DatabaseConnection;
use sqlite::{State, Statement};
use std::error::Error;

// TODO: challenge info url
#[derive(Debug)]
pub struct AcmeDnsChallenge {
    pub id: i64,
    pub order_id: i64,
    pub token: String,
    pub key_authorization: String,
    pub dns_record_name: String,
    pub status: String,
}

impl AcmeDnsChallenge {
    pub fn scan_statement(mut statement: Statement) -> Result<Option<Self>, Box<dyn Error>> {
        if let Ok(State::Row) = statement.next() {
            let challenge = Self {
                id: statement.read::<i64, _>("id")?,
                order_id: statement.read::<i64, _>("order_id")?,
                token: statement.read::<String, _>("token")?,
                key_authorization: statement.read::<String, _>("key_authorization")?,
                dns_record_name: statement.read::<String, _>("dns_record_name")?,
                status: statement.read::<String, _>("status")?,
            };
            return Ok(Some(challenge));
        }
        Ok(None)
    }

    // TODO: challenge url
    pub fn create(
        order_id: i64,
        token: &str,
        key_authorization: &str,
        dns_record_name: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeDnsChallenge, Box<dyn Error>> {
        let sql = r#"
            INSERT INTO acme_dns_challenges (order_id, token, key_authorization, dns_record_name, status)
            VALUES (?1, ?2, ?3, ?4, 'pending') RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, order_id))?;
        statement.bind((2, token))?;
        statement.bind((3, key_authorization))?;
        statement.bind((4, dns_record_name))?;
        Self::scan_statement(statement)?.ok_or_else(|| "INSERT returned no rows".into())
    }

    pub fn get_for_order(
        order_id: i64,
        conn: &DatabaseConnection,
    ) -> Result<Option<AcmeDnsChallenge>, Box<dyn Error>> {
        let sql = r#"
            SELECT * FROM acme_dns_challenges WHERE order_id = ?1 ORDER BY id DESC LIMIT 1;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, order_id))?;
        Self::scan_statement(statement)
    }

    pub fn set_status(
        challenge_id: i64,
        status: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeDnsChallenge, Box<dyn Error>> {
        let sql = r#"
            UPDATE acme_dns_challenges SET status = ?1 WHERE id = ?2 RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, status))?;
        statement.bind((2, challenge_id))?;
        Self::scan_statement(statement)?.ok_or_else(|| "UPDATE returned no rows".into())
    }
}
