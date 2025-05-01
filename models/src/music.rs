use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "music")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub title: String,
    #[sea_orm(primary_key)]
    pub artist: String,
    #[sea_orm(primary_key)]
    pub album: String,
    pub release_date: Date,
    pub genre: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
// Implémentation correcte de Related pour la relation many-to-many avec Playlist

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        super::music_playlist::Relation::Playlist.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::music_playlist::Relation::Music.def().rev())
    }
}
