use crate::{m20250319_093000_create_tbl_users::TblUsers, playlist29_04_2025::Playlist};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SharedPlaylist::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SharedPlaylist::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SharedPlaylist::UserId)
                            .integer()
                            .not_null()
                            .comment("User who has access to the playlist"),
                    )
                    .col(
                        ColumnDef::new(SharedPlaylist::PlaylistId)
                            .integer()
                            .not_null()
                            .comment("Playlist that is shared"),
                    )
                    .col(
                        ColumnDef::new(SharedPlaylist::CreatedOn)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SharedPlaylist::Table, SharedPlaylist::UserId)
                            .to(TblUsers::Table, TblUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SharedPlaylist::Table, SharedPlaylist::PlaylistId)
                            .to(Playlist::Table, Playlist::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_shared_playlist_unique")
                            .col(SharedPlaylist::UserId)
                            .col(SharedPlaylist::PlaylistId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SharedPlaylist::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SharedPlaylist {
    Table,
    Id,
    UserId,
    PlaylistId,
    CreatedOn,
}
