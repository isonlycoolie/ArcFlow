//! ArcFlow HTTP server library (Phase 2-Pro v2 — testable router).

#![allow(clippy::too_many_arguments)]

mod dto;
mod exec_config;
pub mod handlers;
mod middleware;
mod registry;
pub mod state;
pub mod store;

#[cfg(feature = "debug-endpoints")]
pub mod debug;

use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

pub use state::AppState;

fn cors_layer() -> CorsLayer {
    let origins = std::env::var("ARCFLOW_CORS_ORIGINS").unwrap_or_default();
    let layer = CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);
    if origins.trim().is_empty() {
        return layer;
    }
    let list: Vec<_> = origins
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();
    if list.is_empty() {
        return layer;
    }
    layer.allow_origin(AllowOrigin::list(list))
}

/// Builds the full Axum router for integration tests and `main`.
pub fn build_app(state: Arc<AppState>) -> Router {
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
        .route(
            "/v1/runs/:run_id/external/:binding_id",
            post(handlers::external::external_callback),
        )
        .route(
            "/v1/workflows/:name/versions/:version",
            get(handlers::registry::get_workflow_version)
                .put(handlers::registry::publish_workflow),
        )
        .route(
            "/v1/workflows/:name/resolve",
            get(handlers::registry::resolve_workflow),
        )
        .route(
            "/v1/workflows/:name/aliases/:alias",
            post(handlers::registry::set_workflow_alias),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth::require_api_key,
        ))
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    let admin = Router::new()
        .route("/v1/admin/sites", post(handlers::admin::sites::create_site))
        .route(
            "/v1/admin/sites/:site_id",
            get(handlers::admin::sites::get_site).patch(handlers::admin::sites::patch_site),
        )
        .route(
            "/v1/admin/sites/:site_id/tokens/rotate",
            post(handlers::admin::sites::rotate_token),
        )
        .route(
            "/v1/admin/sites/:site_id/knowledge/ingest",
            post(handlers::admin::knowledge::ingest_knowledge),
        )
        .route(
            "/v1/admin/sites/:site_id/workflows/chat/publish",
            post(handlers::admin::workflows::publish_chat),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::admin_auth::require_admin_key,
        ))
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    public.merge(protected).merge(admin).with_state(state).layer(cors_layer())
}
