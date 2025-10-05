use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
use crate::m2025_03_19_create_tbl_users::TblUsers;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Friendships::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Friendships::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Friendships::UserId).integer().not_null())
                    .col(ColumnDef::new(Friendships::FriendId).integer().not_null())
                    .col(
                        ColumnDef::new(Friendships::CreatedOn)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friendships::Table, Friendships::UserId)
                            .to(TblUsers::Table, TblUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friendships::Table, Friendships::FriendId)
                            .to(TblUsers::Table, TblUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_friendships_unique")
                            .col(Friendships::UserId)
                            .col(Friendships::FriendId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Friendships::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Friendships {
    Table,
    Id,
    UserId,
    FriendId,
    CreatedOn,
}
