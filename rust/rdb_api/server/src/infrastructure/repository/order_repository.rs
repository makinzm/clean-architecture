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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};

    #[tokio::test]
    async fn test_create() {
        let now = Utc::now().naive_utc();
        let mock_model = OrderModel {
            id: 1,
            user_id: 10,
            item_name: "Widget".to_string(),
            quantity: 5,
            created_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([vec![mock_model]])
            .append_exec_results([MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection();

        let mut tx = db.begin().await.unwrap();
        let repo = SeaOrmOrderRepository;
        let result = repo.create(&mut tx, 10, "Widget", 5).await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.user_id, 10);
        assert_eq!(result.item_name, "Widget");
        assert_eq!(result.quantity, 5);
    }
}
