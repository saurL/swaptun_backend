use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "music_playlist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub playlist_id: i32,
    pub music_title: String,
    pub music_artist: String,
    pub music_album: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::playlist::Entity",
        from = "Column::PlaylistId",
        to = "super::playlist::Column::Id"
    )]
    Playlist,
    #[sea_orm(
        belongs_to = "super::music::Entity",
        from = "(Column::MusicTitle, Column::MusicArtist, Column::MusicAlbum)",
        to = "(super::music::Column::Title, super::music::Column::Artist, super::music::Column::Album)"
    )]
    Music,
}

impl ActiveModelBehavior for ActiveModel {}
