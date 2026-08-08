use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub env: Option<String>,
    pub color: Option<String>,
    pub icon_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub env: String,
    pub color: Option<String>,
    pub icon_type: Option<String>,
    pub diagrams_count: i64,
    pub created_at: String,
    pub updated_at: String,
}
