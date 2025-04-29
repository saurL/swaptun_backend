use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AddTokenRequest {
    pub user_id: i32,
    pub token: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateTokenRequest {
    pub user_id: i32,
    pub new_token: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteTokenRequest {
    pub user_id: i32,
}
