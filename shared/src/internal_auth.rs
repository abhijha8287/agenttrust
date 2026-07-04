//! Service-to-service auth: deploy-time static secrets, not live JWT issuance.
//!
//! Per the eng review (2026-07-04), no service blocks on identity-service's
//! uptime just to make its next request — the original design's circular
//! bootstrap problem (gateway needing a live token from identity-service,
//! which is itself one of the services behind gateway) is avoided entirely
//! by injecting each service's credential at deploy time. identity-service
//! only handles scheduled rotation, never routine request-time validation.

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

const INTERNAL_SECRET_HEADER: &str = "x-internal-secret";

/// Constant-time comparison — avoids leaking secret length/prefix via timing.
fn secrets_match(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Axum middleware: rejects any request without a valid `X-Internal-Secret`
/// header matching this service's configured secret. Apply to every internal
/// (service-to-service) route; never apply to routes gateway exposes to
/// external clients — those use client JWT/wallet auth instead, a distinct
/// token type that must never be interchangeable with this one.
pub async fn require_internal_secret(
    expected_secret: String,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let provided = request
        .headers()
        .get(INTERNAL_SECRET_HEADER)
        .and_then(|v| v.to_str().ok());

    match provided {
        Some(secret) if secrets_match(secret, &expected_secret) => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_identical_secrets() {
        assert!(secrets_match("abc123", "abc123"));
    }

    #[test]
    fn rejects_different_secrets() {
        assert!(!secrets_match("abc123", "abc124"));
    }

    #[test]
    fn rejects_different_length_secrets() {
        assert!(!secrets_match("abc", "abc123"));
    }
}
