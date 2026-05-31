//! Dev-only site config from `ARCFLOW_RELAY_SITES_JSON`.

use std::collections::HashMap;

use arcflow_server::store::sites::{SiteRecord, SiteStore};

pub fn load_sites_json() -> (HashMap<String, SiteRecord>, HashMap<String, String>) {
    let Ok(raw) = std::env::var("ARCFLOW_RELAY_SITES_JSON") else {
        return (HashMap::new(), HashMap::new());
    };
    #[derive(serde::Deserialize)]
    struct DevSite {
        id: String,
        display_name: String,
        allowed_origins: Vec<String>,
        rate_limit_rpm: i32,
        allow_inline: bool,
        default_workflow_name: Option<String>,
        kb_namespace: String,
        upstream_runtime_key: String,
        token: String,
    }
    let parsed: Vec<DevSite> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "invalid ARCFLOW_RELAY_SITES_JSON");
            return (HashMap::new(), HashMap::new());
        }
    };
    let mut sites = HashMap::new();
    let mut hashes = HashMap::new();
    for s in parsed {
        let record = SiteRecord {
            id: s.id.clone(),
            display_name: s.display_name,
            allowed_origins: s.allowed_origins,
            rate_limit_rpm: s.rate_limit_rpm,
            allow_inline: s.allow_inline,
            default_workflow_name: s.default_workflow_name,
            kb_namespace: s.kb_namespace,
            upstream_runtime_key: s.upstream_runtime_key,
            chat_instructions: None,
            created_at: chrono::Utc::now(),
        };
        hashes.insert(s.id.clone(), SiteStore::hash_token(&s.token));
        sites.insert(s.id, record);
    }
    (sites, hashes)
}
