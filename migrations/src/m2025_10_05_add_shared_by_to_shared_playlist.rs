use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SharedPlaylist::Table)
                    .add_column(
                        ColumnDef::new(SharedPlaylist::SharedByUserId)
                            .integer()
                            .not_null()
                            .comment("User who shared the playlist"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint
        manager
            .alter_table(
                Table::alter()
                    .table(SharedPlaylist::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_shared_playlist_shared_by")
                            .from_tbl(SharedPlaylist::Table)
                            .from_col(SharedPlaylist::SharedByUserId)
                            .to_tbl(TblUsers::Table)
                            .to_col(TblUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SharedPlaylist::Table)
                    .drop_foreign_key(Alias::new("fk_shared_playlist_shared_by"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(SharedPlaylist::Table)
                    .drop_column(SharedPlaylist::SharedByUserId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SharedPlaylist {
    Table,
    SharedByUserId,
}

#[derive(DeriveIden)]
enum TblUsers {
    Table,
    Id,
}
