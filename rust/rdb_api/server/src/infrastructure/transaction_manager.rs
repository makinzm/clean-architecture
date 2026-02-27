use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};

use crate::error::AppResult;
use crate::use_case::transaction_manager::TransactionManager;

pub struct SeaOrmTransactionManager {
    db: DatabaseConnection,
}

impl SeaOrmTransactionManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TransactionManager for SeaOrmTransactionManager {
    type Tx = DatabaseTransaction;

    async fn begin(&self) -> AppResult<Self::Tx> {
        Ok(self.db.begin().await?)
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
