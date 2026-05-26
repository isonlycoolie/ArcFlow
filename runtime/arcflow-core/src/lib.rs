// ArcFlow Runtime Core
//
// This crate is the Rust runtime kernel of ArcFlow. It contains the complete
// orchestration engine, state management, memory system, tool runtime,
// observability engine, and provider abstraction layer.
//
// Architecture: ADR-001 (docs pipeline — URL in contracts/sprint1-acceptance-evidence.md)
// Contract: See contracts/rcs-v1.schema.json
//
// Public API surface: Only types in the `rcs` module are public across
// the SDK boundary. All other public items are for inter-crate use within
// the runtime workspace.

/// Runtime Contract Specification — types and serialization.
/// This module is fully implemented in Sprint 1.
pub mod rcs;

/// Workflow execution engine.
/// Implemented in Sprint 2.
pub mod workflow;

/// Agent runtime and lifecycle management.
/// Implemented in Sprint 2.
pub mod agent;

/// Execution state management.
/// Implemented in Sprint 2.
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

/// LLM provider abstraction layer.
/// Implemented in Sprint 6.
pub mod providers;
