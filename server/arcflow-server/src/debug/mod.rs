//! Debug HTTP handlers (localhost + ARCFLOW_DEBUG only).

mod store;

pub use store::DebugRunStore;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use arcflow_core::debug::DebugSession;
use arcflow_core::rcs::types::{AgentDefinition, WorkflowDefinition};
use arcflow_core::workflow::{ExecutionConfig, WorkflowEngine};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct DebugStartRequest {
    pub workflow: WorkflowDefinition,
    pub agents: Vec<AgentDefinition>,
    pub input: String,
    #[serde(default)]
    pub breakpoints: Vec<String>,
}

#[derive(Serialize)]
pub struct DebugStartResponse {
    pub run_id: String,
}

#[derive(Serialize)]
pub struct DebugStateResponse {
    pub paused: bool,
    pub state: Option<arcflow_core::debug::DebugStateView>,
}

fn reject_non_loopback(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<(), (StatusCode, String)> {
    if !addr.ip().is_loopback() {
        return Err((
            StatusCode::FORBIDDEN,
            "[ArcFlow] debug endpoints are localhost-only".into(),
        ));
    }
    Ok(())
}

pub async fn start_debug_run(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<DebugStartRequest>,
) -> Result<Json<DebugStartResponse>, (StatusCode, String)> {
    reject_non_loopback(ConnectInfo(addr))?;
    let store = state.debug.as_ref().ok_or((
        StatusCode::NOT_FOUND,
        "[ArcFlow] debug endpoints disabled; set ARCFLOW_DEBUG=true".into(),
    ))?;
    if body.input.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "input must be non-empty".into()));
    }
    let run_id = Uuid::new_v4().to_string();
    let session = Arc::new(DebugSession::new());
    session.set_breakpoints(body.breakpoints);
    store.insert(run_id.clone(), session.clone());
    let agent_map: HashMap<Uuid, AgentDefinition> =
        body.agents.iter().map(|a| (a.id, a.clone())).collect();
    let workflow = body.workflow;
    let input = body.input;
    let run_id_spawn = run_id.clone();
    tokio::spawn(async move {
        let exec_config = ExecutionConfig {
            run_id: Uuid::parse_str(&run_id_spawn).ok(),
            debug: Some(session),
            ..ExecutionConfig::default()
        };
        let _ = WorkflowEngine::new().execute_with_config(
            &workflow,
            &agent_map,
            &input,
            None,
            None,
            None,
            arcflow_core::providers::default_max_tokens(),
            arcflow_core::providers::default_temperature(),
            &exec_config,
            None,
        );
    });
    Ok(Json(DebugStartResponse { run_id }))
}

pub async fn get_debug_state(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(run_id): Path<String>,
) -> Result<Json<DebugStateResponse>, (StatusCode, String)> {
    reject_non_loopback(ConnectInfo(addr))?;
    let store = state.debug.as_ref().ok_or((
        StatusCode::NOT_FOUND,
        "[ArcFlow] debug endpoints disabled".into(),
    ))?;
    let session = store.get(&run_id).ok_or((
        StatusCode::NOT_FOUND,
        format!("debug run '{run_id}' not found"),
    ))?;
    let view = session.state_view();
    Ok(Json(DebugStateResponse {
        paused: view.is_some(),
        state: view,
    }))
}

pub async fn continue_debug_run(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(run_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    reject_non_loopback(ConnectInfo(addr))?;
    let store = state.debug.as_ref().ok_or((
        StatusCode::NOT_FOUND,
        "[ArcFlow] debug endpoints disabled".into(),
    ))?;
    let session = store.get(&run_id).ok_or((
        StatusCode::NOT_FOUND,
        format!("debug run '{run_id}' not found"),
    ))?;
    session.continue_run();
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_check_accepts_localhost() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("addr");
        assert!(reject_non_loopback(ConnectInfo(addr)).is_ok());
    }
}
