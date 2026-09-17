use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AiDiagramContext {
    pub dsl: Option<String>,
    pub node_count: Option<usize>,
    pub edge_count: Option<usize>,
    pub selected_node_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AiChatRequest {
    pub workspace_id: String,
    pub diagram_id: String,
    pub mode: String, // "ask" | "analyze" | "modify"
    #[serde(default)]
    pub think: bool,
    pub message: String,
    pub context: Option<AiDiagramContext>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiUsageDto {
    pub used: i32,
    pub limit: i32,
    pub remaining: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiChatResponse {
    pub mode: String,
    pub status: String,
    pub message: String,
    pub flow: Option<String>,
    pub explanation: Option<String>,
    pub thought_process: Option<String>,
    pub usage: AiUsageDto,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiHistoryMessageItem {
    pub id: String,
    pub role: String,
    pub mode: String,
    pub message: String,
    pub flow: Option<String>,
    pub explanation: Option<String>,
    pub thought_process: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiHistoryResponse {
    pub messages: Vec<AiHistoryMessageItem>,
    pub usage: AiUsageDto,
}
