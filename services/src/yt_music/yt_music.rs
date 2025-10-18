use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use swaptun_models::{PlaylistOrigin, UserModel, YoutubeTokenActiveModel, YoutubeTokenModel};
use swaptun_repositories::YoutubeTokenRepository;
use tokio::sync::Mutex;
use ytmapi_rs::{
    auth::OAuthToken,
    common::{AlbumID, VideoID, YoutubeID},
    parse::{GetAlbum, LibraryPlaylist, PlaylistItem, PlaylistSong},
    query::{playlist::PrivacyStatus, CreatePlaylistQuery},
    YtMusic,
};

use crate::AddTokenRequest;
use crate::{
    error::AppError, music::dto::CreateMusicRequest, music::music_service::MusicService,
    playlist::playlist_service::PlaylistService, CreatePlaylistRequest, YoutubeUrlResponse,
};
use apple_music_api::catalog::Artist;
use log::{error, info};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    reqwest, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse, TokenUrl,
};
use std::env::var;
static VERIFIER_STORE: Lazy<Mutex<HashMap<i32, PkceCodeVerifier>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
use ytmapi_rs::query::playlist::GetWatchPlaylistQueryID;

/// Normalize a string for simple comparison (lowercase + alphanumeric only)
fn normalize_string(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Simple permissive match: check if one string contains the other
fn simple_match(str1: &str, str2: &str) -> bool {
    let norm1 = normalize_string(str1);
    let norm2 = normalize_string(str2);

    // Check if one contains the other
    norm1.contains(&norm2) || norm2.contains(&norm1)
}

#[derive(Clone)]
pub struct YoutubeMusicService {
    youtube_token_repository: YoutubeTokenRepository,
    playlist_service: PlaylistService,
    music_service: MusicService,
    db: Arc<DatabaseConnection>,
}

impl YoutubeMusicService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        let youtube_token_repository = YoutubeTokenRepository::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());
        let music_service = MusicService::new(db.clone());

        YoutubeMusicService {
            youtube_token_repository,
            playlist_service,
            music_service,
            db,
        }
    }

    pub async fn get_authorization_url(
        &self,
        user: &UserModel,
    ) -> Result<YoutubeUrlResponse, AppError> {
        let client = self.get_auth_client().await?;

        // Generate a PKCE challenge.
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        VERIFIER_STORE
            .lock()
            .await
            .insert(user.id.clone(), pkce_verifier);

        // Generate the full authorization URL.
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/music".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/userinfo.profile".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/youtube".to_string(),
            ))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();
        let youtube_url_response = YoutubeUrlResponse {
            url: format!(
                "{}{}",
                auth_url.to_string(),
                "&prompt=consent&access_type=offline"
            ),
        };
        Ok(youtube_url_response)
    }

    pub async fn auth_callback(
        &self,
        user: &UserModel,
        req: AddTokenRequest,
    ) -> Result<(), AppError> {
        let verifier = VERIFIER_STORE
            .lock()
            .await
            .remove(&user.id)
            .ok_or(AppError::InternalServerError)?;

        let client = self.get_auth_client().await?;
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");
        let token = match client
            .exchange_code(AuthorizationCode::new(req.token))
            .set_pkce_verifier(verifier)
            .request_async(&http_client)
            .await
        {
            Ok(token) => token,
            Err(e) => {
                error!("Failed to exchange code for token: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
        info!("token acquired {:?}", token);
        let existing_token = self.youtube_token_repository.get_token(user).await?;

        let refresh_token_string = match token.refresh_token() {
            Some(refresh_token) => refresh_token.secret().to_string(),
            None => {
                error!("No refresh token found in the response");
                return Err(AppError::InternalServerError);
            }
        };

        let expires_in_seconds = match token.expires_in() {
            Some(duration) => duration.as_secs() as i64,
            None => 0,
        };

        let mut youtube_active_model = YoutubeTokenActiveModel {
            user_id: Set(user.id),
            access_token: Set(token.access_token().secret().to_string()),
            refresh_token: Set(refresh_token_string),
            expires_in: Set(expires_in_seconds),
            ..Default::default()
        };

        if let Some(db_token) = existing_token {
            youtube_active_model.id = Set(db_token.id);
        }
        self.save(youtube_active_model).await?;

        info!("Successfully saved token for user {}", user.id);

        self.get_user_playlists(&user).await?;

        Ok(())
    }

    pub async fn save(
        &self,
        youtube_active_model: YoutubeTokenActiveModel,
    ) -> Result<(), AppError> {
        self.youtube_token_repository
            .save(youtube_active_model)
            .await
            .map_err(|e| {
                error!("Failed to save youtube token: {:?}", e);
                AppError::InternalServerError
            })?;
        Ok(())
    }

    pub async fn get_token(&self, user: &UserModel) -> Result<Option<YoutubeTokenModel>, AppError> {
        match self.youtube_token_repository.get_token(user).await? {
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

    pub async fn create_token(
        &self,
        youtube_token: YoutubeTokenActiveModel,
        user: &UserModel,
    ) -> Result<YoutubeTokenModel, AppError> {
        if let Some(token) = self.get_token(user).await? {
            return Ok(token);
        }
        self.youtube_token_repository
            .create(youtube_token)
            .await
            .map_err(|e| {
                error!("Failed to create youtube token: {:?}", e);
                AppError::InternalServerError
            })
    }

    pub fn get_client_id(&self) -> Result<String, AppError> {
        var("YOUTUI_OAUTH_CLIENT_ID").map_err(|e| {
            error!(
                "Failed to get YOUTUI_OAUTH_CLIENT_ID from environment: {}",
                e
            );
            AppError::InternalServerError
        })
    }

    pub fn get_client_secret(&self) -> Result<String, AppError> {
        var("YOUTUI_OAUTH_CLIENT_SECRET").map_err(|e| {
            error!(
                "Failed to get YOUTUI_OAUTH_CLIENT_SECRET from environment: {}",
                e
            );
            AppError::InternalServerError
        })
    }

    pub async fn get_auth_client(
        &self,
    ) -> Result<
        Client<
            StandardErrorResponse<BasicErrorResponseType>,
            StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
            StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
            StandardRevocableToken,
            StandardErrorResponse<RevocationErrorResponseType>,
            EndpointSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointSet,
        >,
        AppError,
    > {
        let client_id = self.get_client_id()?;
        let client_secret = self.get_client_secret()?;
        let client: Client<
            StandardErrorResponse<BasicErrorResponseType>,
            StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
            StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
            StandardRevocableToken,
            StandardErrorResponse<RevocationErrorResponseType>,
            EndpointSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointSet,
        > = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(AuthUrl::new(
                "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            )?)
            .set_token_uri(TokenUrl::new(
                "https://oauth2.googleapis.com/token".to_string(),
            )?)
            .set_redirect_uri(RedirectUrl::new(
                "https://swaptun.com/open/youtube".to_string(),
            )?);
        Ok(client)
    }

    pub async fn get_ytmusic_client(
        &self,
        user: &UserModel,
    ) -> Result<YtMusic<OAuthToken>, AppError> {
        let token_model = match self.get_token(user).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                error!("No YouTube token found for user {}", user.id);
                return Err(AppError::NotFound("YouTube token not found".to_string()));
            }
            Err(e) => {
                error!("Failed to get YouTube token for user {}: {:?}", user.id, e);
                return Err(AppError::InternalServerError);
            }
        };
        let client_id = self.get_client_id()?;
        let client_secret = self.get_client_secret()?;
        info!("token model : {:?}", token_model);
        let token = OAuthToken::new(
            "Bearer".to_string(),
            token_model.access_token.clone(),
            token_model.refresh_token.clone(),
            token_model.expires_in.clone() as usize,
            token_model.updated_on.clone().into(),
            client_id,
            client_secret,
        );
        let mut client = YtMusic::from_auth_token(token);

        let token = match client.refresh_token().await {
            Ok(token) => token,
            Err(e) => {
                error!(
                    "Failed to refresh YouTube token for user {}: {:?}",
                    user.id, e
                );
                return Err(AppError::InternalServerError);
            }
        };
        let mut active_token: YoutubeTokenActiveModel = token_model.into_active_model();
        active_token.access_token = Set(token.access_token);
        active_token.refresh_token = Set(token.refresh_token);
        active_token.expires_in = Set(token.expires_in as i64);
        active_token.updated_on = Set(chrono::Utc::now().into());
        self.save(active_token).await?;
        Ok(client)
    }

    pub async fn get_user_playlists(&self, user: &UserModel) -> Result<(), AppError> {
        let client = self.get_ytmusic_client(user).await?;
        let playlists = client.get_library_playlists().await.map_err(|e| {
            error!("Failed to get library playlists: {:?}", e);
            AppError::InternalServerError
        })?;

        info!("Found {} YouTube Music playlists", playlists.len());

        // Spawn background task to import playlists
        let service = self.clone();
        let user_clone = user.clone();
        let playlists_clone = playlists.clone();

        tokio::spawn(async move {
            info!(
                "Starting background import of {} YouTube Music playlists",
                playlists_clone.len()
            );

            match service.get_ytmusic_client(&user_clone).await {
                Ok(client) => {
                    for playlist in playlists_clone {
                        if let Err(e) = service
                            .import_playlist(&playlist, &user_clone, &client)
                            .await
                        {
                            error!(
                                "Error importing YouTube Music playlist {:?}: {:?}",
                                playlist.title, e
                            );
                        } else {
                            info!(
                                "Successfully imported YouTube Music playlist: {:?}",
                                playlist.title
                            );
                        }
                    }
                    info!("Background import of YouTube Music playlists completed");
                }
                Err(e) => {
                    error!(
                        "Failed to get YouTube Music client for background import: {:?}",
                        e
                    );
                }
            }
        });

        Ok(())
    }

    pub async fn get_playlist_track(
        &self,
        playlist: &LibraryPlaylist,
        client: &YtMusic<OAuthToken>,
    ) -> Result<Vec<PlaylistSong>, AppError> {
        let tracks: Vec<PlaylistItem> = match client
            .get_playlist_tracks(playlist.playlist_id.clone())
            .await
        {
            Ok(tracks) => tracks,
            Err(e) => {
                error!("Failed to load playlist {:?}: {}", playlist.playlist_id, e);
                return Err(AppError::InternalServerError);
            }
        };
        let songs: Vec<PlaylistSong> = tracks
            .into_iter()
            .filter_map(|track| {
                if let PlaylistItem::Song(song) = track {
                    Some(song)
                } else {
                    None
                }
            })
            .collect();

        info!("Filtered songs: {:?}", songs);
        Ok(songs)
    }

    pub async fn import_playlist(
        &self,
        playlist: &LibraryPlaylist,
        user: &UserModel,
        client: &YtMusic<OAuthToken>,
    ) -> Result<(), AppError> {
        let request = CreatePlaylistRequest {
            name: playlist.title.clone(),
            origin: PlaylistOrigin::YoutubeMusic,
            description: None,
            origin_id: playlist.playlist_id.get_playlist_id().to_string(),
            image_url: None, // YouTube Music LibraryPlaylist doesn't expose image URLs
        };
        let tracks = self.get_playlist_track(playlist, client).await?;
        if tracks.is_empty() {
            info!("No tracks found in playlist: {}", playlist.title);
            return Ok(());
        }
        let playlist_model = self.playlist_service.create_or_get(request, &user).await?;

        let mut local_tracks = self.music_service.find_by_playlist(&playlist_model).await?;

        for track in tracks {
            let artist = match track.artists.clone().first() {
                Some(artist) => artist.name.clone(),
                None => {
                    continue; // Skip if no artist is found
                }
            };
            if let Some(pos) = local_tracks.iter().position(|local_track| {
                local_track.title == track.title
                    && local_track.artist == artist
                    && local_track.album == track.album.name.clone()
            }) {
                local_tracks.remove(pos);
            }
            let album_info = self.get_album_info(&client, &track.album.id).await?;

            let create_music_request = CreateMusicRequest {
                title: track.title,
                release_date: chrono::NaiveDate::from_ymd_opt(
                    album_info.year.parse::<i32>().unwrap_or_default(),
                    1,
                    1,
                )
                .unwrap_or_default(),
                genre: None,
                artist: artist,
                album: track.album.name,
                description: None,
            };
            let music = self.music_service.create(create_music_request).await?;
            self.playlist_service
                .add_music(&playlist_model, music)
                .await?;
        }
        info!("Tracks to remove: {:?}", local_tracks);

        for local_track in local_tracks {
            info!("Removing track from playlist: {:?}", local_track);
            self.playlist_service
                .remove_music(&playlist_model, &local_track)
                .await?;
        }
        Ok(())
    }

    pub async fn get_album_info<'a>(
        &self,
        client: &'a YtMusic<OAuthToken>,
        album_id: &'a AlbumID<'a>,
    ) -> Result<GetAlbum, AppError> {
        match client.get_album(album_id).await {
            Ok(album) => {
                info!("Album info: {:?}", album);
                Ok(album)
            }
            Err(e) => {
                error!(
                    "Failed to get album info for {}: {:?}",
                    album_id.get_raw(),
                    e
                );
                return Err(AppError::InternalServerError);
            }
        }
    }
    pub async fn import_playlist_in_yt(
        &self,
        user: &UserModel,
        playlist_id: i32,
    ) -> Result<String, AppError> {
        let client = match self.get_ytmusic_client(user).await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get YouTube Music client: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let playlist = self.playlist_service.get_playlist(playlist_id).await?;
        let tracks = self.music_service.find_by_playlist(&playlist).await?;

        if tracks.is_empty() {
            info!("No tracks in playlist, returning early");
            return Ok(String::new());
        }

        let mut youtube_tracks_id = Vec::new();

        for track in tracks {
            match client.search_songs(track.title.clone()).await {
                Ok(youtube_tracks) => {
                    let mut found = false;
                    for youtube_track in youtube_tracks {
                        // Simple permissive matching: check if DB name is included in YouTube name or vice versa
                        let title_matches = simple_match(&youtube_track.title, &track.title);
                        let artist_matches = simple_match(&youtube_track.artist, &track.artist);

                        if title_matches && artist_matches {
                            info!(
                                "Found track on YouTube Music: {} - {} (matched with: {} - {})",
                                track.artist,
                                track.title,
                                youtube_track.artist,
                                youtube_track.title
                            );
                            youtube_tracks_id.push(youtube_track);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        info!(
                            "Track not found on YouTube Music: {} - {}",
                            track.artist, track.title
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to search for track {}: {:?}", track.title, e);
                    continue;
                }
            }
        }

        let query = CreatePlaylistQuery::new(&playlist.name, None, PrivacyStatus::Public);
        let yt_playlist_id = match client.create_playlist(query).await {
            Ok(playlist_id) => {
                info!("Created YouTube playlist: {:?}", playlist_id);
                playlist_id
            }
            Err(e) => {
                error!("Failed to create YouTube playlist: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
        let video_ids: Vec<VideoID> = youtube_tracks_id
            .into_iter()
            .map(|track| track.video_id)
            .collect();
        match client
            .add_video_items_to_playlist(yt_playlist_id.clone(), video_ids)
            .await
        {
            Ok(_) => {
                info!("Successfully added tracks to YouTube playlist: {:?}", yt_playlist_id);
                Ok(yt_playlist_id.get_raw().to_string())
            },
            Err(e) => {
                error!("Failed to add video items to playlist: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn disconnect(&self, user: &UserModel) -> Result<(), AppError> {
        info!("Disconnecting YouTube Music for user {}", user.id);

        // Delete playlists from YouTube Music origin
        self.playlist_service
            .delete_by_origin(user, PlaylistOrigin::YoutubeMusic)
            .await?;

        // Delete YouTube Music tokens
        self.youtube_token_repository
            .delete_by_user_id(user.id)
            .await
            .map_err(|e| {
                error!("Failed to delete youtube token: {:?}", e);
                AppError::InternalServerError
            })?;

        info!(
            "YouTube Music disconnected successfully for user {}",
            user.id
        );
        Ok(())
    }
}
