use std::sync::Arc;
use swaptun_models::{
    MusicEntity, MusicModel, PlaylistActiveModel, PlaylistColumn, PlaylistEntity, PlaylistModel,
    UserModel, playlist::PlaylistOrigin,
};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    ModelTrait, QueryFilter, metric::Info, sea_query::ExprTrait,
};
pub struct PlaylistRepository {
    db: Arc<DatabaseConnection>,
}
use log::info;

impl PlaylistRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<PlaylistModel>, DbErr> {
        PlaylistEntity::find_by_id(id).one(&*self.db).await
    }

    pub async fn find_by_user(&self, user: UserModel) -> Result<Vec<PlaylistModel>, DbErr> {
        user.find_related(PlaylistEntity).all(&*self.db).await
    }

    pub async fn find_by_user_and_origin(
        &self,
        user: UserModel,
        origin: PlaylistOrigin,
    ) -> Result<Vec<PlaylistModel>, DbErr> {
        user.find_related(PlaylistEntity)
            .filter(PlaylistColumn::Origin.eq(origin))
            .all(&*self.db)
            .await
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
}
