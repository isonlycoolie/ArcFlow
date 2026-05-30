//! Shared application state.

use std::sync::Arc;

use arcflow_core::tracing::PostgresTracePersistence;
use sqlx::PgPool;

use crate::store::runs::RunStore;
use crate::store::workflow_registry::WorkflowRegistryStore;

#[derive(Clone)]
pub struct AppState {
    pub api_key: String,
    pub runs: Option<Arc<RunStore>>,
    pub registry: Option<Arc<WorkflowRegistryStore>>,
    pub traces: Option<Arc<PostgresTracePersistence>>,
}

impl AppState {
    pub async fn from_env(api_key: String) -> Self {
        let (runs, registry, traces) = match std::env::var("ARCFLOW_POSTGRESQL_URL") {
            Ok(url) => match PgPool::connect(&url).await {
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
                                    if let Err(e) = persistence.persist_events(&run_id, &[event]).await
                                    {
                                        tracing::warn!(error = %e, %run_id, "trace persist failed");
                                    }
                                });
                            }
                        }
                    }));
                    (
                        Some(Arc::new(RunStore::new(pool.clone()))),
                        Some(Arc::new(WorkflowRegistryStore::new(pool.clone()))),
                        Some(Arc::new(PostgresTracePersistence::new(pool))),
                    )
                }
                Err(e) => {
                    tracing::warn!(error = %e, "postgres unavailable; /v1/runs disabled");
                    (None, None, None)
                }
            },
            Err(_) => (None, None, None),
        };
        Self {
            api_key,
            runs,
            registry,
            traces,
        }
    }
}
