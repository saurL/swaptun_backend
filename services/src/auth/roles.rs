use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl UserRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn is_user_or_above(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::User)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Guest => "guest",
        }
    }

    pub fn is_valid_role(role: &str) -> bool {
        matches!(role.to_lowercase().as_str(), "admin" | "user" | "guest")
    }
}
