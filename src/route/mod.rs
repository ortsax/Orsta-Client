mod billing;
mod instance;

use crate::sql::Orchestrator;
use axum::{Router, routing::{get, patch, post}};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn start_client_api_service(orch: Arc<Mutex<Orchestrator>>) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        // Instance management
        .route("/instances", get(instance::list_instances))
        .route("/instances", post(instance::create_instance))
        .route("/instances/:id/activate", patch(instance::activate_instance))
        .route("/instances/:id/deactivate", patch(instance::deactivate_instance))
        // Billing
        .route("/billing", get(billing::get_billing))
        .with_state(orch)
}
