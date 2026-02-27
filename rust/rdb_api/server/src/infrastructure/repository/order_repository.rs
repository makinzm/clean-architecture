use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};

use crate::domain::entity::order::Order;
use crate::domain::repository::order_repository::OrderRepository;
use crate::error::AppResult;

use crate::infrastructure::entity::order::{Model as OrderModel, ActiveModel as OrderActiveModel};

pub struct SeaOrmOrderRepository;

#[async_trait]
impl OrderRepository<DatabaseTransaction> for SeaOrmOrderRepository {
    async fn create(
        &self,
        tx: &mut DatabaseTransaction,
        user_id: i64,
        item_name: &str,
        quantity: i32,
    ) -> AppResult<Order> {
        let active_model = OrderActiveModel {
            user_id: Set(user_id),
            item_name: Set(item_name.to_owned()),
            quantity: Set(quantity),
            ..Default::default()
        };

        let model = active_model.insert(tx).await?;

        Ok(map_to_domain(model))
    }
}

fn map_to_domain(model: OrderModel) -> Order {
    Order {
        id: model.id,
        user_id: model.user_id,
        item_name: model.item_name,
        quantity: model.quantity,
        created_at: model.created_at,
    }
}
