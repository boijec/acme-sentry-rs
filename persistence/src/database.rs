use common_utils::EnumIterator;
use sqlite::Statement;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::slice::Iter;
use tracing::{debug, info};

pub struct DatabaseConnection {
    connection: sqlite::Connection,
    db_loc: String,
}
// TODO: remove allow
#[allow(dead_code)]
#[derive(Debug)]
enum PreFlightCheckList {
    AcmeUsersTable,
    AcmeUserDirectory,
    AcmeUserOrders,
    AcmeUserCertificates,
}
#[derive(Debug)]
enum SqliteSettings {
    ForeignKeysEnabled,
}
#[derive(Debug)]
enum KeyOperations {
    GetKeyFromKeyId,
}

trait SqlStatement {
    fn get_statement(&self) -> &'static str;
}
impl SqlStatement for SqliteSettings {
    fn get_statement(&self) -> &'static str {
        match self {
            SqliteSettings::ForeignKeysEnabled => "PRAGMA foreign_keys = ON;",
        }
    }
}
impl Display for SqliteSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Display for PreFlightCheckList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Display for KeyOperations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl EnumIterator<PreFlightCheckList> for PreFlightCheckList {
    fn iterator() -> Iter<'static, PreFlightCheckList> {
        static PRE_FLIGHT_CHECK_LIST: &[PreFlightCheckList] = &[
            PreFlightCheckList::AcmeUsersTable,
            PreFlightCheckList::AcmeUserDirectory,
        ];
        PRE_FLIGHT_CHECK_LIST.iter()
    }
}
impl EnumIterator<SqliteSettings> for SqliteSettings {
    fn iterator() -> Iter<'static, SqliteSettings> {
        static SQLITE_SETTINGS: &[SqliteSettings] = &[SqliteSettings::ForeignKeysEnabled];
        SQLITE_SETTINGS.iter()
    }
}
impl SqlStatement for PreFlightCheckList {
    fn get_statement(&self) -> &'static str {
        match self {
            PreFlightCheckList::AcmeUsersTable => {
                r#"
                CREATE TABLE IF NOT EXISTS acme_users(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id TEXT(36) NOT NULL,
                    key_type TEX(15) NOT NULL,
                    key_path TEXT(256) NOT NULL,
                    user_dump_path TEXT(256) NOT NULL
                );
            "#
            }
            PreFlightCheckList::AcmeUserDirectory => {
                r#"
                CREATE TABLE IF NOT EXISTS acme_users_directory(
                    directory_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    new_nonce TEXT(512) NOT NULL,
                    new_account TEXT(512),
                    new_order TEXT(512) NOT NULL,
                    new_authz TEXT(512) NOT NULL,
                    revoke_cert TEXT(512) NOT NULL,
                    key_change TEXT(512) NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES acme_users(id) ON DELETE RESTRICT
                )
            "#
            }
            PreFlightCheckList::AcmeUserOrders => r#"dudde"#,
            PreFlightCheckList::AcmeUserCertificates => r#"dudde"#,
        }
    }
}
impl DatabaseConnection {
    pub fn get_connection(db_loc: &str) -> Result<DatabaseConnection, Box<dyn Error>> {
        debug!("Opening db at: {}/acme-sentry.db", db_loc);
        let connection = sqlite::open((db_loc.to_owned() + "/acme-sentry.db").as_str())?;
        for settings in SqliteSettings::iterator() {
            debug!("Executing setting: {} for Sqlite", settings);
            connection.execute(settings.get_statement())?
        }
        debug!("Database settings have been executed!");
        Ok(DatabaseConnection { connection, db_loc: db_loc.to_owned() })
    }

    pub fn prepare(&self, prepared_statement: &str) -> Result<Statement, Box<dyn Error>> {
        Ok(self.connection.prepare(prepared_statement)?)
    }

    pub fn internal_structure_check(&self) -> Result<(), Box<dyn Error>> {
        for pre_flight in PreFlightCheckList::iterator() {
            debug!("Executing pre-flight script: {}", pre_flight);
            self.connection.execute(pre_flight.get_statement())?;
        }
        info!("Pre-flight database scripts have been executed!");
        Ok(())
    }
}
