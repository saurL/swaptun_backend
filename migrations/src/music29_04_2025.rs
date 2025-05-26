use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Music::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Music::Title).string().not_null())
                    .col(ColumnDef::new(Music::Artist).string().not_null())
                    .col(ColumnDef::new(Music::Album).string().null())
                    .col(ColumnDef::new(Music::ReleaseDate).date().null())
                    .col(ColumnDef::new(Music::Genre).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Music::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Music {
    Table,
    Title,
    Artist,
    Album,
    ReleaseDate,
    Genre,
}
