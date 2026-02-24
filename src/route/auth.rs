use crate::{
    auth::{
        AuthUser, clear_session_cookie, generate_eakey, generate_token, hash_password,
        session_cookie, verify_password,
    },
    sql::{
        Orchestrator,
        billing::NewBilling,
        instance::NewInstance,
        user::{NewUser, User},
        user_property::NewUserProperty,
    },
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub passkey: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// ---------------------------------------------------------------------------
// POST /auth/signup
// ---------------------------------------------------------------------------

pub async fn signup(
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Json(body): Json<SignupRequest>,
) -> impl IntoResponse {
    use crate::schema::users::dsl::*;

    let hashed_password = match hash_password(&body.password) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                Json(serde_json::json!({"error": "Password hashing failed"})),
            );
        }
    };

    let eakey_val = generate_eakey();
    let new_user = NewUser {
        username: body.username.clone(),
        email: body.email.clone(),
        password_hash: hashed_password,
        passkey: body.passkey.clone(),
        eakey: eakey_val.clone(),
    };

    let mut db = orch.lock().await;

    let insert_result = diesel::insert_into(users)
        .values(&new_user)
        .execute(&mut db.sqlite)
        .await;

    if let Err(e) = insert_result {
        let msg = if e.to_string().contains("UNIQUE") {
            "Username or email already exists"
        } else {
            "Failed to create user"
        };
        return (
            StatusCode::CONFLICT,
            HeaderMap::new(),
            Json(serde_json::json!({"error": msg})),
        );
    }

    // Fetch the newly created user row by email
    let user: User = match users
        .filter(email.eq(&body.email))
        .select(User::as_select())
        .first(&mut db.sqlite)
        .await
    {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                Json(serde_json::json!({"error": "Failed to retrieve created user"})),
            );
        }
    };

    // Create companion rows
    let _ = diesel::insert_into(crate::schema::user_property::table)
        .values(&NewUserProperty {
            user_id: user.id,
            instance_status: "inactive".to_string(),
            instance_usage: 0.0,
        })
        .execute(&mut db.sqlite)
        .await;

    let _ = diesel::insert_into(crate::schema::instances::table)
        .values(&NewInstance {
            user_id: user.id,
            instances_count: 0,
            expected_consumption: 0.0,
            instances_overall_consumption: 0.0,
        })
        .execute(&mut db.sqlite)
        .await;

    let _ = diesel::insert_into(crate::schema::billing::table)
        .values(&NewBilling {
            user_id: user.id,
            amount_in_wallet: 0.0,
            amount_spent: 0.0,
            total_amount_spent: 0.0,
            average_hourly_consumption: 0.0,
        })
        .execute(&mut db.sqlite)
        .await;

    let token = match generate_token(user.id, &user.username) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                Json(serde_json::json!({"error": "Token generation failed"})),
            );
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "set-cookie",
        HeaderValue::from_str(&session_cookie(&token)).unwrap(),
    );

    (
        StatusCode::CREATED,
        headers,
        Json(serde_json::json!({
            "token": token,
            "user_id": user.id,
            "username": user.username,
            "eakey": user.eakey,
        })),
    )
}

// ---------------------------------------------------------------------------
// POST /auth/login
// ---------------------------------------------------------------------------

pub async fn login(
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    use crate::schema::users::dsl::*;

    let mut db = orch.lock().await;

    let result: QueryResult<User> = users
        .filter(email.eq(&body.email))
        .select(User::as_select())
        .first(&mut db.sqlite)
        .await;

    let user = match result {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                HeaderMap::new(),
                Json(serde_json::json!({"error": "Invalid credentials"})),
            );
        }
    };

    if !verify_password(&body.password, &user.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            HeaderMap::new(),
            Json(serde_json::json!({"error": "Invalid credentials"})),
        );
    }

    let token = match generate_token(user.id, &user.username) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                Json(serde_json::json!({"error": "Token generation failed"})),
            );
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "set-cookie",
        HeaderValue::from_str(&session_cookie(&token)).unwrap(),
    );

    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({
            "token": token,
            "user_id": user.id,
            "username": user.username,
            "eakey": user.eakey,
        })),
    )
}

// ---------------------------------------------------------------------------
// POST /auth/logout
// ---------------------------------------------------------------------------

pub async fn logout() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "set-cookie",
        HeaderValue::from_str(&clear_session_cookie()).unwrap(),
    );
    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({"ok": true})),
    )
}

// ---------------------------------------------------------------------------
// GET /me  (example authenticated HTTP endpoint)
// ---------------------------------------------------------------------------

pub async fn me(AuthUser(claims): AuthUser) -> impl IntoResponse {
    Json(serde_json::json!({
        "user_id": claims.sub,
        "username": claims.username,
    }))
}
