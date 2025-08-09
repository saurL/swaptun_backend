use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MailResponse {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkMailResponse {
    pub success: bool,
    pub total_sent: usize,
    pub total_failed: usize,
    pub details: Vec<MailResponse>,
}
