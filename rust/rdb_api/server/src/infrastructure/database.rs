use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};

use crate::error::AppResult;

pub async fn create_pool() -> AppResult<Pool<MySql>> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|e| anyhow::anyhow!("DATABASE_URL not set: {e}"))?;

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
