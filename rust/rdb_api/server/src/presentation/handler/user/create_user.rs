use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::domain::entity::user::User;
use crate::error::AppError;
use crate::presentation::state::AppState;
use crate::use_case::user::create_user::CreateUserInput;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// User's display name
    pub name: String,
    /// User's email address (must be unique)
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created", body = User),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<User>, AppError> {
    let user = state
        .create_user
        .execute(CreateUserInput { name: body.name, email: body.email })
        .await?;
    Ok(Json(user))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_case::user::create_user::CreateUserUseCase;
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use std::sync::Arc;
    use crate::use_case::order::create_order::CreateOrderUseCase;
    use crate::use_case::user::get_user::GetUserUseCase;
    use crate::use_case::user::list_users::ListUsersUseCase;

    struct MockCreateUserUseCaseSuccess;
    #[async_trait]
    impl CreateUserUseCase for MockCreateUserUseCaseSuccess {
        async fn execute(&self, input: CreateUserInput) -> crate::error::AppResult<User> {
            Ok(User {
                id: 1,
                name: input.name,
                email: input.email,
                order_count: 0,
                created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
                updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            })
        }
    }

    struct MockCreateUserUseCaseConflict;
    #[async_trait]
    impl CreateUserUseCase for MockCreateUserUseCaseConflict {
        async fn execute(&self, _: CreateUserInput) -> crate::error::AppResult<User> {
            Err(AppError::Conflict("duplicate entry".to_string()))
        }
    }

    struct DummyGetUserUseCase;
    #[async_trait]
    impl GetUserUseCase for DummyGetUserUseCase {
        async fn execute(&self, _: i64) -> crate::error::AppResult<User> { unimplemented!() }
    }

    struct DummyListUsersUseCase;
    #[async_trait]
    impl ListUsersUseCase for DummyListUsersUseCase {
        async fn execute(&self) -> crate::error::AppResult<Vec<User>> { unimplemented!() }
    }

    struct DummyCreateOrderUseCase;
    #[async_trait]
    impl CreateOrderUseCase for DummyCreateOrderUseCase {
        async fn execute(&self, _: crate::use_case::order::create_order::CreateOrderInput) -> crate::error::AppResult<crate::domain::entity::order::Order> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let state = AppState {
            create_user: Arc::new(MockCreateUserUseCaseSuccess),
            get_user: Arc::new(DummyGetUserUseCase),
            list_users: Arc::new(DummyListUsersUseCase),
            create_order: Arc::new(DummyCreateOrderUseCase),
        };
        let body = CreateUserRequest { name: "Alice".to_string(), email: "alice@example.com".to_string() };
        let result = create_user(State(state), Json(body)).await;
        assert!(result.is_ok());
        let Json(user) = result.unwrap();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, "alice@example.com");
    }

    #[tokio::test]
    async fn test_create_user_conflict() {
        let state = AppState {
            create_user: Arc::new(MockCreateUserUseCaseConflict),
            get_user: Arc::new(DummyGetUserUseCase),
            list_users: Arc::new(DummyListUsersUseCase),
            create_order: Arc::new(DummyCreateOrderUseCase),
        };
        let body = CreateUserRequest { name: "Alice".to_string(), email: "alice@example.com".to_string() };
        let result = create_user(State(state), Json(body)).await;
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }
}
