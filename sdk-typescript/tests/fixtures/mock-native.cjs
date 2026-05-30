function stubOutputFromExecConfig(execConfigJson) {
  let output = "[stub] writer (author): Reply briefly.";
  if (!execConfigJson) {
    return output;
  }
  try {
    const cfg = JSON.parse(execConfigJson);
    const stub = cfg.test?.stub_responses?.step_1;
    if (stub && typeof stub.then_output === "string" && stub.fail_times != null) {
      return stub.then_output;
    }
    if (typeof stub?.output === "string") {
      return stub.output;
    }
  } catch {
    // ignore malformed exec config in mock
  }
  return output;
}

exports.executeWorkflow = async (
  _workflowName,
  _workflowId,
  _agents,
  _steps,
  _runInput,
  _provider,
  execConfigJson,
  graphJson,
) => {
  const output = stubOutputFromExecConfig(execConfigJson);
  const stepCount = graphJson ? JSON.parse(graphJson).nodes.length : 1;
  return {
    output,
    runId: "00000000-0000-4000-8000-000000000001",
    stepCount,
    traceEventsJson: "[]",
  };
};

exports.executeResumeWorkflow = async () => ({
  output: "[stub] writer resumed",
  runId: "00000000-0000-4000-8000-000000000002",
  stepCount: 2,
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

exports.executeWorkflowStream = async () => ({
  eventsJson: JSON.stringify([
    { type: "step_start", step_id: "step-1" },
    { type: "step_complete", step_id: "step-1", duration_ms: 1 },
  ]),
  output: "[stub] writer (author): Reply briefly.",
  runId: "00000000-0000-4000-8000-000000000003",
  stepCount: 1,
  traceEventsJson: "[]",
});

exports.startWorkflowStream = (
  _workflowName,
  _workflowId,
  _agents,
  _steps,
  _runInput,
  _provider,
  execConfigJson,
) => {
  const output = stubOutputFromExecConfig(execConfigJson);
  const queued = [
    JSON.stringify({ type: "step_start", step_id: "step-1" }),
    JSON.stringify({ type: "step_complete", step_id: "step-1", duration_ms: 1 }),
  ];
  let index = 0;
  return {
    pollEvent() {
      if (index >= queued.length) {
        return null;
      }
      const next = queued[index];
      index += 1;
      return next;
    },
    finalize() {
      return {
        output,
        runId: "00000000-0000-4000-8000-000000000004",
        stepCount: 1,
        traceEventsJson: "[]",
      };
    },
  };
};
