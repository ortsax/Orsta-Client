use crate::{
    auth::{Claims, COOKIE_NAME, validate_token},
    sql::Orchestrator,
};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
};
use chrono;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Message envelope
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct WsIncoming {
    action: String,
    #[allow(dead_code)]
    payload: Option<Value>,
}

#[derive(Serialize)]
struct WsOutgoing {
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// /ws  upgrade handler
// ---------------------------------------------------------------------------

/// Upgrade to WebSocket. Accepts two auth methods:
///   1. JWT — `Authorization: Bearer <token>` or `orsta_session` cookie
///   2. API Key — `X-Api-Key: <eakey>` (must have api_key_active = true)
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // --- API Key path ---
    if let Some(api_key) = extract_api_key(&headers) {
        return match resolve_api_key(&api_key, &orch).await {
            Some(claims) => ws.on_upgrade(move |socket| handle_socket(socket, claims, orch)).into_response(),
            None => (StatusCode::UNAUTHORIZED, "Invalid or inactive API key").into_response(),
        };
    }

    // --- JWT path ---
    match extract_bearer_or_cookie(&headers).as_deref().map(validate_token) {
        Some(Ok(claims)) => ws.on_upgrade(move |socket| handle_socket(socket, claims, orch)).into_response(),
        _ => (StatusCode::UNAUTHORIZED, "Missing or invalid session token").into_response(),
    }
}

// ---------------------------------------------------------------------------
// Per-connection handler
// ---------------------------------------------------------------------------

async fn handle_socket(
    socket: WebSocket,
    claims: crate::auth::Claims,
    orch: Arc<Mutex<Orchestrator>>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Greet the client.
    let welcome = serde_json::to_string(&WsOutgoing {
        action: "connected".to_string(),
        data: Some(serde_json::json!({
            "user_id": claims.sub,
            "username": claims.username,
        })),
        error: None,
    })
    .unwrap();
    let _ = sender.send(Message::Text(welcome.into())).await;

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                let response = dispatch(&text, &claims, &orch).await;
                let _ = sender
                    .send(Message::Text(
                        serde_json::to_string(&response).unwrap().into(),
                    ))
                    .await;
            }
            Message::Close(_) => break,
            Message::Ping(data) => {
                let _ = sender.send(Message::Pong(data)).await;
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Action dispatcher
// ---------------------------------------------------------------------------

async fn dispatch(
    text: &str,
    claims: &crate::auth::Claims,
    _orch: &Arc<Mutex<Orchestrator>>,
) -> WsOutgoing {
    let incoming: WsIncoming = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => {
            return WsOutgoing {
                action: "error".to_string(),
                data: None,
                error: Some("Invalid JSON message".to_string()),
            };
        }
    };

    match incoming.action.as_str() {
        "ping" => WsOutgoing {
            action: "pong".to_string(),
            data: Some(serde_json::json!({"user": claims.username})),
            error: None,
        },
        "whoami" => WsOutgoing {
            action: "whoami".to_string(),
            data: Some(serde_json::json!({
                "user_id": claims.sub,
                "username": claims.username,
            })),
            error: None,
        },
        unknown => WsOutgoing {
            action: "error".to_string(),
            data: None,
            error: Some(format!("Unknown action: {}", unknown)),
        },
    }
}

// ---------------------------------------------------------------------------
// Token extraction helpers
// ---------------------------------------------------------------------------

/// Try `Authorization: Bearer` header, then cookie.
fn extract_bearer_or_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    // 1. Authorization header
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            if let Some(t) = val.strip_prefix("Bearer ") {
                return Some(t.to_string());
            }
        }
    }
    // 2. Cookie
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(val) = cookie_header.to_str() {
            for pair in val.split(';') {
                let pair = pair.trim();
                if let Some(v) = pair.strip_prefix(&format!("{}=", COOKIE_NAME)) {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Extract the value of the `X-Api-Key` header.
fn extract_api_key(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Look up a user by their eakey and return Claims if api_key_active is true.
async fn resolve_api_key(
    key: &str,
    orch: &Arc<Mutex<Orchestrator>>,
) -> Option<Claims> {
    use crate::schema::{user_property::dsl as prop, users::dsl as udsl};
    use crate::sql::user::User;
    use crate::sql::user_property::UserProperty;

    let mut db = orch.lock().await;

    let user: User = udsl::users
        .filter(udsl::eakey.eq(key))
        .select(User::as_select())
        .first(&mut db.sqlite)
        .await
        .ok()?;

    let property: UserProperty = prop::user_property
        .filter(prop::user_id.eq(user.id))
        .select(UserProperty::as_select())
        .first(&mut db.sqlite)
        .await
        .ok()?;

    if !property.api_key_active {
        return None;
    }

    let now = chrono::Utc::now().timestamp() as usize;
    Some(Claims {
        sub: user.id.to_string(),
        username: user.username,
        exp: now + crate::auth::TOKEN_EXPIRY_SECS as usize,
        iat: now,
    })
}
