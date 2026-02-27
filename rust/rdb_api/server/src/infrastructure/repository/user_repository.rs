use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, Set};

use crate::domain::entity::user::User;
use crate::domain::repository::user_repository::UserRepository;
use crate::error::AppResult;

// Import our new SeaORM Entity
use crate::infrastructure::entity::user::{Entity as UserEntity, Model as UserModel, ActiveModel as UserActiveModel};

pub struct SeaOrmUserRepository;

#[async_trait]
impl UserRepository<DatabaseTransaction> for SeaOrmUserRepository {
    async fn find_by_id(
        &self,
        tx: &mut DatabaseTransaction,
        id: i64,
    ) -> AppResult<Option<User>> {
        let model = UserEntity::find_by_id(id).one(tx).await?;
        Ok(model.map(map_to_domain))
    }

    async fn find_all(&self, tx: &mut DatabaseTransaction) -> AppResult<Vec<User>> {
        let models = UserEntity::find().all(tx).await?;
        Ok(models.into_iter().map(map_to_domain).collect())
    }

    async fn create(
        &self,
        tx: &mut DatabaseTransaction,
        name: &str,
        email: &str,
    ) -> AppResult<User> {
        let active_model = UserActiveModel {
            name: Set(name.to_owned()),
            email: Set(email.to_owned()),
            ..Default::default()
        };
        let model = active_model.insert(tx).await?;
        Ok(map_to_domain(model))
    }

    async fn increment_order_count(
        &self,
        tx: &mut DatabaseTransaction,
        id: i64,
    ) -> AppResult<()> {
        let model = UserEntity::find_by_id(id).one(tx).await?;
        if let Some(user) = model {
            let mut active: UserActiveModel = user.into();
            active.order_count = Set(active.order_count.unwrap() + 1);
            active.update(tx).await?;
        }
        Ok(())
    }
}

fn map_to_domain(model: UserModel) -> User {
    User {
        id: model.id,
        name: model.name,
        email: model.email,
        order_count: model.order_count,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};

    #[tokio::test]
    async fn test_find_by_id() {
        let now = Utc::now().naive_utc();
        let mock_model = UserModel {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            order_count: 0,
            created_at: now,
            updated_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([vec![mock_model]])
            .into_connection();

        let mut tx = db.begin().await.unwrap();
        let repo = SeaOrmUserRepository;
        let result = repo.find_by_id(&mut tx, 1).await.unwrap();

        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Alice");
    }

    #[tokio::test]
    async fn test_find_all() {
        let now = Utc::now().naive_utc();
        let mock_model = UserModel {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            order_count: 0,
            created_at: now,
            updated_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([vec![mock_model]])
            .into_connection();

        let mut tx = db.begin().await.unwrap();
        let repo = SeaOrmUserRepository;
        let result = repo.find_all(&mut tx).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Alice");
    }

    #[tokio::test]
    async fn test_create() {
        let now = Utc::now().naive_utc();
        let mock_model = UserModel {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            order_count: 0,
            created_at: now,
            updated_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([vec![mock_model]])
            .append_exec_results([MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection();

        let mut tx = db.begin().await.unwrap();
        let repo = SeaOrmUserRepository;
        let result = repo.create(&mut tx, "Alice", "alice@example.com").await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.name, "Alice");
        assert_eq!(result.email, "alice@example.com");
    }

    #[tokio::test]
    async fn test_increment_order_count() {
        let now = Utc::now().naive_utc();
        let mock_model = UserModel {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            order_count: 0,
            created_at: now,
            updated_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([vec![mock_model.clone()], vec![mock_model.clone()]])
            .append_exec_results([MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection();

        let mut tx = db.begin().await.unwrap();
        let repo = SeaOrmUserRepository;
        repo.increment_order_count(&mut tx, 1).await.unwrap();
    }
}
