use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TblUsers::Username).string_len(20).not_null())
                    .col(ColumnDef::new(TblUsers::Password).string().not_null())
                    .col(
                        ColumnDef::new(TblUsers::FirstName)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(TblUsers::LastName).string_len(20).not_null())
                    .col(ColumnDef::new(TblUsers::Email).string().not_null())
                    .col(ColumnDef::new(TblUsers::Role).string_len(10).not_null())
                    .col(
                        ColumnDef::new(TblUsers::CreatedOn)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblUsers::UpdatedOn)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblUsers::DeletedOn)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .index(
                        Index::create()
                            .name("idx_username")
                            .col(TblUsers::Username)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_email")
                            .col(TblUsers::Email)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblUsers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TblUsers {
    Table,
    Id,
    Username,
    Password,
    FirstName,
    LastName,
    Email,
    Role,
    CreatedOn,
    UpdatedOn,
    DeletedOn,
}
