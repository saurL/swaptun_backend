use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct AddFriendRequest {
    #[validate(range(min = 1, message = "Friend ID must be a positive integer"))]
    pub friend_id: i32,
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct RemoveFriendRequest {
    #[validate(range(min = 1, message = "Friend ID must be a positive integer"))]
    pub friend_id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FriendshipResponse {
    pub id: i32,
    pub user_id: i32,
    pub friend_id: i32,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FriendListResponse {
    pub friends: Vec<crate::UserBean>,
}
