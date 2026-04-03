use crate::database::DatabaseConnection;
use crate::user::AcmeUser;
use sqlite::{State, Statement};
use std::error::Error;
use tracing::info;

#[derive(Debug)]
pub struct AcmeAccountLink {
    pub id: i64,
    pub user_id: i64,
    pub account_url: String,
}

impl AcmeAccountLink {
    pub fn scan_statement(mut statement: Statement) -> Result<Option<Self>, Box<dyn Error>> {
        if let Ok(State::Row) = statement.next() {
            let link = Self {
                id: statement.read::<i64, _>("id")?,
                user_id: statement.read::<i64, _>("user_id")?,
                account_url: statement.read::<String, _>("account_url")?,
            };
            return Ok(Some(link));
        }
        Ok(None)
    }

    pub fn get_for_user(
        user_id: i64,
        conn: &DatabaseConnection,
    ) -> Result<Option<AcmeAccountLink>, Box<dyn Error>> {
        let sql = r#"
            SELECT * FROM acme_account_links WHERE user_id = ?1 ORDER BY id DESC LIMIT 1;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, user_id))?;
        Self::scan_statement(statement)
    }

    pub fn upsert(
        user_id: i64,
        account_url: &str,
        conn: &DatabaseConnection,
    ) -> Result<AcmeAccountLink, Box<dyn Error>> {
        let existing = Self::get_for_user(user_id, conn)?;
        if let Some(existing) = existing {
            if existing.account_url == account_url {
                info!(
                    "Account link for user_id={} is already up to date, skipping upsert.",
                    user_id
                );
                return Ok(existing);
            }
            let sql = r#"
                UPDATE acme_account_links SET account_url = ?1 WHERE user_id = ?2 RETURNING *;
            "#;
            let mut statement = conn.prepare(sql)?;
            statement.bind((1, account_url))?;
            statement.bind((2, user_id))?;
            return Self::scan_statement(statement)?
                .ok_or_else(|| "UPDATE returned no rows".into());
        }
        let sql = r#"
            INSERT INTO acme_account_links (user_id, account_url)
            VALUES (?1, ?2) RETURNING *;
        "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, user_id))?;
        statement.bind((2, account_url))?;
        Self::scan_statement(statement)?.ok_or_else(|| "INSERT returned no rows".into())
    }

    pub fn upsert_for_user_id(
        user_id_str: &str,
        account_url: &str,
        conn: &DatabaseConnection,
    ) -> anyhow::Result<AcmeAccountLink> {
        let user = AcmeUser::get_user(user_id_str, conn)
            .map_err(|e| anyhow::anyhow!("Failed to look up user '{}': {}", user_id_str, e))?;
        let user = user.ok_or_else(|| {
            anyhow::anyhow!("User '{}' not found when saving account link", user_id_str)
        })?;
        info!(
            "Persisting account link for user '{}' -> {}",
            user_id_str, account_url
        );
        Self::upsert(user.id, account_url, conn)
            .map_err(|e| anyhow::anyhow!("Failed to upsert account link: {}", e))
    }
}
