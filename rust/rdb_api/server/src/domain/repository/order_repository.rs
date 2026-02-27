use async_trait::async_trait;

use crate::domain::entity::order::Order;
use crate::error::AppResult;

#[async_trait]
pub trait OrderRepository<Tx>: Send + Sync {
    async fn create(
        &self,
        tx: &mut Tx,
        user_id: i64,
        item_name: &str,
        quantity: i32,
    ) -> AppResult<Order>;
}
