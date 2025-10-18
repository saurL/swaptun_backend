use serde::{Deserialize, Serialize};

use sea_orm::entity::prelude::*;

#[derive(EnumIter, DeriveActiveEnum, Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "playlist_origin")]
pub enum PlaylistOrigin {
    #[sea_orm(string_value = "Spotify")]
    Spotify,
    #[sea_orm(string_value = "Deezer")]
    Deezer,
    #[sea_orm(string_value = "YoutubeMusic")]
    YoutubeMusic,
    #[sea_orm(string_value = "AppleMusic")]
    AppleMusic,
}
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "playlist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub origin: PlaylistOrigin,
    pub created_on: DateTimeWithTimeZone,
    pub updated_on: DateTimeWithTimeZone,
    pub origin_id: String,
    pub image_url: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::user::Entity",
        from = "Column::UserId",
        to = "crate::user::Column::Id"
    )]
    User,
}

impl Related<crate::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
impl Related<super::music::Entity> for Entity {
    fn to() -> RelationDef {
        super::music_playlist::Relation::Music.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::music_playlist::Relation::Playlist.def().rev())
    }
}
