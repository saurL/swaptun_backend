use crate::m2025_03_19_create_tbl_users::TblUsers;
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblYoutubeToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblYoutubeToken::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TblYoutubeToken::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(TblYoutubeToken::AccessToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblYoutubeToken::RefreshToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblYoutubeToken::ExpiresIn)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblYoutubeToken::CreatedOn)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(TblYoutubeToken::UpdatedOn)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_youtube_token_user")
                            .from(TblYoutubeToken::Table, TblYoutubeToken::UserId)
                            .to(TblUsers::Table, TblUsers::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblYoutubeToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblYoutubeToken {
    Table,
    Id,
    UserId,
    AccessToken,
    RefreshToken,
    ExpiresIn,
    CreatedOn,
    UpdatedOn,
}
