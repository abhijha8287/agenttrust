mod db;
mod handlers;

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
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "identity_service=info,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (identity-service owns its own database)");
    let internal_secret = env::var("INTERNAL_SERVICE_SECRET")
        .expect("INTERNAL_SERVICE_SECRET must be set (deploy-time static secret)");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to identity-service database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run identity-service migrations");

    let state = AppState { pool };

    let app = Router::new()
        .route("/agents", post(handlers::register_agent))
        .route("/agents/:id", get(handlers::get_agent))
        .route_layer(middleware::from_fn(move |req, next| {
            require_internal_secret(internal_secret.clone(), req, next)
        }))
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "identity-service listening");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
