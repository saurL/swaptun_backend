use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
use crate::m2025_03_19_create_tbl_users::TblUsers;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblUserInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblUserInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TblUserInfo::UserId).integer().not_null())
                    .col(ColumnDef::new(TblUserInfo::Birthdate).string().not_null())
                    .col(ColumnDef::new(TblUserInfo::Gender).string().not_null())
                    .col(ColumnDef::new(TblUserInfo::Region).string().not_null())
                    .col(ColumnDef::new(TblUserInfo::Consent).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_info_user_id")
                            .from(TblUserInfo::Table, TblUserInfo::UserId)
                            .to(TblUsers::Table, TblUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblUserInfo::Table).to_owned())
            .await
    }
}

// Identificateurs pour la table user_info
#[derive(DeriveIden)]
pub enum TblUserInfo {
    Table,
    Id,
    UserId,
    Birthdate,
    Gender,
    Region,
    Consent,
}
