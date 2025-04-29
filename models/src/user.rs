use sea_orm::{entity::prelude::*, sqlx::types::chrono::NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tbl_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,

    #[serde(skip_serializing)]
    pub password: String,

    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: String,
    pub created_on: NaiveDateTime,
    pub updated_on: NaiveDateTime,
    pub deleted_on: Option<NaiveDateTime>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::spotify_token::Entity")]
    SpotifyToken,
    #[sea_orm(has_one = "super::deezer_token::Entity")]
    DeezerToken,
    #[sea_orm(has_many = "super::playlist::Entity")]
    Playlist,
}

impl Related<super::spotify_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SpotifyToken.def()
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Playlist.def()
    }
}

impl Related<super::deezer_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DeezerToken.def()
    }
}
