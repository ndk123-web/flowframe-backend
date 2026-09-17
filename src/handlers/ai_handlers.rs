use crate::dtos::ai_dto::AiChatRequest;
use crate::middleware::jwt_auth::AuthUserExtension;
use crate::state::app_state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub workspace_id: String,
    pub diagram_id: String,
}

/// POST /api/ai/chat
pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Json(payload): Json<AiChatRequest>,
) -> impl IntoResponse {
    match state.ai_service.process_chat(&auth_user.user_id, payload).await {
        Ok(resp) => (StatusCode::OK, Json(json!(resp))).into_response(),
        Err(err) => {
            let err_msg = err.to_string();
            let status = if err_msg.contains("limit reached") {
                StatusCode::TOO_MANY_REQUESTS
            } else if err_msg.contains("not found") || err_msg.contains("unauthorized") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            (status, Json(json!({ "error": err_msg }))).into_response()
        }
    }
}

/// GET /api/ai/usage
pub async fn usage_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
) -> impl IntoResponse {
    match state.ai_service.get_user_usage(&auth_user.user_id).await {
        Ok(usage) => (StatusCode::OK, Json(json!(usage))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/ai/history?workspace_id=...&diagram_id=...
pub async fn history_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    match state
        .ai_service
        .get_chat_history(&auth_user.user_id, &query.workspace_id, &query.diagram_id)
        .await
    {
        Ok(history) => (StatusCode::OK, Json(json!(history))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}
