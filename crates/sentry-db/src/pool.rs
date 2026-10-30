use crate::error::DatabaseError;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

pub async fn connect(connection_path: &String) -> Result<Pool<Sqlite>, DatabaseError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .connect(connection_path)
        .await
        .map_err(|e| DatabaseError::ConnectionError(e))?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    Ok(pool)
}



#[cfg(test)]
mod tests {
    use crate::pool::connect;

    #[tokio::test]
    #[ignore]
    async fn test_initialization() {
        let connection_string = "test.db".to_string();
        let t = connect(&connection_string).await.unwrap();
        t.close().await;
        assert!(t.is_closed());
    }

}