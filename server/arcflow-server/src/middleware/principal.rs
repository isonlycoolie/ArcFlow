//! Authenticated caller identity for static runtime keys.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AuthPrincipal {
    Master,
    Runtime(StaticKeyPolicy),
}

#[derive(Debug, Clone)]
pub struct StaticKeyPolicy {
    pub publish: bool,
    pub workflows: Option<Vec<String>>,
}

pub type StaticKeyMap = HashMap<String, StaticKeyPolicy>;

pub fn load_static_keys_from_env() -> StaticKeyMap {
    let Ok(raw) = std::env::var("ARCFLOW_STATIC_RUNTIME_KEYS") else {
        return StaticKeyMap::new();
    };
    let parsed: HashMap<String, StaticKeyPolicyJson> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "invalid ARCFLOW_STATIC_RUNTIME_KEYS JSON");
            return StaticKeyMap::new();
        }
    };
    parsed
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                StaticKeyPolicy {
                    publish: v.publish.unwrap_or(false),
                    workflows: v.workflows,
                },
            )
        })
        .collect()
}

#[derive(Debug, serde::Deserialize)]
struct StaticKeyPolicyJson {
    publish: Option<bool>,
    workflows: Option<Vec<String>>,
}

pub fn workflow_allowed(policy: &StaticKeyPolicy, workflow_name: &str) -> bool {
    match &policy.workflows {
        None => true,
        Some(list) => list.iter().any(|n| n == workflow_name),
    }
}
