use axum::{
    extract::{Path, State},
    Json,
};

use crate::domain::entity::user::User;
use crate::error::AppError;
use crate::presentation::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = state.get_user.execute(id).await?;
    Ok(Json(user))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_case::user::get_user::GetUserUseCase;
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use std::sync::Arc;
    use crate::use_case::order::create_order::CreateOrderUseCase;
    use crate::use_case::user::create_user::CreateUserUseCase;
    use crate::use_case::user::list_users::ListUsersUseCase;

    struct MockGetUserUseCaseFound;
    #[async_trait]
    impl GetUserUseCase for MockGetUserUseCaseFound {
        async fn execute(&self, id: i64) -> crate::error::AppResult<User> {
            Ok(User {
                id,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                order_count: 0,
                created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
                updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            })
        }
    }

    struct MockGetUserUseCaseNotFound;
    #[async_trait]
    impl GetUserUseCase for MockGetUserUseCaseNotFound {
        async fn execute(&self, _: i64) -> crate::error::AppResult<User> {
            Err(AppError::NotFound)
        }
    }

    struct DummyCreateUserUseCase;
    #[async_trait]
    impl CreateUserUseCase for DummyCreateUserUseCase {
        async fn execute(&self, _: crate::use_case::user::create_user::CreateUserInput) -> crate::error::AppResult<User> { unimplemented!() }
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
    async fn test_get_user_found() {
        let state = AppState {
            create_user: Arc::new(DummyCreateUserUseCase),
            get_user: Arc::new(MockGetUserUseCaseFound),
            list_users: Arc::new(DummyListUsersUseCase),
            create_order: Arc::new(DummyCreateOrderUseCase),
        };
        let result = get_user(State(state), Path(1)).await;
        assert!(result.is_ok());
        let Json(user) = result.unwrap();
        assert_eq!(user.name, "Bob");
        assert_eq!(user.id, 1);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let state = AppState {
            create_user: Arc::new(DummyCreateUserUseCase),
            get_user: Arc::new(MockGetUserUseCaseNotFound),
            list_users: Arc::new(DummyListUsersUseCase),
            create_order: Arc::new(DummyCreateOrderUseCase),
        };
        let result = get_user(State(state), Path(999)).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
