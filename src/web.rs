use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;

pub async fn start() {
    let app = Router::new()
        .route("/", get(|| async { "Hello from web server!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}