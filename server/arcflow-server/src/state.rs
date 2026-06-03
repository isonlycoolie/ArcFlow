//! Shared application state.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use arcflow_core::tracing::PostgresTracePersistence;
use sqlx::PgPool;

use crate::middleware::principal::{load_static_keys_from_env, StaticKeyMap};
use crate::store::runs::RunStore;
use crate::store::sites::SiteStore;
use crate::store::workflow_registry::WorkflowRegistryStore;

#[derive(Clone)]
pub struct AppState {
    pub api_key: String,
    pub admin_api_key: Option<String>,
    pub default_upstream_runtime_key: Option<String>,
    pub static_runtime_keys: StaticKeyMap,
    pub webhook_secret: Option<String>,
    /// Shared pool when `ARCFLOW_POSTGRESQL_URL` is set (Phase A).
    pub pg_pool: Option<PgPool>,
    pub runs: Option<Arc<RunStore>>,
    pub registry: Option<Arc<WorkflowRegistryStore>>,
    pub sites: Option<Arc<SiteStore>>,
    pub traces: Option<Arc<PostgresTracePersistence>>,
    pub external_idempotency: Arc<Mutex<HashSet<String>>>,
    #[cfg(feature = "debug-endpoints")]
    pub debug: Option<Arc<crate::debug::DebugRunStore>>,
}

impl AppState {
    pub async fn from_env(api_key: String) -> Self {
        let webhook_secret = std::env::var("ARCFLOW_WEBHOOK_SECRET").ok();
        let max_pg = std::env::var("ARCFLOW_PG_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let (pg_pool, runs, registry, sites, traces) = match std::env::var("ARCFLOW_POSTGRESQL_URL")
        {
            Ok(url) => match sqlx::postgres::PgPoolOptions::new()
                .max_connections(max_pg)
                .connect(&url)
                .await
            {
                Ok(pool) => {
                    arcflow_core::tracing::set_trace_event_persist_hook(Arc::new({
                        let pool = pool.clone();
                        move |run_id: &str, event: &arcflow_core::tracing::TraceEvent| {
                            let pool = pool.clone();
                            let run_id = run_id.to_string();
                            let event = event.clone();
                            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                handle.spawn(async move {
                                    let persistence = PostgresTracePersistence::new(pool);
                                    if let Err(e) =
                                        persistence.persist_events(&run_id, &[event]).await
                                    {
                                        tracing::warn!(error = %e, %run_id, "trace persist failed");
                                    }
                                });
                            }
                        }
                    }));
                    arcflow_core::recovery::install_shared_pool(pool.clone());
                    (
                        Some(pool.clone()),
                        Some(Arc::new(RunStore::new(pool.clone()))),
                        Some(Arc::new(WorkflowRegistryStore::new(pool.clone()))),
                        Some(Arc::new(SiteStore::new(pool.clone()))),
                        Some(Arc::new(PostgresTracePersistence::new(pool))),
                    )
                }
                Err(e) => {
                    tracing::warn!(error = %e, "postgres unavailable; /v1/runs disabled");
                    (None, None, None, None, None)
                }
            },
            Err(_) => (None, None, None, None, None),
        };
        Self {
            api_key,
            admin_api_key: std::env::var("ARCFLOW_ADMIN_API_KEY").ok(),
            default_upstream_runtime_key: std::env::var("ARCFLOW_DEFAULT_UPSTREAM_RUNTIME_KEY")
                .ok(),
            static_runtime_keys: load_static_keys_from_env(),
            webhook_secret,
            pg_pool,
            runs,
            registry,
            sites,
            traces,
            external_idempotency: Arc::new(Mutex::new(HashSet::new())),
            #[cfg(feature = "debug-endpoints")]
            debug: std::env::var("ARCFLOW_DEBUG")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false)
                .then(|| Arc::new(crate::debug::DebugRunStore::default())),
        }
    }
}
