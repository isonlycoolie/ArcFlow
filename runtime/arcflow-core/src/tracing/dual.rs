//! Dual emission to Sprint 4 RCS trace and Sprint 5 store.

#![allow(clippy::too_many_arguments)]

use std::time::Instant;
use uuid::Uuid;

use super::emitter::TraceEmitter;
use super::events::TraceEventKind;
use super::sprint5_emitter::TraceEventEmitter;
use super::types::TokenUsage;

/// Emits tool start + legacy metadata on completion path.
pub fn tool_started(
    sprint5: &mut TraceEventEmitter<'_>,
    run_id: &str,
    step_id: Uuid,
    tool_name: &str,
) {
    sprint5.emit(TraceEventKind::ToolCallStarted {
        run_id: run_id.to_string(),
        step_id: step_id.to_string(),
        tool_name: tool_name.to_string(),
        input_schema_hash: String::new(),
    });
}

/// Records tool completion on both traces.
pub fn tool_finished(
    legacy: &mut TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_id: &str,
    step_id: Option<Uuid>,
    tool_name: &str,
    ok: bool,
    duration_ms: u64,
    output_size_bytes: usize,
    error_code: Option<&str>,
) {
    let status = if ok { "ok" } else { "failed" };
    legacy.tool_executed(step_id, tool_name, status, duration_ms);
    let sid = step_id.map(|u| u.to_string()).unwrap_or_default();
    if ok {
        sprint5.emit(TraceEventKind::ToolCallCompleted {
            run_id: run_id.to_string(),
            step_id: sid,
            tool_name: tool_name.to_string(),
            duration_ms,
            output_size_bytes,
        });
    } else {
        sprint5.emit(TraceEventKind::ToolCallFailed {
            run_id: run_id.to_string(),
            step_id: sid,
            tool_name: tool_name.to_string(),
            duration_ms,
            failure_reason: error_code.unwrap_or("failed").to_string(),
            error_code: error_code.unwrap_or("tool_failed").to_string(),
        });
    }
}

/// Memory read on both traces (no values).
pub fn memory_read(
    legacy: &mut TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_id: &str,
    step_id: Option<Uuid>,
    agent_name: &str,
    memory_type: &str,
    key: &str,
    hit: bool,
    started: Instant,
) {
    legacy.memory_read(step_id, memory_type, key.len());
    sprint5.emit(TraceEventKind::MemoryRead {
        run_id: run_id.to_string(),
        step_id: step_id.map(|u| u.to_string()).unwrap_or_default(),
        agent_name: agent_name.to_string(),
        memory_type: memory_type.to_string(),
        key: key.to_string(),
        hit,
        duration_ms: started.elapsed().as_millis() as u64,
    });
}

/// Memory write on both traces (no values).
pub fn memory_write(
    legacy: &mut TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_id: &str,
    step_id: Option<Uuid>,
    agent_name: &str,
    memory_type: &str,
    key: &str,
    started: Instant,
) {
    legacy.memory_write(step_id, memory_type, key.len());
    sprint5.emit(TraceEventKind::MemoryWrite {
        run_id: run_id.to_string(),
        step_id: step_id.map(|u| u.to_string()).unwrap_or_default(),
        agent_name: agent_name.to_string(),
        memory_type: memory_type.to_string(),
        key: key.to_string(),
        duration_ms: started.elapsed().as_millis() as u64,
    });
}

/// Stub token event until Sprint 6 provider wiring.
pub fn tokens_consumed(
    sprint5: &mut TraceEventEmitter<'_>,
    run_id: &str,
    step_id: Uuid,
    agent_name: &str,
) {
    sprint5.emit(TraceEventKind::TokensConsumed {
        run_id: run_id.to_string(),
        step_id: step_id.to_string(),
        agent_name: agent_name.to_string(),
        tokens: TokenUsage::default(),
    });
}
