use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct SpotifyUrlResponse {
    pub url: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct GetAuthorizationUrlRequest {
    pub port: u16,
}
