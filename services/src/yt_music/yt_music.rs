use std::{collections::HashMap, sync::Arc};

use futures::future::join_all;
use once_cell::sync::Lazy;
use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use swaptun_models::{
    PlaylistOrigin, UserModel, YoutubeTokenActiveModel, YoutubeTokenModel,
};
use swaptun_repositories::YoutubeTokenRepository;
use tokio::sync::Mutex;
use ytmapi_rs::{
    auth::OAuthToken, common::{AlbumID, YoutubeID}, parse::{GetAlbum, LibraryPlaylist, PlaylistItem, PlaylistSong}, query::watch_playlist::GetWatchPlaylistQueryID, YtMusic
};

use crate::{
    dto::{CreateMusicRequest, CreatePlaylistRequest},
    error::AppError,
    music::music_service::MusicService,
    playlist::playlist_service::PlaylistService,
    YoutubeUrlResponse,
};
use log::{error, info};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType}, reqwest, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields, EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse, TokenUrl
};
use std::env::var;

use crate::AddTokenRequest;
static VERIFIER_STORE: Lazy<Mutex<HashMap<i32, PkceCodeVerifier>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct YoutubeMusicService {
    youtube_token_repository: YoutubeTokenRepository,
    playlist_service: PlaylistService,
    music_service: MusicService,
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
               url: format!("{}{}",auth_url.to_string(), "&prompt=consent&access_type=offline"),
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
            .await{
            Ok(token) => token,
            Err(e) => {
                error!("Failed to exchange code for token: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };
        info!("token acquired {:?}", token);
        let existing_token = self
            .youtube_token_repository
            .get_token(user)
            .await?;

        let refresh_token_string = match token
            .refresh_token(){
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

    pub async fn  save(&self, youtube_active_model: YoutubeTokenActiveModel) -> Result<(), AppError> {
        self.youtube_token_repository
            .save(youtube_active_model)
            .await
            .map_err(|e| {
                error!("Failed to save youtube token: {:?}", e);
                AppError::InternalServerError
            })?;
            Ok(())
    }

    pub async fn get_token(
        &self,
        user: &UserModel,
    ) -> Result<Option<YoutubeTokenModel>, AppError> {
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

    pub async fn create_token(&self, youtube_token: YoutubeTokenActiveModel,user: &UserModel    ) -> Result<YoutubeTokenModel, AppError> {
        if let Some(token) = self.get_token(user) .await? {
            return Ok(token);
        }
        self.youtube_token_repository.create(youtube_token).await.map_err(|e| {
            error!("Failed to create youtube token: {:?}", e);
            AppError::InternalServerError
        })
    }

    pub fn get_client_id(&self) -> Result<String, AppError> {
        var("YOUTUI_OAUTH_CLIENT_ID").map_err(|e| {
            error!("Failed to get YOUTUI_OAUTH_CLIENT_ID from environment: {}", e);
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

    pub async fn get_auth_client(&self) -> Result<Client<StandardErrorResponse<BasicErrorResponseType>, StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>, StandardRevocableToken, StandardErrorResponse<RevocationErrorResponseType>, EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> , AppError> {
        let client_id = self.get_client_id()?;
        let client_secret = self.get_client_secret()?;
        let client: Client<StandardErrorResponse<BasicErrorResponseType>, StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>, StandardRevocableToken, StandardErrorResponse<RevocationErrorResponseType>, EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> = BasicClient::new(ClientId::new(client_id))
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
        let token_model = match self.get_token(user).await{
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
        let token = OAuthToken::new("Bearer".to_string(), token_model.access_token.clone(), token_model.refresh_token.clone(), token_model.expires_in.clone() as usize     , token_model.updated_on.clone().and_utc().into(), client_id, client_secret);
        let mut client = YtMusic::from_auth_token(token);

        let token = match client.refresh_token().await {
    
                    Ok(token) => token,
            Err(e) => {
                error!("Failed to refresh YouTube token for user {}: {:?}", user.id, e);
                return Err(AppError::InternalServerError);
            }
        };
        let mut active_token: YoutubeTokenActiveModel = token_model.into_active_model();
        active_token.access_token = Set(token.access_token);
        active_token.refresh_token = Set(token.refresh_token);
        active_token.expires_in = Set(token.expires_in as i64);  
        active_token.updated_on = Set(chrono::Utc::now().naive_utc());
        Ok(client)
    }

    pub async fn get_user_playlists(
        &self,
        user: &UserModel,
    ) -> Result<(), AppError> {
        let client = self.get_ytmusic_client(user).await?;
        let playlists = client.get_library_playlists().await.map_err(|e| {
            error!("Failed to get library playlists: {:?}", e);
            AppError::InternalServerError
        })?;
        let mut import_futures = Vec::new();

        for playlist in &playlists {
            let import_future =self.import_playlist(playlist, user, &client);
            import_futures.push(import_future);
        }

        let import_results = join_all(import_futures).await;

                // Vérifier s'il y a des erreurs d'importation
                for result in import_results {
                    if let Err(e) = result {
                        error!("Error importing playlist: {:?}", e);
                        // Vous pouvez choisir de retourner une erreur ici si vous voulez
                        // return Err(e);
                    }
                }

        Ok(())
    }



    pub async fn get_playlist_track(&self, playlist: &LibraryPlaylist,client: &YtMusic<OAuthToken>) -> Result<Vec<PlaylistSong> , AppError> {
        let playlist = match client.get_playlist(playlist.playlist_id.clone()).await {
            Ok(playlist) => playlist,
            Err(e) => {
                error!("Failed to load playlist {:?}: {}", playlist.playlist_id, e);
                return Err(AppError::InternalServerError);
            }
        };
        let tracks = playlist.tracks;
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
        };
        let playlist_model = self.playlist_service.create_or_get(request, &user).await?;
        let tracks = self.get_playlist_track(playlist, client).await?;
        let mut local_tracks = self
            .music_service
            .find_by_playlist(&playlist_model)
            .await?;

        for track in tracks {
            if let Some(pos) = local_tracks.iter().position(|local_track| {
                local_track.title == track.title
                    && local_track.artist
                        == track
                            .artists
                            .clone()
                            .first()
                            .unwrap()
                            .name
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
                artist: track
                    .artists
                    .clone()
                    .first()
                    .unwrap()
                    .name
                    .clone(),
                album: track.album.name,
                description: None,
            };
            let music = self.music_service.create(create_music_request).await?;
            self.playlist_service
                .add_music(&playlist_model, music)
                .await?;
        }

        for local_track in local_tracks {
            self.playlist_service
                .remove_music(&playlist_model, &local_track)
                .await?;
        }
        Ok(())
    }

    pub async fn get_album_info<'a>(&self, client: &'a YtMusic<OAuthToken>, album_id: &'a AlbumID<'a>) -> Result<GetAlbum, AppError> {
        match client.get_album(album_id).await{
            Ok(album) => {
                info!("Album info: {:?}", album);
                Ok(album)
            }
            Err(e) => {
                error!("Failed to get album info for {}: {:?}", album_id.get_raw(), e);
                return Err(AppError::InternalServerError);
            }
        }

    }
}