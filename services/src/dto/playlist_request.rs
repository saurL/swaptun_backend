use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub playlist_id: i32,

    pub description: Option<String>,
}
#[derive(Deserialize, Serialize, Validate)]
pub struct DeletePlaylistRequest {
    pub id: i32,
}
