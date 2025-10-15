use std::sync::Arc;

use super::{
    CreatePlaylistRequest, DeletePlaylistRequest, GetPlaylistMusicsResponse,
    GetPlaylistResponse, GetPlaylistsParams, PlaylistWithMusics,
    UpdatePlaylistRequest,
};
use crate::{error::AppError, SharedPlaylist, SharedPlaylistsResponse, UserInfo};

use log::error;
use sea_orm::{DatabaseConnection, DbErr, DeleteResult, IntoActiveModel};
use swaptun_models::{
    music_playlist, playlist::PlaylistOrigin, MusicModel, PlaylistActiveModel, PlaylistModel,
    UserModel,
};
use swaptun_repositories::{MusicPlaylistRepository, MusicRepository, PlaylistRepository};

#[derive(Clone)]
pub struct PlaylistService {
    pub playlist_repository: PlaylistRepository,
    pub music_playlist_repository: MusicPlaylistRepository,
    pub music_repository: MusicRepository,
}

impl PlaylistService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            playlist_repository: PlaylistRepository::new(db.clone()),
            music_playlist_repository: MusicPlaylistRepository::new(db.clone()),
            music_repository: MusicRepository::new(db),
        }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<PlaylistModel>, DbErr> {
        self.playlist_repository.find_by_id(id).await
    }

    async fn _create(&self, model: PlaylistActiveModel) -> Result<PlaylistModel, DbErr> {
        self.playlist_repository.create(model).await
    }

    async fn _update(&self, model: PlaylistActiveModel) -> Result<PlaylistModel, DbErr> {
        self.playlist_repository.update(model).await
    }

    async fn _delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        self.playlist_repository.delete(id).await
    }

    pub async fn get_user_playlist(
        &self,
        user: UserModel,
        params: GetPlaylistsParams,
    ) -> Result<GetPlaylistResponse, AppError> {
        let playlists = self
            .playlist_repository
            .find_by_user(&user, params.origin)
            .await?;

        let playlists_with_musics = if params.include_musics {
            let mut result = Vec::new();
            for playlist in playlists {
                let musics = self.music_repository.find_by_playlist(&playlist).await?;
                result.push(PlaylistWithMusics {
                    playlist,
                    musics: Some(musics),
                });
            }
            result
        } else {
            playlists
                .into_iter()
                .map(|playlist| PlaylistWithMusics {
                    playlist,
                    musics: None,
                })
                .collect()
        };

        Ok(GetPlaylistResponse {
            playlists: playlists_with_musics,
        })
    }


    pub async fn get_playlist_musics(
        &self,
        playlist_id: i32,
    ) -> Result<GetPlaylistMusicsResponse, AppError> {
        let playlist = self.get_playlist(playlist_id).await?;
        let musics = self.music_repository.find_by_playlist(&playlist).await?;

        Ok(GetPlaylistMusicsResponse {
            playlist_id,
            musics,
        })
    }

    pub async fn get_shared_playlists(
        &self,
        user: UserModel,
    ) -> Result<GetPlaylistResponse, AppError> {
        let playlists = self
            .playlist_repository
            .find_shared_playlist(&user)
            .await?;

        let playlists_without_musics = playlists
            .into_iter()
            .map(|playlist| PlaylistWithMusics {
                playlist,
                musics: None,
            })
            .collect();

        Ok(GetPlaylistResponse {
            playlists: playlists_without_musics,
        })
    }

    pub async fn get_shared_playlists_with_details(
        &self,
        user: &UserModel,
    ) -> Result<SharedPlaylistsResponse, AppError> {
        let details = self
            .playlist_repository
            .find_shared_playlist_with_details(user)
            .await?;

        let shared_playlists = details
            .into_iter()
            .map(|(shared, playlist, shared_by)| SharedPlaylist {
                id: shared.id,
                playlist,
                shared_by: shared_by.into(),
                shared_at: shared.created_on.into(),
            })
            .collect();

        Ok(SharedPlaylistsResponse { shared_playlists })
    }

    pub async fn create(
        &self,
        request: CreatePlaylistRequest,
        user_id: i32,
    ) -> Result<PlaylistModel, AppError> {
        let model = PlaylistActiveModel {
            name: sea_orm::ActiveValue::Set(request.name),
            description: sea_orm::ActiveValue::Set(request.description),
            user_id: sea_orm::ActiveValue::Set(user_id),
            origin: sea_orm::ActiveValue::Set(request.origin),
            origin_id: sea_orm::ActiveValue::Set(request.origin_id),
            ..Default::default()
        };

        match self._create(model).await {
            Ok(playlist) => Ok(playlist),
            Err(e) => {
                error!("Error creating playlist: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
    pub async fn create_or_get(
        &self,
        request: CreatePlaylistRequest,
        user: &UserModel,
    ) -> Result<PlaylistModel, AppError> {
        if let Some(playlist) = self
            .playlist_repository
            .find_by_user(&user, None)
            .await?
            .into_iter()
            .find(|p| p.origin_id == request.origin_id)
        {
            return Ok(playlist);
        }

        self.create(request, user.id).await
    }

    pub async fn get_playlist(&self, id: i32) -> Result<PlaylistModel, AppError> {
        match self.find_by_id(id).await {
            Ok(Some(playlist)) => Ok(playlist),
            Ok(None) => Err(AppError::NotFound(format!(
                "Playlist with id {} not found",
                id
            ))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update(
        &self,
        request: UpdatePlaylistRequest,
        user_id: i32,
    ) -> Result<(), AppError> {
        let playlist_id = request.playlist_id;
        let mut playlist = match self.find_by_id(playlist_id).await? {
            Some(p) => {
                if p.user_id != user_id {
                    return Err(AppError::Unauthorized(
                        "You do not have permission to update this playlist".to_string(),
                    ));
                }
                p.into_active_model()
            }
            None => {
                return Err(AppError::NotFound(format!(
                    "Playlist with id {} not found",
                    playlist_id
                )));
            }
        };

        if let Some(name) = request.name {
            playlist.name = sea_orm::ActiveValue::Set(name);
        }

        if let Some(description) = request.description {
            playlist.description = sea_orm::ActiveValue::Set(Some(description));
        }

        match self._update(playlist).await {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete(
        &self,
        request: DeletePlaylistRequest,
        user_id: i32,
    ) -> Result<(), AppError> {
        let playlist_id = request.id;
        match self.find_by_id(playlist_id).await? {
            Some(playlist) => {
                if playlist.user_id != user_id {
                    return Err(AppError::Unauthorized(
                        "You do not have permission to delete this playlist".to_string(),
                    ));
                }
                match self._delete(playlist_id).await {
                    Ok(_) => Ok(()),
                    Err(_) => Err(AppError::InternalServerError),
                }
            }
            None => Err(AppError::NotFound(format!(
                "Playlist with id {} not found",
                playlist_id
            ))),
        }
    }

    pub async fn add_music(
        &self,
        playlist: &PlaylistModel,
        music: MusicModel,
    ) -> Result<(), AppError> {
        if let Some(_) = self
            .music_playlist_repository
            .find_relation(playlist.clone(), music.clone())
            .await?
        {
            return Ok(());
        }
        let music_playlist = music_playlist::ActiveModel {
            playlist_id: sea_orm::ActiveValue::Set(playlist.id),
            music_album: sea_orm::ActiveValue::Set(music.album),
            music_artist: sea_orm::ActiveValue::Set(music.artist),
            music_title: sea_orm::ActiveValue::Set(music.title),
            ..Default::default()
        };

        match self.music_playlist_repository.create(music_playlist).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error adding music to playlist {}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
    pub async fn remove_music(
        &self,
        playlist: &PlaylistModel,
        music: &MusicModel,
    ) -> Result<(), AppError> {
        if let Some(_) = self
            .music_playlist_repository
            .find_relation(playlist.clone(), music.clone())
            .await?
        {
            match self
                .music_playlist_repository
                .delete_relation(playlist, music)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Error removing music from playlist: {:?}", e);
                    Err(AppError::InternalServerError)
                }
            }
        } else {
            Err(AppError::NotFound(format!(
                "Music {} not found in playlist with id {}",
                music.title, playlist.id
            )))
        }
    }

    pub async fn share_playlist(
        &self,
        shared_with_user: &UserModel,
        playlist: &PlaylistModel,
        shared_by_user: &UserModel,
    ) -> Result<(), AppError> {
        self.playlist_repository
            .create_shared_link(shared_with_user, playlist, shared_by_user)
            .await?;
        Ok(())
    }

    pub async fn unshare_playlist(
        &self,
        user: &UserModel,
        playlist: &PlaylistModel,
    ) -> Result<(), AppError> {
        match self
            .playlist_repository
            .delete_shared_link(user, playlist)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error unsharing playlist: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_by_origin(
        &self,
        user: &UserModel,
        origin: PlaylistOrigin,
    ) -> Result<(), AppError> {
        self.playlist_repository
            .delete_by_user_and_origin(user, origin)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }
}
