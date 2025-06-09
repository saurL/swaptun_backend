use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoRequest {
    pub user_id: i32,
    pub birthdate: String,
    pub gender: String,
    pub region: String,
    pub interests: Vec<String>,
    pub listening_minutes_per_day: i32,
    pub main_devices: Vec<String>,
    pub consent: bool,
}
