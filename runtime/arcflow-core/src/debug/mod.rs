//! Local-only step-through debugging (Phase 2.4).

use std::collections::HashSet;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::StateSnapshot;

/// Masked workflow state exposed to debug clients (values redacted).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DebugStateView {
    pub run_id: String,
    pub step_id: String,
    pub step_index: usize,
    pub committed_step_ids: Vec<String>,
    pub masked_outputs: Vec<MaskedStepOutput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaskedStepOutput {
    pub step_id: String,
    pub agent_id: String,
    pub content_preview: String,
    pub status: String,
}

fn mask_content(content: &str) -> String {
    let _ = content.len();
    "***".into()
}

fn state_view(
    run_id: &str,
    step_id: Uuid,
    step_index: usize,
    snapshot: &StateSnapshot,
) -> DebugStateView {
    DebugStateView {
        run_id: run_id.to_string(),
        step_id: step_id.to_string(),
        step_index,
        committed_step_ids: snapshot
            .steps
            .iter()
            .map(|s| s.step_id.to_string())
            .collect(),
        masked_outputs: snapshot
            .steps
            .iter()
            .map(|s| MaskedStepOutput {
                step_id: s.step_id.to_string(),
                agent_id: s.agent_id.to_string(),
                content_preview: mask_content(&s.content),
                status: format!("{:?}", s.status),
            })
            .collect(),
    }
}

struct PausedState {
    view: DebugStateView,
}

/// In-process debug session for breakpoint pause/resume.
#[derive(Clone)]
pub struct DebugSession {
    breakpoints: Arc<Mutex<HashSet<String>>>,
    paused: Arc<Mutex<Option<PausedState>>>,
    gate: Arc<(Mutex<bool>, Condvar)>,
}

impl Default for DebugSession {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugSession {
    pub fn new() -> Self {
        Self {
            breakpoints: Arc::new(Mutex::new(HashSet::new())),
            paused: Arc::new(Mutex::new(None)),
            gate: Arc::new((Mutex::new(true), Condvar::new())),
        }
    }

    pub fn set_breakpoints(&self, step_ids: impl IntoIterator<Item = String>) {
        let mut bp = self.breakpoints.lock().expect("breakpoints lock");
        bp.clear();
        bp.extend(step_ids);
    }

    pub fn pause_before_step(
        &self,
        run_id: &str,
        step_id: Uuid,
        step_index: usize,
        snapshot: &StateSnapshot,
    ) {
        let should_pause = self
            .breakpoints
            .lock()
            .expect("breakpoints lock")
            .contains(&step_id.to_string());
        if !should_pause {
            return;
        }
        {
            let mut paused = self.paused.lock().expect("paused lock");
            *paused = Some(PausedState {
                view: state_view(run_id, step_id, step_index, snapshot),
            });
        }
        let (lock, cv) = &*self.gate;
        let mut ready = lock.lock().expect("gate lock");
        *ready = false;
        while !*ready {
            ready = cv
                .wait_timeout(ready, Duration::from_millis(200))
                .expect("gate wait")
                .0;
        }
        let mut paused = self.paused.lock().expect("paused lock");
        *paused = None;
    }

    pub fn continue_run(&self) {
        let (lock, cv) = &*self.gate;
        let mut ready = lock.lock().expect("gate lock");
        *ready = true;
        cv.notify_all();
    }

    pub fn state_view(&self) -> Option<DebugStateView> {
        self.paused
            .lock()
            .expect("paused lock")
            .as_ref()
            .map(|p| p.view.clone())
    }
}

impl std::fmt::Debug for DebugSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugSession").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rcs::types::ExecutionStatus;
    use crate::state::ExecutionStepOutput;
    use std::thread;

    #[test]
    fn breakpoint_pauses_until_continue() {
        let session = DebugSession::new();
        let step_id = Uuid::new_v4();
        session.set_breakpoints([step_id.to_string()]);
        let session_bg = session.clone();
        let handle = thread::spawn(move || {
            session_bg.pause_before_step(
                "run",
                step_id,
                0,
                &StateSnapshot { steps: vec![] },
            );
        });
        thread::sleep(Duration::from_millis(50));
        assert!(session.state_view().is_some());
        session.continue_run();
        handle.join().expect("pause thread");
        assert!(session.state_view().is_none());
    }

    #[test]
    fn masks_long_step_content() {
        let step_id = Uuid::new_v4();
        let snap = StateSnapshot {
            steps: vec![ExecutionStepOutput {
                step_id,
                agent_id: Uuid::new_v4(),
                content: "secret-api-key-value-12345".into(),
                status: ExecutionStatus::Completed,
            }],
        };
        let view = state_view("run", step_id, 1, &snap);
        assert!(view.masked_outputs[0].content_preview.contains("***"));
        assert!(!view.masked_outputs[0].content_preview.contains("secret-api-key"));
    }
}
