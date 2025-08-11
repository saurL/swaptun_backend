use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::validators::user_validators::validate_password;

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}
#[derive(Deserialize, Serialize, Debug, Validate)]

pub struct ResetPasswordRequest {
    #[validate(custom(function = validate_password))]
    pub password: String,
}
