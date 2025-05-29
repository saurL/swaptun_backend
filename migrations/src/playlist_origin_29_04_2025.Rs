use sea_orm::{sea_query::extension::postgres::Type, ActiveEnum, DbBackend, Schema};
use sea_orm_migration::prelude::*;
use swaptun_models::playlist::PlaylistOrigin;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the enum type
        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<PlaylistOrigin>())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Playlist::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Playlist::Origin).custom(PlaylistOrigin::name()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the origin column
        manager
            .alter_table(
                Table::alter()
                    .table(Playlist::Table)
                    .drop_column(Playlist::Origin)
                    .to_owned(),
            )
            .await?;

        // Drop the enum type
        manager
            .drop_type(Type::drop().name(PlaylistOrigin::name()).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Playlist {
    Table,
    Origin,
}
