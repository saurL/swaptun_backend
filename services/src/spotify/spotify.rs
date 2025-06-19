use crate::dto::{AddTokenRequest, DeleteTokenRequest, UpdateTokenRequest};
use crate::error::AppError;
use crate::{
    get_track_metadata, CreateMusicRequest, CreatePlaylistRequest, MusicService, PlaylistService, SpotifyUrlResponse
};
use futures::StreamExt;
use futures::future::join_all;
use log::{error, info};
use rspotify::model::{PlayableItem, SimplifiedPlaylist};
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

pub struct SpotifyService {
    spotify_code_repository: SpotifyCodeRepository,
    spotify_token_repository: SpotifyTokenRepository,
    playlist_service: PlaylistService,
    music_service: MusicService,
}

impl SpotifyService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            spotify_code_repository: SpotifyCodeRepository::new(db.clone()),
            spotify_token_repository: SpotifyTokenRepository::new(db.clone()),
            playlist_service: PlaylistService::new(db.clone()),
            music_service: MusicService::new(db.clone()),
        }
    }

    pub async fn add_code(&self, code: String, user: &UserModel) -> Result<(), AppError> {
        info!("Adding code");
        match self.get_code(user.clone()).await {
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
                active_model.expires_at = Set(token.expires_at.unwrap().naive_utc());
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
                    expires_at: Set(token.expires_at.unwrap().naive_utc()),
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

    pub async fn get_code(&self, user_model: UserModel) -> Option<SpotifyCodeModel> {
        match self.spotify_code_repository.get_code(user_model).await {
            Ok(code) => Some(code),
            Err(_) => None,
        }
    }

    pub async fn get_authorization_url(&self, port: u16) -> Result<SpotifyUrlResponse, AppError> {
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
        let oauth: OAuth = OAuth {
            redirect_uri: format!("http://127.0.0.1:{}", port),
            scopes: scopes!("playlist-read-private playlist-modify-public playlist-modify-private user-read-email"),
            ..Default::default()
        };

        let spotify = AuthCodeSpotify::new(creds, oauth);

        // Obtaining the access token
        let url = spotify.get_authorize_url(false).unwrap();
        let response = SpotifyUrlResponse { url };
        Ok(response)
    }

    pub async fn get_user_playlists(
        &self,
        user: UserModel,
    ) -> Result<Vec<SimplifiedPlaylist>, AppError> {
        let spotify = self.get_spotify_client_connected(&user).await?;
        match spotify.me().await {
            Ok(spotify_user) => {
                let mut playlist_models = Vec::new();
                let mut import_futures = Vec::new();
                let mut playlists = spotify.user_playlists(spotify_user.id);

                while let Some(playlist_result) = playlists.next().await {
                    if let Ok(playlist) = playlist_result {
                        playlist_models.push(playlist.clone());
                        // Créer la future d'importation sans l'attendre
                        let import_future = self.import_playlist(playlist, &user,&spotify);
                        import_futures.push(import_future);
                    }
                }

                // Attendre que toutes les importations soient terminées et collecter les erreurs
                let import_results = join_all(import_futures).await;

                // Vérifier s'il y a des erreurs d'importation
                for result in import_results {
                    if let Err(e) = result {
                        error!("Error importing playlist: {:?}", e);
                        // Vous pouvez choisir de retourner une erreur ici si vous voulez
                        // return Err(e);
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
        spotify: &AuthCodeSpotify
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
                            
                            let artist_name = track.artists.first().map(|a| a.name.clone()).unwrap_or_default();
                            let track_title = track.name.clone();

                            let genre = match get_track_metadata(&track_title, &artist_name).await {
                                Ok(Some(metadata)) => metadata.genre,
                                Ok(None) => {
                                    info!("Pas de genre trouvé pour {} - {}", artist_name, track_title);
                                    None
                                },
                                Err(e) => {
                                    error!("Erreur lors de la récupération du genre via MusicBrainz: {:?}", e);
                                    None
                                }
                            };

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
            self.playlist_service
                .remove_music(&playlist, &local_track)
                .await?;
        }
        Ok(())
    }

    pub async fn get_spotify_client(&self) -> Result<AuthCodeSpotify, AppError> {
        let creds: Credentials = Credentials::from_env().unwrap();
        let oauth: OAuth = OAuth {
            redirect_uri: format!("http://127.0.0.1:8000"),
            scopes: scopes!("playlist-read-private playlist-modify-public playlist-modify-private"),
            ..Default::default()
        };
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
            let expires_in = token.expires_at.and_utc().signed_duration_since(Utc::now());
            spotify.token = Arc::new(rspotify::sync::Mutex::new(Some(Token {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                expires_at: Some(token.expires_at.and_utc()),
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
}
