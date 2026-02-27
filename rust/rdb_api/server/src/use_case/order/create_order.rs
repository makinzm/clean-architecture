use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::order::Order;
use crate::domain::repository::order_repository::OrderRepository;
use crate::domain::repository::user_repository::UserRepository;
use crate::error::{AppError, AppResult};
use crate::use_case::transaction_manager::TransactionManager;

pub struct CreateOrderInput {
    pub user_id: i64,
    pub item_name: String,
    pub quantity: i32,
}

#[async_trait]
pub trait CreateOrderUseCase: Send + Sync {
    async fn execute(&self, input: CreateOrderInput) -> AppResult<Order>;
}

pub struct CreateOrderUseCaseImpl<TM: TransactionManager> {
    tx_manager: Arc<TM>,
    user_repo: Arc<dyn UserRepository<TM::Tx>>,
    order_repo: Arc<dyn OrderRepository<TM::Tx>>,
}

impl<TM: TransactionManager + 'static> CreateOrderUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    pub fn new(
        tx_manager: Arc<TM>,
        user_repo: Arc<dyn UserRepository<TM::Tx>>,
        order_repo: Arc<dyn OrderRepository<TM::Tx>>,
    ) -> Self {
        Self { tx_manager, user_repo, order_repo }
    }
}

#[async_trait]
impl<TM: TransactionManager + 'static> CreateOrderUseCase for CreateOrderUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    async fn execute(&self, input: CreateOrderInput) -> AppResult<Order> {
        let mut tx = self.tx_manager.begin().await?;

        // Verify user exists
        match self.user_repo.find_by_id(&mut tx, input.user_id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = self.tx_manager.rollback(tx).await;
                return Err(AppError::NotFound);
            }
            Err(e) => {
                let _ = self.tx_manager.rollback(tx).await;
                return Err(e);
            }
        }

        // Create order
        let order = match self
            .order_repo
            .create(&mut tx, input.user_id, &input.item_name, input.quantity)
            .await
        {
            Ok(o) => o,
            Err(e) => {
                let _ = self.tx_manager.rollback(tx).await;
                return Err(e);
            }
        };

        // Increment user's order_count in the same transaction
        if let Err(e) = self.user_repo.increment_order_count(&mut tx, input.user_id).await {
            let _ = self.tx_manager.rollback(tx).await;
            return Err(e);
        }

        self.tx_manager.commit(tx).await?;
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use async_trait::async_trait;
    use chrono::NaiveDateTime;

    use super::*;
    use crate::domain::entity::order::Order;
    use crate::domain::entity::user::User;
    use crate::error::{AppError, AppResult};
    use crate::use_case::transaction_manager::TransactionManager;

    // --- test doubles ---

    struct TestTx;

    struct TestTxManager;

    #[async_trait]
    impl TransactionManager for TestTxManager {
        type Tx = TestTx;
        async fn begin(&self) -> AppResult<TestTx> { Ok(TestTx) }
        async fn commit(&self, _: TestTx) -> AppResult<()> { Ok(()) }
        async fn rollback(&self, _: TestTx) -> AppResult<()> { Ok(()) }
    }

    fn make_user(id: i64) -> User {
        User {
            id,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            order_count: 0,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            updated_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        }
    }

    fn make_order(id: i64, user_id: i64) -> Order {
        Order {
            id,
            user_id,
            item_name: "Widget".to_string(),
            quantity: 2,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        }
    }

    /// Tracks how many times increment_order_count was called.
    struct TrackingUserRepo {
        user: Option<User>,
        increment_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl UserRepository<TestTx> for TrackingUserRepo {
        async fn find_by_id(&self, _: &mut TestTx, _: i64) -> AppResult<Option<User>> {
            Ok(self.user.clone())
        }
        async fn find_all(&self, _: &mut TestTx) -> AppResult<Vec<User>> { Ok(vec![]) }
        async fn create(&self, _: &mut TestTx, _: &str, _: &str) -> AppResult<User> {
            unimplemented!()
        }
        async fn increment_order_count(&self, _: &mut TestTx, _: i64) -> AppResult<()> {
            self.increment_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct StubOrderRepo(Order);

    #[async_trait]
    impl OrderRepository<TestTx> for StubOrderRepo {
        async fn create(
            &self,
            _: &mut TestTx,
            _: i64,
            _: &str,
            _: i32,
        ) -> AppResult<Order> {
            Ok(self.0.clone())
        }
    }

    // --- tests ---

    #[tokio::test]
    async fn test_create_order_increments_user_order_count_in_same_transaction() {
        let increment_calls = Arc::new(AtomicUsize::new(0));
        let user_repo = Arc::new(TrackingUserRepo {
            user: Some(make_user(1)),
            increment_calls: increment_calls.clone(),
        });
        let order_repo = Arc::new(StubOrderRepo(make_order(1, 1)));

        let use_case = CreateOrderUseCaseImpl::new(
            Arc::new(TestTxManager),
            user_repo,
            order_repo,
        );

        let result = use_case
            .execute(CreateOrderInput {
                user_id: 1,
                item_name: "Widget".to_string(),
                quantity: 2,
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(
            increment_calls.load(Ordering::SeqCst),
            1,
            "increment_order_count must be called exactly once within the same transaction"
        );
    }

    #[tokio::test]
    async fn test_create_order_returns_not_found_when_user_missing() {
        let user_repo = Arc::new(TrackingUserRepo {
            user: None,
            increment_calls: Arc::new(AtomicUsize::new(0)),
        });
        let order_repo = Arc::new(StubOrderRepo(make_order(1, 999)));

        let use_case = CreateOrderUseCaseImpl::new(
            Arc::new(TestTxManager),
            user_repo,
            order_repo,
        );

        let result = use_case
            .execute(CreateOrderInput {
                user_id: 999,
                item_name: "Widget".to_string(),
                quantity: 1,
            })
            .await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
