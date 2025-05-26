use crate::error::AppError;
use sea_orm::{ActiveValue::Set, DatabaseConnection};
use std::sync::Arc;
use swaptun_models::{TokenActiveModel, TokenModel, UserModel};
use swaptun_repositories::token_repository::TokenRepository;

pub struct TokenService {
    token_repository: TokenRepository,
}

impl TokenService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            token_repository: TokenRepository::new(db),
        }
    }

    pub async fn get_user_token(&self, user_id: i32) -> Result<Option<TokenModel>, AppError> {
        self.token_repository
            .find_by_user_id(user_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn add_token(
        &self,
        user: UserModel,
        access_token: String,
        refresh_token: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        token_type: String,
        scope: Option<String>,
    ) -> Result<TokenModel, AppError> {
        match self.get_user_token(user.id).await? {
            Some(_) => {
                // Token exists, update it
                self.update_token(
                    user.id,
                    access_token,
                    refresh_token,
                    expires_at,
                    token_type,
                    scope,
                )
                .await
            }
            None => {
                // Create new token
                let model = TokenActiveModel {
                    user_id: Set(user.id),
                    access_token: Set(access_token),
                    refresh_token: Set(refresh_token),
                    expires_at: Set(expires_at),
                    token_type: Set(token_type),
                    scope: Set(scope),
                    ..Default::default()
                };
                self.token_repository
                    .create(model)
                    .await
                    .map_err(AppError::from)
            }
        }
    }

    pub async fn update_token(
        &self,
        user_id: i32,
        access_token: String,
        refresh_token: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        token_type: String,
        scope: Option<String>,
    ) -> Result<TokenModel, AppError> {
        let token = self.get_user_token(user_id).await?;
        let token_id = match token {
            Some(token) => token.id,
            None => return Err(AppError::NotFound("Token not found".to_string())),
        };

        let model = TokenActiveModel {
            id: Set(token_id),
            user_id: Set(user_id),
            access_token: Set(access_token),
            refresh_token: Set(refresh_token),
            expires_at: Set(expires_at),
            token_type: Set(token_type),
            scope: Set(scope),
            ..Default::default()
        };

        self.token_repository
            .update(model)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_token(&self, user_id: i32) -> Result<(), AppError> {
        let token = self.get_user_token(user_id).await?;
        match token {
            Some(token) => self
                .token_repository
                .delete(token.id)
                .await
                .map(|_| ())
                .map_err(AppError::from),
            None => Err(AppError::NotFound("Token not found".to_string())),
        }
    }

    pub async fn is_token_expired(&self, user_id: i32) -> Result<bool, AppError> {
        let token = self.get_user_token(user_id).await?;
        match token {
            Some(token) => {
                let now = chrono::Utc::now();
                Ok(token.expires_at < now)
            }
            None => Err(AppError::NotFound("Token not found".to_string())),
        }
    }
}
