use crate::dtos::signin_dto::SignInRequest;
use crate::dtos::signup_dto::SignUpRequest;
use crate::state::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

pub async fn signup_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignUpRequest>,
) -> impl IntoResponse {
    match state.auth_service.signup(payload).await {
        Ok(res) => (StatusCode::CREATED, Json(res)).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn signin_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignInRequest>,
) -> impl IntoResponse {
    match state.auth_service.signin(payload).await {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(err) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}
