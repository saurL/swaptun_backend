use futures::SinkExt;
use sea_orm::entity::prelude::Date;
use serde::{Deserialize, Serialize};
use validator::Validate;
#[derive(Deserialize, Serialize, Validate)]
pub struct CreateMusicRequest {
    pub title: String,
    pub description: Option<String>,

    pub artist: String,
    pub album: String,
    pub release_date: Date,
    pub genre: Option<String>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UpdateMusicRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub release_date: Option<Date>,
    pub genre: Option<String>,
}
#[derive(Deserialize, Serialize, Validate)]
pub struct DeleteMusicRequest {
    pub id: i32,
}
