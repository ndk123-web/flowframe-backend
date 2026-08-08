use crate::dtos::workspace_dto::CreateWorkspaceRequest;
use crate::middleware::jwt_auth::AuthUserExtension;
use crate::state::app_state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn create_workspace_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> impl IntoResponse {
    match state
        .workspace_service
        .create_workspace(&auth_user.user_id, payload)
        .await
    {
        Ok(res) => (StatusCode::CREATED, Json(json!(res))).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_user_workspaces_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
) -> impl IntoResponse {
    match state
        .workspace_service
        .get_user_workspaces(&auth_user.user_id)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(json!(res))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_workspace_by_id_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state
        .workspace_service
        .get_workspace_by_id(&id, &auth_user.user_id)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(json!(res))).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_workspace_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state
        .workspace_service
        .delete_workspace(&id, &auth_user.user_id)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({ "message": "Workspace deleted successfully" })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Workspace not found or not owned by user" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}
