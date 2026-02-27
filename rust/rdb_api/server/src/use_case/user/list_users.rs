use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::user::User;
use crate::domain::repository::user_repository::UserRepository;
use crate::error::AppResult;
use crate::use_case::transaction_manager::TransactionManager;

#[async_trait]
pub trait ListUsersUseCase: Send + Sync {
    async fn execute(&self) -> AppResult<Vec<User>>;
}

pub struct ListUsersUseCaseImpl<TM: TransactionManager> {
    tx_manager: Arc<TM>,
    user_repo: Arc<dyn UserRepository<TM::Tx>>,
}

impl<TM: TransactionManager + 'static> ListUsersUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    pub fn new(tx_manager: Arc<TM>, user_repo: Arc<dyn UserRepository<TM::Tx>>) -> Self {
        Self { tx_manager, user_repo }
    }
}

#[async_trait]
impl<TM: TransactionManager + 'static> ListUsersUseCase for ListUsersUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    async fn execute(&self) -> AppResult<Vec<User>> {
        let mut tx = self.tx_manager.begin().await?;
        let result = self.user_repo.find_all(&mut tx).await;
        match result {
            Ok(users) => {
                self.tx_manager.commit(tx).await?;
                Ok(users)
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
    use crate::error::AppResult;
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

    fn make_user(id: i64, name: &str) -> User {
        User {
            id,
            name: name.to_string(),
            email: format!("{name}@example.com"),
            order_count: 0,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            updated_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        }
    }

    struct StubRepo(Vec<User>);

    #[async_trait]
    impl UserRepository<TestTx> for StubRepo {
        async fn find_by_id(&self, _: &mut TestTx, _: i64) -> AppResult<Option<User>> {
            unimplemented!()
        }
        async fn find_all(&self, _: &mut TestTx) -> AppResult<Vec<User>> {
            Ok(self.0.clone())
        }
        async fn create(&self, _: &mut TestTx, _: &str, _: &str) -> AppResult<User> {
            unimplemented!()
        }
        async fn increment_order_count(&self, _: &mut TestTx, _: i64) -> AppResult<()> {
            Ok(())
        }
    }

    // --- tests ---

    #[tokio::test]
    async fn test_list_users_returns_all_users() {
        let users = vec![make_user(1, "Alice"), make_user(2, "Bob")];
        let use_case =
            ListUsersUseCaseImpl::new(Arc::new(TestTxManager), Arc::new(StubRepo(users)));

        let result = use_case.execute().await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Alice");
        assert_eq!(list[1].name, "Bob");
    }

    #[tokio::test]
    async fn test_list_users_returns_empty_when_no_users() {
        let use_case =
            ListUsersUseCaseImpl::new(Arc::new(TestTxManager), Arc::new(StubRepo(vec![])));

        let result = use_case.execute().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
