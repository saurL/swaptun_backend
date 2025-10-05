use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    ModelTrait, QueryFilter,
};
use std::sync::Arc;
use swaptun_models::{
    YoutubeTokenActiveModel, YoutubeTokenColumn, YoutubeTokenEntity, YoutubeTokenModel, UserModel,
};

#[derive(Clone)]
pub struct YoutubeTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl YoutubeTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<YoutubeTokenModel>, DbErr> {
        YoutubeTokenEntity::find().all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<YoutubeTokenModel>, DbErr> {
        YoutubeTokenEntity::find_by_id(id)
            .one(self.db.as_ref())
            .await
    }
    pub async fn save(
        &self,
        token: YoutubeTokenActiveModel,
    ) -> Result<YoutubeTokenActiveModel, DbErr> {
        token.save(self.db.as_ref()).await
    }
    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<YoutubeTokenModel>, DbErr> {
        YoutubeTokenEntity::find()
            .filter(YoutubeTokenColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: YoutubeTokenActiveModel) -> Result<YoutubeTokenModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: YoutubeTokenActiveModel) -> Result<YoutubeTokenModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        YoutubeTokenEntity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
    }

    pub async fn delete_by_user_id(&self, user_id: i32) -> Result<DeleteResult, DbErr> {
        YoutubeTokenEntity::delete_many()
            .filter(YoutubeTokenColumn::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
    }

    pub async fn get_token(&self, user_model: &UserModel) -> Result<Option<YoutubeTokenModel>, DbErr> {
        let token = user_model
            .find_related(YoutubeTokenEntity)
            .one(self.db.as_ref())
            .await?;

        Ok(token)
    }
}