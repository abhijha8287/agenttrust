mod db;
mod handlers;
mod worker;

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
    contracts_dir: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blockchain_service=info,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (blockchain-service owns its own database)");
    let identity_service_url = env::var("IDENTITY_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    let internal_secret = env::var("INTERNAL_SERVICE_SECRET")
        .expect("INTERNAL_SERVICE_SECRET must be set (deploy-time static secret)");
    // Where contracts/squads/chain-worker.ts lives — Squads v4 has no Rust
    // SDK, so the actual on-chain calls are shelled out to that script.
    let contracts_dir = env::var("CONTRACTS_DIR")
        .expect("CONTRACTS_DIR must be set (path to the contracts/ directory)");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8085".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to blockchain-service database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run blockchain-service migrations");

    let state = AppState {
        pool,
        http: reqwest::Client::new(),
        identity_service_url,
        internal_secret: internal_secret.clone(),
        contracts_dir,
    };

    let app = Router::new()
        .route("/agents/:id/anchor", post(handlers::anchor_execution))
        .route("/agents/:id/anchors", get(handlers::list_anchors))
        .route_layer(middleware::from_fn(move |req, next| {
            require_internal_secret(internal_secret.clone(), req, next)
        }))
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "blockchain-service listening");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");
    axum::serve(listener, app).await.expect("server error");
}
