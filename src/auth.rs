use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

/// How long a session token is valid (seconds).
pub const TOKEN_EXPIRY_SECS: i64 = 7 * 24 * 3600; // 7 days
pub const COOKIE_NAME: &str = "orsta_session";

// ---------------------------------------------------------------------------
// JWT claims
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject — user id (as string).
    pub sub: String,
    /// Username for convenience.
    pub username: String,
    /// Expiry timestamp (Unix seconds).
    pub exp: usize,
    /// Issued-at timestamp (Unix seconds).
    pub iat: usize,
}

fn jwt_secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .unwrap_or_else(|_| "orsta_default_secret_CHANGE_ME".to_string())
        .into_bytes()
}

/// Generate a signed JWT for the given user.
pub fn generate_token(user_id: i32, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: now + TOKEN_EXPIRY_SECS as usize,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret()),
    )
}

/// Validate and decode a JWT, returning the claims on success.
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&jwt_secret()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

// ---------------------------------------------------------------------------
// Password hashing
// ---------------------------------------------------------------------------

/// Hash a plaintext password with Argon2id.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// Verify a plaintext password against a stored Argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

// ---------------------------------------------------------------------------
// Encrypted Access Key (eakey)
// ---------------------------------------------------------------------------

/// Generate a unique, random 32-byte hex string to serve as the user's eakey.
pub fn generate_eakey() -> String {
    let bytes: [u8; 32] = rand::random();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ---------------------------------------------------------------------------
// Cookie helpers
// ---------------------------------------------------------------------------

/// Build a `Set-Cookie` header value for the session token.
pub fn session_cookie(token: &str) -> String {
    format!(
        "{}={}; HttpOnly; SameSite=Strict; Max-Age={}; Path=/",
        COOKIE_NAME, token, TOKEN_EXPIRY_SECS,
    )
}

/// Build a `Set-Cookie` header value that expires the session immediately.
pub fn clear_session_cookie() -> String {
    format!(
        "{}=; HttpOnly; SameSite=Strict; Max-Age=0; Path=/",
        COOKIE_NAME
    )
}

// ---------------------------------------------------------------------------
// Extractor: pull JWT from cookie or Authorization header
// ---------------------------------------------------------------------------

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

/// Axum extractor that validates the session token and yields the claims.
pub struct AuthUser(pub Claims);

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts).ok_or((StatusCode::UNAUTHORIZED, "Missing token"))?;
        let claims =
            validate_token(&token).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;
        Ok(AuthUser(claims))
    }
}

fn extract_token(parts: &Parts) -> Option<String> {
    // 1. Try cookie
    if let Some(cookie_header) = parts.headers.get("cookie") {
        if let Ok(val) = cookie_header.to_str() {
            for pair in val.split(';') {
                let pair = pair.trim();
                if let Some(v) = pair.strip_prefix(&format!("{}=", COOKIE_NAME)) {
                    return Some(v.to_string());
                }
            }
        }
    }
    // 2. Try Authorization: Bearer <token>
    if let Some(auth) = parts.headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            if let Some(token) = val.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    None
}
