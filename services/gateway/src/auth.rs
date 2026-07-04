//! External client auth (client -> gateway). Distinct token type from the
//! internal service-to-service secret in `shared::internal_auth` — the two
//! must never be interchangeable (eng review, 2026-07-04).
//!
//! This is JWT bearer auth as the working mechanism today. Wallet-signature
//! auth is still an open question in the design doc (which wallets, which
//! chains for user-side auth vs. agent-side custody) — this module is the
//! seam that gets extended once that's decided, not a placeholder standing
//! in for it.

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn require_client_jwt(
    jwt_secret: String,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
