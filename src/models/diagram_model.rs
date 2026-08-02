use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Diagram {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub workspace_id: ObjectId,
    pub user_id: ObjectId,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_version")]
    pub version: String,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    #[serde(default = "default_configs")]
    pub configs: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<serde_json::Value>,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
    #[serde(default = "DateTime::now")]
    pub updated_at: DateTime,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_configs() -> serde_json::Value {
    serde_json::json!({})
}
