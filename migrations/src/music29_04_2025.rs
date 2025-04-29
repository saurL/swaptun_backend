use crate::playlist29_04_2025::Playlist;
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
                    .col(
                        ColumnDef::new(Music::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Music::Title).string().not_null())
                    .col(ColumnDef::new(Music::Artist).string().not_null())
                    .col(ColumnDef::new(Music::Album).string().null())
                    .col(ColumnDef::new(Music::ReleaseDate).date().null())
                    .col(ColumnDef::new(Music::Genre).string().not_null())
                    .col(ColumnDef::new(Music::PlaylistId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Music::Table, Music::PlaylistId)
                            .to(Playlist::Table, Playlist::Id),
                    )
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
    Id,
    Title,
    Artist,
    Album,
    ReleaseDate,
    Genre,
    PlaylistId,
}
