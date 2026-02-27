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
