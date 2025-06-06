use std::sync::Arc;
use swaptun_models::{MusicActiveModel, MusicEntity, MusicModel, PlaylistModel};

use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, ModelTrait};
pub struct MusicRepository {
    db: Arc<DatabaseConnection>,
}

impl MusicRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(
        &self,
        name: String,
        artist: String,
        album: String,
    ) -> Result<Option<MusicModel>, DbErr> {
        MusicEntity::find_by_id((name, artist, album))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_all(&self) -> Result<Vec<MusicModel>, DbErr> {
        MusicEntity::find().all(self.db.as_ref()).await
    }

    pub async fn create(&self, music: MusicActiveModel) -> Result<MusicModel, DbErr> {
        music.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, music: MusicActiveModel) -> Result<MusicModel, DbErr> {
        music.update(self.db.as_ref()).await
    }

    pub async fn delete(
        &self,
        name: String,
        artist: String,
        album: String,
    ) -> Result<DeleteResult, DbErr> {
        MusicEntity::delete_by_id((name, artist, album))
            .exec(self.db.as_ref())
            .await
    }

    pub async fn find_by_playlist(
        &self,
        playlist_model: &PlaylistModel,
    ) -> Result<Vec<MusicModel>, DbErr> {
        playlist_model
            .find_related(MusicEntity)
            .all(self.db.as_ref())
            .await
    }
}
