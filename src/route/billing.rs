use crate::schema;
use crate::sql::{BillingRecord, Orchestrator};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct UserQuery {
    user_id: i32,
}

/// Summary returned by `GET /billing`.
#[derive(Serialize)]
pub struct BillingSummary {
    pub user_id: i32,
    /// All billing windows (open and closed) for this user.
    pub records: Vec<BillingRecord>,
    /// Total charge across all *closed* windows (in cents).
    pub total_cents: i32,
}

/// `GET /billing?user_id=<id>` – retrieve billing records and running total
/// for a user.
///
/// Only *closed* windows (where `ended_at IS NOT NULL`) contribute to
/// `total_cents`; open windows are included in `records` but their
/// `amount_cents` will be 0 until the instance is deactivated.
pub async fn get_billing(
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Query(params): Query<UserQuery>,
) -> impl IntoResponse {
    let mut db = orch.lock().await;
    let conn = db.sqlite();

    match schema::billing_records::table
        .filter(schema::billing_records::user_id.eq(params.user_id))
        .select(BillingRecord::as_select())
        .load(conn)
        .await
    {
        Ok(records) => {
            let total_cents: i32 = records.iter().map(|r| r.amount_cents).sum();
            let summary = BillingSummary {
                user_id: params.user_id,
                records,
                total_cents,
            };
            (StatusCode::OK, Json(summary)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
