use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::user::User;
use crate::domain::repository::user_repository::UserRepository;
use crate::error::{AppError, AppResult};
use crate::use_case::transaction_manager::TransactionManager;

#[async_trait]
pub trait GetUserUseCase: Send + Sync {
    async fn execute(&self, id: i64) -> AppResult<User>;
}

pub struct GetUserUseCaseImpl<TM: TransactionManager> {
    tx_manager: Arc<TM>,
    user_repo: Arc<dyn UserRepository<TM::Tx>>,
}

impl<TM: TransactionManager + 'static> GetUserUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    pub fn new(tx_manager: Arc<TM>, user_repo: Arc<dyn UserRepository<TM::Tx>>) -> Self {
        Self { tx_manager, user_repo }
    }
}

#[async_trait]
impl<TM: TransactionManager + 'static> GetUserUseCase for GetUserUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    async fn execute(&self, id: i64) -> AppResult<User> {
        let mut tx = self.tx_manager.begin().await?;
        let result = self.user_repo.find_by_id(&mut tx, id).await;
        match result {
            Ok(Some(user)) => {
                self.tx_manager.commit(tx).await?;
                Ok(user)
            }
            Ok(None) => {
                let _ = self.tx_manager.rollback(tx).await;
                Err(AppError::NotFound)
            }
            Err(e) => {
                let _ = self.tx_manager.rollback(tx).await;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::NaiveDateTime;

    use super::*;
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

    fn sample_user() -> User {
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            order_count: 0,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            updated_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        }
    }

    struct FoundRepo(User);

    #[async_trait]
    impl UserRepository<TestTx> for FoundRepo {
        async fn find_by_id(&self, _: &mut TestTx, _: i64) -> AppResult<Option<User>> {
            Ok(Some(self.0.clone()))
        }
        async fn find_all(&self, _: &mut TestTx) -> AppResult<Vec<User>> { Ok(vec![]) }
        async fn create(&self, _: &mut TestTx, _: &str, _: &str) -> AppResult<User> {
            unimplemented!()
        }
        async fn increment_order_count(&self, _: &mut TestTx, _: i64) -> AppResult<()> {
            Ok(())
        }
    }

    struct NotFoundRepo;

    #[async_trait]
    impl UserRepository<TestTx> for NotFoundRepo {
        async fn find_by_id(&self, _: &mut TestTx, _: i64) -> AppResult<Option<User>> {
            Ok(None)
        }
        async fn find_all(&self, _: &mut TestTx) -> AppResult<Vec<User>> { Ok(vec![]) }
        async fn create(&self, _: &mut TestTx, _: &str, _: &str) -> AppResult<User> {
            unimplemented!()
        }
        async fn increment_order_count(&self, _: &mut TestTx, _: i64) -> AppResult<()> {
            Ok(())
        }
    }

    // --- tests ---

    #[tokio::test]
    async fn test_get_user_returns_user_when_found() {
        let use_case = GetUserUseCaseImpl::new(
            Arc::new(TestTxManager),
            Arc::new(FoundRepo(sample_user())),
        );

        let result = use_case.execute(1).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Alice");
    }

    #[tokio::test]
    async fn test_get_user_returns_not_found_when_user_missing() {
        let use_case =
            GetUserUseCaseImpl::new(Arc::new(TestTxManager), Arc::new(NotFoundRepo));

        let result = use_case.execute(999).await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
