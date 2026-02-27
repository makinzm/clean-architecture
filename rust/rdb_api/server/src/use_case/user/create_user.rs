use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::user::User;
use crate::domain::repository::user_repository::UserRepository;
use crate::error::AppResult;
use crate::use_case::transaction_manager::TransactionManager;

pub struct CreateUserInput {
    pub name: String,
    pub email: String,
}

#[async_trait]
pub trait CreateUserUseCase: Send + Sync {
    async fn execute(&self, input: CreateUserInput) -> AppResult<User>;
}

pub struct CreateUserUseCaseImpl<TM: TransactionManager> {
    tx_manager: Arc<TM>,
    user_repo: Arc<dyn UserRepository<TM::Tx>>,
}

impl<TM: TransactionManager + 'static> CreateUserUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    pub fn new(tx_manager: Arc<TM>, user_repo: Arc<dyn UserRepository<TM::Tx>>) -> Self {
        Self { tx_manager, user_repo }
    }
}

#[async_trait]
impl<TM: TransactionManager + 'static> CreateUserUseCase for CreateUserUseCaseImpl<TM>
where
    TM::Tx: 'static,
{
    async fn execute(&self, input: CreateUserInput) -> AppResult<User> {
        let mut tx = self.tx_manager.begin().await?;
        let result = self.user_repo.create(&mut tx, &input.name, &input.email).await;
        match result {
            Ok(user) => {
                self.tx_manager.commit(tx).await?;
                Ok(user)
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

    fn created_user(name: &str, email: &str) -> User {
        User {
            id: 1,
            name: name.to_string(),
            email: email.to_string(),
            order_count: 0,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            updated_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        }
    }

    struct SuccessRepo;

    #[async_trait]
    impl UserRepository<TestTx> for SuccessRepo {
        async fn find_by_id(&self, _: &mut TestTx, _: i64) -> AppResult<Option<User>> {
            unimplemented!()
        }
        async fn find_all(&self, _: &mut TestTx) -> AppResult<Vec<User>> { unimplemented!() }
        async fn create(&self, _: &mut TestTx, name: &str, email: &str) -> AppResult<User> {
            Ok(created_user(name, email))
        }
        async fn increment_order_count(&self, _: &mut TestTx, _: i64) -> AppResult<()> {
            Ok(())
        }
    }

    struct DuplicateEmailRepo;

    #[async_trait]
    impl UserRepository<TestTx> for DuplicateEmailRepo {
        async fn find_by_id(&self, _: &mut TestTx, _: i64) -> AppResult<Option<User>> {
            unimplemented!()
        }
        async fn find_all(&self, _: &mut TestTx) -> AppResult<Vec<User>> { unimplemented!() }
        async fn create(&self, _: &mut TestTx, _: &str, _: &str) -> AppResult<User> {
            Err(AppError::Conflict("duplicate entry".to_string()))
        }
        async fn increment_order_count(&self, _: &mut TestTx, _: i64) -> AppResult<()> {
            Ok(())
        }
    }

    // --- tests ---

    #[tokio::test]
    async fn test_create_user_returns_created_user() {
        let use_case =
            CreateUserUseCaseImpl::new(Arc::new(TestTxManager), Arc::new(SuccessRepo));

        let result = use_case
            .execute(CreateUserInput {
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Bob");
        assert_eq!(user.email, "bob@example.com");
    }

    #[tokio::test]
    async fn test_create_user_propagates_conflict_on_duplicate_email() {
        let use_case =
            CreateUserUseCaseImpl::new(Arc::new(TestTxManager), Arc::new(DuplicateEmailRepo));

        let result = use_case
            .execute(CreateUserInput {
                name: "Bob".to_string(),
                email: "existing@example.com".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AppError::Conflict(_))));
    }
}
