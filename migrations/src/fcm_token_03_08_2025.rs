use sea_orm_migration::prelude::*;

use crate::m20250319_093000_create_tbl_users::TblUsers;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblFcmTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblFcmTokens::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TblFcmTokens::UserId).integer().not_null())
                    .col(ColumnDef::new(TblFcmTokens::Token).string().not_null())
                    .col(ColumnDef::new(TblFcmTokens::DeviceId).string())
                    .col(ColumnDef::new(TblFcmTokens::Platform).string_len(20))
                    .col(
                        ColumnDef::new(TblFcmTokens::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(TblFcmTokens::CreatedOn)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TblFcmTokens::UpdatedOn)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TblFcmTokens::Table, TblFcmTokens::UserId)
                            .to(TblUsers::Table, TblUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblFcmTokens::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TblFcmTokens {
    Table,
    Id,
    UserId,
    Token,
    DeviceId,
    Platform,
    IsActive,
    CreatedOn,
    UpdatedOn,
}
