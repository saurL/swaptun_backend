use std::sync::Arc;

use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::DeleteResult;
use sea_orm::IntoActiveModel;
use swaptun_models::MusicModel;
use swaptun_models::PlaylistActiveModel;
use swaptun_models::music_playlist;
use swaptun_models::{PlaylistModel, UserModel};
use swaptun_repositories::MusicPlaylistRepository;
use swaptun_repositories::PlaylistRepository;

use crate::CreatePlaylistRequest;
use crate::DeletePlaylistRequest;
use crate::UpdatePlaylistRequest;
use crate::error::AppError;

pub struct PlaylistService {
    pub playlist_repository: PlaylistRepository,
    pub music_playlist_repository: MusicPlaylistRepository,
}

impl PlaylistService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            playlist_repository: PlaylistRepository::new(db.clone()),
            music_playlist_repository: MusicPlaylistRepository::new(db),
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
        user_model: UserModel,
    ) -> Result<Vec<PlaylistModel>, DbErr> {
        self.playlist_repository.find_by_user(user_model).await
    }

    pub async fn create(
        &self,
        request: CreatePlaylistRequest,
        user_id: i32,
    ) -> Result<(), AppError> {
        let model = PlaylistActiveModel {
            name: sea_orm::ActiveValue::Set(request.name),
            description: sea_orm::ActiveValue::Set(request.description),
            user_id: sea_orm::ActiveValue::Set(user_id),
            ..Default::default()
        };

        match self._create(model).await {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::InternalServerError),
        }
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
        playlist: PlaylistModel,
        music: MusicModel,
    ) -> Result<(), AppError> {
        if let Some(_) = self
            .music_playlist_repository
            .find_by_relation(playlist.clone(), music.clone())
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
            Err(_) => Err(AppError::InternalServerError),
        }
    }
    pub async fn remove_music(
        &self,
        playlist: PlaylistModel,
        music: MusicModel,
    ) -> Result<(), AppError> {
        if let Some(_) = self
            .music_playlist_repository
            .find_by_relation(playlist.clone(), music.clone())
            .await?
        {
            match self
                .music_playlist_repository
                .delete_by_relation(playlist, music)
                .await
            {
                Ok(_) => Ok(()),
                Err(_) => Err(AppError::InternalServerError),
            }
        } else {
            Err(AppError::NotFound(format!(
                "Music not found in playlist with id {}",
                playlist.id
            )))
        }
    }
}
