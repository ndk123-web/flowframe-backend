use crate::handlers::workspace_handlers::{
    create_workspace_handler, delete_workspace_handler, get_user_workspaces_handler,
    get_workspace_by_id_handler,
};
use crate::middleware::jwt_auth::jwt_auth_middleware;
use crate::state::app_state::AppState;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn workspace_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            post(create_workspace_handler).get(get_user_workspaces_handler),
        )
        .route(
            "/{id}",
            get(get_workspace_by_id_handler).delete(delete_workspace_handler),
        )
        .layer(from_fn_with_state(state, jwt_auth_middleware))
}
