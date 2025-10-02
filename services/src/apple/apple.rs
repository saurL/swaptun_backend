use std::sync::Arc;

use apple_music_api::config::ClientConfigBuilder;
use apple_music_api::library::LibraryPlaylistsResponse;
use futures::future::join_all;
use sea_orm::{DatabaseConnection, Set};
use swaptun_models::{playlist, AppleTokenActiveModel, AppleTokenModel, UserModel};
use swaptun_repositories::AppleTokenRepository;

use crate::{error::AppError, GetDeveloperToken};
use crate::{
    AddTokenRequest, CreateMusicRequest, CreatePlaylistRequest, MusicService, PlaylistService,
};
use apple_music_api::{create_developer_token, AppleMusicClient};
use log::{error, info};

pub struct AppleMusicService {
    apple_token_repository: AppleTokenRepository,
    playlist_service: PlaylistService,
    music_service: MusicService,
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
        let token = AppleTokenActiveModel {
            user_id: Set(user_id),
            access_token: Set(request.token),

            ..Default::default()
        };
        self.save(token).await
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
        info!("playlist list of apple music {:?}", playlists.data);
        let mut import_futures = Vec::new();
        // Process playlists sequentially but tracks asynchronously within each playlist
        for playlist in playlists.data {
            let id: String = playlist.id.clone();
            let import_future = self.import_playlist(&user, &client, id);
            import_futures.push(import_future);
        }

        let import_results = join_all(import_futures).await;
        info!("Import results: {:?}", import_results);
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

    pub async fn import_playlist(
        &self,
        user: &UserModel,
        client: &AppleMusicClient,
        playlist_id: String,
    ) -> Result<(), AppError> {
        let playlist = client.get_playlist_with_tracks(&playlist_id).await?;

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

            // Check if track already exists
            if let Some(pos) = local_tracks.iter().position(|local_track| {
                local_track.title == attributes.name
                    && local_track.artist == attributes.artist_name
                    && local_track.album == attributes.album_name
            }) {
                local_tracks.remove(pos);
                return Ok(());
            }

            let create_music_request = CreateMusicRequest {
                title: attributes.name.clone(),
                artist: attributes.artist_name.clone(),
                album: attributes.album_name.clone(),
                description: None,
                genre: attributes.genre_names.get(0).cloned(),
                release_date: attributes.release_date.naive_local().into(),
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
}
