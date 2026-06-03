// ArcFlow Runtime Core
//
// This crate is the Rust runtime kernel of ArcFlow. It contains the complete
// orchestration engine, state management, memory system, tool runtime,
// observability engine, and provider abstraction layer.
//
// Architecture: ADR-001 (docs pipeline — see docs/architecture/ADR-001.md locally)
// Contract: See contracts/normative/rcs/v1.schema.json
//
// Public API surface: Only types in the `rcs` module are public across
// the SDK boundary. All other public items are for inter-crate use within
// the runtime workspace.

#![allow(clippy::too_many_arguments, clippy::result_large_err)]

/// Runtime Contract Specification — types and serialization.
/// This module is fully implemented in Sprint 1.
pub mod rcs;

/// Named limits shared across runtime subsystems (Sprint 5).
pub mod constants;

/// Typed runtime errors for orchestration (Sprint 2).
pub mod error;

/// Workflow execution engine (Sprint 2 — `WorkflowEngine`, deterministic stub agents).
pub mod workflow;

/// Agent runtime and lifecycle management (Sprint 2 — `AgentRuntime` stub).
pub mod agent;

/// Execution state management (Sprint 2 — see `crate::state::StateEngine`).
pub mod state;

/// Memory subsystem — session, shared, persistent, vector.
/// Implemented in Sprint 4.
pub mod memory;

/// Tool execution runtime.
/// Implemented in Sprint 4.
pub mod tools;

/// Observability and tracing engine.
/// Implemented in Sprint 5.
pub mod tracing;

pub use tracing::get_execution_trace;

/// LLM provider abstraction layer.
/// Implemented in Sprint 6.
pub mod providers;

/// Internal streaming engine (Sprint 6).
pub mod streaming;

/// Retry and timeout enforcement (Sprint 7).
pub mod retry;

/// Partial execution recovery (Sprint 7).
pub mod recovery;

/// Human-in-the-loop pause and resume (Phase 1.4).
pub mod human;

/// External orchestration — bindings, callbacks, recovery (Phase 2-Pro v2).
pub mod external;

/// Local-only step-through debugging (Phase 2.4).
pub mod debug;

/// Embedding providers for vector memory (Phase 1.5).
pub mod embedding;

/// PostgreSQL schema migrations (Phase A).
pub mod migrate;
