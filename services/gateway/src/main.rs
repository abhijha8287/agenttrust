mod auth;

use axum::{
    body::{to_bytes, Body},
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::env;

#[derive(Clone)]
pub struct AppState {
    http: reqwest::Client,
    identity_service_url: String,
    agent_core_url: String,
    audit_engine_url: String,
    blockchain_service_url: String,
    internal_secret: String,
    jwt_secret: String,
}

const MAX_PROXY_BODY_BYTES: usize = 1024 * 1024; // 1 MiB — agent registration payloads are small

async fn proxy_register_agent(
    State(state): State<AppState>,
    body: Body,
) -> Result<Response, StatusCode> {
    let bytes = to_bytes(body, MAX_PROXY_BODY_BYTES)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let resp = state
        .http
        .post(format!("{}/agents", state.identity_service_url))
        .header("x-internal-secret", &state.internal_secret)
        .header("content-type", "application/json")
        .body(bytes)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "identity-service request failed");
            StatusCode::BAD_GATEWAY
        })?;

    forward_response(resp).await
}

async fn proxy_get_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let resp = state
        .http
        .get(format!("{}/agents/{}", state.identity_service_url, id))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "identity-service request failed");
            StatusCode::BAD_GATEWAY
        })?;

    forward_response(resp).await
}

async fn proxy_list_agents(State(state): State<AppState>) -> Result<Response, StatusCode> {
    let resp = state
        .http
        .get(format!("{}/agents", state.identity_service_url))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "identity-service request failed");
            StatusCode::BAD_GATEWAY
        })?;

    forward_response(resp).await
}

async fn proxy_execute_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Body,
) -> Result<Response, StatusCode> {
    let bytes = to_bytes(body, MAX_PROXY_BODY_BYTES)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let resp = state
        .http
        .post(format!("{}/agents/{}/execute", state.agent_core_url, id))
        .header("x-internal-secret", &state.internal_secret)
        .header("content-type", "application/json")
        .body(bytes)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "agent-core request failed");
            StatusCode::BAD_GATEWAY
        })?;

    forward_response(resp).await
}

async fn proxy_list_executions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let resp = state
        .http
        .get(format!("{}/agents/{}/executions", state.agent_core_url, id))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "agent-core request failed");
            StatusCode::BAD_GATEWAY
        })?;

    forward_response(resp).await
}

async fn proxy_list_audits(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let resp = state
        .http
        .get(format!("{}/agents/{}/audits", state.audit_engine_url, id))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "audit-engine request failed");
            StatusCode::BAD_GATEWAY
        })?;

    forward_response(resp).await
}

async fn proxy_list_anchors(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let resp = state
        .http
        .get(format!(
            "{}/agents/{}/anchors",
            state.blockchain_service_url, id
        ))
        .header("x-internal-secret", &state.internal_secret)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "blockchain-service request failed");
            StatusCode::BAD_GATEWAY
        })?;

    forward_response(resp).await
}

async fn forward_response(resp: reqwest::Response) -> Result<Response, StatusCode> {
    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let bytes = resp
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok((status, bytes).into_response())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gateway=info,tower_http=info".into()),
        )
        .init();

    let identity_service_url = env::var("IDENTITY_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    let agent_core_url =
        env::var("AGENT_CORE_URL").unwrap_or_else(|_| "http://localhost:8083".to_string());
    let audit_engine_url =
        env::var("AUDIT_ENGINE_URL").unwrap_or_else(|_| "http://localhost:8084".to_string());
    let blockchain_service_url = env::var("BLOCKCHAIN_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8085".to_string());
    let internal_secret = env::var("INTERNAL_SERVICE_SECRET")
        .expect("INTERNAL_SERVICE_SECRET must be set (deploy-time static secret)");
    let jwt_secret = env::var("CLIENT_JWT_SECRET")
        .expect("CLIENT_JWT_SECRET must be set (external client auth)");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let state = AppState {
        http: reqwest::Client::new(),
        identity_service_url,
        agent_core_url,
        audit_engine_url,
        blockchain_service_url,
        internal_secret,
        jwt_secret: jwt_secret.clone(),
    };

    let protected = Router::new()
        .route(
            "/agents",
            post(proxy_register_agent).get(proxy_list_agents),
        )
        .route("/agents/:id", get(proxy_get_agent))
        .route("/agents/:id/execute", post(proxy_execute_action))
        .route("/agents/:id/executions", get(proxy_list_executions))
        .route("/agents/:id/audits", get(proxy_list_audits))
        .route("/agents/:id/anchors", get(proxy_list_anchors))
        .route_layer(middleware::from_fn(move |req, next| {
            auth::require_client_jwt(jwt_secret.clone(), req, next)
        }));

    let app = Router::new()
        .merge(protected)
        .route("/dev/session", post(auth::issue_dev_session))
        .route("/health", get(health))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "gateway listening");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");
    axum::serve(listener, app).await.expect("server error");
}
