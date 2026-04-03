use crate::database::DatabaseConnection;
use sqlite::{State, Statement};
use std::error::Error;
use tracing::info;

#[derive(Debug)]
pub struct AcmeOrder {
    pub id: i64,
    pub user_id: i64,
    pub domain: String,
    pub order_url: Option<String>,
    pub authorization_url: Option<String>,
    pub finalize_url: Option<String>,
    pub certificate_url: Option<String>,
    pub status: String,
    pub created_at: String,
}

impl AcmeOrder {
    pub fn scan_statement(mut statement: Statement) -> Result<Option<Self>, Box<dyn Error>> {
        if let Ok(State::Row) = statement.next() {
            let order = Self {
                id: statement.read::<i64, _>("id")?,
                user_id: statement.read::<i64, _>("user_id")?,
                domain: statement.read::<String, _>("domain")?,
                order_url: statement.read::<Option<String>, _>("order_url")?,
                authorization_url: statement.read::<Option<String>, _>("authorization_url")?,
                finalize_url: statement.read::<Option<String>, _>("finalize_url")?,
                certificate_url: statement.read::<Option<String>, _>("certificate_url")?,
                status: statement.read::<String, _>("status")?,
                created_at: statement.read::<String, _>("created_at")?,
            };
            return Ok(Some(order));
        }
        Ok(None)
    }

    pub fn create(
        user_id: i64,
        domain: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeOrder, Box<dyn Error>> {
        let now = chrono_now();
        let sql = r#"
            INSERT INTO acme_orders (user_id, domain, status, created_at)
            VALUES (?1, ?2, 'pending', ?3) RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, user_id))?;
        statement.bind((2, domain))?;
        statement.bind((3, now.as_str()))?;
        Self::scan_statement(statement)?.ok_or_else(|| "INSERT returned no rows".into())
    }

    pub fn get_by_id(
        order_id: i64,
        conn: &DatabaseConnection,
    ) -> Result<Option<AcmeOrder>, Box<dyn Error>> {
        let sql = r#"SELECT * FROM acme_orders WHERE id = ?1;"#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, order_id))?;
        Self::scan_statement(statement)
    }

    pub fn get_pending_for_user(
        user_id: i64,
        conn: &DatabaseConnection,
    ) -> Result<Option<AcmeOrder>, Box<dyn Error>> {
        let sql = r#"
            SELECT * FROM acme_orders
            WHERE user_id = ?1 AND status = 'pending'
            ORDER BY id DESC LIMIT 1;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, user_id))?;
        Self::scan_statement(statement)
    }

    pub fn update_order_urls(
        order_id: i64,
        order_url: &str,
        authorization_url: &str,
        finalize_url: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeOrder, Box<dyn Error>> {
        info!(
            "Updating order id={} with order_url, authorization_url, finalize_url",
            order_id
        );
        let sql = r#"
            UPDATE acme_orders
            SET order_url = ?1, authorization_url = ?2, finalize_url = ?3, status = 'ready'
            WHERE id = ?4 RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, order_url))?;
        statement.bind((2, authorization_url))?;
        statement.bind((3, finalize_url))?;
        statement.bind((4, order_id))?;
        Self::scan_statement(statement)?.ok_or_else(|| "UPDATE returned no rows".into())
    }

    pub fn set_certificate_url(
        order_id: i64,
        certificate_url: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeOrder, Box<dyn Error>> {
        let sql = r#"
            UPDATE acme_orders
            SET certificate_url = ?1, status = 'valid'
            WHERE id = ?2 RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, certificate_url))?;
        statement.bind((2, order_id))?;
        Self::scan_statement(statement)?.ok_or_else(|| "UPDATE returned no rows".into())
    }

    pub fn set_status(
        order_id: i64,
        status: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeOrder, Box<dyn Error>> {
        let sql = r#"
            UPDATE acme_orders SET status = ?1 WHERE id = ?2 RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, status))?;
        statement.bind((2, order_id))?;
        Self::scan_statement(statement)?.ok_or_else(|| "UPDATE returned no rows".into())
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}
