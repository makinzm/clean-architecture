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
