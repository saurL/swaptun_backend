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

#[derive(Deserialize, Serialize, Debug)]
pub struct SendPlaylistResponse {
    pub platform: PlaylistOrigin,
    pub playlist_id: String,
}

#[derive(Deserialize, Serialize, Validate, Debug)]
pub struct SharePlaylistRequest {
    pub user_id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SharedPlaylistsResponse {
    pub shared_playlists: Vec<SharedPlaylist>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SharedPlaylist {
    pub id: i32,
    pub playlist: PlaylistModel,
    pub shared_by: UserInfo,
    pub shared_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
}

impl From<UserModel> for UserInfo {
    fn from(user: UserModel) -> Self {
        UserInfo {
            id: user.id,
            username: user.username,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_playlist_response_serialization() {
        let response = SendPlaylistResponse {
            platform: PlaylistOrigin::Spotify,
            playlist_id: "test_id_123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test_id_123"));
        assert!(json.contains("Spotify"));
    }

    #[test]
    fn test_send_playlist_response_deserialization() {
        let json = r#"{"platform":"YoutubeMusic","playlist_id":"yt_playlist_456"}"#;
        let response: SendPlaylistResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.platform, PlaylistOrigin::YoutubeMusic);
        assert_eq!(response.playlist_id, "yt_playlist_456");
    }

    #[test]
    fn test_send_playlist_response_all_platforms() {
        let platforms = vec![
            PlaylistOrigin::Spotify,
            PlaylistOrigin::AppleMusic,
            PlaylistOrigin::YoutubeMusic,
        ];

        for platform in platforms {
            let response = SendPlaylistResponse {
                platform: platform.clone(),
                playlist_id: format!("{:?}_test_id", platform),
            };

            let json = serde_json::to_string(&response).unwrap();
            let deserialized: SendPlaylistResponse = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.platform, platform);
            assert!(deserialized.playlist_id.contains("test_id"));
        }
    }
}
