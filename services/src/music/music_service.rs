use std::sync::Arc;

use crate::CreateMusicRequest;
use crate::UpdateMusicRequest;
use sea_orm::ActiveValue;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::DeleteResult;
use sea_orm::IntoActiveModel;
use swaptun_models::{MusicActiveModel, MusicModel, PlaylistModel};
use swaptun_repositories::MusicRepository;

pub struct MusicService {
    pub music_repository: MusicRepository,
}

impl MusicService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            music_repository: MusicRepository::new(db),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<MusicModel>, DbErr> {
        self.music_repository.find_all().await
    }

    pub async fn find_by_id(
        &self,
        name: String,
        artist: String,
        album: String,
    ) -> Result<Option<MusicModel>, DbErr> {
        self.music_repository.find_by_id(name, artist, album).await
    }

    pub async fn _create(&self, model: MusicActiveModel) -> Result<MusicModel, DbErr> {
        self.music_repository.create(model).await
    }

    pub async fn _update(&self, model: MusicActiveModel) -> Result<MusicModel, DbErr> {
        self.music_repository.update(model).await
    }

    pub async fn delete(
        &self,
        name: String,
        artist: String,
        album: String,
    ) -> Result<DeleteResult, DbErr> {
        self.music_repository.delete(name, artist, album).await
    }

    pub async fn find_by_playlist(
        &self,
        playlist_model: &PlaylistModel,
    ) -> Result<Vec<MusicModel>, DbErr> {
        self.music_repository.find_by_playlist(playlist_model).await
    }

    pub async fn create(&self, request: CreateMusicRequest) -> Result<MusicModel, DbErr> {
        if let Some(music_model) = self
            .music_repository
            .find_by_id(
                request.title.clone(),
                request.artist.clone(),
                request.album.clone(),
            )
            .await?
        {
            return Ok(music_model);
        }
        let new_music = MusicActiveModel {
            title: ActiveValue::Set(request.title),
            artist: ActiveValue::Set(request.artist),
            album: ActiveValue::Set(request.album),
            release_date: ActiveValue::Set(request.release_date),
            genre: ActiveValue::Set(request.genre),

            ..Default::default()
        };

        self.music_repository.create(new_music).await
    }

    pub async fn update(&self, request: UpdateMusicRequest) -> Result<MusicModel, DbErr> {
        let mut existing_music = self
            .music_repository
            .find_by_id(request.title, request.artist, request.album)
            .await?
            .ok_or(DbErr::Custom("Music not found".to_string()))?;

        if let Some(release_date) = request.release_date {
            existing_music.release_date = release_date;
        }
        if let Some(genre) = request.genre {
            existing_music.genre = Some(genre);
        }

        self.music_repository
            .update(existing_music.into_active_model())
            .await
    }
}
