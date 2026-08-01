use crate::handlers::auth_handlers::{signin_handler, signup_handler};
use crate::state::app_state::AppState;
use axum::{routing::post, Router};
use std::sync::Arc;

pub fn auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler))
}