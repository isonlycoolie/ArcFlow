//! ArcFlow Relay HTTP router.

mod handlers;
mod middleware;
mod state;

use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

pub use state::RelayState;

pub fn build_app(state: Arc<RelayState>) -> Router {
    let public = Router::new().route("/health", get(handlers::health::health));

    let site_routes = Router::new()
        .route("/v1/sites/:site_id/runs", post(handlers::proxy::create_run))
        .route(
            "/v1/sites/:site_id/runs/:run_id",
            get(handlers::proxy::get_run),
        )
        .route(
            "/v1/sites/:site_id/runs/:run_id/trace",
            get(handlers::proxy::get_run_trace),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::site_auth::require_site_auth,
        ))
        .layer(RequestBodyLimitLayer::new(512 * 1024));

    public
        .merge(site_routes)
        .with_state(state)
        .layer(cors_layer())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_origin(AllowOrigin::any())
}
