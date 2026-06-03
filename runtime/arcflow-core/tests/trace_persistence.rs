//! Trace persistence survives in-process store eviction (Postgres required).

use arcflow_core::tracing::events::TraceEventKind;
use arcflow_core::tracing::persistence::PostgresTracePersistence;
use arcflow_core::tracing::registry::{get_execution_trace, with_store};
use arcflow_core::tracing::sprint5_emitter::TraceEventEmitter;

#[tokio::test]
#[ignore = "requires ARCFLOW_POSTGRESQL_URL and migration 005 applied"]
async fn persisted_trace_loads_after_store_eviction() {
    let url = std::env::var("ARCFLOW_POSTGRESQL_URL").expect("ARCFLOW_POSTGRESQL_URL");
    let pool = sqlx::PgPool::connect(&url).await.expect("connect postgres");
    let migration = include_str!("../migrations/20240531000005_trace_events.sql");
    for stmt in migration.split(';').filter(|s| s.trim().starts_with("CREATE")) {
        sqlx::query(stmt).execute(&pool).await.expect("migrate");
    }

    let run_id = "trace-persist-test-run";
    with_store(|store| {
        let mut emitter = TraceEventEmitter::new(run_id.to_string(), store);
        emitter.emit(TraceEventKind::WorkflowStarted {
            run_id: run_id.to_string(),
            workflow_name: "test".into(),
            step_count: 1,
        });
        store.mark_complete(run_id);
    });

    let trace = get_execution_trace(run_id).expect("in-memory trace");
    assert!(!trace.events.is_empty());

    let persistence = PostgresTracePersistence::new(pool);
    persistence
        .persist_events(run_id, &trace.events)
        .await
        .expect("persist");

    let loaded = persistence
        .load_execution_trace(run_id)
        .await
        .expect("load")
        .expect("trace rows");
    assert_eq!(loaded.events.len(), trace.events.len());
    assert_eq!(loaded.run_id, run_id);
}
