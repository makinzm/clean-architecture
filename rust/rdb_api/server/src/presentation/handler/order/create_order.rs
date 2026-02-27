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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_case::order::create_order::CreateOrderUseCase;
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use std::sync::Arc;
    use crate::use_case::user::create_user::CreateUserUseCase;
    use crate::use_case::user::get_user::GetUserUseCase;
    use crate::use_case::user::list_users::ListUsersUseCase;

    struct MockCreateOrderUseCaseSuccess;
    #[async_trait]
    impl CreateOrderUseCase for MockCreateOrderUseCaseSuccess {
        async fn execute(&self, input: CreateOrderInput) -> crate::error::AppResult<Order> {
            Ok(Order {
                id: 1,
                user_id: input.user_id,
                item_name: input.item_name,
                quantity: input.quantity,
                created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            })
        }
    }

    struct MockCreateOrderUseCaseUserNotFound;
    #[async_trait]
    impl CreateOrderUseCase for MockCreateOrderUseCaseUserNotFound {
        async fn execute(&self, _: CreateOrderInput) -> crate::error::AppResult<Order> {
            Err(AppError::NotFound)
        }
    }

    struct DummyGetUserUseCase;
    #[async_trait]
    impl GetUserUseCase for DummyGetUserUseCase {
        async fn execute(&self, _: i64) -> crate::error::AppResult<crate::domain::entity::user::User> { unimplemented!() }
    }

    struct DummyListUsersUseCase;
    #[async_trait]
    impl ListUsersUseCase for DummyListUsersUseCase {
        async fn execute(&self) -> crate::error::AppResult<Vec<crate::domain::entity::user::User>> { unimplemented!() }
    }

    struct DummyCreateUserUseCase;
    #[async_trait]
    impl CreateUserUseCase for DummyCreateUserUseCase {
        async fn execute(&self, _: crate::use_case::user::create_user::CreateUserInput) -> crate::error::AppResult<crate::domain::entity::user::User> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_create_order_success() {
        let state = AppState {
            create_user: Arc::new(DummyCreateUserUseCase),
            get_user: Arc::new(DummyGetUserUseCase),
            list_users: Arc::new(DummyListUsersUseCase),
            create_order: Arc::new(MockCreateOrderUseCaseSuccess),
        };
        let body = CreateOrderRequest { user_id: 1, item_name: "Apple".to_string(), quantity: 5 };
        let result = create_order(State(state), Json(body)).await;
        assert!(result.is_ok());
        let Json(order) = result.unwrap();
        assert_eq!(order.item_name, "Apple");
        assert_eq!(order.quantity, 5);
        assert_eq!(order.user_id, 1);
    }

    #[tokio::test]
    async fn test_create_order_user_not_found() {
        let state = AppState {
            create_user: Arc::new(DummyCreateUserUseCase),
            get_user: Arc::new(DummyGetUserUseCase),
            list_users: Arc::new(DummyListUsersUseCase),
            create_order: Arc::new(MockCreateOrderUseCaseUserNotFound),
        };
        let body = CreateOrderRequest { user_id: 999, item_name: "Apple".to_string(), quantity: 5 };
        let result = create_order(State(state), Json(body)).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
