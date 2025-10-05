use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use swaptun_models::{PlaylistModel, PlaylistOrigin, UserModel};
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
    pub destination: PlaylistOrigin,
}
#[derive(Deserialize, Serialize, Validate, Debug)]
pub struct SharePlaylistRequest {
    pub user_id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SharedPlaylistResponse {
    pub id: i32,
    pub playlist: PlaylistModel,
    pub shared_by: UserInfo,
    pub shared_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}

impl From<UserModel> for UserInfo {
    fn from(user: UserModel) -> Self {
        UserInfo {
            id: user.id,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
        }
    }
}
