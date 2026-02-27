use async_trait::async_trait;
use sqlx::{MySql, Transaction};

use crate::domain::entity::order::Order;
use crate::domain::repository::order_repository::OrderRepository;
use crate::error::AppResult;

pub struct SqlxOrderRepository;

#[async_trait]
impl OrderRepository<Transaction<'static, MySql>> for SqlxOrderRepository {
    async fn create(
        &self,
        tx: &mut Transaction<'static, MySql>,
        user_id: i64,
        item_name: &str,
        quantity: i32,
    ) -> AppResult<Order> {
        sqlx::query("INSERT INTO orders (user_id, item_name, quantity) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(item_name)
            .bind(quantity)
            .execute(&mut **tx)
            .await?;

        let order = sqlx::query_as::<_, Order>(
            "SELECT id, user_id, item_name, quantity, created_at FROM orders WHERE id = LAST_INSERT_ID()",
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(order)
    }
}
