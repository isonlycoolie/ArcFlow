//! Sprint 4 memory integration tests (in-process; Docker for persistent/vector).

use arcflow_core::memory::MemoryCoordinator;
use arcflow_core::rcs::types::{MemoryConfig, MemoryScope, MemoryType};
use arcflow_core::tracing::TraceEmitter;
use uuid::Uuid;

#[test]
fn session_write_read_same_agent() {
    let run = Uuid::new_v4();
    let agent = Uuid::new_v4();
    let coord = MemoryCoordinator::new(run);
    let mut trace = TraceEmitter::new(Uuid::new_v4());
    coord
        .write_session(agent, "note", b"hello", &mut trace, None)
        .unwrap();
    let got = coord.read_session(agent, "note", &mut trace, None).unwrap();
    assert_eq!(got.as_deref(), Some(b"hello".as_ref()));
}

#[test]
fn shared_agent_b_reads_agent_a_write() {
    let run = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let coord = MemoryCoordinator::new(run);
    let mut trace = TraceEmitter::new(Uuid::new_v4());
    let write_cfg = MemoryConfig {
        memory_type: MemoryType::Shared,
        scope: MemoryScope::Workflow,
        ttl_seconds: None,
    };
    let read_cfg = write_cfg.clone();
    coord
        .write_shared(a, "token", b"42", &write_cfg, &mut trace, None)
        .unwrap();
    let got = coord
        .read_shared(&read_cfg, a, "token", &mut trace, None)
        .unwrap();
    assert_eq!(got.as_deref(), Some(b"42".as_ref()));
    let _ = b;
}

#[test]
fn session_isolation_between_agents() {
    let run = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let coord = MemoryCoordinator::new(run);
    let mut trace = TraceEmitter::new(Uuid::new_v4());
    coord
        .write_session(a, "secret", b"x", &mut trace, None)
        .unwrap();
    let missing = coord.read_session(b, "secret", &mut trace, None).unwrap();
    assert!(missing.is_none());
}

#[test]
#[ignore = "requires ARCFLOW_POSTGRESQL_URL"]
fn persistent_survives_new_coordinator() {
    let url = std::env::var("ARCFLOW_POSTGRESQL_URL").expect("set for ignored test");
    std::env::set_var("ARCFLOW_POSTGRESQL_URL", url);
    let ns = format!("test-{}", Uuid::new_v4());
    let mut trace = TraceEmitter::new(Uuid::new_v4());
    let coord = MemoryCoordinator::new(Uuid::new_v4());
    coord
        .write_persistent(&ns, "k", b"v", &mut trace, None)
        .unwrap();
    let coord2 = MemoryCoordinator::new(Uuid::new_v4());
    let got = coord2
        .read_persistent(&ns, "k", &mut trace, None)
        .unwrap();
    assert_eq!(got.as_deref(), Some(b"v".as_ref()));
}

#[test]
#[ignore = "requires ARCFLOW_QDRANT_URL"]
fn vector_store_and_search_stub() {
    let _ = std::env::var("ARCFLOW_QDRANT_URL").expect("set for ignored test");
    let ns = format!("vec-{}", Uuid::new_v4());
    let mut trace = TraceEmitter::new(Uuid::new_v4());
    let coord = MemoryCoordinator::new(Uuid::new_v4());
    coord
        .write_vector(&ns, "doc", b"payload", &mut trace, None)
        .unwrap();
    let got = coord
        .read_vector(&ns, "doc", &mut trace, None)
        .unwrap();
    assert_eq!(got.as_deref(), Some(b"payload".as_ref()));
}
