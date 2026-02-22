use std::error::Error;
use sqlite::{State, Statement};

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
}

#[derive(Debug)]
pub struct AcmeDirectory {
    pub directory_id: i64,
    pub user_id: String,
    pub key_change: String,
    pub new_authz: Option<String>,
    pub new_nonce: String,
    pub new_account: String,
    pub new_order: String,
    pub revoke_cert: String,
}

impl AcmeDirectory {
    pub fn scan_statement(mut statement: Statement) -> Result<Self, Box<dyn Error>> {
        if let Ok(State::Row) = statement.next() {
            let dir = Self {
                directory_id: statement.read::<i64, _>("directory_id")?,
                user_id: statement.read::<String, _>("user_id")?,
                key_change: statement.read::<String, _>("key_change")?,
                new_nonce: statement.read::<String, _>("new_nonce")?,
                new_account: statement.read::<String, _>("new_account")?,
                new_order: statement.read::<String, _>("new_order")?,
                new_authz: statement.read::<Option<String>, _>("new_authz")?,
                revoke_cert: statement.read::<String, _>("revoke_cert")?,
            };
            return Ok(dir);
        }
        Err(statement.next().unwrap_err().into())
    }
}