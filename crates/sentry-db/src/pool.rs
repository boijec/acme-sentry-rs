use crate::error::DatabaseError;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

pub const DB_FILE_NAME: &str = "acme-sentry.db";

pub async fn connect(connection_path: &String) -> Result<Pool<Sqlite>, DatabaseError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .connect(connection_path)
        .await
        .map_err(|e| DatabaseError::ConnectionError(e))?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use tempfile::tempdir;
    use crate::pool::connect;

    #[tokio::test]
    async fn test_initialization() {
        let tmp_dir = tempdir().unwrap();
        let fp = tmp_dir.path().join("test.db");
        let _ = File::create(&fp).unwrap();
        let connection_string = fp.into_string().unwrap();
        let t = connect(&connection_string).await.unwrap();
        t.close().await;
        assert!(t.is_closed());
    }
}
