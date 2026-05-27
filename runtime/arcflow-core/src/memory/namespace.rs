//! Key composition for memory backends.

use uuid::Uuid;

/// Session or shared key: `{run_id}:{agent_id}:{logical_key}`.
pub fn session_key(run_id: Uuid, agent_id: Uuid, logical_key: &str) -> String {
    format!("{run_id}:{agent_id}:{logical_key}")
}

/// Persistent or vector key: `{namespace}:{logical_key}`.
pub fn durable_key(namespace: &str, logical_key: &str) -> String {
    format!("{namespace}:{logical_key}")
}
