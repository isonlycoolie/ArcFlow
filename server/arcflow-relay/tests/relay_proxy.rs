//! Relay policy enforcement tests.

use std::collections::HashMap;
use std::sync::Arc;

use arcflow_relay::{build_app, RelayState};
use arcflow_server::store::sites::{SiteRecord, SiteStore};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use tower::ServiceExt;

fn dev_state() -> Arc<RelayState> {
    let site = SiteRecord {
        id: "s_test".into(),
        display_name: "Test".into(),
        allowed_origins: vec!["http://localhost:5173".into()],
        rate_limit_rpm: 60,
        allow_inline: false,
        default_workflow_name: Some("chat".into()),
        kb_namespace: "site-s_test-kb".into(),
        upstream_runtime_key: "runtime-key".into(),
        chat_instructions: None,
        created_at: Utc::now(),
    };
    let mut sites = HashMap::new();
    sites.insert("s_test".into(), site);
    let mut token_hashes = HashMap::new();
    token_hashes.insert("s_test".into(), SiteStore::hash_token("st_live_test"));
    Arc::new(RelayState::for_test(sites, token_hashes))
}

#[tokio::test]
async fn relay_rejects_missing_token() {
    let app = build_app(dev_state());
    let req = Request::builder()
        .method("POST")
        .uri("/v1/sites/s_test/runs")
        .header("Origin", "http://localhost:5173")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn relay_rejects_bad_origin() {
    let app = build_app(dev_state());
    let req = Request::builder()
        .method("POST")
        .uri("/v1/sites/s_test/runs")
        .header("Authorization", "Bearer st_live_test")
        .header("Origin", "https://evil.example")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
