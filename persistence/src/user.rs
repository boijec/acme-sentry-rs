use std::error::Error;
use std::path::PathBuf;
use sqlite::{State, Statement};
use crate::database::DatabaseConnection;

#[derive(Debug)]
pub struct AcmeUser {
    pub id: i64,
    pub user_id: String,
    pub key_type: String,
    pub key_path: String,
    pub user_dump_path: String,
}

impl AcmeUser {
    pub fn scan_statement(mut statement: Statement) -> Result<Self, Box<dyn Error>> {
        if let Ok(State::Row) = statement.next() {
            let user = Self {
                id: statement.read::<i64, _>("id")?,
                user_id: statement.read::<String, _>("user_id")?,
                key_type: statement.read::<String, _>("key_type")?,
                key_path: statement.read::<String, _>("key_path")?,
                user_dump_path: statement.read::<String, _>("user_dump_path")?,
            };
            return Ok(user)
        }
        Err("Failed to execute statement".into())
    }
    pub fn get_user(user_id: &str, conn: &DatabaseConnection) -> anyhow::Result<Option<AcmeUser>, Box<dyn Error>> {
        let sql = r#"
            SELECT * FROM acme_users WHERE user_id = ?1
            "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, user_id))?;
        Ok(Some(AcmeUser::scan_statement(statement)?))
    }
    pub fn new_user(key_path: PathBuf, dump_path: PathBuf, user_id: String, key_type: String, conn: &DatabaseConnection) -> Result<Option<AcmeUser>, Box<dyn Error>> {
        let sql = r#"
            INSERT INTO acme_users (user_id, key_type, key_path, user_dump_path)
            VALUES (?1, ?2, ?3, ?4) RETURNING *;
            "#;
        let mut statement = conn.prepare(sql)?;
        statement.bind((1, user_id.as_str()))?;
        statement.bind((2, key_type.as_str()))?;
        statement.bind((3, key_path.as_path().to_str().unwrap()))?;
        statement.bind((4, dump_path.as_path().to_str().unwrap()))?;
        Ok(Some(AcmeUser::scan_statement(statement)?))
    }
}