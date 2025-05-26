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
                    .table(TblSpotifyCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblSpotifyCodes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TblSpotifyCodes::UserId).integer().not_null())
                    .col(ColumnDef::new(TblSpotifyCodes::Token).string().not_null())
                    .col(
                        ColumnDef::new(TblSpotifyCodes::CreatedOn)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TblSpotifyCodes::UpdatedOn)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TblSpotifyCodes::Table, TblSpotifyCodes::UserId)
                            .to(TblUsers::Table, TblUsers::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblSpotifyCodes::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TblSpotifyCodes {
    Table,
    Id,
    UserId,
    Token,
    CreatedOn,
    UpdatedOn,
}
