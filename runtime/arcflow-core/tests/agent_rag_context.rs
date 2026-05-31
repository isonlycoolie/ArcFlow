//! Vector memory ingest/search for agent RAG path (Phase 2-Pro).

use uuid::Uuid;

use arcflow_core::agent::AgentRuntime;
use arcflow_core::memory::MemoryCoordinator;
use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionStatus, MemoryConfig, MemoryRetrievalConfig, MemoryScope,
    MemoryType, RetrievalModeSpec,
};
use arcflow_core::state::StateSnapshot;
use arcflow_core::tracing::{
    emitter::TraceEmitter, sprint5_emitter::TraceEventEmitter, store::TraceStore,
};
use arcflow_core::workflow::ExecutionContext;

fn vector_memory_config(namespace: &str) -> MemoryConfig {
    MemoryConfig {
        memory_type: MemoryType::Vector,
        scope: MemoryScope::Agent,
        namespace: Some(namespace.into()),
        ttl_seconds: None,
        embedding: Some("stub/384".into()),
        retrieval: Some(MemoryRetrievalConfig {
            mode: RetrievalModeSpec::Dense,
            dense_weight: 1.0,
            sparse_weight: 0.0,
            rerank: None,
            top_k: Some(3),
        }),
        chunking: None,
    }
}

#[test]
#[ignore = "requires ARCFLOW_QDRANT_URL"]
fn vector_agent_retrieves_chunks_in_context() {
    let _ = std::env::var("ARCFLOW_QDRANT_URL").expect("set for ignored test");
    let run = Uuid::new_v4();
    let step_id = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let namespace = "course-kb";
    let config = vector_memory_config(namespace);

    let coord = MemoryCoordinator::new(run);
    let mut store = TraceStore::new();
    let run_key = run.to_string();
    let mut legacy = TraceEmitter::new(run);
    let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);

    coord
        .write_vector_document(
            &config,
            "syllabus",
            "Module 3 covers retrieval-augmented generation patterns in ArcFlow.",
            "ingest",
            &mut legacy,
            &mut sprint5,
            &run_key,
            None,
        )
        .expect("ingest");

    let agent = AgentDefinition {
        id: agent_id,
        name: "qa".into(),
        role: "tutor".into(),
        instructions: "Answer using retrieved knowledge.".into(),
        tools: None,
        memory_config: Some(config),
        context: None,
        tool_execution: None,
    };

    let mut memory = MemoryCoordinator::new(run);
    // Re-ingest on the agent coordinator used during execution.
    memory
        .write_vector_document(
            agent.memory_config.as_ref().unwrap(),
            "syllabus",
            "Module 3 covers retrieval-augmented generation patterns in ArcFlow.",
            "ingest",
            &mut legacy,
            &mut sprint5,
            &run_key,
            None,
        )
        .expect("ingest on run coord");

    let mut exec_ctx = ExecutionContext {
        tool_runtime: None,
        tool_invoker: None,
        memory: &mut memory,
        legacy: &mut legacy,
        sprint5: &mut sprint5,
        run_id: run_key,
        provider: None,
        provider_max_tokens: 256,
        provider_temperature: 0.0,
        retry_config: None,
        step_timeout: None,
        workflow_deadline: None,
        step_order: 1,
        test_config: None,
        test_attempt: 1,
        stream_tx: None,
        graph_state: None,
    };

    let state = StateSnapshot { steps: vec![] };
    let out = AgentRuntime::new()
        .execute_with_context(
            &agent,
            step_id,
            &state,
            "retrieval-augmented generation",
            Some(&mut exec_ctx),
        )
        .unwrap();

    assert!(out.content.contains("## Retrieved knowledge"));
    assert!(out.content.contains("retrieval-augmented generation"));
    assert_eq!(out.status, ExecutionStatus::Completed);
}
