use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod config;
mod db;
mod dtos;
mod handlers;
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

#[tokio::main]
async fn main() {
    // Load .env from src/.env or root folder
    dotenvy::from_filename("src/.env").ok();
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let db = create_database(&config.database_url, &config.database_name).await;

    // why Arc
    /*
     * Rust restricts that variable only will have 1 owner
     * in axum runs multiple threads for handling requests, so rust will restrict when we do not use Arc
     * after use Arc now means add that data into heap and then all threads shares ownership same data
     */
    let app_state = Arc::new(AppState::new(config, db));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index_fn))
        .nest("/api/auth", auth_router())
        .layer(cors)
        .with_state(app_state);

    let tcp_listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Issue in tcp_listener");

    println!("SERVER RUNNING: http://127.0.0.1:8000");

    axum::serve(tcp_listener, app)
        .await
        .expect("Axum not able to serve");
}

async fn index_fn() -> &'static str {
    "Hello FlowFrame Server"
}
