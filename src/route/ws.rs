use crate::{
    auth::{COOKIE_NAME, validate_token},
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

/// Upgrade to WebSocket only when a valid session token is present.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(orch): State<Arc<Mutex<Orchestrator>>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let token = extract_bearer_or_cookie(&headers);
    match token.as_deref().map(validate_token) {
        Some(Ok(claims)) => ws.on_upgrade(move |socket| handle_socket(socket, claims, orch)),
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
