use async_trait::async_trait;
use sqlx::{MySql, Pool, Transaction};

use crate::error::AppResult;
use crate::use_case::transaction_manager::TransactionManager;

pub struct SqlxTransactionManager {
    pool: Pool<MySql>,
}

impl SqlxTransactionManager {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionManager for SqlxTransactionManager {
    type Tx = Transaction<'static, MySql>;

    async fn begin(&self) -> AppResult<Self::Tx> {
        Ok(self.pool.begin().await?)
    }

    async fn commit(&self, tx: Self::Tx) -> AppResult<()> {
        tx.commit().await?;
        Ok(())
    }

    async fn rollback(&self, tx: Self::Tx) -> AppResult<()> {
        tx.rollback().await?;
        Ok(())
    }
}
