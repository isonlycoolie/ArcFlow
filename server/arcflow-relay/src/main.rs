//! ArcFlow Relay — browser edge proxy for static sites.

use std::net::SocketAddr;
use std::sync::Arc;

use arcflow_relay::{build_app, RelayState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arcflow_relay=info".into()),
        )
        .init();

    let port: u16 = std::env::var("ARCFLOW_RELAY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);

    let state = Arc::new(RelayState::from_env().await);
    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!(%addr, "arcflow-relay listening");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
