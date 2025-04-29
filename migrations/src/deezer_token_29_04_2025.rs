use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblDeezerTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblDeezerTokens::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TblDeezerTokens::UserId).integer().not_null())
                    .col(ColumnDef::new(TblDeezerTokens::Token).string().not_null())
                    .col(
                        ColumnDef::new(TblDeezerTokens::CreatedOn)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TblDeezerTokens::UpdatedOn)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TblDeezerTokens::Table, TblDeezerTokens::UserId)
                            .to(TblUsers::Table, TblUsers::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblDeezerTokens::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TblDeezerTokens {
    Table,
    Id,
    UserId,
    Token,
    CreatedOn,
    UpdatedOn,
}

#[derive(Iden)]
enum TblUsers {
    Table,
    Id,
}
