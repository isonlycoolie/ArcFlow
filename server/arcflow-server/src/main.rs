//! ArcFlow runtime HTTP server (Sprint 8 + Phase 1.3).

use std::net::SocketAddr;
use std::sync::Arc;

use arcflow_server::{build_app, AppState};

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
    let app = build_app(state.clone());

    #[cfg(feature = "debug-endpoints")]
    let app = if std::env::var("ARCFLOW_DEBUG")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        use axum::routing::{get, post};
        use axum::Router;
        let debug_routes = Router::new()
            .route("/v1/debug/runs/start", post(arcflow_server::debug::start_debug_run))
            .route(
                "/v1/debug/runs/:run_id/state",
                get(arcflow_server::debug::get_debug_state),
            )
            .route(
                "/v1/debug/runs/:run_id/continue",
                post(arcflow_server::debug::continue_debug_run),
            )
            .with_state(state);
        app.merge(debug_routes)
    } else {
        app
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!(%addr, "arcflow-server listening");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("serve");
}
