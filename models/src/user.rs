use sea_orm::{entity::prelude::*, sqlx::types::chrono::NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, Eq, Hash)]
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
    #[sea_orm(has_one = "super::spotify_code::Entity")]
    SpotifyCode,
    #[sea_orm(has_one = "super::spotify_token::Entity")]
    SpotifyToken,
    #[sea_orm(has_one = "super::deezer_token::Entity")]
    DeezerToken,
    #[sea_orm(has_one = "super::fcm_token::Entity")]
    FcmToken,
    #[sea_orm(has_many = "super::playlist::Entity")]
    Playlist,
    #[sea_orm(has_one = "super::youtube_token::Entity")]
    YoutubeToken,
    #[sea_orm(has_one = "super::apple_token::Entity")]
    AppleToken,
}

impl Related<super::spotify_code::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SpotifyCode.def()
    }
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

impl Related<super::fcm_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FcmToken.def()
    }
}

impl Related<super::youtube_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::YoutubeToken.def()
    }
}

impl Related<super::apple_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppleToken.def()
    }
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        super::friendship::Relation::Friend.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::friendship::Relation::User.def().rev())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserBean {
    pub id: i32,
    pub username: String,
}

impl Into<UserBean> for Model {
    fn into(self) -> UserBean {
        UserBean {
            id: self.id,
            username: self.username,
        }
    }
}

pub struct SharedPlaylist;

impl Linked for SharedPlaylist {
    type FromEntity = Entity;

    type ToEntity = super::playlist::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::shared_playlist::Relation::User.def().rev(),
            super::shared_playlist::Relation::Playlist.def(),
        ]
    }
}
