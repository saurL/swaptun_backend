use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use validator::Validate;

#[derive(Deserialize, Serialize, Validate, Debug)]
pub struct AppleUrlResponse {
    pub url: String,
}

#[derive(Deserialize, Serialize, Validate, Debug)]
pub struct GetDeveloperToken {
    pub developer_token: String,
}
