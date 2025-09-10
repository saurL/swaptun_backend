use std::sync::Arc;

use sea_orm::{DatabaseConnection, Set};
use swaptun_models::{AppleTokenActiveModel, AppleTokenModel, UserModel};
use swaptun_repositories::AppleTokenRepository;

use crate::AddTokenRequest;
use crate::{error::AppError, GetDeveloperToken};
use apple_music_api::create_developer_token;
use log::{error, info};

pub struct AppleMusicService {
    apple_token_repository: AppleTokenRepository,
}

impl AppleMusicService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        let apple_token_repository = AppleTokenRepository::new(db.clone());

        AppleMusicService {
            apple_token_repository,
        }
    }

    pub async fn generate_developer_token(&self) -> Result<GetDeveloperToken, AppError> {
        let team_id = std::env::var("APPLE_TEAM_ID").map_err(|_| {
            error!("APPLE_TEAM_ID not set");
            AppError::InternalServerError
        })?;
        let key_id = std::env::var("APPLE_KEY_ID").map_err(|_| {
            error!("APPLE_KEY_ID not set");
            AppError::InternalServerError
        })?;
        let private_key = include_str!("../../certs/AuthKey_apple_music.p8");
        match create_developer_token(&team_id, &key_id, &private_key) {
            Ok(token) => Ok(GetDeveloperToken {
                developer_token: token,
            }),
            Err(e) => {
                error!("Failed to create developer token: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn add_user_token(
        &self,
        request: AddTokenRequest,
        user_id: i32,
    ) -> Result<(), AppError> {
        let token = AppleTokenActiveModel {
            user_id: Set(user_id),
            access_token: Set(request.token),

            ..Default::default()
        };
        self.save(token).await
    }

    pub async fn save(&self, apple_active_model: AppleTokenActiveModel) -> Result<(), AppError> {
        self.apple_token_repository
            .save(apple_active_model)
            .await
            .map_err(|e| {
                error!("Failed to save apple token: {:?}", e);
                AppError::InternalServerError
            })?;
        Ok(())
    }

    pub async fn get_token(&self, user: &UserModel) -> Result<Option<AppleTokenModel>, AppError> {
        match self.apple_token_repository.get_token(user).await? {
            Some(token) => {
                info!("Found token for user {}", user.id);
                Ok(Some(token))
            }
            None => {
                info!("No token found for user {}", user.id);
                Ok(None)
            }
        }
    }
}
