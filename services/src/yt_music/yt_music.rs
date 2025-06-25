use std::sync::Arc;

use sea_orm::DatabaseConnection;
use crate::{error::AppError, YoutubeUrlResponse};

pub struct YoutubeMusicService;

impl YoutubeMusicService {
    pub fn new(_db: Arc<DatabaseConnection>) -> Self {
        YoutubeMusicService
    }

    pub async fn get_authorization_url(&self) -> Result<YoutubeUrlResponse, AppError> {
        let client = ytmapi_rs::Client::new().unwrap();
        // A Client ID and Client Secret must be provided - see `youtui` README.md.
        // In this example, I assume they were put in environment variables beforehand.
        let client_id = std::env::var("YOUTUI_OAUTH_CLIENT_ID").unwrap();
        let _client_secret = std::env::var("YOUTUI_OAUTH_CLIENT_SECRET").unwrap();
        match ytmapi_rs::generate_oauth_code_and_url(&client, &client_id).await{
            Ok((_, url)) => {
                // The code is used to get an access token, which is not needed here.
                // The URL can be used to redirect the user to the authorization page.
                let response= YoutubeUrlResponse {
                    url: url.clone(),
                };
                Ok(response)
            }
            Err(_) => Err(AppError::InternalServerError),
        }    
    }
}