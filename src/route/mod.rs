use crate::sql::Orchestrator;
use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn start_client_api_service(orch: Arc<Mutex<Orchestrator>>) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .with_state(orch)
}
