use common_utils::EnumIterator;
use std::error::Error;
use std::slice::Iter;

pub struct DatabaseConnection {
    connection: sqlite::Connection,
}
// TODO: remove allow
#[allow(dead_code)]
enum PreFlightCheckList {
    AcmeUsersTable,
    AcmeUserDirectory,
    AcmeUserOrders,
    AcmeUserCertificates,
}
enum SqliteSettings {
    ForeignKeysEnabled,
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
        static SQLITE_SETTINGS: &[SqliteSettings] = &[
            SqliteSettings::ForeignKeysEnabled
        ];
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
                    key_id INTEGER NOT NULL,
                    key_path TEXT(256) NOT NULL
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
    pub fn get_connection() -> Result<DatabaseConnection, Box<dyn Error>> {
        let connection = sqlite::open("acme-sentry.db")?;
        for settings in SqliteSettings::iterator() {
            connection.execute(settings.get_statement())?
        }
        Ok(DatabaseConnection { connection })
    }

    pub fn internal_structure_check(&self) -> Result<(), Box<dyn Error>> {
        for pre_flight in PreFlightCheckList::iterator() {
            self.connection.execute(pre_flight.get_statement())?;
        }
        Ok(())
    }
}
