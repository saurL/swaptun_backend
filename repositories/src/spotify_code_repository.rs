use sea_orm::DeleteResult;
use sea_orm::ModelTrait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;
use swaptun_models::{
    SpotifyCodeActiveModel, SpotifyCodeColumn, SpotifyCodeEntity, SpotifyCodeModel, UserModel,
};

pub struct SpotifyCodeRepository {
    db: Arc<DatabaseConnection>,
}

impl SpotifyCodeRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<SpotifyCodeModel>, DbErr> {
        SpotifyCodeEntity::find().all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<SpotifyCodeModel>, DbErr> {
        SpotifyCodeEntity::find_by_id(id)
            .one(self.db.as_ref())
            .await
    }

    pub async fn save(
        &self,
        code: SpotifyCodeActiveModel,
    ) -> Result<SpotifyCodeActiveModel, DbErr> {
        code.save(self.db.as_ref()).await
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<SpotifyCodeModel>, DbErr> {
        SpotifyCodeEntity::find()
            .filter(SpotifyCodeColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: SpotifyCodeActiveModel) -> Result<SpotifyCodeModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: SpotifyCodeActiveModel) -> Result<SpotifyCodeModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        SpotifyCodeEntity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
    }

    pub async fn get_code(&self, user_model: &UserModel) -> Result<SpotifyCodeModel, DbErr> {
        let code = user_model
            .find_related(SpotifyCodeEntity)
            .one(self.db.as_ref())
            .await?;

        match code {
            Some(code) => Ok(code),
            None => Err(DbErr::Custom("code not found".to_string())),
        }
    }
}
