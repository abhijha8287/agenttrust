mod db;
mod handlers;
mod judge;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use shared::internal_auth::require_internal_secret;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Clone)]
pub struct AppState {
    pool: sqlx::PgPool,
    http: reqwest::Client,
    identity_service_url: String,
    internal_secret: String,
    gemini_api_key: String,
    gemini_model: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "audit_engine=info,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (audit-engine owns its own database)");
    let identity_service_url = env::var("IDENTITY_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    let internal_secret = env::var("INTERNAL_SERVICE_SECRET")
        .expect("INTERNAL_SERVICE_SECRET must be set (deploy-time static secret)");
    let gemini_api_key = env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY must be set (audit-engine calls real judge models)");
    // Overridable, not hardcoded — if this default model is ever retired,
    // fixing it is an env var change, not a redeploy.
    let gemini_model =
        env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8084".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to audit-engine database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run audit-engine migrations");

    let state = AppState {
        pool,
        http: reqwest::Client::new(),
        identity_service_url,
        internal_secret: internal_secret.clone(),
        gemini_api_key,
        gemini_model,
    };

    let app = Router::new()
        .route("/audit", post(handlers::audit_execution))
        .route("/agents/:id/audits", get(handlers::list_audits))
        .route_layer(middleware::from_fn(move |req, next| {
            require_internal_secret(internal_secret.clone(), req, next)
        }))
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "audit-engine listening");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");
    axum::serve(listener, app).await.expect("server error");
}
