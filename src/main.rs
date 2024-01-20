use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use axum::{
    routing,
    Router,
    response::Redirect,
};

mod get;
mod ring;

#[tokio::main]
async fn main() {
    let mut state = ring::Ring::new();
    state.initialize_from_json_file("members.json").await;
    state.link_members().await;

    let state = Arc::new(RwLock::new(state));

    let app = Router::new()
        .route("/", 
            routing::get(|| async { Redirect::temporary("https://github.com/umaring/umaring") })
        )
        .route("/all", routing::get(get::all))
        .route("/:id", routing::get(get::one))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

