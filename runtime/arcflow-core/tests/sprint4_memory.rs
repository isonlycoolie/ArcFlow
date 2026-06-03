//! Sprint 4 memory integration tests (in-process; Docker for persistent/vector).

use arcflow_core::memory::MemoryCoordinator;
use arcflow_core::rcs::types::{MemoryConfig, MemoryScope, MemoryType};
use arcflow_core::tracing::{
    emitter::TraceEmitter, sprint5_emitter::TraceEventEmitter, store::TraceStore,
};
use uuid::Uuid;

fn with_trace<F>(run_id: Uuid, f: F)
where
    F: FnOnce(&mut TraceEmitter, &mut TraceEventEmitter<'_>, &str),
{
    let mut store = TraceStore::new();
    let run_key = run_id.to_string();
    let mut legacy = TraceEmitter::new(run_id);
    let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
    f(&mut legacy, &mut sprint5, &run_key);
}

#[test]
fn session_write_read_same_agent() {
    let run = Uuid::new_v4();
    let agent = Uuid::new_v4();
    let coord = MemoryCoordinator::new(run);
    with_trace(run, |legacy, sprint5, run_key| {
        coord
            .write_session(
                agent, "note", b"hello", "agent", legacy, sprint5, run_key, None,
            )
            .unwrap();
        let got = coord
            .read_session(agent, "note", "agent", legacy, sprint5, run_key, None)
            .unwrap();
        assert_eq!(got.as_deref(), Some(b"hello".as_ref()));
    });
}

#[test]
fn shared_agent_b_reads_agent_a_write() {
    let run = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let coord = MemoryCoordinator::new(run);
    with_trace(run, |legacy, sprint5, run_key| {
        let write_cfg = MemoryConfig {
            memory_type: MemoryType::Shared,
            scope: MemoryScope::Workflow,
            namespace: None,
            ttl_seconds: None,
            embedding: None,
            retrieval: None,
            chunking: None,
        };
        let read_cfg = write_cfg.clone();
        coord
            .write_shared(
                a, "token", b"42", &write_cfg, "agent-a", legacy, sprint5, run_key, None,
            )
            .unwrap();
        let got = coord
            .read_shared(
                &read_cfg, a, "token", "agent-b", legacy, sprint5, run_key, None,
            )
            .unwrap();
        assert_eq!(got.as_deref(), Some(b"42".as_ref()));
        let _ = b;
    });
}

#[test]
fn session_isolation_between_agents() {
    let run = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let coord = MemoryCoordinator::new(run);
    with_trace(run, |legacy, sprint5, run_key| {
        coord
            .write_session(a, "secret", b"x", "agent-a", legacy, sprint5, run_key, None)
            .unwrap();
        let missing = coord
            .read_session(b, "secret", "agent-b", legacy, sprint5, run_key, None)
            .unwrap();
        assert!(missing.is_none());
    });
}

#[test]
#[ignore = "requires ARCFLOW_POSTGRESQL_URL"]
fn persistent_survives_new_coordinator() {
    let url = std::env::var("ARCFLOW_POSTGRESQL_URL").expect("set for ignored test");
    std::env::set_var("ARCFLOW_POSTGRESQL_URL", url);
    let ns = format!("test-{}", Uuid::new_v4());
    let run = Uuid::new_v4();
    with_trace(run, |legacy, sprint5, run_key| {
        let coord = MemoryCoordinator::new(run);
        coord
            .write_persistent(&ns, "k", b"v", "agent", legacy, sprint5, run_key, None)
            .unwrap();
        let coord2 = MemoryCoordinator::new(Uuid::new_v4());
        let got = coord2
            .read_persistent(&ns, "k", "agent", legacy, sprint5, run_key, None)
            .unwrap();
        assert_eq!(got.as_deref(), Some(b"v".as_ref()));
    });
}

#[test]
#[ignore = "requires ARCFLOW_QDRANT_URL"]
fn vector_store_and_search_stub() {
    let _ = std::env::var("ARCFLOW_QDRANT_URL").expect("set for ignored test");
    let ns = format!("vec-{}", Uuid::new_v4());
    let run = Uuid::new_v4();
    with_trace(run, |legacy, sprint5, run_key| {
        let coord = MemoryCoordinator::new(run);
        coord
            .write_vector(
                &ns,
                "chunk-1",
                b"search me",
                "agent",
                legacy,
                sprint5,
                run_key,
                None,
            )
            .unwrap();
        let got = coord
            .read_vector(&ns, "search me", "agent", legacy, sprint5, run_key, None)
            .unwrap();
        assert_eq!(got.as_deref(), Some(b"search me".as_ref()));
    });
}
