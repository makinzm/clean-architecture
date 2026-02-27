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
