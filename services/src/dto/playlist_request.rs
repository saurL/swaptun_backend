use serde::{Deserialize, Serialize};
use swaptun_models::{PlaylistModel, PlaylistOrigin};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub origin: PlaylistOrigin,
    pub origin_id: String,
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

#[derive(Deserialize, Serialize, Validate)]
pub struct GetPlaylistsParams {
    pub origin: Option<PlaylistOrigin>,
}

#[derive(Deserialize, Serialize, Validate, Clone)]
pub struct GetPlaylistResponse {
    pub vec: Vec<PlaylistModel>,
}

#[derive(Deserialize, Serialize, Validate, Debug)]
pub struct SendPlaylistRequest {
    pub origin: PlaylistOrigin,
}
