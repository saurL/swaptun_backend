use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    ModelTrait, QueryFilter,
};
use std::sync::Arc;
use swaptun_models::{
    AppleTokenActiveModel, AppleTokenColumn, AppleTokenEntity, AppleTokenModel, UserModel,
};

#[derive(Clone)]
pub struct AppleTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl AppleTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<AppleTokenModel>, DbErr> {
        AppleTokenEntity::find().all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<AppleTokenModel>, DbErr> {
        AppleTokenEntity::find_by_id(id).one(self.db.as_ref()).await
    }
    pub async fn save(&self, token: AppleTokenActiveModel) -> Result<AppleTokenActiveModel, DbErr> {
        token.save(self.db.as_ref()).await
    }
    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<AppleTokenModel>, DbErr> {
        AppleTokenEntity::find()
            .filter(AppleTokenColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: AppleTokenActiveModel) -> Result<AppleTokenModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: AppleTokenActiveModel) -> Result<AppleTokenModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        AppleTokenEntity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
    }

    pub async fn delete_by_user_id(&self, user_id: i32) -> Result<DeleteResult, DbErr> {
        AppleTokenEntity::delete_many()
            .filter(AppleTokenColumn::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
    }

    pub async fn get_token(
        &self,
        user_model: &UserModel,
    ) -> Result<Option<AppleTokenModel>, DbErr> {
        let token = user_model
            .find_related(AppleTokenEntity)
            .one(self.db.as_ref())
            .await?;

        Ok(token)
    }
}
