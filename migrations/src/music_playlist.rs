use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MusicPlaylist::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MusicPlaylist::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MusicPlaylist::PlaylistId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MusicPlaylist::MusicTitle)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MusicPlaylist::MusicArtist)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MusicPlaylist::MusicAlbum)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(MusicPlaylist::Table, MusicPlaylist::PlaylistId)
                            .to(Playlist::Table, Playlist::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MusicPlaylist::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum MusicPlaylist {
    Table,
    Id,
    PlaylistId,
    MusicTitle,
    MusicArtist,
    MusicAlbum,
}

#[derive(Iden)]
enum Playlist {
    Table,
    Id,
}
