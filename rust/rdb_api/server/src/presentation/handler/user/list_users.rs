use axum::{extract::State, Json};

use crate::domain::entity::user::User;
use crate::error::AppError;
use crate::presentation::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = state.list_users.execute().await?;
    Ok(Json(users))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_case::user::list_users::ListUsersUseCase;
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use std::sync::Arc;
    use crate::use_case::order::create_order::CreateOrderUseCase;
    use crate::use_case::user::create_user::CreateUserUseCase;
    use crate::use_case::user::get_user::GetUserUseCase;

    struct MockListUsersUseCase;

    #[async_trait]
    impl ListUsersUseCase for MockListUsersUseCase {
        async fn execute(&self) -> crate::error::AppResult<Vec<User>> {
            Ok(vec![User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                order_count: 0,
                created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
                updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            }])
        }
    }

    struct DummyCreateUserUseCase;
    #[async_trait]
    impl CreateUserUseCase for DummyCreateUserUseCase {
        async fn execute(&self, _: crate::use_case::user::create_user::CreateUserInput) -> crate::error::AppResult<User> { unimplemented!() }
    }

    struct DummyGetUserUseCase;
    #[async_trait]
    impl GetUserUseCase for DummyGetUserUseCase {
        async fn execute(&self, _: i64) -> crate::error::AppResult<User> { unimplemented!() }
    }

    struct DummyCreateOrderUseCase;
    #[async_trait]
    impl CreateOrderUseCase for DummyCreateOrderUseCase {
        async fn execute(&self, _: crate::use_case::order::create_order::CreateOrderInput) -> crate::error::AppResult<crate::domain::entity::order::Order> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_list_users_success() {
        let state = AppState {
            create_user: Arc::new(DummyCreateUserUseCase),
            get_user: Arc::new(DummyGetUserUseCase),
            list_users: Arc::new(MockListUsersUseCase),
            create_order: Arc::new(DummyCreateOrderUseCase),
        };
        let result = list_users(State(state)).await;
        assert!(result.is_ok());
        let Json(users) = result.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].name, "Alice");
    }
}
