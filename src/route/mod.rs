pub mod auth;
pub mod user;
pub mod ws;

use crate::sql::Orchestrator;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

pub fn start_client_api_service(orch: Arc<Mutex<Orchestrator>>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check (no auth required)
        .route("/health", get(|| async { "OK" }))
        // Auth — plain HTTP (cookie + JSON response)
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        // Authenticated HTTP example
        .route("/me", get(auth::me))
        // WebSocket service — token required
        .route("/ws", get(ws::ws_handler))
        .with_state(orch)
        .layer(cors)
}
