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
struct AppState {
    http: reqwest::Client,
    identity_service_url: String,
    internal_secret: String,
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
        internal_secret,
    };

    let app = Router::new()
        .route("/agents", post(proxy_register_agent))
        .route("/agents/:id", get(proxy_get_agent))
        .route_layer(middleware::from_fn(move |req, next| {
            auth::require_client_jwt(jwt_secret.clone(), req, next)
        }))
        .route("/health", get(health))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "gateway listening");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");
    axum::serve(listener, app).await.expect("server error");
}
