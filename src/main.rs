use axum::{
    response::{Redirect, Response},
    routing, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

mod get;
mod health;
mod ring;

#[tokio::main]
async fn main() {
    let ring = ring::Ring::new(include_str!("../members.toml"));

    let ring = Arc::new(RwLock::new(ring));

    // Shuffle the ring every hour
    let ring_clone = ring.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await;

            let mut ring = ring_clone.write().await;

            ring.shuffle();
        }
    });

    // Initial health scan of all members on startup
    let ring_clone = ring.clone();
    tokio::spawn(async move {
        health::scan_all(ring_clone.clone()).await;
        // After initial scan, start the per-minute check loop
        health::health_check_loop(ring_clone).await;
    });

    let app = Router::new()
        .route(
            "/",
            routing::get(|| async { Redirect::temporary("https://github.com/umacabal/umaring") }),
        )
        .route("/health", routing::get(health))
        .route("/status", routing::get(get::status))
        .route("/all", routing::get(get::all))
        .route("/:id", routing::get(get::one))
        .route("/:id/prev", routing::get(get::prev))
        .route("/:id/next", routing::get(get::next))
        .route("/ring.js", routing::get(get::ring_js))
        .layer(CorsLayer::permissive())
        .with_state(ring);

    let bind_addr = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();

    println!("Listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Response<String> {
    let commit = std::env::var("COMMIT").unwrap_or("unknown".to_string());

    Response::builder()
        .header("Content-Type", "text/plain")
        .body(format!("OK\n{}", commit))
        .unwrap()
}
