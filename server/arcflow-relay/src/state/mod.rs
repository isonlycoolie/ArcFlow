//! Shared relay application state.

mod sites_json;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use arcflow_server::store::sites::{SiteRecord, SiteStore};
use sqlx::PgPool;
use subtle::ConstantTimeEq;

pub use sites_json::load_sites_json;

#[derive(Clone)]
pub struct RelayState {
    pub upstream_url: String,
    pub sites_db: Option<Arc<SiteStore>>,
    pub sites_json: HashMap<String, SiteRecord>,
    pub site_token_hashes: HashMap<String, String>,
    rate_buckets: Arc<Mutex<HashMap<String, RateBucket>>>,
    pub http: reqwest::Client,
}

#[derive(Debug, Clone)]
struct RateBucket {
    window_start: Instant,
    count: u32,
}

impl RelayState {
    pub async fn from_env() -> Self {
        let upstream_url = std::env::var("ARCFLOW_UPSTREAM_URL")
            .unwrap_or_else(|_| "http://localhost:8080".into())
            .trim_end_matches('/')
            .to_string();

        let (sites_json, site_token_hashes) = load_sites_json();
        let sites_db = match std::env::var("ARCFLOW_POSTGRESQL_URL") {
            Ok(url) => PgPool::connect(&url)
                .await
                .ok()
                .map(|pool| Arc::new(SiteStore::new(pool))),
            Err(_) => None,
        };

        Self {
            upstream_url,
            sites_db,
            sites_json,
            site_token_hashes,
            rate_buckets: Arc::new(Mutex::new(HashMap::new())),
            http: reqwest::Client::new(),
        }
    }

    pub async fn resolve_site(&self, site_id: &str, token: &str) -> Option<SiteRecord> {
        if let Some(store) = &self.sites_db {
            if let Ok(Some(site)) = store.verify_token(site_id, token).await {
                return Some(site);
            }
        }
        let site = self.sites_json.get(site_id)?.clone();
        let stored = self.site_token_hashes.get(site_id)?;
        let computed = SiteStore::hash_token(token);
        if computed.as_bytes().ct_eq(stored.as_bytes()).into() {
            return Some(site);
        }
        None
    }

    pub fn check_rate_limit(&self, site: &SiteRecord) -> bool {
        let rpm = site.rate_limit_rpm.max(1) as u32;
        let mut buckets = self.rate_buckets.lock().unwrap();
        let bucket = buckets
            .entry(site.id.clone())
            .or_insert_with(|| RateBucket {
                window_start: Instant::now(),
                count: 0,
            });
        if bucket.window_start.elapsed().as_secs() >= 60 {
            bucket.window_start = Instant::now();
            bucket.count = 0;
        }
        if bucket.count >= rpm {
            return false;
        }
        bucket.count += 1;
        true
    }

    /// Test-only constructor for integration tests.
    pub fn for_test(
        sites_json: HashMap<String, SiteRecord>,
        site_token_hashes: HashMap<String, String>,
    ) -> Self {
        Self {
            upstream_url: "http://127.0.0.1:1".into(),
            sites_db: None,
            sites_json,
            site_token_hashes,
            rate_buckets: Arc::new(Mutex::new(HashMap::new())),
            http: reqwest::Client::new(),
        }
    }
}
