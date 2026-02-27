use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::domain::entity::order::Order;
use crate::error::AppError;
use crate::presentation::state::AppState;
use crate::use_case::order::create_order::CreateOrderInput;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrderRequest {
    /// User ID to create the order for
    pub user_id: i64,
    /// Name of the item being ordered
    pub item_name: String,
    /// Quantity of items
    pub quantity: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/orders",
    tag = "orders",
    request_body = CreateOrderRequest,
    responses(
        (status = 200, description = "Order created", body = Order),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn create_order(
    State(state): State<AppState>,
    Json(body): Json<CreateOrderRequest>,
) -> Result<Json<Order>, AppError> {
    let order = state
        .create_order
        .execute(CreateOrderInput {
            user_id: body.user_id,
            item_name: body.item_name,
            quantity: body.quantity,
        })
        .await?;
    Ok(Json(order))
}
