use common_utils::{EnumIterator, APPLICATION_CONFIG};
use sqlite::Statement;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::slice::Iter;
use tracing::{debug, info};

pub struct DatabaseConnection {
    connection: sqlite::Connection
}
// TODO: remove allow
#[allow(dead_code)]
#[derive(Debug)]
enum PreFlightCheckList {
    AcmeUsersTable,
    AcmeUserDirectory,
    AcmeAccountLinks,
    AcmeOrders,
    AcmeDnsChallenges,
    AcmeCertificates,
}
#[derive(Debug)]
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
impl EnumIterator<PreFlightCheckList> for PreFlightCheckList {
    fn iterator() -> Iter<'static, PreFlightCheckList> {
        static PRE_FLIGHT_CHECK_LIST: &[PreFlightCheckList] = &[
            PreFlightCheckList::AcmeUsersTable,
            PreFlightCheckList::AcmeUserDirectory,
            PreFlightCheckList::AcmeAccountLinks,
            PreFlightCheckList::AcmeOrders,
            PreFlightCheckList::AcmeDnsChallenges,
            PreFlightCheckList::AcmeCertificates,
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
                    directory_base_url TEXT(512) NOT NULL,
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
            PreFlightCheckList::AcmeAccountLinks => {
                r#"
                CREATE TABLE IF NOT EXISTS acme_account_links(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    account_url TEXT(512) NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES acme_users(id) ON DELETE RESTRICT
                );
            "#
            }
            PreFlightCheckList::AcmeOrders => {
                r#"
                CREATE TABLE IF NOT EXISTS acme_orders(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    domain TEXT(256) NOT NULL,
                    order_url TEXT(512),
                    authorization_url TEXT(512),
                    finalize_url TEXT(512),
                    certificate_url TEXT(512),
                    status TEXT(32) NOT NULL DEFAULT 'pending',
                    created_at TEXT(32) NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES acme_users(id) ON DELETE RESTRICT
                );
            "#
            }
            PreFlightCheckList::AcmeDnsChallenges => {
                r#"
                CREATE TABLE IF NOT EXISTS acme_dns_challenges(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    order_id INTEGER NOT NULL,
                    token TEXT(512) NOT NULL,
                    key_authorization TEXT(1024) NOT NULL,
                    dns_record_name TEXT(512) NOT NULL,
                    status TEXT(32) NOT NULL DEFAULT 'pending',
                    FOREIGN KEY (order_id) REFERENCES acme_orders(id) ON DELETE RESTRICT
                );
            "#
            }
            PreFlightCheckList::AcmeCertificates => {
                r#"
                CREATE TABLE IF NOT EXISTS acme_certificates(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    order_id INTEGER NOT NULL,
                    certificate_path TEXT(256) NOT NULL,
                    downloaded_at TEXT(32) NOT NULL,
                    FOREIGN KEY (order_id) REFERENCES acme_orders(id) ON DELETE RESTRICT
                );
            "#
            }
        }
    }
}
impl DatabaseConnection {
    pub fn get_connection() -> Result<DatabaseConnection, Box<dyn Error>> {
        let config = APPLICATION_CONFIG.get().unwrap();
        debug!("Opening db at: {}/acme-sentry.db", config.base_dir.as_str());
        let connection = sqlite::open((config.base_dir.to_owned() + "/acme-sentry.db").as_str())?;
        for settings in SqliteSettings::iterator() {
            debug!("Executing setting: {} for Sqlite", settings);
            connection.execute(settings.get_statement())?
        }
        debug!("Database settings have been executed!");
        Ok(DatabaseConnection { connection })
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
