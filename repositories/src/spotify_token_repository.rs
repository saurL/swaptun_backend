use sea_orm::DeleteResult;
use sea_orm::ModelTrait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;
use swaptun_models::{
    Model, SpotifyTokenActiveModel, SpotifyTokenColumn, SpotifyTokenEntity, SpotifyTokenModel,
};
pub struct SpotifyTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl SpotifyTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<SpotifyTokenModel>, DbErr> {
        SpotifyTokenEntity::find().all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<SpotifyTokenModel>, DbErr> {
        SpotifyTokenEntity::find_by_id(id)
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<SpotifyTokenModel>, DbErr> {
        SpotifyTokenEntity::find()
            .filter(SpotifyTokenColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: SpotifyTokenActiveModel) -> Result<SpotifyTokenModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: SpotifyTokenActiveModel) -> Result<SpotifyTokenModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        SpotifyTokenEntity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
    }

    pub async fn get_token(&self, user_model: Model) -> Result<SpotifyTokenModel, DbErr> {
        let token = user_model
            .find_related(SpotifyTokenEntity)
            .one(self.db.as_ref())
            .await?;

        match token {
            Some(token) => Ok(token),
            None => Err(DbErr::Custom("Token not found".to_string())),
        }
    }
}
