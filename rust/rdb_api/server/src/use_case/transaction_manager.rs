use async_trait::async_trait;

use crate::error::AppResult;

#[async_trait]
pub trait TransactionManager: Send + Sync {
    type Tx: Send;

    async fn begin(&self) -> AppResult<Self::Tx>;
    async fn commit(&self, tx: Self::Tx) -> AppResult<()>;
    async fn rollback(&self, tx: Self::Tx) -> AppResult<()>;
}
