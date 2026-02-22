use crate::sql::Orchestrator;
use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn u_handler(State(orch): State<Arc<Mutex<Orchestrator>>>) -> impl IntoResponse {
    let mut db = orch.lock().await;
    // db.sync_write(...).await;
    "Success"
}
