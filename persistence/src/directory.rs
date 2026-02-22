use crate::database::DatabaseConnection;
use crate::user::AcmeUser;
use acme_client::comms::directory::AcmeDirectoryApi;
use common_utils::{CompareFields, FieldDiff};
use sqlite::{State, Statement};
use std::error::Error;
use tracing::info;

#[derive(Debug)]
pub struct AcmeDirectory {
    pub directory_id: i64,
    pub user_id: String,
    pub directory_base_url: String,
    pub key_change: String,
    pub new_authz: Option<String>,
    pub new_nonce: String,
    pub new_account: String,
    pub new_order: String,
    pub revoke_cert: String,
}
impl CompareFields<AcmeDirectoryApi> for AcmeDirectory {
    fn compare_fields(&self, other: &AcmeDirectoryApi) -> Vec<FieldDiff> {
        let auths = self.new_authz.clone().unwrap().as_str().to_string();
        let other_auths = other.new_authz.clone().unwrap_or("".to_string()).as_str().to_string();
        vec![
            FieldDiff {
                field: "key_change",
                are_equal: self.key_change == other.key_change,
            },
            FieldDiff {
                field: "new_authz",
                are_equal: auths == other_auths,
            },
            FieldDiff {
                field: "new_nonce",
                are_equal: self.new_nonce == other.new_nonce,
            },
            FieldDiff {
                field: "new_account",
                are_equal: self.new_account == other.new_account,
            },
            FieldDiff {
                field: "new_order",
                are_equal: self.new_order == other.new_order,
            },
            FieldDiff {
                field: "revoke_cert",
                are_equal: self.revoke_cert == other.revoke_cert,
            },
        ]
    }
}

impl AcmeDirectory {
    pub fn scan_statement(mut statement: Statement) -> Result<Option<Self>, Box<dyn Error>> {
        if let Ok(State::Row) = statement.next() {
            let dir = Self {
                directory_id: statement.read::<i64, _>("directory_id")?,
                user_id: statement.read::<String, _>("user_id")?,
                directory_base_url: statement.read::<String, _>("directory_base_url")?,
                key_change: statement.read::<String, _>("key_change")?,
                new_nonce: statement.read::<String, _>("new_nonce")?,
                new_account: statement.read::<String, _>("new_account")?,
                new_order: statement.read::<String, _>("new_order")?,
                new_authz: statement.read::<Option<String>, _>("new_authz")?,
                revoke_cert: statement.read::<String, _>("revoke_cert")?,
            };
            return Ok(Some(dir));
        } else if let Ok(State::Done) = statement.next() {
            return Ok(None)
        }
        Err(statement.next().unwrap_err().into())
    }
    pub fn get_existing(
        user_id: i64,
        connection: &DatabaseConnection,
    ) -> anyhow::Result<Option<AcmeDirectory>> {
        let sql = r#"
            SELECT * FROM acme_users_directory WHERE user_id = $1;
            "#;
        let mut statement = connection.prepare(sql).unwrap();
        statement.bind((1, user_id))?;
        Ok(Self::scan_statement(statement).unwrap())
    }
    pub fn get_dir_from_user(user_id: String, connection: &DatabaseConnection) -> anyhow::Result<Option<AcmeDirectory>> {
        let user = AcmeUser::get_user(user_id.as_str(), &connection).unwrap();
        if user.is_none() {
            return Err(anyhow::anyhow!(
                "Could not complete directory job since queried user could not be found!"
            ))?;
        }
        let user = user.unwrap();
        let existing_dir = Self::get_existing(user.id, &connection)?;
        Ok(existing_dir)
    }
    pub fn refresh_if_diff(
        acme_directory: AcmeDirectoryApi,
        user_id: String,
        acme_dir_url: String,
        connection: &DatabaseConnection,
    ) -> anyhow::Result<Option<AcmeDirectory>> {
        let user = AcmeUser::get_user(user_id.as_str(), &connection).unwrap();
        if user.is_none() {
            return Err(anyhow::anyhow!(
                "Could not complete directory job since queried user could not be found!"
            ))?;
        }
        let user = user.unwrap();
        let existing_dir = Self::get_existing(user.id, &connection)?;
        let sql;
        if existing_dir.is_none() {
            sql = r#"
            INSERT INTO acme_users_directory(
                user_id,
                directory_base_url,
                new_nonce,
                new_account,
                new_order,
                new_authz,
                revoke_cert,
                key_change
            ) VALUES (?1, ?8, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING *;
            "#;
        } else {
            let existing = existing_dir.unwrap();
            info!("Existing acme directory found for user_id: {} - directory id: {}", user.id, existing.directory_id);
            if existing.is_equal_to(&acme_directory) {
                info!(
                    "Existing acme directory was found to be up to date - skipping refresh..."
                );
                return Ok(Some(existing));
            }
            info!("Acme directory found not to be equal to request - refreshing...");
            sql = Self::get_update();
        }
        let mut statement = connection.prepare(sql).unwrap();
        statement.bind((1, user.id))?;
        statement.bind((2, acme_directory.new_nonce.as_str()))?;
        statement.bind((3, acme_directory.new_account.as_str()))?;
        statement.bind((4, acme_directory.new_order.as_str()))?;
        statement.bind((
            5,
            acme_directory.new_authz.unwrap_or("".to_string()).as_str(),
        ))?;
        statement.bind((6, acme_directory.revoke_cert.as_str()))?;
        statement.bind((7, acme_directory.key_change.as_str()))?;
        statement.bind((8, acme_dir_url.as_str()))?;
        Ok(Self::scan_statement(statement).unwrap())
    }
    pub fn get_update() -> &'static str {
        r#"
        UPDATE acme_users_directory SET
            directory_base_url = ?8
            new_nonce = ?2,
            new_account = ?3,
            new_order = ?4,
            new_authz = ?5,
            revoke_cert = ?6,
            key_change = ?7
        WHERE user_id = ?1 RETURNING *;
        "#
    }
}