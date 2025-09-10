use crate::m20250319_093000_create_tbl_users::TblUsers;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblAppleToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblAppleToken::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TblAppleToken::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(TblAppleToken::AccessToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblAppleToken::CreatedOn)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(TblAppleToken::UpdatedOn)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_apple_token_user")
                            .from(TblAppleToken::Table, TblAppleToken::UserId)
                            .to(TblUsers::Table, TblUsers::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblAppleToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblAppleToken {
    Table,
    Id,
    UserId,
    AccessToken,
    CreatedOn,
    UpdatedOn,
}
