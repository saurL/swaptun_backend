use std::sync::Arc;

use crate::dto::{AddTokenRequest, DeleteTokenRequest, UpdateTokenRequest};
use crate::error::AppError;
use sea_orm::IntoActiveModel;
use sea_orm::{ActiveValue::Set, DatabaseConnection};
use swaptun_models::{DeezerTokenActiveModel, UserModel};
use swaptun_repositories::deezer_token_repository::DeezerTokenRepository;

pub struct DeezerService {
    deezer_token_repository: DeezerTokenRepository,
}

impl DeezerService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            deezer_token_repository: DeezerTokenRepository::new(db),
        }
    }

    pub async fn add_token(
        &self,
        request: AddTokenRequest,
        user: UserModel,
    ) -> Result<DeezerTokenActiveModel, AppError> {
        match self.get_token(user.clone()).await {
            Ok(model) => {
                let mut active_model = model.into_active_model();
                active_model.token = Set(request.token);
                self.deezer_token_repository
                    .save(active_model.into())
                    .await
                    .map_err(AppError::from)
            }
            Err(_) => {
                let model = DeezerTokenActiveModel {
                    user_id: Set(user.id),
                    token: Set(request.token),
                    ..Default::default()
                };
                self.deezer_token_repository
                    .save(model)
                    .await
                    .map_err(AppError::from)
            }
        }
    }

    pub async fn get_user_token(
        &self,
        user_id: i32,
    ) -> Result<Option<swaptun_models::DeezerTokenModel>, AppError> {
        self.deezer_token_repository
            .find_by_user_id(user_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_token(&self, request: UpdateTokenRequest) -> Result<(), AppError> {
        let token_id = match self.get_user_token(request.user_id).await? {
            Some(token) => token.id,
            None => return Err(AppError::NotFound("Token doesn't exist".to_string())),
        };
        let model = DeezerTokenActiveModel {
            id: Set(token_id),
            user_id: Set(request.user_id),
            token: Set(request.new_token),
            ..Default::default()
        };
        self.deezer_token_repository
            .update(model)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    pub async fn delete_token(&self, request: DeleteTokenRequest) -> Result<(), AppError> {
        let user_id: i32 = request.user_id;

        self.deezer_token_repository
            .delete_by_user_id(user_id)
            .await
            .map(|_| ())
            .map_err(AppError::from)
    }

    pub async fn get_token_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<swaptun_models::DeezerTokenModel>, AppError> {
        self.deezer_token_repository
            .find_by_user_id(user_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_token(
        &self,
        user_model: UserModel,
    ) -> Result<swaptun_models::DeezerTokenModel, AppError> {
        self.deezer_token_repository
            .get_token(user_model)
            .await
            .map_err(AppError::from)
    }
}
