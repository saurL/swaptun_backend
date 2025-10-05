use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shared_playlist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(comment = "User who has access to the playlist")]
    pub user_id: i32,
    #[sea_orm(comment = "Playlist that is shared")]
    pub playlist_id: i32,
    #[sea_orm(comment = "User who shared the playlist")]
    pub shared_by_user_id: i32,
    pub created_on: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::playlist::Entity",
        from = "Column::PlaylistId",
        to = "super::playlist::Column::Id"
    )]
    Playlist,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SharedByUserId",
        to = "super::user::Column::Id"
    )]
    SharedByUser,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Playlist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
