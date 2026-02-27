use async_trait::async_trait;
use sqlx::{MySql, Transaction};

use crate::domain::entity::user::User;
use crate::domain::repository::user_repository::UserRepository;
use crate::error::AppResult;

pub struct SqlxUserRepository;

#[async_trait]
impl UserRepository<Transaction<'static, MySql>> for SqlxUserRepository {
    async fn find_by_id(
        &self,
        tx: &mut Transaction<'static, MySql>,
        id: i64,
    ) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, order_count, created_at, updated_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;
        Ok(user)
    }

    async fn find_all(&self, tx: &mut Transaction<'static, MySql>) -> AppResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, name, email, order_count, created_at, updated_at FROM users ORDER BY id",
        )
        .fetch_all(&mut **tx)
        .await?;
        Ok(users)
    }

    async fn create(
        &self,
        tx: &mut Transaction<'static, MySql>,
        name: &str,
        email: &str,
    ) -> AppResult<User> {
        sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
            .bind(name)
            .bind(email)
            .execute(&mut **tx)
            .await?;

        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, order_count, created_at, updated_at FROM users WHERE id = LAST_INSERT_ID()",
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(user)
    }

    async fn increment_order_count(
        &self,
        tx: &mut Transaction<'static, MySql>,
        id: i64,
    ) -> AppResult<()> {
        sqlx::query("UPDATE users SET order_count = order_count + 1 WHERE id = ?")
            .bind(id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}
