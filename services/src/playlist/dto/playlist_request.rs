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
    #[serde(default)]
    pub include_musics: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlaylistWithMusics {
    pub playlist: PlaylistModel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musics: Option<Vec<swaptun_models::MusicModel>>,
}

#[derive(Deserialize, Serialize, Validate, Clone)]
pub struct GetPlaylistResponse {
    pub playlists: Vec<PlaylistWithMusics>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetPlaylistMusicsResponse {
    pub playlist_id: i32,
    pub musics: Vec<swaptun_models::MusicModel>,
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

    #[test]
    fn test_get_playlists_params_default_include_musics() {
        let json = r#"{"origin":null}"#;
        let params: GetPlaylistsParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.include_musics, false);
        assert_eq!(params.origin, None);
    }

    #[test]
    fn test_get_playlists_params_with_include_musics() {
        let json = r#"{"origin":"Spotify","include_musics":true}"#;
        let params: GetPlaylistsParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.include_musics, true);
        assert_eq!(params.origin, Some(PlaylistOrigin::Spotify));
    }

    #[test]
    fn test_get_playlist_musics_response_serialization() {
        use swaptun_models::MusicModel;
        use chrono::NaiveDate;

        let music = MusicModel {
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            release_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            genre: Some("Rock".to_string()),
        };

        let response = GetPlaylistMusicsResponse {
            playlist_id: 42,
            musics: vec![music],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Test Song"));
        assert!(json.contains("Test Artist"));
        assert!(json.contains("\"playlist_id\":42"));
    }

    #[test]
    fn test_playlist_with_musics_serialization_without_musics() {
        use swaptun_models::PlaylistModel;
        use chrono::{FixedOffset, Utc};

        let playlist = PlaylistModel {
            id: 1,
            name: "Test Playlist".to_string(),
            description: Some("Test Description".to_string()),
            user_id: 1,
            origin: PlaylistOrigin::Spotify,
            origin_id: "test_id".to_string(),
            created_on: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
            updated_on: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
        };

        let playlist_with_musics = PlaylistWithMusics {
            playlist,
            musics: None,
        };

        let json = serde_json::to_string(&playlist_with_musics).unwrap();
        assert!(json.contains("Test Playlist"));
        assert!(!json.contains("musics"));
    }

    #[test]
    fn test_playlist_with_musics_serialization_with_musics() {
        use swaptun_models::{MusicModel, PlaylistModel};
        use chrono::{FixedOffset, NaiveDate, Utc};

        let playlist = PlaylistModel {
            id: 1,
            name: "Test Playlist".to_string(),
            description: Some("Test Description".to_string()),
            user_id: 1,
            origin: PlaylistOrigin::Spotify,
            origin_id: "test_id".to_string(),
            created_on: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
            updated_on: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
        };

        let music = MusicModel {
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            release_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            genre: Some("Rock".to_string()),
        };

        let playlist_with_musics = PlaylistWithMusics {
            playlist,
            musics: Some(vec![music]),
        };

        let json = serde_json::to_string(&playlist_with_musics).unwrap();
        assert!(json.contains("Test Playlist"));
        assert!(json.contains("musics"));
        assert!(json.contains("Test Song"));
        assert!(json.contains("Test Artist"));
    }

    #[test]
    fn test_get_playlist_response_serialization() {
        use swaptun_models::PlaylistModel;
        use chrono::{FixedOffset, Utc};

        let playlist = PlaylistModel {
            id: 1,
            name: "Test Playlist".to_string(),
            description: Some("Test Description".to_string()),
            user_id: 1,
            origin: PlaylistOrigin::Spotify,
            origin_id: "spotify_id".to_string(),
            created_on: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
            updated_on: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
        };

        let response = GetPlaylistResponse {
            playlists: vec![PlaylistWithMusics {
                playlist,
                musics: None,
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("playlists"));
        assert!(json.contains("Test Playlist"));
    }
}
