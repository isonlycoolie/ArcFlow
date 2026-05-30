//! Sprint 5 trace event kinds. Schema: contracts/normative/observability/trace-events-v1.md

use serde::{Deserialize, Serialize};

use super::types::TokenUsage;

/// All observable events in the ArcFlow execution lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "PascalCase")]
pub enum TraceEventKind {
    WorkflowStarted {
        run_id: String,
        workflow_name: String,
        step_count: usize,
    },
    WorkflowCompleted {
        run_id: String,
        duration_ms: u64,
        total_tokens: TokenUsage,
    },
    WorkflowFailed {
        run_id: String,
        duration_ms: u64,
        failed_step_index: Option<usize>,
        error_code: String,
    },
    WorkflowValidationFailed {
        run_id: String,
        reason: String,
    },
    StepStarted {
        run_id: String,
        step_id: String,
        step_index: usize,
        agent_name: String,
        agent_role: String,
    },
    StepCompleted {
        run_id: String,
        step_id: String,
        step_index: usize,
        duration_ms: u64,
        tokens: TokenUsage,
        output_size_bytes: usize,
    },
    StepFailed {
        run_id: String,
        step_id: String,
        step_index: usize,
        duration_ms: u64,
        error_code: String,
        error_message: String,
    },
    StateCommitted {
        run_id: String,
        step_id: String,
        committed_step_count: usize,
    },
    AgentInvoked {
        run_id: String,
        step_id: String,
        agent_name: String,
        input_size_bytes: usize,
    },
    AgentResponseReceived {
        run_id: String,
        step_id: String,
        agent_name: String,
        output_size_bytes: usize,
    },
    TokensConsumed {
        run_id: String,
        step_id: String,
        agent_name: String,
        tokens: TokenUsage,
    },
    ProviderRequestSent {
        run_id: String,
        step_id: String,
        provider_id: String,
        model_id: String,
        max_tokens: u32,
        prompt_size_bytes: usize,
    },
    ProviderResponseReceived {
        run_id: String,
        step_id: String,
        provider_id: String,
        model_id: String,
        tokens: TokenUsage,
        latency_ms: u64,
    },
    ProviderRateLimited {
        run_id: String,
        step_id: String,
        provider_id: String,
        retry_after_seconds: Option<u64>,
    },
    ProviderError {
        run_id: String,
        step_id: String,
        provider_id: String,
        error_code: String,
        error_message: String,
    },
    ToolCallStarted {
        run_id: String,
        step_id: String,
        tool_name: String,
        input_schema_hash: String,
    },
    ToolCallCompleted {
        run_id: String,
        step_id: String,
        tool_name: String,
        duration_ms: u64,
        output_size_bytes: usize,
    },
    ToolCallFailed {
        run_id: String,
        step_id: String,
        tool_name: String,
        duration_ms: u64,
        failure_reason: String,
        error_code: String,
    },
    ToolInputValidationFailed {
        run_id: String,
        step_id: String,
        tool_name: String,
        violation_description: String,
    },
    MemoryWrite {
        run_id: String,
        step_id: String,
        agent_name: String,
        memory_type: String,
        key: String,
        duration_ms: u64,
    },
    MemoryRead {
        run_id: String,
        step_id: String,
        agent_name: String,
        memory_type: String,
        key: String,
        hit: bool,
        duration_ms: u64,
    },
    MemoryDegraded {
        run_id: String,
        memory_type: String,
        backend: String,
        reason: String,
    },
    MemoryEvicted {
        run_id: String,
        memory_type: String,
        key: String,
        eviction_reason: String,
    },
    TraceStorageWarning {
        run_id: String,
        events_dropped: u32,
        capacity_limit: u32,
    },
    RetryAttempted {
        run_id: String,
        step_id: String,
        attempt_number: u32,
        max_attempts: u32,
        backoff_ms: u64,
        trigger_error_code: String,
    },
    RetryExhausted {
        run_id: String,
        step_id: String,
        total_attempts: u32,
        last_error_code: String,
    },
    TimeoutEnforced {
        run_id: String,
        step_id: String,
        timeout_type: String,
        configured_ms: u64,
        elapsed_ms: u64,
    },
    StepFallbackActivated {
        run_id: String,
        step_id: String,
        primary_agent_name: String,
        fallback_agent_name: String,
    },
    WorkflowRecoveryStarted {
        run_id: String,
        original_run_id: String,
        resume_from_step: usize,
    },
    WorkflowRecoveryCompleted {
        run_id: String,
        original_run_id: String,
        steps_re_executed: usize,
    },
    /// Streaming chunk metadata only — no token text (RCS v0.4, SEC-1).
    StreamChunkReceived {
        run_id: String,
        step_id: String,
        chunk_bytes: usize,
    },
    /// Token count delta during streaming — counts only, no text (RCS v0.4, SEC-1).
    TokenEmitted {
        run_id: String,
        step_id: String,
        completion_token_delta: u32,
        prompt_token_delta: u32,
    },
}
