use crate::database::DatabaseConnection;
use sqlite::{State, Statement};
use std::error::Error;

#[derive(Debug)]
pub struct AcmeCertificate {
    pub id: i64,
    pub order_id: i64,
    pub certificate_path: String,
    pub downloaded_at: String,
}

impl AcmeCertificate {
    pub fn scan_statement(mut statement: Statement) -> Result<Option<Self>, Box<dyn Error>> {
        if let Ok(State::Row) = statement.next() {
            let cert = Self {
                id: statement.read::<i64, _>("id")?,
                order_id: statement.read::<i64, _>("order_id")?,
                certificate_path: statement.read::<String, _>("certificate_path")?,
                downloaded_at: statement.read::<String, _>("downloaded_at")?,
            };
            return Ok(Some(cert));
        }
        Ok(None)
    }

    pub fn record(
        order_id: i64,
        certificate_path: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeCertificate, Box<dyn Error>> {
        let now = unix_now();
        let sql = r#"
            INSERT INTO acme_certificates (order_id, certificate_path, downloaded_at)
            VALUES (?1, ?2, ?3) RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, order_id))?;
        statement.bind((2, certificate_path))?;
        statement.bind((3, now.as_str()))?;
        Self::scan_statement(statement)?.ok_or_else(|| "INSERT returned no rows".into())
    }

    pub fn get_for_order(
        order_id: i64,
        conn: &DatabaseConnection,
    ) -> Result<Option<AcmeCertificate>, Box<dyn Error>> {
        let sql = r#"
            SELECT * FROM acme_certificates WHERE order_id = ?1 ORDER BY id DESC LIMIT 1;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, order_id))?;
        Self::scan_statement(statement)
    }
}

fn unix_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        .to_string()
}
