use crate::sql::Orchestrator;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub async fn u_handler(State(_orch): State<Arc<Mutex<Orchestrator>>>) -> impl IntoResponse {
    "Success"
}
