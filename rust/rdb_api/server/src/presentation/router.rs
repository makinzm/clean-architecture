use axum::{
    Json,
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

use crate::presentation::handler::order;
use crate::presentation::handler::user;
use crate::presentation::openapi::ApiDoc;
use crate::presentation::state::AppState;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn swagger_ui() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html>
<head>
  <title>API Documentation</title>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.onload = function() {
      SwaggerUIBundle({ url: "/api-docs/openapi.json", dom_id: '#swagger-ui' });
    }
  </script>
</body>
</html>"#,
    )
}

pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        .route(
            "/api/v1/users",
            get(user::list_users::list_users).post(user::create_user::create_user),
        )
        .route("/api/v1/users/{id}", get(user::get_user::get_user))
        .route("/api/v1/orders", post(order::create_order::create_order))
        .with_state(state);

    Router::new()
        .merge(api)
        .route("/swagger-ui", get(swagger_ui))
        .route("/api-docs/openapi.json", get(openapi_json))
        .layer(TraceLayer::new_for_http())
}
