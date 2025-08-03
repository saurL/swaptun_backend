use fcm::ErrorReason;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NotificationRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,

    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Body is required"))]
    pub body: String,

    pub data: Option<HashMap<String, String>>,
    pub image: Option<String>,
    pub sound: Option<String>,
    pub badge: Option<String>,
    pub click_action: Option<String>,
    pub priority: Option<NotificationPriority>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MulticastNotificationRequest {
    #[validate(length(min = 1, message = "At least one token is required"))]
    pub tokens: Vec<String>,

    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Body is required"))]
    pub body: String,

    pub data: Option<HashMap<String, String>>,
    pub image: Option<String>,
    pub sound: Option<String>,
    pub badge: Option<String>,
    pub click_action: Option<String>,
    pub priority: Option<NotificationPriority>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TopicNotificationRequest {
    #[validate(length(min = 1, message = "Topic is required"))]
    pub topic: String,

    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Body is required"))]
    pub body: String,

    pub data: Option<HashMap<String, String>>,
    pub image: Option<String>,
    pub sound: Option<String>,
    pub badge: Option<String>,
    pub click_action: Option<String>,
    pub priority: Option<NotificationPriority>,
    pub condition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NotificationPriority {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "high")]
    High,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        NotificationPriority::Normal
    }
}

#[derive(Debug, Deserialize)]
pub struct NotificationResponse {
    pub success: bool,
    pub message_id: Option<u64>,
    pub error: Option<ErrorReason>,
    pub multicast_id: Option<i64>,
    pub success_count: Option<usize>,
    pub failure_count: Option<usize>,
    pub canonical_ids: Option<usize>,
    pub results: Option<Vec<NotificationResult>>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationResult {
    pub message_id: Option<u64>,
    pub registration_id: Option<String>,
    pub error: Option<ErrorReason>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SubscribeToTopicRequest {
    #[validate(length(min = 1, message = "At least one token is required"))]
    pub tokens: Vec<String>,

    #[validate(length(min = 1, message = "Topic is required"))]
    pub topic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicManagementResponse {
    pub success: bool,
    pub error: Option<String>,
    pub results: Option<Vec<TopicManagementResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicManagementResult {
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterFcmTokenRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,

    pub device_id: Option<String>,

    #[validate(length(min = 1, message = "Platform must be specified if provided"))]
    pub platform: Option<String>, // "android", "ios", "web"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterFcmTokenResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendTestNotificationRequest {
    #[validate(range(min = 1, message = "User ID must be positive"))]
    pub user_id: i32,

    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    #[validate(length(min = 1, message = "Body is required"))]
    pub body: String,

    pub data: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTestNotificationResponse {
    pub success: bool,
    pub message: String,
    pub notification_sent: bool,
}
