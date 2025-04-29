use std::sync::Arc;

use crate::dto::{AddTokenRequest, DeleteTokenRequest, UpdateTokenRequest};
use crate::error::AppError;
use sea_orm::{ActiveValue::Set, DatabaseConnection};
use swaptun_models::{Model, SpotifyTokenActiveModel, SpotifyTokenModel};
use swaptun_repositories::spotify_token_repository::SpotifyTokenRepository;
pub struct SpotifyService {
    spotify_token_repository: SpotifyTokenRepository,
}

impl SpotifyService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            spotify_token_repository: SpotifyTokenRepository::new(db),
        }
    }

    pub async fn add_token(&self, request: AddTokenRequest) -> Result<(), AppError> {
        let model = SpotifyTokenActiveModel {
            user_id: Set(request.user_id),
            token: Set(request.token),
            ..Default::default()
        };
        self.spotify_token_repository
            .create(model)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    pub async fn get_user_token(
        &self,
        user_id: i32,
    ) -> Result<Option<swaptun_models::SpotifyTokenModel>, AppError> {
        self.spotify_token_repository
            .find_by_user_id(user_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_token(&self, request: UpdateTokenRequest) -> Result<(), AppError> {
        let token_id = match self.get_user_token(request.user_id).await? {
            Some(token) => token.id,
            None => return Err(AppError::NotFound("Token doesn't exsits".to_string())),
        };
        let model = SpotifyTokenActiveModel {
            id: Set(token_id),
            user_id: Set(request.user_id),
            token: Set(request.new_token),
            ..Default::default()
        };
        self.spotify_token_repository
            .update(model)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    pub async fn delete_token(&self, request: DeleteTokenRequest) -> Result<(), AppError> {
        let user_id: i32 = request.user_id;

        self.spotify_token_repository
            .delete(user_id)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    pub async fn get_token_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<swaptun_models::SpotifyTokenModel>, AppError> {
        self.spotify_token_repository
            .find_by_user_id(user_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_token(&self, user_model: Model) -> Result<SpotifyTokenModel, AppError> {
        self.spotify_token_repository
            .get_token(user_model)
            .await
            .map_err(AppError::from)
    }
}
