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
