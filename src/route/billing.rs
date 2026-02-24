use crate::{
    auth::AuthUser,
    payment::{PaymentDetails, PaymentProvider},
    sql::Orchestrator,
};
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct EnableApiKeyRequest {
    /// Payment amount to charge for API key activation.
    pub amount: f64,
    /// Human-readable reason shown in payment receipt.
    pub description: Option<String>,
    /// Provider-specific metadata (card token, etc.).
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// POST /billing/enable-api-key
// ---------------------------------------------------------------------------

pub async fn enable_api_key(
    AuthUser(claims): AuthUser,
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Extension(payment): Extension<Arc<dyn PaymentProvider>>,
    Json(body): Json<EnableApiKeyRequest>,
) -> impl IntoResponse {
    use crate::schema::{billing::dsl as bdsl, user_property::dsl::*};

    let uid: i32 = match claims.sub.parse() {
        Ok(v) => v,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token"}))),
    };

    // --- Attempt payment ---
    let details = PaymentDetails {
        amount: body.amount,
        description: body.description.unwrap_or_else(|| "Orsta API key activation".to_string()),
        metadata: body.metadata,
    };

    let outcome = payment.charge(&details).await;

    if !outcome.success {
        warn!(
            provider = outcome.provider,
            user_id = uid,
            "API key activation payment failed: {}",
            outcome.message
        );
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(serde_json::json!({
                "error": "Payment failed",
                "reason": outcome.message,
                "provider": outcome.provider,
            })),
        );
    }

    info!(
        provider = outcome.provider,
        txn = outcome.transaction_id,
        user_id = uid,
        amount = body.amount,
        "API key activation payment succeeded."
    );

    let mut db = orch.lock().await;

    // Activate API key
    let _ = diesel::update(user_property.filter(user_id.eq(uid)))
        .set(api_key_active.eq(true))
        .execute(&mut db.sqlite)
        .await;

    // Update billing records
    let _ = diesel::update(bdsl::billing.filter(bdsl::user_id.eq(uid)))
        .set((
            bdsl::amount_spent.eq(body.amount),
            bdsl::total_amount_spent.eq(
                bdsl::total_amount_spent + body.amount,
            ),
        ))
        .execute(&mut db.sqlite)
        .await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "message": "API key activated",
            "transaction_id": outcome.transaction_id,
            "provider": outcome.provider,
            "amount_charged": body.amount,
        })),
    )
}

// ---------------------------------------------------------------------------
// POST /billing/disable-api-key
// ---------------------------------------------------------------------------

pub async fn disable_api_key(
    AuthUser(claims): AuthUser,
    State(orch): State<Arc<Mutex<Orchestrator>>>,
) -> impl IntoResponse {
    use crate::schema::user_property::dsl::*;

    let uid: i32 = match claims.sub.parse() {
        Ok(v) => v,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token"}))),
    };

    let mut db = orch.lock().await;

    let result = diesel::update(user_property.filter(user_id.eq(uid)))
        .set(api_key_active.eq(false))
        .execute(&mut db.sqlite)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ok": true, "message": "API key deactivated"}))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to deactivate API key"}))),
    }
}

// ---------------------------------------------------------------------------
// GET /billing/api-key-status
// ---------------------------------------------------------------------------

pub async fn api_key_status(
    AuthUser(claims): AuthUser,
    State(orch): State<Arc<Mutex<Orchestrator>>>,
) -> impl IntoResponse {
    use crate::schema::{user_property::dsl as pdsl, users::dsl as udsl};
    use crate::sql::{user::User, user_property::UserProperty};

    let uid: i32 = match claims.sub.parse() {
        Ok(v) => v,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token"}))),
    };

    let mut db = orch.lock().await;

    let user_res = udsl::users
        .filter(udsl::id.eq(uid))
        .select(User::as_select())
        .first(&mut db.sqlite)
        .await;

    let prop_res: Result<UserProperty, _> = pdsl::user_property
        .filter(pdsl::user_id.eq(uid))
        .select(UserProperty::as_select())
        .first(&mut db.sqlite)
        .await;

    match (user_res, prop_res) {
        (Ok(user), Ok(prop)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "api_key": user.eakey,
                "active": prop.api_key_active,
            })),
        ),
        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "User not found"}))),
    }
}

// ---------------------------------------------------------------------------
// GET /billing/summary
// ---------------------------------------------------------------------------

pub async fn summary(
    AuthUser(claims): AuthUser,
    State(orch): State<Arc<Mutex<Orchestrator>>>,
) -> impl IntoResponse {
    use crate::schema::billing::dsl::*;
    use crate::sql::billing::Billing;

    let uid: i32 = match claims.sub.parse() {
        Ok(v) => v,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token"}))),
    };

    let mut db = orch.lock().await;

    let result: Result<Billing, _> = billing
        .filter(user_id.eq(uid))
        .select(Billing::as_select())
        .first(&mut db.sqlite)
        .await;

    match result {
        Ok(b) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "amount_in_wallet": b.amount_in_wallet,
                "amount_spent": b.amount_spent,
                "total_amount_spent": b.total_amount_spent,
                "average_hourly_consumption": b.average_hourly_consumption,
            })),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Billing record not found"}))),
    }
}

