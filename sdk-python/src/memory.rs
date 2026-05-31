//! PyO3 vector memory bindings (Phase 2-Pro).

use std::sync::Mutex;

use pyo3::prelude::*;
use pyo3::types::PyList;
use uuid::Uuid;

use arcflow_core::memory::MemoryCoordinator;
use arcflow_core::rcs::types::{
    MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, RetrievalModeSpec,
};
use arcflow_core::tracing::{
    emitter::TraceEmitter, sprint5_emitter::TraceEventEmitter, store::TraceStore,
};

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

/// Native vector store handle (one coordinator per instance).
#[pyclass]
pub struct PyVectorStore {
    coord: Mutex<MemoryCoordinator>,
    run_id: Uuid,
}

#[pymethods]
impl PyVectorStore {
    #[new]
    fn new() -> Self {
        let run_id = Uuid::new_v4();
        Self {
            coord: Mutex::new(MemoryCoordinator::new(run_id)),
            run_id,
        }
    }

    /// Ingest text into a namespace; returns chunk count.
    fn ingest(&self, namespace: &str, key: &str, text: &str) -> PyResult<usize> {
        let config = vector_config(namespace);
        let mut store = TraceStore::new();
        let run_key = self.run_id.to_string();
        let mut legacy = TraceEmitter::new(self.run_id);
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
        let coord = self.coord.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("VectorStore lock poisoned: {e}"))
        })?;
        coord
            .write_vector_document(
                &config,
                key,
                text,
                "sdk",
                &mut legacy,
                &mut sprint5,
                &run_key,
                None,
            )
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// Search a namespace; returns list of chunk text strings.
    fn search(&self, namespace: &str, query: &str, top_k: usize) -> PyResult<Vec<String>> {
        let config = vector_config(namespace);
        let mut store = TraceStore::new();
        let run_key = self.run_id.to_string();
        let mut legacy = TraceEmitter::new(self.run_id);
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
        let coord = self.coord.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("VectorStore lock poisoned: {e}"))
        })?;
        let hits = coord
            .search_vector(
                &config,
                query,
                top_k,
                "sdk",
                &mut legacy,
                &mut sprint5,
                &run_key,
                None,
            )
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(hits
            .into_iter()
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .collect())
    }

    /// Search returning `(text, byte_len)` tuples for SDK parity.
    fn search_hits<'py>(
        &self,
        py: Python<'py>,
        namespace: &str,
        query: &str,
        top_k: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        let hits = self.search(namespace, query, top_k)?;
        let list = PyList::empty_bound(py);
        for text in hits {
            list.append((text.clone(), text.len()))?;
        }
        Ok(list)
    }
}
