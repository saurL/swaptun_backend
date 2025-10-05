use crate::error::AppError;
use crate::{
    music::dto::CreateMusicRequest, CreatePlaylistRequest, MusicService, PlaylistService,
    SpotifyUrlResponse,
};
use crate::{AddTokenRequest, DeleteTokenRequest, UpdateTokenRequest};
use futures::StreamExt;
use log::{error, info};
use rspotify::model::{PlayableId, PlayableItem, SearchResult, SimplifiedPlaylist};
use rspotify::prelude::{BaseClient, OAuthClient};
use sea_orm::IntoActiveModel;
use std::sync::Arc;

use sea_orm::{ActiveValue::Set, DatabaseConnection};
use swaptun_models::{
    PlaylistOrigin, SpotifyCodeActiveModel, SpotifyCodeModel, SpotifyTokenActiveModel,
    SpotifyTokenModel, UserModel,
};
use swaptun_repositories::{
    spotify_code_repository::SpotifyCodeRepository,
    spotify_token_repository::SpotifyTokenRepository,
};

use chrono::{NaiveDate, Utc};
use rspotify::{scopes, AuthCodeSpotify, Credentials, OAuth, Token};

#[derive(Clone)]
pub struct SpotifyService {
    spotify_code_repository: SpotifyCodeRepository,
    spotify_token_repository: SpotifyTokenRepository,
    playlist_service: PlaylistService,
    music_service: MusicService,
    db: Arc<DatabaseConnection>,
}

impl SpotifyService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            spotify_code_repository: SpotifyCodeRepository::new(db.clone()),
            spotify_token_repository: SpotifyTokenRepository::new(db.clone()),
            playlist_service: PlaylistService::new(db.clone()),
            music_service: MusicService::new(db.clone()),
            db,
        }
    }

    pub async fn add_code(&self, code: String, user: &UserModel) -> Result<(), AppError> {
        info!("Adding code");
        match self.get_code(&user).await {
            Some(model) => {
                let mut active_model = model.into_active_model();
                active_model.token = Set(code.clone());
                self.spotify_code_repository
                    .save(active_model.into())
                    .await
                    .map_err(AppError::from)?;
            }
            None => {
                let model = SpotifyCodeActiveModel {
                    user_id: Set(user.id),
                    token: Set(code.clone()),
                    ..Default::default()
                };
                self.spotify_code_repository
                    .save(model)
                    .await
                    .map_err(AppError::from)?;
            }
        }
        Ok(())
    }
    pub async fn add_refresh_token(
        &self,
        user: &UserModel,
        spotify_client: &AuthCodeSpotify,
    ) -> Result<(), AppError> {
        match self.get_token(&user).await {
            Some(token_model) => {
                let arc_token = spotify_client.get_token();
                let guard = arc_token.lock().await.unwrap();
                let token = guard.as_ref().unwrap();
                let mut active_model = token_model.into_active_model();
                active_model.access_token = Set(token.access_token.clone());
                active_model.refresh_token = Set(token.refresh_token.clone());
                active_model.expires_at = Set(token.expires_at.unwrap().into());
                active_model.scope = Set(Some(
                    token
                        .scopes
                        .iter()
                        .next()
                        .unwrap_or(&"".to_string())
                        .to_string(),
                ));
                info!("Saving token dans ok{:?}", active_model);
                self.spotify_token_repository
                    .save(active_model)
                    .await
                    .map_err(AppError::from)?;
            }
            None => {
                let arc_token = spotify_client.get_token();
                let guard = arc_token.lock().await.unwrap();
                let token = guard.as_ref().unwrap();

                let active_model = SpotifyTokenActiveModel {
                    user_id: Set(user.id),
                    access_token: Set(token.access_token.clone()),
                    refresh_token: Set(token.refresh_token.clone()),
                    expires_at: Set(token.expires_at.unwrap().into()),
                    scope: Set(Some(
                        token
                            .scopes
                            .iter()
                            .next()
                            .unwrap_or(&"".to_string())
                            .to_string(),
                    )),
                    ..Default::default()
                };
                info!("Saving token dans error{:?}", active_model);
                self.spotify_token_repository
                    .save(active_model)
                    .await
                    .map_err(AppError::from)?;
            }
        }
        Ok(())
    }
    pub async fn add_token(
        &self,
        request: AddTokenRequest,
        user: UserModel,
    ) -> Result<(), AppError> {
        info!("Requesting token");

        let spotify_client: AuthCodeSpotify = self.get_spotify_client().await?;
        self.add_code(request.token.clone(), &user).await?;
        match spotify_client.request_token(&request.token).await {
            Ok(_) => {
                self.add_refresh_token(&user, &spotify_client).await?;
            }
            Err(e) => {
                error!("Error requesting token {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }
        Ok(())
    }
    pub async fn get_user_code(&self, user_id: i32) -> Result<Option<SpotifyCodeModel>, AppError> {
        self.spotify_code_repository
            .find_by_user_id(user_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_token(&self, request: UpdateTokenRequest) -> Result<(), AppError> {
        let token_id = match self.get_user_code(request.user_id).await? {
            Some(token) => token.id,
            None => return Err(AppError::NotFound("Token doesn't exist".to_string())),
        };
        let model = SpotifyCodeActiveModel {
            id: Set(token_id),
            user_id: Set(request.user_id),
            token: Set(request.new_token),
            ..Default::default()
        };
        self.spotify_code_repository
            .update(model)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    pub async fn delete_token(&self, request: DeleteTokenRequest) -> Result<(), AppError> {
        let user_id: i32 = request.user_id;

        self.spotify_code_repository
            .delete(user_id)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    pub async fn get_code_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<SpotifyCodeModel>, AppError> {
        self.spotify_code_repository
            .find_by_user_id(user_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_code(&self, user_model: &UserModel) -> Option<SpotifyCodeModel> {
        match self.spotify_code_repository.get_code(user_model).await {
            Ok(code) => Some(code),
            Err(_) => None,
        }
    }

    pub async fn get_authorization_url(&self) -> Result<SpotifyUrlResponse, AppError> {
        let creds = Credentials::from_env().unwrap();

        // Same for RSPOTIFY_REDIRECT_URI. You can also set it explictly:
        //
        // ```
        // let oauth = OAuth {
        //     redirect_uri: "http://localhost:8888/callback".to_string(),
        //     scopes: scopes!("user-read-recently-played"),
        //     ..Default::default(),
        // };
        // ```

        let oauth: OAuth = self.get_oauth();

        let spotify = AuthCodeSpotify::new(creds, oauth);

        // Obtaining the access token
        let url = spotify.get_authorize_url(true).unwrap();
        let response = SpotifyUrlResponse { url };
        Ok(response)
    }

    // Collecter les playlists

    pub async fn get_user_playlists(
        &self,
        user: UserModel,
    ) -> Result<Vec<SimplifiedPlaylist>, AppError> {
        let spotify = self.get_spotify_client_connected(&user).await?;
        match spotify.me().await {
            Ok(spotify_user) => {
                let mut playlist_models = Vec::new();
                let mut playlists = spotify.user_playlists(spotify_user.id);

                // Collecter les playlists
                while let Some(playlist_result) = playlists.next().await {
                    if let Ok(playlist) = playlist_result {
                        playlist_models.push(playlist.clone());

                        // Spawn background task to import playlists
                        let service = self.clone();
                        let user_clone = user.clone();

                        tokio::spawn(async move {
                            match service.get_spotify_client_connected(&user_clone).await {
                                Ok(spotify_client) => {
                                    if let Err(e) = service
                                        .import_playlist(
                                            playlist.clone(),
                                            &user_clone,
                                            &spotify_client,
                                        )
                                        .await
                                    {
                                        error!(
                                            "Error importing playlist {}: {:?}",
                                            playlist.name, e
                                        );
                                    } else {
                                        info!("Successfully imported playlist: {}", playlist.name);
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to get Spotify client for background import: {:?}",
                                        e
                                    );
                                }
                            }
                        });
                    }
                }

                Ok(playlist_models)
            }
            Err(e) => {
                error!("Error getting user playlists: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn import_playlist(
        &self,
        playlist: SimplifiedPlaylist,
        user: &UserModel,
        spotify: &AuthCodeSpotify,
    ) -> Result<(), AppError> {
        let mut tracks = spotify.playlist_items(playlist.id.clone(), None, None);
        let request = CreatePlaylistRequest {
            name: playlist.name,
            origin: PlaylistOrigin::Spotify,
            description: None,
            origin_id: playlist.id.to_string(),
        };
        let playlist = self.playlist_service.create_or_get(request, &user).await?;

        let mut local_tracks = self.music_service.find_by_playlist(&playlist).await?;
        while let Some(track) = tracks.next().await {
            if let Ok(track) = track {
                if let Some(track) = track.track {
                    match track {
                        PlayableItem::Track(track) => {
                            // Vérifier si la musique existe déjà dans la playlist
                            if let Some(index) = local_tracks.iter().position(|t| {
                                t.title == track.name
                                    && t.artist
                                        == track.artists.first().map_or("", |a| a.name.as_str())
                                    && t.album == track.album.name
                            }) {
                                info!(
                                    "La musique {} - {} existe déjà dans la playlist",
                                    track.artists.first().map_or("", |a| a.name.as_str()),
                                    track.name
                                );
                                local_tracks.remove(index);
                                continue;
                            };
                            let artist_name = track
                                .artists
                                .first()
                                .map(|a| a.name.clone())
                                .unwrap_or_default();
                            let track_title = track.name.clone();

                            let genre = None;

                            let create_music_request = CreateMusicRequest {
                                title: track_title,
                                release_date: track
                                    .album
                                    .release_date
                                    .unwrap_or_default()
                                    .parse::<NaiveDate>()
                                    .unwrap_or_default(),
                                genre,
                                artist: artist_name,
                                album: track.album.name,
                                description: None,
                            };
                            let music = self.music_service.create(create_music_request).await?;
                            self.playlist_service.add_music(&playlist, music).await?;
                        }

                        _ => {}
                    }
                }
            }
        }
        for local_track in local_tracks {
            info!(
                "La musique {} n'existe plus dans la nouvelle playlist",
                local_track.title
            );
            self.playlist_service
                .remove_music(&playlist, &local_track)
                .await?;
        }

        Ok(())
    }

    pub async fn get_spotify_client(&self) -> Result<AuthCodeSpotify, AppError> {
        let creds: Credentials = Credentials::from_env().unwrap();
        let oauth: OAuth = self.get_oauth();
        let spotify = AuthCodeSpotify::new(creds, oauth);

        Ok(spotify)
    }
    pub async fn get_spotify_client_connected(
        &self,
        user_model: &UserModel,
    ) -> Result<AuthCodeSpotify, AppError> {
        let mut spotify = self.get_spotify_client().await?;
        let token: Option<SpotifyTokenModel> = self.get_token(&user_model).await;
        if let Some(token) = token {
            let expires_in = token.expires_at.signed_duration_since(Utc::now());
            spotify.token = Arc::new(rspotify::sync::Mutex::new(Some(Token {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                expires_at: Some(token.expires_at.into()),
                expires_in: expires_in.clone(),
                scopes: scopes!(
                    "playlist-read-private playlist-modify-public playlist-modify-private user-read-email"
                ),
            })));
            if expires_in.num_seconds() < 0 {
                match spotify.refresh_token().await {
                    Ok(_) => {
                        self.add_refresh_token(&user_model, &spotify).await?;
                    }
                    Err(e) => {
                        error!("Error refreshing token: {:?}", e);
                        return Err(AppError::InternalServerError);
                    }
                }
            }
        } else {
            return Err(AppError::NotFound("Token not found".to_string()));
        }
        Ok(spotify)
    }
    pub async fn get_token(&self, user_model: &UserModel) -> Option<SpotifyTokenModel> {
        match self
            .spotify_token_repository
            .get_token(user_model.clone())
            .await
            .map_err(AppError::from)
        {
            Ok(token) => Some(token),
            Err(_) => None,
        }
    }

    pub fn get_oauth(&self) -> OAuth {
        OAuth {
            redirect_uri: "https://swaptun.com/open/spotify".to_string(),
            scopes: scopes!("playlist-read-private playlist-modify-public playlist-modify-private"),
            ..Default::default()
        }
    }

    /// Creates a playlist on Spotify from a database playlist and adds matching tracks
    pub async fn create_spotify_playlist_from_db(
        &self,
        playlist_id: i32,
        user: &UserModel,
    ) -> Result<String, AppError> {
        // Get the database playlist
        let playlist = self.playlist_service.get_playlist(playlist_id).await?;

        // Get Spotify client
        let spotify = self.get_spotify_client_connected(user).await?;

        // Get user's Spotify ID
        let spotify_user = spotify.me().await.map_err(|e| {
            error!("Error getting Spotify user: {:?}", e);
            AppError::InternalServerError
        })?;

        // Create the playlist on Spotify
        let new_playlist = spotify
            .user_playlist_create(
                spotify_user.id.clone(),
                &playlist.name,
                Some(false), // public
                Some(false), // collaborative
                playlist.description.as_deref(),
            )
            .await
            .map_err(|e| {
                error!("Error creating Spotify playlist: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("Created Spotify playlist with ID: {}", new_playlist.id);

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
            return Ok(new_playlist.id.to_string());
        }

        // Search for tracks on Spotify and collect their IDs
        let mut spotify_track_ids = Vec::new();
        let mut not_found_tracks = Vec::new();

        for track in tracks {
            // Search for the track on Spotify
            let query = format!("track:{} artist:{}", track.title, track.artist);

            match spotify
                .search(
                    &query,
                    rspotify::model::SearchType::Track,
                    None,
                    None,
                    Some(5),
                    None,
                )
                .await
            {
                Ok(SearchResult::Tracks(track_result)) => {
                    for spotify_track in track_result.items.iter().take(5) {
                        if let Some(track_id) = &spotify_track.id {
                            if spotify_track.name.to_lowercase() == track.title.to_lowercase()
                                && spotify_track.artists.first().map(|a| a.name.to_lowercase())
                                    == Some(track.artist.to_lowercase())
                            {
                                spotify_track_ids.push(track_id.clone());
                                info!("Found track on Spotify: {} - {}", track.artist, track.title);
                                break;
                            } else {
                                info!(
                                    "Track found but artist/title does not match: Spotify: {} - {}, DB: {} - {}",
                                    spotify_track.artists.first().map_or("", |a| a.name.as_str()),
                                    spotify_track.name,
                                    track.artist,
                                    track.title
                                );
                            }
                        } else {
                            info!(
                                "Track found but has no ID: {} - {}",
                                track.artist, track.title
                            );
                        }
                    }
                    // Si aucun des 5 premiers ne correspond, on ajoute à not_found_tracks
                    if !spotify_track_ids.iter().any(|id| {
                        track_result
                            .items
                            .iter()
                            .take(5)
                            .any(|t| t.id.as_ref() == Some(id))
                    }) {
                        not_found_tracks.push(format!("{} - {}", track.artist, track.title));
                        info!(
                            "Track not found on Spotify: {} - {}",
                            track.artist, track.title
                        );
                    }
                }
                Ok(_) => {
                    not_found_tracks.push(format!("{} - {}", track.artist, track.title));
                    info!(
                        "Unexpected search result for track: {} - {}",
                        track.artist, track.title
                    );
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

        // Add tracks to the Spotify playlist in batches of 100 (Spotify's limit)
        if !spotify_track_ids.is_empty() {
            for chunk in spotify_track_ids.chunks(100) {
                let track_ids: Vec<PlayableId> = chunk
                    .iter()
                    .map(|id| PlayableId::from(id.clone()))
                    .collect();

                match spotify
                    .playlist_add_items(new_playlist.id.clone(), track_ids, None)
                    .await
                {
                    Ok(_) => {
                        info!("Added {} tracks to Spotify playlist", chunk.len());
                    }
                    Err(e) => {
                        error!("Error adding tracks to Spotify playlist: {:?}", e);
                        return Err(AppError::InternalServerError);
                    }
                }
            }
        }

        // Log tracks that weren't found
        if !not_found_tracks.is_empty() {
            info!("The following tracks were not found on Spotify:");
            for track in &not_found_tracks {
                info!("  - {}", track);
            }
        }

        Ok(new_playlist.id.to_string())
    }

    pub async fn disconnect(&self, user: &UserModel) -> Result<(), AppError> {
        info!("Disconnecting Spotify for user {}", user.id);

        // Delete playlists from Spotify origin
        self.playlist_service
            .delete_by_origin(user, PlaylistOrigin::Spotify)
            .await?;

        // Delete Spotify tokens
        self.spotify_token_repository
            .delete_by_user_id(user.id)
            .await
            .map_err(AppError::from)?;

        // Delete Spotify codes
        self.spotify_code_repository
            .delete_by_user_id(user.id)
            .await
            .map_err(AppError::from)?;

        info!("Spotify disconnected successfully for user {}", user.id);
        Ok(())
    }
}
