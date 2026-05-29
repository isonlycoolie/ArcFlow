exports.executeWorkflow = async () => ({
  output: "[stub] writer (author): Reply briefly.",
  runId: "00000000-0000-4000-8000-000000000001",
  stepCount: 1,
  traceEventsJson: "[]",
});

exports.getExecutionTraceJson = () =>
  JSON.stringify({
    run_id: "00000000-0000-4000-8000-000000000001",
    workflow_name: "ts-trace",
    status: "Completed",
    started_at: "2026-01-01T00:00:00Z",
    completed_at: "2026-01-01T00:00:01Z",
    duration_ms: 1000,
    total_tokens: { total_tokens: 0 },
    steps: [
      {
        step_index: 0,
        agent_name: "writer",
        agent_role: "author",
        status: "Completed",
        started_at: "2026-01-01T00:00:00Z",
        completed_at: "2026-01-01T00:00:01Z",
        duration_ms: 1000,
        tokens: { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
      },
    ],
  });

exports.getVersion = () => "0.1.0";
