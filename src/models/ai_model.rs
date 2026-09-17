use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiUsage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub request_count: i32,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
    #[serde(default = "DateTime::now")]
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiChatMessage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub workspace_id: ObjectId,
    pub diagram_id: ObjectId,
    pub role: String, // "user" | "assistant"
    pub mode: String, // "ask" | "analyze" | "modify"
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_process: Option<String>,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
}
