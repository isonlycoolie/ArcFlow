use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn ready() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ready", "version": env!("CARGO_PKG_VERSION") })))
}
