//! Coordinates session, shared, persistent, and vector memory for a run.

#![allow(clippy::too_many_arguments)] // dual trace emission shares legacy, sprint5, and run_id

use std::cell::RefCell;
use std::sync::OnceLock;
use std::time::Instant;

use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::rcs::types::{MemoryConfig, MemoryScope, MemoryType};
use crate::tracing::{
    emitter::TraceEmitter, memory_read, memory_write, sprint5_emitter::TraceEventEmitter,
};

use super::error::MemoryError;
use super::persistent::PersistentMemory;
use super::session::SessionMemory;
use super::shared::SharedMemory;
use super::vector::VectorMemory;

/// In-process and lazy remote memory for one workflow execution.
pub struct MemoryCoordinator {
    run_id: Uuid,
    session: SessionMemory,
    shared: SharedMemory,
    persistent: RefCell<PersistentMemory>,
    vector: RefCell<VectorMemory>,
    rt: OnceLock<Runtime>,
}

impl MemoryCoordinator {
    /// Binds coordinator to a workflow run id.
    pub fn new(run_id: Uuid) -> Self {
        Self {
            run_id,
            session: SessionMemory::default(),
            shared: SharedMemory::default(),
            persistent: RefCell::new(PersistentMemory::new()),
            vector: RefCell::new(VectorMemory::new()),
            rt: OnceLock::new(),
        }
    }

    fn runtime(&self) -> Result<&Runtime, MemoryError> {
        if self.rt.get().is_none() {
            let rt = Runtime::new().map_err(|e| MemoryError::OperationFailed {
                reason: format!("tokio runtime for memory: {e}"),
            })?;
            let _ = self.rt.set(rt);
        }
        self.rt.get().ok_or(MemoryError::OperationFailed {
            reason: "tokio runtime unavailable".into(),
        })
    }

    /// Writes session memory for `agent_id`.
    pub fn write_session(
        &self,
        agent_id: Uuid,
        logical_key: &str,
        value: &[u8],
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        #[cfg(feature = "otel")]
        let _mem = crate::tracing::otel_live::memory_span(
            run_id,
            step_id.map(|s| s.to_string()).as_deref(),
            "session",
            "write",
        );
        let started = Instant::now();
        self.session
            .write(self.run_id, agent_id, logical_key, value)?;
        memory_write(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "session",
            logical_key,
            started,
        );
        Ok(())
    }

    /// Reads session memory (same agent only).
    pub fn read_session(
        &self,
        agent_id: Uuid,
        logical_key: &str,
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        #[cfg(feature = "otel")]
        let _mem = crate::tracing::otel_live::memory_span(
            run_id,
            step_id.map(|s| s.to_string()).as_deref(),
            "session",
            "read",
        );
        let started = Instant::now();
        let out = self
            .session
            .read(self.run_id, agent_id, agent_id, logical_key)?;
        memory_read(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "session",
            logical_key,
            out.is_some(),
            started,
        );
        Ok(out)
    }

    /// Writes shared memory when config allows workflow scope.
    pub fn write_shared(
        &self,
        writer_agent_id: Uuid,
        logical_key: &str,
        value: &[u8],
        config: &MemoryConfig,
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        if config.memory_type != MemoryType::Shared {
            return Err(MemoryError::OperationFailed {
                reason: "not shared memory config".into(),
            });
        }
        let started = Instant::now();
        self.shared
            .write(self.run_id, writer_agent_id, logical_key, value)?;
        memory_write(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "shared",
            logical_key,
            started,
        );
        Ok(())
    }

    /// Reads shared memory from another agent when scope is workflow-wide.
    pub fn read_shared(
        &self,
        reader_config: &MemoryConfig,
        owner_agent_id: Uuid,
        logical_key: &str,
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        if reader_config.memory_type != MemoryType::Shared {
            return Err(MemoryError::OperationFailed {
                reason: "not shared memory config".into(),
            });
        }
        if reader_config.scope != MemoryScope::Workflow {
            return Err(MemoryError::ScopeDenied);
        }
        let started = Instant::now();
        let out = self.shared.read(self.run_id, owner_agent_id, logical_key)?;
        memory_read(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "shared",
            logical_key,
            out.is_some(),
            started,
        );
        Ok(out)
    }

    /// Writes persistent memory (requires namespace on config).
    pub fn write_persistent(
        &self,
        namespace: &str,
        logical_key: &str,
        value: &[u8],
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let started = Instant::now();
        self.runtime()?
            .block_on(
                self.persistent
                    .borrow_mut()
                    .write(namespace, logical_key, value),
            )?;
        memory_write(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "persistent",
            logical_key,
            started,
        );
        Ok(())
    }

    /// Reads persistent memory.
    pub fn read_persistent(
        &self,
        namespace: &str,
        logical_key: &str,
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let started = Instant::now();
        let out = self
            .runtime()?
            .block_on(self.persistent.borrow_mut().read(namespace, logical_key))?;
        memory_read(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "persistent",
            logical_key,
            out.is_some(),
            started,
        );
        Ok(out)
    }

    /// Writes vector memory.
    pub fn write_vector(
        &self,
        namespace: &str,
        logical_key: &str,
        value: &[u8],
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let started = Instant::now();
        self.runtime()?.block_on(
            self.vector
                .borrow_mut()
                .write(namespace, logical_key, value),
        )?;
        memory_write(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "vector",
            logical_key,
            started,
        );
        Ok(())
    }

    /// Reads vector memory.
    pub fn read_vector(
        &self,
        namespace: &str,
        logical_key: &str,
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let started = Instant::now();
        let out = self
            .runtime()?
            .block_on(self.vector.borrow_mut().read(namespace, logical_key))?;
        memory_read(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "vector",
            logical_key,
            out.is_some(),
            started,
        );
        Ok(out)
    }

    /// Ingests a document into vector memory using agent chunking config (Phase 2.5).
    pub fn write_vector_document(
        &self,
        config: &MemoryConfig,
        logical_key: &str,
        text: &str,
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<usize, MemoryError> {
        let namespace = config
            .namespace
            .as_deref()
            .ok_or(MemoryError::NamespaceRequired)?;
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let started = Instant::now();
        let mut vector = self.vector.borrow_mut();
        if config.retrieval.is_some() || config.chunking.is_some() || config.embedding.is_some() {
            let rebuilt = super::vector::VectorMemory::from_memory_config(config).map_err(|e| {
                MemoryError::OperationFailed {
                    reason: e.to_string(),
                }
            })?;
            *vector = rebuilt;
        }
        let count =
            self.runtime()?
                .block_on(vector.write_document(namespace, logical_key, text))?;
        memory_write(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "vector",
            logical_key,
            started,
        );
        Ok(count)
    }

    /// Hybrid search with optional rerank from agent memory config (Phase 2.5).
    pub fn search_vector(
        &self,
        config: &MemoryConfig,
        query: &str,
        top_k: usize,
        agent_name: &str,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<Vec<Vec<u8>>, MemoryError> {
        let namespace = config
            .namespace
            .as_deref()
            .ok_or(MemoryError::NamespaceRequired)?;
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let started = Instant::now();
        let mut vector = self.vector.borrow_mut();
        if config.retrieval.is_some() || config.chunking.is_some() || config.embedding.is_some() {
            let rebuilt = super::vector::VectorMemory::from_memory_config(config).map_err(|e| {
                MemoryError::OperationFailed {
                    reason: e.to_string(),
                }
            })?;
            *vector = rebuilt;
        }
        let hits = self
            .runtime()?
            .block_on(vector.search_reranked(namespace, query, top_k))?;
        memory_read(
            legacy,
            sprint5,
            run_id,
            step_id,
            agent_name,
            "vector",
            query,
            !hits.is_empty(),
            started,
        );
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::{sprint5_emitter::TraceEventEmitter, store::TraceStore, TraceEmitter};

    #[test]
    fn session_write_read_round_trip() {
        let run_id = Uuid::new_v4();
        let coord = MemoryCoordinator::new(run_id);
        let mut store = TraceStore::new();
        let run_key = run_id.to_string();
        let mut legacy = TraceEmitter::new(run_id);
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
        let aid = Uuid::new_v4();
        coord
            .write_session(
                aid,
                "k",
                b"v",
                "agent",
                &mut legacy,
                &mut sprint5,
                &run_key,
                None,
            )
            .unwrap();
        let got = coord
            .read_session(aid, "k", "agent", &mut legacy, &mut sprint5, &run_key, None)
            .unwrap();
        assert_eq!(got.as_deref(), Some(b"v".as_slice()));
    }

    #[test]
    fn shared_scope_denied_when_not_workflow() {
        let run_id = Uuid::new_v4();
        let coord = MemoryCoordinator::new(run_id);
        let mut store = TraceStore::new();
        let run_key = run_id.to_string();
        let mut legacy = TraceEmitter::new(run_id);
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
        let cfg = MemoryConfig {
            memory_type: MemoryType::Shared,
            scope: MemoryScope::Agent,
            namespace: None,
            ttl_seconds: None,
            embedding: None,
            retrieval: None,
            chunking: None,
        };
        let err = coord
            .read_shared(
                &cfg,
                Uuid::new_v4(),
                "k",
                "agent",
                &mut legacy,
                &mut sprint5,
                &run_key,
                None,
            )
            .unwrap_err();
        assert!(matches!(err, MemoryError::ScopeDenied));
    }
}
