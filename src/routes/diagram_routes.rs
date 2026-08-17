use crate::handlers::diagram_handlers::{
    create_diagram_handler, delete_diagram_handler, get_diagram_by_id_handler,
    get_public_diagram_handler, get_recent_diagrams_handler, get_workspace_diagrams_handler,
    update_diagram_handler,
};
use crate::middleware::jwt_auth::jwt_auth_middleware;
use crate::state::app_state::AppState;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn diagram_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/recent",
            get(get_recent_diagrams_handler),
        )
        .route(
            "/{workspace_id}/diagrams",
            post(create_diagram_handler).get(get_workspace_diagrams_handler),
        )
        .route(
            "/{workspace_id}/diagrams/{diagram_id}",
            get(get_diagram_by_id_handler)
                .put(update_diagram_handler)
                .delete(delete_diagram_handler),
        )
        .layer(from_fn_with_state(state, jwt_auth_middleware))
}

pub fn share_diagram_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{diagram_id}", get(get_public_diagram_handler))
        .route("/diagrams/{diagram_id}", get(get_public_diagram_handler))
}
