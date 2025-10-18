use std::sync::Arc;

use crate::{error::AppError, GetDeveloperToken};
use crate::{
    AddTokenRequest, CreateMusicRequest, CreatePlaylistRequest, MusicService, PlaylistService,
};
use apple_music_api::catalog::Song;
use apple_music_api::config::ClientConfigBuilder;
use apple_music_api::library::LibraryPlaylistsResponse;
use futures::{stream, StreamExt};

use apple_music_api::{create_developer_token, AppleMusicClient};
use log::{error, info};

use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use swaptun_models::{playlist, AppleTokenActiveModel, AppleTokenModel, MusicModel, UserModel};
use swaptun_repositories::AppleTokenRepository;
/// Normalize a string for fuzzy matching
fn normalize_string(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if two strings match with fuzzy logic
fn fuzzy_match(str1: &str, str2: &str) -> bool {
    let norm1 = normalize_string(str1);
    let norm2 = normalize_string(str2);

    if norm1 == norm2 {
        return true;
    }

    if norm1.contains(&norm2) || norm2.contains(&norm1) {
        return true;
    }

    false
}

/// Check if artist matches (Apple Music artist is a String)
fn artist_matches_apple(apple_artist: &str, db_artist: &str) -> bool {
    let db_artist_norm = normalize_string(db_artist);
    let apple_artist_norm = normalize_string(apple_artist);

    if fuzzy_match(&apple_artist_norm, &db_artist_norm) {
        return true;
    }

    let db_artist_parts: Vec<&str> = db_artist
        .split(&['&', ','][..])
        .chain(db_artist.split(" feat "))
        .chain(db_artist.split(" feat. "))
        .chain(db_artist.split(" ft "))
        .chain(db_artist.split(" ft. "))
        .chain(db_artist.split(" with "))
        .map(|s| s.trim())
        .collect();

    for db_part in &db_artist_parts {
        let db_part_norm = normalize_string(db_part);
        if fuzzy_match(&apple_artist_norm, &db_part_norm) {
            return true;
        }
    }

    false
}

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

        // Extract artwork URL if available (use 600x600 as default size)
        let image_url = playlist
            .attributes
            .artwork
            .map(|artwork| artwork.url_square(600));

        let request = CreatePlaylistRequest {
            name: playlist.attributes.name,
            origin_id: playlist.id.clone(),
            description: None,
            origin: playlist::PlaylistOrigin::AppleMusic,
            image_url,
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

        // Get tracks from the database playlist
        let tracks = self
            .music_service
            .find_by_playlist(&playlist)
            .await
            .map_err(|e| {
                error!("Error getting playlist tracks: {:?}", e);
                AppError::InternalServerError
            })?;

        let concurrency_limit = 3; // ajuste selon les quotas de l’API Apple

        let songs: Vec<Song> = stream::iter(tracks.clone().into_iter().map(|track| {
            let client = client.clone(); // si ton client est clonable
            async move {
                let query = format!("{} {}", track.title, track.artist);
                self.search_song(&client, query, track).await
            }
        }))
        .buffer_unordered(concurrency_limit)
        .filter_map(|res| async move { res }) // garde seulement les Some(Song)
        .collect()
        .await;
        let apple_track_ids: Vec<String> =
            songs.iter().map(|opt_song| opt_song.id.clone()).collect();
        // Create the playlist on Apple Music in the Swaptun folder
        let apple_playlist = client
            .create_library_playlist(
                &playlist.name,
                playlist.description,
                Some(apple_track_ids),
                None::<String>,
            )
            .await
            .map_err(|e| {
                error!("Error creating Apple Music playlist: {:?}", e);
                AppError::InternalServerError
            })?;

        info!(
            "Created Apple Music playlist with ID: {}",
            apple_playlist.id
        );

        Ok(apple_playlist.id)
    }

    pub async fn search_song(
        &self,
        client: &AppleMusicClient,
        query: String,
        track: MusicModel,
    ) -> Option<Song> {
        match client.search_songs(&query).await {
            Ok(songs) => {
                for apple_song in songs.iter().take(5) {
                    // Use fuzzy matching for title and artist
                    let title_matches = fuzzy_match(&apple_song.attributes.name, &track.title);
                    let artist_matches =
                        artist_matches_apple(&apple_song.attributes.artist_name, &track.artist);

                    if title_matches && artist_matches {
                        // Add song to library first

                        info!(
                            "Found track on Apple Music: {} - {} (matched with: {} - {})",
                            track.artist,
                            track.title,
                            apple_song.attributes.artist_name,
                            apple_song.attributes.name
                        );
                        return Some(apple_song.clone());
                    } else {
                        info!(
                                "Track found but does not match - Title match: {}, Artist match: {} | Apple: {} - {}, DB: {} - {}",
                                title_matches,
                                artist_matches,
                                apple_song.attributes.artist_name,
                                apple_song.attributes.name,
                                track.artist,
                                track.title
                            );
                    }
                }

                return None;
            }
            Err(e) => {
                error!(
                    "Error searching for track {} - {}: {:?}",
                    track.artist, track.title, e
                );

                None
            }
        }
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
