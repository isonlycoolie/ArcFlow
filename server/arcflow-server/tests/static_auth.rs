//! Static auth: bearer header, runtime keys, publish denial.

use std::sync::Arc;

use arcflow_server::{build_app, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

async fn test_state(master: &str, static_keys: &str) -> Arc<AppState> {
    std::env::set_var("ARCFLOW_STATIC_RUNTIME_KEYS", static_keys);
    Arc::new(AppState::from_env(master.to_string()).await)
}

#[tokio::test]
async fn invalid_key_rejected_on_protected_route() {
    let app = build_app(test_state("master-key", "{}").await);
    let req = Request::builder()
        .method("POST")
        .uri("/v1/runs")
        .header("X-ArcFlow-Api-Key", "wrong")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn bearer_auth_reaches_handler() {
    let app = build_app(test_state("master-key", "{}").await);
    let req = Request::builder()
        .method("POST")
        .uri("/v1/runs")
        .header("Authorization", "Bearer master-key")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn runtime_key_denies_publish() {
    let keys = r#"{"runtime-only":{"publish":false,"workflows":["chat"]}}"#;
    let app = build_app(test_state("master-key", keys).await);
    let payload = serde_json::json!({
        "workflow": {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "chat",
            "steps": [],
            "execution_mode": "linear"
        },
        "agents": []
    });
    let req = Request::builder()
        .method("PUT")
        .uri("/v1/workflows/chat/versions/1.0.0")
        .header("X-ArcFlow-Api-Key", "runtime-only")
        .header("Content-Type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
