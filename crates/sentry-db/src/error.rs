use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Sentry persistence error: {0}")]
    GenericDatabaseError(String),
    #[error("DB connection error: {0}")]
    ConnectionError(sqlx::Error),
    #[error("Database migration error: {0}")]
    MigrationError(#[from] MigrateError),
}