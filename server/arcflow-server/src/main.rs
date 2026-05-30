//! ArcFlow runtime HTTP server (Sprint 8 + Phase 1.3).

mod dto;
mod exec_config;
mod handlers;
mod middleware;
mod state;
mod store;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

use state::AppState;

#[tokio::main]
async fn main() {
    arcflow_core::tracing::otel_live::init_tracing_subscriber("arcflow_server=info");

    let api_key = std::env::var("ARCFLOW_SERVER_API_KEY").unwrap_or_else(|_| {
        eprintln!("[ArcFlow] ARCFLOW_SERVER_API_KEY must be set.");
        std::process::exit(1);
    });
    let port: u16 = std::env::var("ARCFLOW_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let state = Arc::new(AppState::from_env(api_key).await);
    let public = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/ready", get(handlers::ready::ready));

    let protected = Router::new()
        .route("/v1/workflows/run", post(handlers::workflow::run_workflow))
        .route("/v1/runs", post(handlers::runs::create_run))
        .route("/v1/runs/:run_id", get(handlers::runs::get_run))
        .route(
            "/v1/runs/:run_id/trace",
            get(handlers::runs::get_run_trace),
        )
        .route(
            "/v1/runs/:run_id/approve/:approval_key",
            post(handlers::approve::approve_run),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth::require_api_key,
        ))
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    let app = public.merge(protected).with_state(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!(%addr, "arcflow-server listening");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
