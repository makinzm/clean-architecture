mod domain;
mod error;
mod infrastructure;
mod presentation;
mod use_case;

use std::sync::Arc;

use dotenvy::dotenv;
use sqlx::migrate::MigrateDatabase;
use sqlx::MySql;
use tracing_subscriber::EnvFilter;

use crate::infrastructure::database::create_pool;
use crate::infrastructure::repository::order_repository::SqlxOrderRepository;
use crate::infrastructure::repository::user_repository::SqlxUserRepository;
use crate::infrastructure::transaction_manager::SqlxTransactionManager;
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

    // --reset: drop + recreate DB (used by `make e2e` for a clean slate), then exit
    if std::env::args().any(|a| a == "--reset") {
        MySql::drop_database(&database_url).await.unwrap_or(());
        MySql::create_database(&database_url).await.expect("Failed to create database");
        println!("Database reset.");
        return;
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Ensure DB exists on first run
    if !MySql::database_exists(&database_url).await.unwrap_or(false) {
        MySql::create_database(&database_url).await.expect("Failed to create database");
    }

    let pool = create_pool().await.expect("Failed to create database pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let tx_manager = Arc::new(SqlxTransactionManager::new(pool));
    let user_repo = Arc::new(SqlxUserRepository);
    let order_repo = Arc::new(SqlxOrderRepository);

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");

    tracing::info!("Server listening on http://0.0.0.0:8080");
    tracing::info!("Swagger UI: http://localhost:8080/swagger-ui");

    axum::serve(listener, app).await.expect("Server error");
}
