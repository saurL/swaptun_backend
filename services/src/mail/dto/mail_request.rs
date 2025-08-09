use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct MailRequest {
    #[validate(nested)]
    pub to: Vec<String>,

    #[validate(nested)]
    pub cc: Option<Vec<String>>,

    #[validate(nested)]
    pub bcc: Option<Vec<String>>,

    #[validate(length(min = 1, message = "Subject is required"))]
    pub subject: String,

    #[validate(length(min = 1, message = "Body is required"))]
    pub body: String,

    pub is_html: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct BulkMailRequest {
    #[validate(length(min = 1, message = "At least one recipient is required"))]
    pub recipients: Vec<MailRecipient>,

    #[validate(length(min = 1, message = "Subject is required"))]
    pub subject: String,

    #[validate(length(min = 1, message = "Body is required"))]
    pub body: String,

    pub is_html: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct MailRecipient {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    pub name: Option<String>,
}
