use axum::routing::{get, post};
use axum::Router;

pub fn create_routes() -> Router {
    Router::new()
        .route("/query", post(execute_query))
        .route("/health", get(health_check))
}

async fn execute_query() -> &'static str {
    "Query execution not yet implemented"
}

async fn health_check() -> &'static str {
    "OK"
}