//! Admin site API integration tests.

use std::sync::Arc;

use arcflow_server::{build_app, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

async fn apply_migrations(pool: &PgPool) {
    arcflow_core::migrate::run(pool).await.expect("migrate");
}

async fn test_state() -> Option<Arc<AppState>> {
    let db_url = std::env::var("ARCFLOW_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("ARCFLOW_POSTGRESQL_URL"))
        .unwrap_or_else(|_| "postgres://arcflow:arcflow@127.0.0.1:5432/arcflow".into());
    let pool = PgPool::connect(&db_url).await.ok()?;
    apply_migrations(&pool).await;
    std::env::set_var("ARCFLOW_POSTGRESQL_URL", &db_url);
    std::env::set_var("ARCFLOW_ADMIN_API_KEY", "admin-test-key");
    std::env::set_var("ARCFLOW_DEFAULT_UPSTREAM_RUNTIME_KEY", "runtime-key");
    Some(Arc::new(AppState::from_env("master-key".into()).await))
}

#[tokio::test]
async fn admin_create_and_get_site() {
    let Some(state) = test_state().await else {
        eprintln!("skip admin_create_and_get_site: postgres unavailable");
        return;
    };
    let app = build_app(state);
    let create = serde_json::json!({
        "display_name": "Test Site",
        "allowed_origins": ["http://localhost:5173"]
    });
    let req = Request::builder()
        .method("POST")
        .uri("/v1/admin/sites")
        .header("X-ArcFlow-Admin-Key", "admin-test-key")
        .header("Content-Type", "application/json")
        .body(Body::from(create.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let site_id = created["site_id"].as_str().unwrap();

    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/admin/sites/{site_id}"))
        .header("X-ArcFlow-Admin-Key", "admin-test-key")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_rejects_missing_key() {
    std::env::set_var("ARCFLOW_ADMIN_API_KEY", "admin-test-key");
    let app = build_app(Arc::new(AppState::from_env("master-key".into()).await));
    let req = Request::builder()
        .method("POST")
        .uri("/v1/admin/sites")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
