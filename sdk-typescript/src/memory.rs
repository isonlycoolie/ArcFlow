//! N-API vector memory bindings (Phase 2-Pro).

use std::sync::Mutex;

use arcflow_core::memory::MemoryCoordinator;
use arcflow_core::rcs::types::{
    MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, RetrievalModeSpec,
};
use arcflow_core::tracing::{
    emitter::TraceEmitter, sprint5_emitter::TraceEventEmitter, store::TraceStore,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use uuid::Uuid;

fn vector_config(namespace: &str) -> MemoryConfig {
    MemoryConfig {
        memory_type: MemoryType::Vector,
        scope: MemoryScope::Agent,
        namespace: Some(namespace.to_string()),
        ttl_seconds: None,
        embedding: Some("stub/384".into()),
        retrieval: Some(MemoryRetrievalConfig {
            mode: RetrievalModeSpec::Dense,
            dense_weight: 1.0,
            sparse_weight: 0.0,
            rerank: None,
            top_k: Some(5),
        }),
        chunking: None,
    }
}

#[napi]
pub struct JsVectorStore {
    coord: Mutex<MemoryCoordinator>,
    run_id: Uuid,
}

#[napi]
impl JsVectorStore {
    #[napi(constructor)]
    pub fn new() -> Self {
        let run_id = Uuid::new_v4();
        Self {
            coord: Mutex::new(MemoryCoordinator::new(run_id)),
            run_id,
        }
    }

    #[napi]
    pub fn ingest(&self, namespace: String, key: String, text: String) -> Result<u32> {
        let config = vector_config(&namespace);
        let mut store = TraceStore::new();
        let run_key = self.run_id.to_string();
        let mut legacy = TraceEmitter::new(self.run_id);
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
        let coord = self
            .coord
            .lock()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let count = coord
            .write_vector_document(
                &config,
                &key,
                &text,
                "sdk",
                &mut legacy,
                &mut sprint5,
                &run_key,
                None,
            )
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(count as u32)
    }

    #[napi]
    pub fn search(&self, namespace: String, query: String, top_k: u32) -> Result<Vec<String>> {
        let config = vector_config(&namespace);
        let mut store = TraceStore::new();
        let run_key = self.run_id.to_string();
        let mut legacy = TraceEmitter::new(self.run_id);
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
        let coord = self
            .coord
            .lock()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let hits = coord
            .search_vector(
                &config,
                &query,
                top_k as usize,
                "sdk",
                &mut legacy,
                &mut sprint5,
                &run_key,
                None,
            )
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(hits
            .into_iter()
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .collect())
    }
}
