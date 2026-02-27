mod domain;
mod error;
mod infrastructure;
mod presentation;
mod use_case;

use std::sync::Arc;

use dotenvy::dotenv;
use sea_orm::{Database, ConnectionTrait};
use tracing_subscriber::EnvFilter;

use crate::infrastructure::database::create_pool;
use crate::infrastructure::repository::order_repository::SeaOrmOrderRepository;
use crate::infrastructure::repository::user_repository::SeaOrmUserRepository;
use crate::infrastructure::transaction_manager::SeaOrmTransactionManager;
use crate::presentation::router::create_router;
use crate::presentation::state::AppState;
use crate::use_case::order::create_order::CreateOrderUseCaseImpl;
use crate::use_case::user::create_user::CreateUserUseCaseImpl;
use crate::use_case::user::get_user::GetUserUseCaseImpl;
use crate::use_case::user::list_users::ListUsersUseCaseImpl;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let pool = create_pool().await.expect("Failed to create database connection pool");

    let tx_manager = Arc::new(SeaOrmTransactionManager::new(pool));
    let user_repo = Arc::new(SeaOrmUserRepository);
    let order_repo = Arc::new(SeaOrmOrderRepository);

    let state = AppState {
        create_user: Arc::new(CreateUserUseCaseImpl::new(
            tx_manager.clone(),
            user_repo.clone(),
        )),
        get_user: Arc::new(GetUserUseCaseImpl::new(
            tx_manager.clone(),
            user_repo.clone(),
        )),
        list_users: Arc::new(ListUsersUseCaseImpl::new(
            tx_manager.clone(),
            user_repo.clone(),
        )),
        create_order: Arc::new(CreateOrderUseCaseImpl::new(
            tx_manager.clone(),
            user_repo.clone(),
            order_repo.clone(),
        )),
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind port 8080");

    tracing::info!("Server listening on http://127.0.0.1:8080");
    tracing::info!("Swagger UI: http://localhost:8080/swagger-ui");

    axum::serve(listener, app).await.expect("Server error");
}
