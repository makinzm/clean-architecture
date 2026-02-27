use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub item_name: String,
    pub quantity: i32,
    pub created_at: NaiveDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let now = chrono::Utc::now().naive_utc();
        let order = Order {
            id: 1,
            user_id: 10,
            item_name: "Widget".to_string(),
            quantity: 5,
            created_at: now,
        };

        assert_eq!(order.id, 1);
        assert_eq!(order.user_id, 10);
        assert_eq!(order.item_name, "Widget");
        assert_eq!(order.quantity, 5);
    }
}
