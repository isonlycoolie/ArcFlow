//! In-memory debug run sessions (Phase 2.4).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use arcflow_core::debug::DebugSession;

#[derive(Default)]
pub struct DebugRunStore {
    sessions: Mutex<HashMap<String, Arc<DebugSession>>>,
}

impl DebugRunStore {
    pub fn insert(&self, run_id: String, session: Arc<DebugSession>) {
        self.sessions
            .lock()
            .expect("debug store lock")
            .insert(run_id, session);
    }

    pub fn get(&self, run_id: &str) -> Option<Arc<DebugSession>> {
        self.sessions
            .lock()
            .expect("debug store lock")
            .get(run_id)
            .cloned()
    }

    pub fn remove(&self, run_id: &str) {
        self.sessions
            .lock()
            .expect("debug store lock")
            .remove(run_id);
    }
}
