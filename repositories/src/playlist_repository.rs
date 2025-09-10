use log::info;
use std::sync::Arc;
use swaptun_models::{
    playlist::PlaylistOrigin, user::SharedPlaylist, MusicEntity, MusicModel, PlaylistActiveModel,
    PlaylistColumn, PlaylistEntity, PlaylistModel, SharedPlaylistActiveModel, SharedPlaylistColumn,
    SharedPlaylistEntity, UserModel,
};

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, DeleteResult,
    EntityTrait, ModelTrait, QueryFilter,
};
pub struct PlaylistRepository {
    db: Arc<DatabaseConnection>,
}

impl PlaylistRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<PlaylistModel>, DbErr> {
        PlaylistEntity::find_by_id(id).one(&*self.db).await
    }

    pub async fn find_by_user(
        &self,
        user: &UserModel,
        origin: Option<PlaylistOrigin>,
    ) -> Result<Vec<PlaylistModel>, DbErr> {
        match origin {
            Some(origin) => {
                user.find_related(PlaylistEntity)
                    .filter(PlaylistColumn::Origin.eq(origin))
                    .all(&*self.db)
                    .await
            }
            None => user.find_related(PlaylistEntity).all(&*self.db).await,
        }
    }

    pub async fn create(&self, model: PlaylistActiveModel) -> Result<PlaylistModel, DbErr> {
        model.insert(&*self.db).await
    }

    pub async fn update(&self, model: PlaylistActiveModel) -> Result<PlaylistModel, DbErr> {
        model.update(&*self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        PlaylistEntity::delete_by_id(id).exec(&*self.db).await
    }

    pub async fn get_music(&self, playist_model: PlaylistModel) -> Result<Vec<MusicModel>, DbErr> {
        playist_model
            .find_related(MusicEntity)
            .all(self.db.as_ref())
            .await
    }

    pub async fn create_shared_link(
        &self,
        user: &UserModel,
        playlist: &PlaylistModel,
    ) -> Result<(), DbErr> {
        if self.is_playlist_shared(user, playlist).await? {
            info!(
                "Playlist {} is already shared with user {}",
                playlist.id, user.id
            );
            return Ok(());
        }
        // Implementation for creating a shared link for the playlist
        SharedPlaylistActiveModel {
            user_id: Set(user.id),
            playlist_id: Set(playlist.id),
            ..Default::default()
        }
        .insert(&*self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_shared_link(
        &self,
        user: &UserModel,
        playlist: &PlaylistModel,
    ) -> Result<DeleteResult, DbErr> {
        SharedPlaylistEntity::delete_many()
            .filter(
                SharedPlaylistColumn::UserId
                    .eq(user.id)
                    .and(SharedPlaylistColumn::PlaylistId.eq(playlist.id)),
            )
            .exec(&*self.db)
            .await
    }

    pub async fn find_shared_playlist(
        &self,
        user: &UserModel,
    ) -> Result<Vec<PlaylistModel>, DbErr> {
        user.find_linked(SharedPlaylist).all(&*self.db).await
    }

    async fn is_playlist_shared(
        &self,
        user: &UserModel,
        playlist: &PlaylistModel,
    ) -> Result<bool, DbErr> {
        let shared_playlists = self.find_shared_playlist(user).await?;
        Ok(shared_playlists
            .iter()
            .any(|shared| shared.id == playlist.id))
    }
}
