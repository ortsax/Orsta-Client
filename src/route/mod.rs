pub mod auth;
pub mod billing;
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
        .route("/health", get(|| async { "OK" }))
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/me", get(auth::me))
        .route("/billing/enable-api-key", post(billing::enable_api_key))
        .route("/billing/disable-api-key", post(billing::disable_api_key))
        .route("/billing/api-key-status", get(billing::api_key_status))
        .route("/billing/summary", get(billing::summary))
        .route("/ws", get(ws::ws_handler))
        .with_state(orch)
        .layer(cors)
}
