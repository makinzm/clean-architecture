# rdb_api — Implementation Plan

## Phase 1 — Users CRUD
- [x] docker-compose.yml (TiDB)
- [x] Cargo.toml (axum 0.8, sqlx 0.8, utoipa 5, etc.)
- [x] .env
- [x] migrations/0001_create_users.sql
- [x] error.rs (AppError, AppResult)
- [x] domain/entity/user.rs
- [x] domain/repository/user_repository.rs (trait)
- [x] use_case/transaction_manager.rs (trait — key constraint)
- [x] use_case/user/list_users.rs + tests
- [x] use_case/user/get_user.rs + tests
- [x] use_case/user/create_user.rs + tests
- [x] infrastructure/database.rs
- [x] infrastructure/transaction_manager.rs (SqlxTransactionManager)
- [x] infrastructure/repository/user_repository.rs (SqlxUserRepository)
- [x] presentation/state.rs (AppState with Arc<dyn UseCase>)
- [x] presentation/router.rs
- [x] presentation/openapi.rs
- [x] presentation/handler/user/{list,get,create}_user.rs
- [x] main.rs

## Phase 2 — Orders + cross-repo transaction
- [x] migrations/0002_create_orders.sql
- [x] domain/entity/order.rs
- [x] domain/repository/order_repository.rs (trait)
- [x] use_case/order/create_order.rs + tests
- [x] infrastructure/repository/order_repository.rs
- [x] presentation/handler/order/create_order.rs

## Test results
```
running 8 tests
test use_case::user::create_user::tests::test_create_user_returns_created_user ... ok
test use_case::user::create_user::tests::test_create_user_propagates_conflict_on_duplicate_email ... ok
test use_case::order::create_order::tests::test_create_order_returns_not_found_when_user_missing ... ok
test use_case::order::create_order::tests::test_create_order_increments_user_order_count_in_same_transaction ... ok
test use_case::user::get_user::tests::test_get_user_returns_not_found_when_user_missing ... ok
test use_case::user::get_user::tests::test_get_user_returns_user_when_found ... ok
test use_case::user::list_users::tests::test_list_users_returns_all_users ... ok
test use_case::user::list_users::tests::test_list_users_returns_empty_when_no_users ... ok
```

## 注意点
- utoipa-swagger-ui 8.1.0 は axum 0.7 に依存しており axum 0.8 と非互換
  → Swagger UI は CDN HTML で代替 (/swagger-ui + /api-docs/openapi.json)
