//! Inline workflow policy enforcement for relay proxy.

use axum::http::StatusCode;
use serde_json::Value;

use crate::middleware::site_auth::SiteContext;

pub fn enforce_policy(ctx: &SiteContext, payload: &Value) -> Result<(), (StatusCode, String)> {
    let has_inline = payload.get("workflow").is_some() || payload.get("agents").is_some();
    if has_inline && !ctx.site.allow_inline {
        return Err((
            StatusCode::FORBIDDEN,
            "[ArcFlow Relay] Inline workflows are disabled for this site.".into(),
        ));
    }
    if let Some(wf_ref) = payload.get("workflow_ref") {
        let name = wf_ref.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(expected) = &ctx.site.default_workflow_name {
            if !expected.is_empty() && name != expected {
                return Err((
                    StatusCode::FORBIDDEN,
                    format!("[ArcFlow Relay] Workflow '{name}' is not allowed for this site."),
                ));
            }
        }
    }
    Ok(())
}
