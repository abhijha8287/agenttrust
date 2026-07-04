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
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct DevSessionRequest {
    pub sub: String,
}

#[derive(Debug, Serialize)]
pub struct DevSessionResponse {
    pub token: String,
}

/// Dev-only session issuance — mints a JWT for any caller-supplied `sub`
/// with no verification of who they claim to be. Stand-in for the
/// wallet-signature auth that's still an open design-doc question (see
/// module doc comment); this exists so the frontend has a token to send
/// before that's built, not a decision that this is how auth will work.
pub async fn issue_dev_session(
    State(state): State<crate::AppState>,
    Json(req): Json<DevSessionRequest>,
) -> Result<Json<DevSessionResponse>, StatusCode> {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims { sub: req.sub, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(DevSessionResponse { token }))
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
