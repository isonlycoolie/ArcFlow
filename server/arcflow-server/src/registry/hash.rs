//! Canonical JSON hashing for published workflow bundles.
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Returns a stable SHA-256 digest over canonically sorted JSON.
pub fn schema_hash(value: &Value) -> String {
    let canonical = canonicalize(value);
    let bytes = serde_json::to_vec(&canonical).expect("canonical json serializes");
    let digest = Sha256::digest(bytes);
    format!("sha256:{:x}", digest)
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                if let Some(v) = map.get(&key) {
                    sorted.insert(key, canonicalize(v));
                }
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hash_is_order_independent() {
        let a = json!({"agents": [], "workflow": {"name": "demo"}});
        let b = json!({"workflow": {"name": "demo"}, "agents": []});
        assert_eq!(schema_hash(&a), schema_hash(&b));
    }
}
