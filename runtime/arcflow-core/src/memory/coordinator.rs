//! Coordinates session, shared, persistent, and vector memory for a run.

use std::cell::RefCell;
use std::sync::OnceLock;

use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::rcs::types::{MemoryConfig, MemoryScope, MemoryType};
use crate::tracing::TraceEmitter;

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
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        self.session
            .write(self.run_id, agent_id, logical_key, value)?;
        trace.memory_write(step_id, "session", value.len());
        Ok(())
    }

    /// Reads session memory (same agent only).
    pub fn read_session(
        &self,
        agent_id: Uuid,
        logical_key: &str,
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        let out = self
            .session
            .read(self.run_id, agent_id, agent_id, logical_key)?;
        trace.memory_read(step_id, "session", logical_key.len());
        Ok(out)
    }

    /// Writes shared memory when config allows workflow scope.
    pub fn write_shared(
        &self,
        writer_agent_id: Uuid,
        logical_key: &str,
        value: &[u8],
        config: &MemoryConfig,
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        if config.memory_type != MemoryType::Shared {
            return Err(MemoryError::OperationFailed {
                reason: "not shared memory config".into(),
            });
        }
        self.shared
            .write(self.run_id, writer_agent_id, logical_key, value)?;
        trace.memory_write(step_id, "shared", value.len());
        Ok(())
    }

    /// Reads shared memory from another agent when scope is workflow-wide.
    pub fn read_shared(
        &self,
        reader_config: &MemoryConfig,
        owner_agent_id: Uuid,
        logical_key: &str,
        trace: &mut TraceEmitter,
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
        let out = self.shared.read(self.run_id, owner_agent_id, logical_key)?;
        trace.memory_read(step_id, "shared", logical_key.len());
        Ok(out)
    }

    /// Writes persistent memory (requires namespace on config).
    pub fn write_persistent(
        &self,
        namespace: &str,
        logical_key: &str,
        value: &[u8],
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        self.runtime()?
            .block_on(
                self.persistent
                    .borrow_mut()
                    .write(namespace, logical_key, value),
            )?;
        trace.memory_write(step_id, "persistent", value.len());
        Ok(())
    }

    /// Reads persistent memory.
    pub fn read_persistent(
        &self,
        namespace: &str,
        logical_key: &str,
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let out = self
            .runtime()?
            .block_on(self.persistent.borrow_mut().read(namespace, logical_key))?;
        trace.memory_read(step_id, "persistent", logical_key.len());
        Ok(out)
    }

    /// Writes vector memory.
    pub fn write_vector(
        &self,
        namespace: &str,
        logical_key: &str,
        value: &[u8],
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<(), MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        self.runtime()?.block_on(
            self.vector
                .borrow_mut()
                .write(namespace, logical_key, value),
        )?;
        trace.memory_write(step_id, "vector", value.len());
        Ok(())
    }

    /// Reads vector memory.
    pub fn read_vector(
        &self,
        namespace: &str,
        logical_key: &str,
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        if namespace.is_empty() {
            return Err(MemoryError::NamespaceRequired);
        }
        let out = self
            .runtime()?
            .block_on(self.vector.borrow_mut().read(namespace, logical_key))?;
        trace.memory_read(step_id, "vector", logical_key.len());
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::SessionMemory;
    use crate::rcs::types::{MemoryConfig, MemoryScope, MemoryType};
    use crate::tracing::TraceEmitter;

    #[test]
    fn session_isolation_blocks_cross_agent_read() {
        let run = Uuid::new_v4();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let coord = MemoryCoordinator::new(run);
        let mut trace = TraceEmitter::new(Uuid::new_v4());
        coord.write_session(a, "k", b"1", &mut trace, None).unwrap();
        let err = SessionMemory::default().read(run, b, a, "k").unwrap_err();
        assert_eq!(err, MemoryError::SessionIsolationViolation);
    }

    #[test]
    fn shared_requires_workflow_scope() {
        let run = Uuid::new_v4();
        let a = Uuid::new_v4();
        let coord = MemoryCoordinator::new(run);
        let mut trace = TraceEmitter::new(Uuid::new_v4());
        let cfg = MemoryConfig {
            memory_type: MemoryType::Shared,
            scope: MemoryScope::Agent,
            ttl_seconds: None,
        };
        coord
            .write_shared(a, "k", b"v", &cfg, &mut trace, None)
            .unwrap();
        let err = coord
            .read_shared(&cfg, a, "k", &mut trace, None)
            .unwrap_err();
        assert_eq!(err, MemoryError::ScopeDenied);
    }
}
