use tokio::net::{TcpListener};
use axum::{routing::get, Router};
mod models;
mod shared;
mod state;
mod config;

#[tokio::main]
async fn main() {

    let app = Router::new()
                .route("/", get(index_fn));
    
    let tcp_listener = TcpListener::bind("127.0.0.1:8000").await.expect("Issue in tcp_listener");

    println!("SERVER RUNNING: 127.0.0.1:8000");

    axum::serve(tcp_listener, app).await.expect("Axum not able to serve");
}

async fn index_fn() -> &'static str {
    return "Hello Index Page";
}
