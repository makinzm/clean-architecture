use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use std::time::Duration;

use crate::error::AppResult;

pub async fn create_pool() -> AppResult<DatabaseConnection> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|e| anyhow::anyhow!("DATABASE_URL not set: {e}"))?;

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(10)
       .min_connections(2)
       .connect_timeout(Duration::from_secs(8))
       .acquire_timeout(Duration::from_secs(8))
       .idle_timeout(Duration::from_secs(8))
       .max_lifetime(Duration::from_secs(8))
       .sqlx_logging(true)
       .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;

    Ok(db)
}

