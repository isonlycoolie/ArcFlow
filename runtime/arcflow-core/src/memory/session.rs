//! In-process session memory (per workflow run, per agent).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use super::error::MemoryError;
use super::namespace::session_key;

type Store = Arc<Mutex<HashMap<String, Vec<u8>>>>;

/// Ephemeral session store for one workflow run.
#[derive(Debug, Clone, Default)]
pub struct SessionMemory {
    store: Store,
}

impl SessionMemory {
    /// Writes bytes for an agent-scoped key.
    pub fn write(
        &self,
        run_id: Uuid,
        agent_id: Uuid,
        logical_key: &str,
        value: &[u8],
    ) -> Result<(), MemoryError> {
        let key = session_key(run_id, agent_id, logical_key);
        self.store
            .lock()
            .map_err(|_| MemoryError::OperationFailed {
                reason: "session lock poisoned".into(),
            })?
            .insert(key, value.to_vec());
        Ok(())
    }

    /// Reads bytes; enforces that `reader_agent_id` matches the key's agent segment.
    pub fn read(
        &self,
        run_id: Uuid,
        reader_agent_id: Uuid,
        owner_agent_id: Uuid,
        logical_key: &str,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        if reader_agent_id != owner_agent_id {
            return Err(MemoryError::SessionIsolationViolation);
        }
        let key = session_key(run_id, owner_agent_id, logical_key);
        let guard = self
            .store
            .lock()
            .map_err(|_| MemoryError::OperationFailed {
                reason: "session lock poisoned".into(),
            })?;
        Ok(guard.get(&key).cloned())
    }
}
