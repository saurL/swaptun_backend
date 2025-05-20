use std::sync::Arc;

use crate::SpotifyUrlResponse;
use crate::dto::{AddTokenRequest, DeleteTokenRequest, UpdateTokenRequest};
use crate::error::AppError;
use sea_orm::{ActiveValue::Set, DatabaseConnection};
use swaptun_models::{SpotifyTokenActiveModel, SpotifyTokenModel, UserModel};
use swaptun_repositories::spotify_token_repository::SpotifyTokenRepository;

use rspotify::{
    AuthCodeSpotify, Credentials, OAuth,
    model::{AdditionalType, Country, Market},
    prelude::*,
    scopes,
};
pub struct SpotifyService {
    spotify_token_repository: SpotifyTokenRepository,
}

impl SpotifyService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            spotify_token_repository: SpotifyTokenRepository::new(db.clone()),
        }
    }

    pub async fn add_token(
        &self,
        request: AddTokenRequest,
        user: UserModel,
    ) -> Result<(), AppError> {
        let model = SpotifyTokenActiveModel {
            user_id: Set(user.id),
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

    pub async fn get_token(&self, user_model: UserModel) -> Result<SpotifyTokenModel, AppError> {
        self.spotify_token_repository
            .get_token(user_model)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_authorization_url(&self, port: u16) -> Result<SpotifyUrlResponse, AppError> {
        let creds = Credentials::from_env().unwrap();

        // Same for RSPOTIFY_REDIRECT_URI. You can also set it explictly:
        //
        // ```
        // let oauth = OAuth {
        //     redirect_uri: "http://localhost:8888/callback".to_string(),
        //     scopes: scopes!("user-read-recently-played"),
        //     ..Default::default(),
        // };
        // ```
        let oauth: OAuth = OAuth {
            redirect_uri: format!("https://127.0.0.1:{}", port),
            scopes: scopes!("user-read-recently-played"),
            ..Default::default()
        };

        let spotify = AuthCodeSpotify::new(creds, oauth);

        // Obtaining the access token
        let url = spotify.get_authorize_url(false).unwrap();
        let response = SpotifyUrlResponse { url: url.clone() };
        Ok(response)
    }
}
