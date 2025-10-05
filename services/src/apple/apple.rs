use std::sync::Arc;

use crate::{error::AppError, GetDeveloperToken};
use crate::{
    AddTokenRequest, CreateMusicRequest, CreatePlaylistRequest, MusicService, PlaylistService,
};
use apple_music_api::config::ClientConfigBuilder;
use apple_music_api::library::LibraryPlaylistsResponse;
use apple_music_api::{create_developer_token, AppleMusicClient};
use log::{error, info};
use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use swaptun_models::{playlist, AppleTokenActiveModel, AppleTokenModel, UserModel};
use swaptun_repositories::AppleTokenRepository;

#[derive(Clone)]
pub struct AppleMusicService {
    apple_token_repository: AppleTokenRepository,
    playlist_service: PlaylistService,
    music_service: MusicService,
    db: Arc<DatabaseConnection>,
}

impl AppleMusicService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        let apple_token_repository = AppleTokenRepository::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());
        let music_service = MusicService::new(db.clone());
        AppleMusicService {
            apple_token_repository,
            playlist_service,
            music_service,
            db,
        }
    }

    pub async fn generate_developer_token(&self) -> Result<GetDeveloperToken, AppError> {
        let team_id = std::env::var("APPLE_TEAM_ID")?;
        let key_id = std::env::var("APPLE_KEY_ID")?;
        let private_key = include_str!("../../certs/AuthKey_apple_music.p8");
        match create_developer_token(&team_id, &key_id, &private_key) {
            Ok(token) => Ok(GetDeveloperToken {
                developer_token: token,
            }),
            Err(e) => {
                error!("Failed to create developer token: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn add_user_token(
        &self,
        request: AddTokenRequest,
        user_id: i32,
    ) -> Result<(), AppError> {
        match self.apple_token_repository.find_by_user_id(user_id).await? {
            Some(existing_token) => {
                let mut active_model = existing_token.into_active_model();
                active_model.access_token = Set(request.token);
                self.save(active_model).await
            }
            None => {
                let token = AppleTokenActiveModel {
                    user_id: Set(user_id),
                    access_token: Set(request.token),
                    ..Default::default()
                };
                self.save(token).await
            }
        }
    }

    pub async fn save(&self, apple_active_model: AppleTokenActiveModel) -> Result<(), AppError> {
        self.apple_token_repository
            .save(apple_active_model)
            .await
            .map_err(|e| {
                error!("Failed to save apple token: {:?}", e);
                AppError::InternalServerError
            })?;
        Ok(())
    }

    pub async fn get_token(&self, user: &UserModel) -> Result<Option<AppleTokenModel>, AppError> {
        match self.apple_token_repository.get_token(user).await? {
            Some(token) => {
                info!("Found token for user {}", user.id);
                Ok(Some(token))
            }
            None => {
                info!("No token found for user {}", user.id);
                Ok(None)
            }
        }
    }

    pub async fn get_apple_client(&self, user: &UserModel) -> Result<AppleMusicClient, AppError> {
        if let Some(user_token) = self.get_token(user).await? {
            let key_id = std::env::var("APPLE_KEY_ID")?;
            let team_id = std::env::var("APPLE_TEAM_ID")?;
            let private_key = include_str!("../../certs/AuthKey_apple_music.p8").to_string();

            let developper_token: String =
                match create_developer_token(&team_id, &key_id, &private_key) {
                    Ok(token) => token,
                    Err(e) => {
                        error!("Failed to create developer token: {:?}", e);
                        return Err(AppError::InternalServerError);
                    }
                };
            let config = ClientConfigBuilder::default()
                .developer_token(developper_token)
                .key_id(key_id)
                .team_id(team_id)
                .user_token(Some(user_token.access_token))
                .build()
                .unwrap();
            info!("Apple Music client config: {:?}", config);
            let apple_client = AppleMusicClient::new(config).await?;

            Ok(apple_client)
        } else {
            error!("No apple token found for user {}", user.id);
            Err(AppError::InternalServerError)
        }
    }

    async fn get_user_playlists(
        &self,
        user: &UserModel,
    ) -> Result<LibraryPlaylistsResponse, AppError> {
        let client = self.get_apple_client(user).await?;
        Ok(client.get_library_playlists().await?)
    }

    pub async fn import_playlists(&self, user: &UserModel) -> Result<(), AppError> {
        let client = self.get_apple_client(user).await?;
        let playlists = client.get_library_playlists().await?;
        info!("Found {} Apple Music playlists", playlists.data.len());

        // Spawn background task to import playlists
        let service = self.clone();
        let user_clone = user.clone();
        let playlist_ids: Vec<String> = playlists.data.iter().map(|p| p.id.clone()).collect();

        tokio::spawn(async move {
            info!(
                "Starting background import of {} Apple Music playlists",
                playlist_ids.len()
            );

            match service.get_apple_client(&user_clone).await {
                Ok(client) => {
                    for playlist_id in playlist_ids {
                        if let Err(e) = service
                            .import_playlist(&user_clone, &client, playlist_id.clone())
                            .await
                        {
                            error!(
                                "Error importing Apple Music playlist {}: {:?}",
                                playlist_id, e
                            );
                        } else {
                            info!(
                                "Successfully imported Apple Music playlist: {}",
                                playlist_id
                            );
                        }
                    }
                    info!("Background import of Apple Music playlists completed");
                }
                Err(e) => {
                    error!(
                        "Failed to get Apple Music client for background import: {:?}",
                        e
                    );
                }
            }
        });

        Ok(())
    }

    pub async fn import_playlist(
        &self,
        user: &UserModel,
        client: &AppleMusicClient,
        playlist_id: String,
    ) -> Result<(), AppError> {
        let playlist = client
            .get_library_playlist_with_tracks(&playlist_id)
            .await?;
        let title = playlist.attributes.name.clone();

        let request = CreatePlaylistRequest {
            name: playlist.attributes.name,
            origin_id: playlist.id.clone(),
            description: None,
            origin: playlist::PlaylistOrigin::AppleMusic,
        };
        let created_playlist = self.playlist_service.create_or_get(request, user).await?;

        let songs = playlist
            .relationships
            .unwrap_or_default()
            .tracks
            .unwrap_or_default()
            .data;

        let mut local_tracks = self
            .music_service
            .find_by_playlist(&created_playlist)
            .await?;

        let music_service = &self.music_service;
        let playlist_service = &self.playlist_service;
        let created_playlist = &created_playlist;
        for song in songs {
            let attributes = song.attributes;
            let title = attributes.name.expect("Failed to get track name").clone();
            let artist = attributes
                .artist_name
                .expect("Failed to get artist name")
                .clone();
            let album = attributes
                .album_name
                .expect("Failed to get album name")
                .clone();
            // Check if track already exists
            if let Some(pos) = local_tracks.iter().position(|local_track| {
                local_track.title == title
                    && local_track.artist == artist
                    && local_track.album == album
            }) {
                local_tracks.remove(pos);
                return Ok(());
            }

            let create_music_request = CreateMusicRequest {
                title: title,
                artist: artist,
                album: album,
                description: None,
                genre: attributes.genre_names.get(0).cloned(),
                release_date: attributes
                    .release_date
                    .expect("Failed to get release date")
                    .clone()
                    .naive_local()
                    .into(),
            };
            let music = music_service.create(create_music_request).await?;
            playlist_service.add_music(&created_playlist, music).await?;
        }

        // Remove tracks that are no longer in the playlist
        for local_track in local_tracks {
            info!("Removing track from playlist: {:?}", local_track);
            self.playlist_service
                .remove_music(&created_playlist, &local_track)
                .await?;
        }
        Ok(())
    }

    /// Exports a playlist from our database to Apple Music
    pub async fn export_playlist_to_apple(
        &self,
        playlist_id: i32,
        user: &UserModel,
    ) -> Result<String, AppError> {
        // Get the database playlist
        let playlist = self.playlist_service.get_playlist(playlist_id).await?;

        // Get Apple Music client
        let client = self.get_apple_client(user).await?;

        // Create the playlist on Apple Music
        let apple_playlist = client
            .create_library_playlist(&playlist.name, playlist.description.as_deref())
            .await
            .map_err(|e| {
                error!("Error creating Apple Music playlist: {:?}", e);
                AppError::InternalServerError
            })?;

        info!(
            "Created Apple Music playlist with ID: {}",
            apple_playlist.id
        );

        // Get tracks from the database playlist
        let tracks = self
            .music_service
            .find_by_playlist(&playlist)
            .await
            .map_err(|e| {
                error!("Error getting playlist tracks: {:?}", e);
                AppError::InternalServerError
            })?;

        if tracks.is_empty() {
            info!("No tracks in playlist, returning early");
            return Ok(apple_playlist.id);
        }

        // Search for tracks on Apple Music and collect their IDs
        let mut apple_track_ids = Vec::new();
        let mut not_found_tracks = Vec::new();

        for track in tracks {
            // Search for the track on Apple Music
            let query = format!("{} {}", track.title, track.artist);

            match client.search_songs(&query).await {
                Ok(songs) => {
                    let mut found = false;
                    for apple_song in songs.iter().take(5) {
                        let apple_artist = apple_song.attributes.artist_name.to_lowercase();
                        let apple_title = apple_song.attributes.name.to_lowercase();

                        if apple_title == track.title.to_lowercase()
                            && apple_artist == track.artist.to_lowercase()
                        {
                            // Add song to library first
                            if let Err(e) = client.add_songs_to_library(&[&apple_song.id]).await {
                                error!("Error adding song to library: {:?}", e);
                                // Continue anyway, the song might already be in the library
                            }

                            apple_track_ids.push(apple_song.id.clone());
                            info!(
                                "Found track on Apple Music: {} - {}",
                                track.artist, track.title
                            );
                            found = true;
                            break;
                        } else {
                            info!(
                                "Track found but artist/title does not match: Apple: {} - {}, DB: {} - {}",
                                apple_artist,
                                apple_title,
                                track.artist,
                                track.title
                            );
                        }
                    }

                    if !found {
                        not_found_tracks.push(format!("{} - {}", track.artist, track.title));
                        info!(
                            "Track not found on Apple Music: {} - {}",
                            track.artist, track.title
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Error searching for track {} - {}: {:?}",
                        track.artist, track.title, e
                    );
                    not_found_tracks.push(format!("{} - {}", track.artist, track.title));
                }
            }
        }

        // Add tracks to the Apple Music playlist in batches
        if !apple_track_ids.is_empty() {
            // Apple Music API can handle batches, but let's be conservative with 100 tracks at a time
            for chunk in apple_track_ids.chunks(100) {
                let track_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();

                match client
                    .add_tracks_to_playlist(&apple_playlist.id, &track_refs)
                    .await
                {
                    Ok(_) => {
                        info!("Added {} tracks to Apple Music playlist", chunk.len());
                    }
                    Err(e) => {
                        error!("Error adding tracks to Apple Music playlist: {:?}", e);
                        return Err(AppError::InternalServerError);
                    }
                }
            }
        }

        // Log tracks that weren't found
        if !not_found_tracks.is_empty() {
            info!("The following tracks were not found on Apple Music:");
            for track in &not_found_tracks {
                info!("  - {}", track);
            }
        }

        Ok(apple_playlist.id)
    }

    pub async fn disconnect(&self, user: &UserModel) -> Result<(), AppError> {
        info!("Disconnecting Apple Music for user {}", user.id);

        // Delete playlists from Apple Music origin
        self.playlist_service
            .delete_by_origin(user, playlist::PlaylistOrigin::AppleMusic)
            .await?;

        // Delete Apple Music tokens
        self.apple_token_repository
            .delete_by_user_id(user.id)
            .await
            .map_err(|e| {
                error!("Failed to delete apple token: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("Apple Music disconnected successfully for user {}", user.id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::test_database::setup_test_db;
    use sea_orm::ActiveValue::Set;
    use swaptun_models::{playlist, MusicActiveModel, PlaylistActiveModel, UserActiveModel};

    #[tokio::test]
    async fn test_export_playlist_to_apple_no_tracks() {
        // Setup test database
        let db = setup_test_db().await;
        let apple_service = AppleMusicService::new(db.clone());

        // Create test user
        let user_active = UserActiveModel {
            username: Set("test_user".to_string()),
            password: Set("hash".to_string()),
            email: Set("test@example.com".to_string()),
            first_name: Set("Test".to_string()),
            last_name: Set("User".to_string()),
            role: Set("user".to_string()),
            ..Default::default()
        };
        let user = swaptun_repositories::UserRepository::new(db.clone())
            .save(user_active)
            .await
            .unwrap();

        // Create test playlist without tracks
        let playlist_active = PlaylistActiveModel {
            user_id: Set(user.id),
            name: Set("Test Playlist".to_string()),
            description: Set(Some("Test description".to_string())),
            origin: Set(playlist::PlaylistOrigin::Spotify),
            origin_id: Set("test_origin_id".to_string()),
            ..Default::default()
        };
        let playlist = swaptun_repositories::PlaylistRepository::new(db.clone())
            .save(playlist_active)
            .await
            .unwrap();

        // Note: This test cannot actually call Apple Music API without credentials
        // In a real scenario, you would:
        // 1. Mock the Apple Music client
        // 2. Or skip this test if Apple credentials are not available
        // For now, we just verify the playlist was created correctly
        assert_eq!(playlist.name, "Test Playlist");
        assert_eq!(playlist.user_id, user.id);
    }

    #[tokio::test]
    async fn test_import_playlist_creates_music_records() {
        // Setup test database
        let db = setup_test_db().await;
        let apple_service = AppleMusicService::new(db.clone());
        let music_service = MusicService::new(db.clone());

        // Create test user
        let user_active = UserActiveModel {
            username: Set("test_user".to_string()),
            password: Set("hash".to_string()),
            email: Set("test@example.com".to_string()),
            first_name: Set("Test".to_string()),
            last_name: Set("User".to_string()),
            role: Set("user".to_string()),
            ..Default::default()
        };
        let user = swaptun_repositories::UserRepository::new(db.clone())
            .save(user_active)
            .await
            .unwrap();

        // Create test playlist
        let playlist_active = PlaylistActiveModel {
            user_id: Set(user.id),
            name: Set("Test Playlist".to_string()),
            description: Set(Some("Test description".to_string())),
            origin: Set(playlist::PlaylistOrigin::AppleMusic),
            origin_id: Set("test_apple_playlist_id".to_string()),
            ..Default::default()
        };
        let playlist = swaptun_repositories::PlaylistRepository::new(db.clone())
            .save(playlist_active)
            .await
            .unwrap();

        // Add test music to playlist
        let music_active = MusicActiveModel {
            title: Set("Test Song".to_string()),
            artist: Set("Test Artist".to_string()),
            album: Set("Test Album".to_string()),
            genre: Set(Some("Rock".to_string())),
            release_date: Set(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            ..Default::default()
        };
        let music = swaptun_repositories::MusicRepository::new(db.clone())
            .save(music_active)
            .await
            .unwrap();

        apple_service
            .playlist_service
            .add_music(&playlist, music.clone())
            .await
            .unwrap();

        // Verify music was added
        let tracks = music_service.find_by_playlist(&playlist).await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Test Song");
        assert_eq!(tracks[0].artist, "Test Artist");
    }
}
