use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use validator::Validate;

#[derive(Deserialize, Serialize, Validate, Debug)]
pub struct YoutubeUrlResponse {
    pub url: String,
}
