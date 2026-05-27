//! In-process shared memory for a workflow run.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use super::error::MemoryError;
use super::namespace::session_key;

type Store = Arc<Mutex<HashMap<String, Vec<u8>>>>;

/// Shared key-value store visible across agents in one run when scope allows.
#[derive(Debug, Clone, Default)]
pub struct SharedMemory {
    store: Store,
}

impl SharedMemory {
    /// Writes a shared value (any agent may write).
    pub fn write(
        &self,
        run_id: Uuid,
        writer_agent_id: Uuid,
        logical_key: &str,
        value: &[u8],
    ) -> Result<(), MemoryError> {
        let key = session_key(run_id, writer_agent_id, logical_key);
        self.store
            .lock()
            .map_err(|_| MemoryError::OperationFailed {
                reason: "shared lock poisoned".into(),
            })?
            .insert(key, value.to_vec());
        Ok(())
    }

    /// Reads a shared value written by `owner_agent_id`.
    pub fn read(
        &self,
        run_id: Uuid,
        owner_agent_id: Uuid,
        logical_key: &str,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        let key = session_key(run_id, owner_agent_id, logical_key);
        let guard = self.store.lock().map_err(|_| MemoryError::OperationFailed {
            reason: "shared lock poisoned".into(),
        })?;
        Ok(guard.get(&key).cloned())
    }
}
