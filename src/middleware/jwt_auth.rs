use crate::state::app_state::AppState;
use crate::utils::jwt::verify_jwt;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AuthUserExtension {
    pub user_id: String,
    pub email: String,
}

pub async fn jwt_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header_val) if header_val.starts_with("Bearer ") => {
            header_val.trim_start_matches("Bearer ").trim()
        }
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid authorization header" })),
            )
                .into_response();
        }
    };

    match verify_jwt(token, &state.config.jwt_secret) {
        Ok(claims) => {
            let auth_user = AuthUserExtension {
                user_id: claims.sub,
                email: claims.email,
            };
            req.extensions_mut().insert(auth_user);
            next.run(req).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired access token" })),
        )
            .into_response(),
    }
}
