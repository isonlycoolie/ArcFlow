//! Shared application state.

use std::sync::Arc;

use sqlx::PgPool;

use crate::store::runs::RunStore;

#[derive(Clone)]
pub struct AppState {
    pub api_key: String,
    pub runs: Option<Arc<RunStore>>,
}

impl AppState {
    pub async fn from_env(api_key: String) -> Self {
        let runs = match std::env::var("ARCFLOW_POSTGRESQL_URL") {
            Ok(url) => match PgPool::connect(&url).await {
                Ok(pool) => Some(Arc::new(RunStore::new(pool))),
                Err(e) => {
                    tracing::warn!(error = %e, "postgres unavailable; /v1/runs disabled");
                    None
                }
            },
            Err(_) => None,
        };
        Self { api_key, runs }
    }
}
