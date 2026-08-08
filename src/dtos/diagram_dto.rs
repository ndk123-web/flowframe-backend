use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDiagramRequest {
    pub title: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub nodes: Option<serde_json::Value>,
    pub edges: Option<serde_json::Value>,
    pub configs: Option<serde_json::Value>,
    pub viewport: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDiagramRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub nodes: Option<serde_json::Value>,
    pub edges: Option<serde_json::Value>,
    pub configs: Option<serde_json::Value>,
    pub viewport: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagramResponse {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub version: String,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    pub configs: serde_json::Value,
    pub viewport: Option<serde_json::Value>,
    pub nodes_count: usize,
    pub edges_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentDiagramResponse {
    pub id: String,
    pub workspace_id: String,
    pub workspace_name: String,
    pub title: String,
    pub env: String,
    pub nodes_count: usize,
    pub updated_at: String,
}
