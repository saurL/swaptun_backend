use serde::{Deserialize, Serialize};
use swaptun_models::UserBean;
use validator::Validate;

use crate::{
    model::SearchField,
    validators::user_validators::{validate_no_spaces, validate_password},
};

#[derive(Deserialize, Serialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be at least 3 characters"))]
    #[validate(custom(function = validate_no_spaces))]
    pub username: Option<String>,

    #[validate(length(min = 3, max = 20, message = "First name cannot exceed 20 characters"))]
    pub first_name: Option<String>,

    #[validate(length(min = 3, max = 20, message = "Last name cannot exceed 20 characters"))]
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetUsersRequest {
    pub include_deleted: Option<bool>,
    pub search: Option<String>,
    pub search_field: Option<SearchField>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub friends_priority: bool,
    pub exclude_friends: bool,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(
        min = 3,
        max = 200,
        message = "Username must be between 3 and 50 characters"
    ))]
    #[validate(custom(function = validate_no_spaces))]
    pub username: String,

    #[validate(custom(function = validate_password))]
    pub password: String,

    #[validate(length(
        min = 1,
        max = 20,
        message = "First name is required and cannot exceed 20 characters"
    ))]
    pub first_name: String,

    #[validate(length(
        min = 1,
        max = 20,
        message = "Last name is required and cannot exceed 20 characters"
    ))]
    pub last_name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct LoginEmailRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserBean,
    pub role: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VerifyTokenResponse {
    pub valid: bool,
}
