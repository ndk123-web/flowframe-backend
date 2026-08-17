use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod config;
mod db;
mod dtos;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;
mod shared;
mod state;
mod utils;

use config::configs::Config;
use db::connections::create_database;
use routes::auth_routes::auth_router;
use routes::diagram_routes::{diagram_router, share_diagram_router};
use routes::workspace_routes::workspace_router;
use state::app_state::AppState;

/// Custom Logger Middleware: Prints Method, Path, Status Code, and Latency for EVERY Request
async fn request_response_logger(req: Request, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let latency = start.elapsed();
    let status = response.status();

    let status_icon = if status.is_success() {
        "✅"
    } else if status.is_client_error() {
        "⚠️"
    } else {
        "❌"
    };

    println!(
        "{} [HTTP] {} {} -> Status: {} ({:?})",
        status_icon, method, uri, status, latency
    );

    response
}

#[tokio::main]
async fn main() {
    // Load single root .env file
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let db = create_database(&config.database_url, &config.database_name).await;

    println!("{:?}/{:?}", config.database_url, config.database_name);

    let app_state = Arc::new(AppState::new(config, db));

    let frontend_url = std::env::var("FRONTEND_URL").ok();
    let cors = if let Some(ref origin_url) = frontend_url {
        if let Ok(header_val) = origin_url.parse::<axum::http::HeaderValue>() {
            CorsLayer::new()
                .allow_origin(header_val)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)
        }
    } else {
        CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)
    };

    // Assembly of routes
    let app = Router::new()
        .route("/", get(index_fn))
        .nest("/api/auth", auth_router())
        .nest("/api/share", share_diagram_router())
        .nest("/api/workspaces", workspace_router(app_state.clone()))
        .nest("/api/workspaces", diagram_router(app_state.clone()))
        .nest("/api/diagrams", diagram_router(app_state.clone()))
        .layer(from_fn(request_response_logger))
        .layer(cors)
        .with_state(app_state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_addr = format!("{}:{}", host, port);

    let tcp_listener = TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|_| panic!("Issue binding TcpListener to {}", bind_addr));

    println!("🚀 FLOWFRAME SERVER RUNNING: http://{}", bind_addr);

    axum::serve(tcp_listener, app)
        .await
        .expect("Axum not able to serve");
}

async fn index_fn() -> &'static str {
    "FlowFrame Server Status: 200 OK"
}
