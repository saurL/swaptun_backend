use sea_orm::DeleteResult;
use sea_orm::ModelTrait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;
use swaptun_models::{TokenActiveModel, TokenColumn, TokenEntity, TokenModel, UserModel};

pub struct TokenRepository {
    db: Arc<DatabaseConnection>,
}

impl TokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<TokenModel>, DbErr> {
        TokenEntity::find().all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<TokenModel>, DbErr> {
        TokenEntity::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn save(&self, token: TokenActiveModel) -> Result<TokenActiveModel, DbErr> {
        token.save(self.db.as_ref()).await
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<TokenModel>, DbErr> {
        TokenEntity::find()
            .filter(TokenColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: TokenActiveModel) -> Result<TokenModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: TokenActiveModel) -> Result<TokenModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        TokenEntity::delete_by_id(id).exec(self.db.as_ref()).await
    }

    pub async fn get_token(&self, user_model: UserModel) -> Result<TokenModel, DbErr> {
        let token = user_model
            .find_related(TokenEntity)
            .one(self.db.as_ref())
            .await?;

        match token {
            Some(token) => Ok(token),
            None => Err(DbErr::Custom("Token not found".to_string())),
        }
    }
}
