use crate::dtos::diagram_dto::{CreateDiagramRequest, UpdateDiagramRequest};
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

pub async fn create_diagram_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Path(workspace_id): Path<String>,
    Json(payload): Json<CreateDiagramRequest>,
) -> impl IntoResponse {
    match state
        .diagram_service
        .create_diagram(&workspace_id, &auth_user.user_id, payload)
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

pub async fn get_workspace_diagrams_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Path(workspace_id): Path<String>,
) -> impl IntoResponse {
    match state
        .diagram_service
        .get_workspace_diagrams(&workspace_id, &auth_user.user_id)
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

pub async fn get_diagram_by_id_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Path((_workspace_id, diagram_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match state
        .diagram_service
        .get_diagram_by_id(&diagram_id, &auth_user.user_id)
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

pub async fn update_diagram_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Path((_workspace_id, diagram_id)): Path<(String, String)>,
    Json(payload): Json<UpdateDiagramRequest>,
) -> impl IntoResponse {
    match state
        .diagram_service
        .update_diagram(&diagram_id, &auth_user.user_id, payload)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(json!(res))).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_diagram_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
    Path((_workspace_id, diagram_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match state
        .diagram_service
        .delete_diagram(&diagram_id, &auth_user.user_id)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({ "message": "Diagram deleted successfully" })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Diagram not found or not owned by user" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_recent_diagrams_handler(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserExtension>,
) -> impl IntoResponse {
    match state
        .diagram_service
        .get_recent_diagrams(&auth_user.user_id, 4)
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
