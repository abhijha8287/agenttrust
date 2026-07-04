//! policy-engine: the single source of truth for "is this agent allowed to
//! do this?" It owns no database of its own — permissions live in
//! identity-service (eng review Finding 1: each service owns its own data),
//! so this service calls identity-service's API for the current permission
//! set and applies decision rules on top. Stateless by design, not by
//! omission: there's nothing here that needs its own storage.

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use shared::{internal_auth::require_internal_secret, AppError};
use std::env;
use uuid::Uuid;

const KNOWN_RESOURCES: &[&str] = &[
    "filesystem",
    "terminal",
    "github",
    "database",
    "email",
    "cloud",
    "browser",
    "wallet",
];

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    identity_service_url: String,
    internal_secret: String,
}

#[derive(Debug, Deserialize)]
struct EvaluateRequest {
    agent_id: Uuid,
    resource: String,
}

#[derive(Debug, Serialize)]
struct EvaluateResponse {
    decision: String,
    resource: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct Permission {
    resource: String,
    mode: String,
}

#[derive(Debug, Deserialize)]
struct AgentDetail {
    permissions: Vec<Permission>,
}

async fn evaluate(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, AppError> {
    let resource = req.resource.to_lowercase();
    if !KNOWN_RESOURCES.contains(&resource.as_str()) {
        return Err(AppError::BadRequest(format!(
            "unknown resource '{resource}'"
        )));
    }

    let resp = state
        .http
        .get(format!(
            "{}/agents/{}",
            state.identity_service_url, req.agent_id
        ))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("identity-service request failed: {e}")))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::NotFound(format!(
            "agent {} not found",
            req.agent_id
        )));
    }
    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "identity-service returned {}",
            resp.status()
        )));
    }

    let detail: AgentDetail = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("bad identity-service response: {e}")))?;

    let permission = detail.permissions.iter().find(|p| p.resource == resource);

    // No explicit grant means deny — permissions are opt-in, never opt-out
    // (matches identity-service's registration contract).
    let (decision, reason) = match permission.map(|p| p.mode.as_str()) {
        Some("allow") => ("allow", "explicit allow grant for this resource".to_string()),
        Some("conditional") => (
            "conditional",
            "resource is conditional — allowed, flagged for audit review".to_string(),
        ),
        Some("deny") | None => (
            "deny",
            "no allow/conditional grant found for this resource".to_string(),
        ),
        Some(other) => (
            "deny",
            format!("unrecognized permission mode '{other}', denying by default"),
        ),
    };

    Ok(Json(EvaluateResponse {
        decision: decision.to_string(),
        resource,
        reason,
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "policy_engine=info,tower_http=info".into()),
        )
        .init();

    let identity_service_url = env::var("IDENTITY_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    let internal_secret = env::var("INTERNAL_SERVICE_SECRET")
        .expect("INTERNAL_SERVICE_SECRET must be set (deploy-time static secret)");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let state = AppState {
        http: reqwest::Client::new(),
        identity_service_url,
        internal_secret: internal_secret.clone(),
    };

    let app = Router::new()
        .route("/evaluate", post(evaluate))
        .route_layer(middleware::from_fn(move |req, next| {
            require_internal_secret(internal_secret.clone(), req, next)
        }))
        .route("/health", get(health))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "policy-engine listening");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");
    axum::serve(listener, app).await.expect("server error");
}
