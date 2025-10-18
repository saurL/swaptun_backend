use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Playlist::Table)
                    .add_column(string_null(Playlist::ImageUrl))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Playlist::Table)
                    .drop_column(Playlist::ImageUrl)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Playlist {
    Table,
    ImageUrl,
}
