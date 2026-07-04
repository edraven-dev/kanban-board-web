use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("0.0.0.0:5000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
