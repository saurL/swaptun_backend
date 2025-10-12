use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the unique index (which is implemented as a constraint in PostgreSQL)
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE friendships DROP CONSTRAINT IF EXISTS idx_friendships_unique")
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recreate the unique constraint
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_friendships_unique")
                    .table(Friendships::Table)
                    .col(Friendships::UserId)
                    .col(Friendships::FriendId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Friendships {
    Table,
    UserId,
    FriendId,
}
