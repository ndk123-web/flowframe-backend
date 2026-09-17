use crate::handlers::ai_handlers::{chat_handler, history_handler, usage_handler};
use crate::middleware::jwt_auth::jwt_auth_middleware;
use crate::state::app_state::AppState;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn ai_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/usage", get(usage_handler))
        .route("/history", get(history_handler))
        .route_layer(from_fn_with_state(app_state.clone(), jwt_auth_middleware))
}
