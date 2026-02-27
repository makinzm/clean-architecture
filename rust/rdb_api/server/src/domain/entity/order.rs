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
