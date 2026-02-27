use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id).big_integer())
                    .col(string(Users::Name))
                    .col(string(Users::Email).unique_key())
                    .col(integer(Users::OrderCount).default(0))
                    .col(date_time(Users::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(Users::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        // Create orders table
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(pk_auto(Orders::Id).big_integer())
                    .col(big_integer(Orders::UserId))
                    .col(string(Orders::ItemName))
                    .col(integer(Orders::Quantity))
                    .col(date_time(Orders::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_user_id")
                            .from(Orders::Table, Orders::UserId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Name,
    Email,
    OrderCount,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    UserId,
    ItemName,
    Quantity,
    CreatedAt,
}
