use crate::schema;
use crate::sql::{Instance, NewInstance, Orchestrator};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_secs() as i64
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct UserQuery {
    user_id: i32,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /instances?user_id=<id>` – list all instances owned by a user.
pub async fn list_instances(
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Query(params): Query<UserQuery>,
) -> impl IntoResponse {
    let mut db = orch.lock().await;
    let conn = db.sqlite();

    match schema::instances::table
        .filter(schema::instances::user_id.eq(params.user_id))
        .select(Instance::as_select())
        .load(conn)
        .await
    {
        Ok(instances) => (StatusCode::OK, Json(instances)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// `POST /instances` – provision a new WhatsApp bot instance.
///
/// Body: `{ "user_id": 1, "country_code": "US", "phone_number": "+15550001234" }`
pub async fn create_instance(
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Json(new_instance): Json<NewInstance>,
) -> impl IntoResponse {
    let mut db = orch.lock().await;
    let conn = db.sqlite();

    let insert_result = diesel::insert_into(schema::instances::table)
        .values(&new_instance)
        .execute(conn)
        .await;

    match insert_result {
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Ok(_) => {}
    }

    // Fetch the newly created row (most recent for this user + phone number).
    match schema::instances::table
        .filter(schema::instances::user_id.eq(new_instance.user_id))
        .filter(schema::instances::phone_number.eq(&new_instance.phone_number))
        .order(schema::instances::id.desc())
        .select(Instance::as_select())
        .first(conn)
        .await
    {
        Ok(instance) => (StatusCode::CREATED, Json(instance)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// `PATCH /instances/:id/activate` – mark an instance as active and open a
/// billing window.
pub async fn activate_instance(
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    use crate::sql::NewBillingRecord;

    let now = now_unix();
    let mut db = orch.lock().await;
    let conn = db.sqlite();

    // Fetch the instance to get user_id and current state.
    let instance: Instance = match schema::instances::table
        .find(id)
        .select(Instance::as_select())
        .first(conn)
        .await
    {
        Ok(i) => i,
        Err(diesel::result::Error::NotFound) => {
            return (StatusCode::NOT_FOUND, "instance not found").into_response()
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if instance.active == 1 {
        return (StatusCode::CONFLICT, "instance is already active").into_response();
    }

    // Mark active.
    if let Err(e) = diesel::update(schema::instances::table.find(id))
        .set(schema::instances::active.eq(1))
        .execute(conn)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Open a new billing window.
    let new_record = NewBillingRecord {
        instance_id: id,
        user_id: instance.user_id,
        started_at: now,
    };

    match diesel::insert_into(crate::schema::billing_records::table)
        .values(&new_record)
        .execute(conn)
        .await
    {
        Ok(_) => (StatusCode::OK, "instance activated").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// `PATCH /instances/:id/deactivate` – mark an instance as inactive, close
/// the open billing window and compute the charge.
pub async fn deactivate_instance(
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    use crate::sql::calculate_charge_cents;

    let now = now_unix();
    let mut db = orch.lock().await;
    let conn = db.sqlite();

    // Fetch the instance.
    let instance: Instance = match schema::instances::table
        .find(id)
        .select(Instance::as_select())
        .first(conn)
        .await
    {
        Ok(i) => i,
        Err(diesel::result::Error::NotFound) => {
            return (StatusCode::NOT_FOUND, "instance not found").into_response()
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if instance.active == 0 {
        return (StatusCode::CONFLICT, "instance is already inactive").into_response();
    }

    // Fetch the user to determine promotion eligibility.
    let user: crate::sql::User = match schema::users::table
        .find(instance.user_id)
        .select(crate::sql::User::as_select())
        .first(conn)
        .await
    {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Fetch the open billing record for this instance.
    let open_record: crate::sql::BillingRecord = match crate::schema::billing_records::table
        .filter(crate::schema::billing_records::instance_id.eq(id))
        .filter(crate::schema::billing_records::ended_at.is_null())
        .select(crate::sql::BillingRecord::as_select())
        .first(conn)
        .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let duration_secs = now - open_record.started_at;
    let charge = calculate_charge_cents(duration_secs, user.created_at, open_record.started_at);

    // Close the billing record and persist the charge.
    if let Err(e) = diesel::update(
        crate::schema::billing_records::table.find(open_record.id),
    )
    .set((
        crate::schema::billing_records::ended_at.eq(now),
        crate::schema::billing_records::amount_cents.eq(charge),
    ))
    .execute(conn)
    .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Mark instance inactive.
    if let Err(e) = diesel::update(schema::instances::table.find(id))
        .set(schema::instances::active.eq(0))
        .execute(conn)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    (StatusCode::OK, "instance deactivated").into_response()
}
