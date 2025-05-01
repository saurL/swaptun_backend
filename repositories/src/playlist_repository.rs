use std::sync::Arc;
use swaptun_models::{
    MusicEntity, MusicModel, PlaylistActiveModel, PlaylistEntity, PlaylistModel, UserModel,
};

use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, ModelTrait};
pub struct PlaylistRepository {
    db: Arc<DatabaseConnection>,
}

impl PlaylistRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<PlaylistModel>, DbErr> {
        PlaylistEntity::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn create(&self, playlist: PlaylistActiveModel) -> Result<PlaylistModel, DbErr> {
        playlist.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, playlist: PlaylistActiveModel) -> Result<PlaylistModel, DbErr> {
        playlist.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        PlaylistEntity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
    }

    pub async fn find_by_user(&self, user_model: UserModel) -> Result<Vec<PlaylistModel>, DbErr> {
        user_model
            .find_related(PlaylistEntity)
            .all(self.db.as_ref())
            .await
    }

    pub async fn get_music(&self, playist_model: PlaylistModel) -> Result<Vec<MusicModel>, DbErr> {
        playist_model
            .find_related(MusicEntity)
            .all(self.db.as_ref())
            .await
    }
}
