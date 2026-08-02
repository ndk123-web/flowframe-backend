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
    // Load .env
    dotenvy::from_filename("src/.env").ok();
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let db = create_database(&config.database_url, &config.database_name).await;

    let app_state = Arc::new(AppState::new(config, db));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Assembly of routes
    let app = Router::new()
        .route("/", get(index_fn))
        .nest("/api/auth", auth_router())
        .layer(from_fn(request_response_logger))
        .layer(cors)
        .with_state(app_state);

    let tcp_listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Issue in tcp_listener");

    println!("🚀 FLOWFRAME SERVER RUNNING: http://127.0.0.1:8000");

    axum::serve(tcp_listener, app)
        .await
        .expect("Axum not able to serve");
}

async fn index_fn() -> &'static str {
    "FlowFrame Server Status: 200 OK"
}
