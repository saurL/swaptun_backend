use sea_orm::ModelTrait;
use sea_orm::error::DbErr;
use sea_orm::*;
use std::sync::Arc;
use swaptun_models::{
    DeezerTokenActiveModel, DeezerTokenColumn, DeezerTokenEntity, DeezerTokenModel, UserModel,
};

pub struct DeezerTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl DeezerTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<DeezerTokenModel>, DbErr> {
        DeezerTokenEntity::find()
            .filter(DeezerTokenColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: DeezerTokenActiveModel) -> Result<DeezerTokenModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: DeezerTokenActiveModel) -> Result<DeezerTokenModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete_by_user_id(&self, user_id: i32) -> Result<DeleteResult, DbErr> {
        DeezerTokenEntity::delete_many()
            .filter(DeezerTokenColumn::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
    }

    pub async fn get_token(&self, user_model: UserModel) -> Result<DeezerTokenModel, DbErr> {
        let token = user_model
            .find_related(DeezerTokenEntity)
            .one(self.db.as_ref())
            .await?;

        match token {
            Some(token) => Ok(token),
            None => Err(DbErr::Custom("Token not found".to_string())),
        }
    }
}
