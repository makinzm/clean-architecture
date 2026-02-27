use async_trait::async_trait;

use crate::domain::entity::user::User;
use crate::error::AppResult;

#[async_trait]
pub trait UserRepository<Tx>: Send + Sync {
    async fn find_by_id(&self, tx: &mut Tx, id: i64) -> AppResult<Option<User>>;
    async fn find_all(&self, tx: &mut Tx) -> AppResult<Vec<User>>;
    async fn create(&self, tx: &mut Tx, name: &str, email: &str) -> AppResult<User>;
    async fn increment_order_count(&self, tx: &mut Tx, id: i64) -> AppResult<()>;
}
