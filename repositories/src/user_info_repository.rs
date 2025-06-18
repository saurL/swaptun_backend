use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
    DeleteResult,
};
use std::sync::Arc;
use swaptun_models::{user_info::{Entity as UserInfoEntity, Model as UserInfoModel, ActiveModel as UserInfoActiveModel, Column as UserInfoColumn}};

pub struct UserInfoRepository {
    db: Arc<DatabaseConnection>,
}

impl UserInfoRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }

    pub async fn find_all(&self) -> Result<Vec<UserInfoModel>, DbErr> {
        UserInfoEntity::find().all(self.db.as_ref()).await
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<UserInfoModel>, DbErr> {
        UserInfoEntity::find()
            .filter(UserInfoColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: UserInfoActiveModel) -> Result<UserInfoModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: UserInfoActiveModel) -> Result<UserInfoModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        UserInfoEntity::delete_by_id(id).exec(self.db.as_ref()).await
    }
}
