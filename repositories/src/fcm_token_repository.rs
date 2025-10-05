use chrono::DateTime;
use sea_orm::sqlx::types::uuid::timestamp;
use sea_orm::DeleteResult;
use sea_orm::ModelTrait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;
use swaptun_models::{
    FcmTokenActiveModel, FcmTokenColumn, FcmTokenEntity, FcmTokenModel, UserModel,
};

pub struct FcmTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl FcmTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<FcmTokenModel>, DbErr> {
        FcmTokenEntity::find().all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<FcmTokenModel>, DbErr> {
        FcmTokenEntity::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<FcmTokenModel>, DbErr> {
        FcmTokenEntity::find()
            .filter(FcmTokenColumn::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_token(&self, token: String) -> Result<Option<FcmTokenModel>, DbErr> {
        FcmTokenEntity::find()
            .filter(FcmTokenColumn::Token.eq(token))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_active_tokens(&self) -> Result<Vec<FcmTokenModel>, DbErr> {
        FcmTokenEntity::find()
            .filter(FcmTokenColumn::IsActive.eq(true))
            .all(self.db.as_ref())
            .await
    }

    pub async fn find_active_by_user_id(&self, user_id: i32) -> Result<Vec<FcmTokenModel>, DbErr> {
        FcmTokenEntity::find()
            .filter(FcmTokenColumn::UserId.eq(user_id))
            .filter(FcmTokenColumn::IsActive.eq(true))
            .all(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: FcmTokenActiveModel) -> Result<FcmTokenModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: FcmTokenActiveModel) -> Result<FcmTokenModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn save(&self, model: FcmTokenActiveModel) -> Result<FcmTokenActiveModel, DbErr> {
        model.save(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        FcmTokenEntity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
    }

    pub async fn deactivate_token(&self, id: i32) -> Result<Option<FcmTokenModel>, DbErr> {
        let token = self.find_by_id(id).await?;
        if let Some(token) = token {
            let mut active_model: FcmTokenActiveModel = token.into();
            active_model.is_active = sea_orm::Set(false);
            active_model.updated_on = sea_orm::Set(chrono::Utc::now().fixed_offset());

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn activate_token(&self, id: i32) -> Result<Option<FcmTokenModel>, DbErr> {
        let token = self.find_by_id(id).await?;

        if let Some(token) = token {
            let mut active_model: FcmTokenActiveModel = token.into();
            active_model.is_active = sea_orm::Set(true);
            active_model.updated_on = sea_orm::Set(chrono::Utc::now().fixed_offset());

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_token_by_user(&self, user_model: UserModel) -> Result<FcmTokenModel, DbErr> {
        let token = user_model
            .find_related(FcmTokenEntity)
            .one(self.db.as_ref())
            .await?;

        match token {
            Some(token) => Ok(token),
            None => Err(DbErr::Custom("FCM Token not found".to_string())),
        }
    }

    pub async fn upsert_token(
        &self,
        user_id: i32,
        token: String,
        device_id: Option<String>,
        platform: Option<String>,
    ) -> Result<FcmTokenModel, DbErr> {
        // Chercher un token existant pour cet utilisateur
        if let Some(existing_token) = self.find_by_user_id(user_id).await? {
            // Mettre à jour le token existant
            let mut active_model: FcmTokenActiveModel = existing_token.into();
            active_model.token = sea_orm::Set(token);
            active_model.device_id = sea_orm::Set(device_id);
            active_model.platform = sea_orm::Set(platform);
            active_model.is_active = sea_orm::Set(true);
            active_model.updated_on = sea_orm::Set(chrono::Utc::now().fixed_offset());

            active_model.update(self.db.as_ref()).await
        } else {
            // Créer un nouveau token
            let now = chrono::Utc::now().fixed_offset();
            let new_token = FcmTokenActiveModel {
                id: sea_orm::NotSet,
                user_id: sea_orm::Set(user_id),
                token: sea_orm::Set(token),
                device_id: sea_orm::Set(device_id),
                platform: sea_orm::Set(platform),
                is_active: sea_orm::Set(true),
                created_on: sea_orm::Set(now),
                updated_on: sea_orm::Set(now),
            };

            new_token.insert(self.db.as_ref()).await
        }
    }
}
