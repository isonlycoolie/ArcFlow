//! ArcFlow WASM edge runtime (alpha).
//!
//! Stub linear workflow execution for Cloudflare Workers and similar hosts.
//! Full `arcflow-core` linkage is deferred until native deps are wasm-gated.

pub mod types;
pub mod runner;
